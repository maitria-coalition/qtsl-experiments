//! simreplay — re-derive an AtariCircuit replay claim's pinned witnesses
//! from a user-supplied ROM, through the Verilator sim, and compare them
//! against the committed claim.cbor.
//!
//! This is the *sim side* of the AIG-vs-Verilator artefact-pair (see
//! `certs/RESULTS.md`): the committed claims pin sha256 hashes of trace
//! (and, for the pixel-witnessed row, frame) witnesses that are never
//! committed because they embed ROM-derived contents. Given your own ROM
//! image, this tool replays every lane of the claim through the sim binary,
//! independently recomputes the reward stream from the RAM trace (the same
//! cross-check the producer ran), and compares:
//!
//!   - per-lane trace record counts against the claim,
//!   - per-lane returns against `claimed_returns`,
//!   - the min-over-lanes bound against `claimed_bound`,
//!   - sha256 of each re-derived raw `.atrc` trace against the pinned hash
//!     (the pinned hash is of the *uncompressed* trace bytes),
//!   - sha256 of each re-derived raw `.afrm` frame witness, when the claim
//!     pins frame witnesses,
//!   - optionally (`--aig`), sha256 of a synthesized netlist against the
//!     claim's `aig_sha256`.
//!
//! An ACCEPT means: from your ROM, this machine re-derived byte-identical
//! witnesses and the exact claimed returns. It does NOT replace the AIG
//! replay checker (the trusted side of the pair — see the trust-base
//! statement in `certs/RESULTS.md`); it establishes that the committed
//! claim is about the ROM you hold.
//!
//! Usage:
//!   simreplay <bundle-dir|claim.cbor> --rom <rom.bin|rom.hex> --sim <sim-binary>
//!             [--jobs N] [--aig <netlist.aig>] [--keep <dir>] [--json <out.json>]
//!   simreplay --rom2hex <rom.bin> <out.hex>
//!
//! ROM handling: a `.hex` file (one hex byte per line, `$readmemh` format)
//! is used as-is; a binary image is converted internally — 2K images are
//! doubled (the 4K mirror baked in), 4K and 8K pass through verbatim.

use ciborium::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const NOOP: i64 = 0;
const ATRC_REC: usize = 132;
const RAM: usize = 128;
const AFRM_W: usize = 160;
const AFRM_H: usize = 239;
const AFRM_Y0: u16 = 22;

// ---------------------------------------------------------------- claim

#[derive(Debug, Clone)]
struct WitnessRef {
    sha256: String,
    frames: usize,
}

#[derive(Debug)]
struct Claim {
    env: String,
    aig_sha256: String,
    reward: String,
    paddle: [i64; 4],
    policy_kind: String,
    policy_name: String,
    actions: Vec<i64>,               // script claims
    actions_per_lane: Vec<Vec<i64>>, // script-family claims
    init_set: Vec<i64>,
    horizon: usize,
    claimed_returns: Vec<i64>,
    claimed_bound: i64,
    traces: Vec<WitnessRef>,
    frames: Vec<WitnessRef>, // empty unless the pixel-witnessed species
}

fn as_map(v: &Value, what: &str) -> Result<BTreeMap<String, Value>, String> {
    match v {
        Value::Map(m) => {
            let mut out = BTreeMap::new();
            for (k, val) in m {
                if let Value::Text(t) = k {
                    out.insert(t.clone(), val.clone());
                }
            }
            Ok(out)
        }
        _ => Err(format!("{what}: expected a CBOR map")),
    }
}

fn get<'a>(m: &'a BTreeMap<String, Value>, k: &str) -> Result<&'a Value, String> {
    m.get(k).ok_or_else(|| format!("claim missing field `{k}`"))
}

fn as_text(v: &Value, what: &str) -> Result<String, String> {
    match v {
        Value::Text(t) => Ok(t.clone()),
        _ => Err(format!("{what}: expected text")),
    }
}

fn as_int(v: &Value, what: &str) -> Result<i64, String> {
    match v {
        Value::Integer(i) => i128::from(*i)
            .try_into()
            .map_err(|_| format!("{what}: out of i64 range")),
        _ => Err(format!("{what}: expected integer")),
    }
}

fn as_int_vec(v: &Value, what: &str) -> Result<Vec<i64>, String> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|x| as_int(x, what))
            .collect::<Result<Vec<_>, _>>(),
        _ => Err(format!("{what}: expected array")),
    }
}

fn witness_refs(v: &Value, what: &str) -> Result<Vec<WitnessRef>, String> {
    let Value::Array(a) = v else {
        return Err(format!("{what}: expected array"));
    };
    a.iter()
        .map(|x| {
            let m = as_map(x, what)?;
            Ok(WitnessRef {
                sha256: as_text(get(&m, "sha256")?, "sha256")?,
                frames: as_int(get(&m, "frames")?, "frames")? as usize,
            })
        })
        .collect()
}

/// The boot protocol + fixed inputs this sim driver implements. A claim
/// declaring anything else cannot be replayed by this tool: refuse rather
/// than replay the wrong environment variant.
fn check_fixed_protocol(claim: &BTreeMap<String, Value>) -> Result<(), String> {
    let schema = as_text(get(claim, "schema")?, "schema")?;
    if schema != "atari-circuit-u1-replay/v0" {
        return Err(format!("unsupported claim schema `{schema}`"));
    }
    let init_latches = as_text(get(claim, "init_latches")?, "init_latches")?;
    if init_latches != "zero" {
        return Err(format!("unsupported init_latches `{init_latches}`"));
    }
    let boot = as_map(get(claim, "boot")?, "boot")?;
    let expect = [
        ("protocol", Value::Text("ale".into())),
        ("reset_hold_clks", Value::Integer(2000.into())),
        ("noop_frames", Value::Integer(60.into())),
        ("reset_press_frames", Value::Integer(8.into())),
    ];
    for (k, want) in expect {
        let got = get(&boot, k)?;
        if *got != want {
            return Err(format!(
                "boot.{k} differs from the protocol this sim implements"
            ));
        }
    }
    let inputs = as_map(get(claim, "inputs")?, "inputs")?;
    if as_int(get(&inputs, "switches")?, "switches")? != 0xF {
        return Err("inputs.switches differs from the sim's fixed 0xF".into());
    }
    if as_int(get(&inputs, "joy1")?, "joy1")? != 0x7F {
        return Err("inputs.joy1 differs from the sim's fixed 0x7F".into());
    }
    Ok(())
}

fn load_claim(path: &Path) -> Result<Claim, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value =
        ciborium::from_reader(&bytes[..]).map_err(|e| format!("{}: {e}", path.display()))?;
    let m = as_map(&v, "claim")?;
    check_fixed_protocol(&m)?;

    let inputs = as_map(get(&m, "inputs")?, "inputs")?;
    let paddle_v = as_int_vec(get(&inputs, "paddle")?, "inputs.paddle")?;
    if paddle_v.len() != 4 {
        return Err("inputs.paddle: expected 4 entries".into());
    }

    let policy = as_map(get(&m, "policy")?, "policy")?;
    let policy_kind = as_text(get(&policy, "kind")?, "policy.kind")?;
    let actions = as_int_vec(get(&policy, "actions")?, "policy.actions")?;
    let actions_per_lane = match policy.get("actions_per_lane") {
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| as_int_vec(x, "policy.actions_per_lane[]"))
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    match policy_kind.as_str() {
        "script" if !actions.is_empty() => {}
        "script-family" if !actions_per_lane.is_empty() => {}
        _ => return Err(format!("policy kind `{policy_kind}` with empty script")),
    }

    Ok(Claim {
        env: as_text(get(&m, "env")?, "env")?,
        aig_sha256: as_text(get(&m, "aig_sha256")?, "aig_sha256")?,
        reward: as_text(get(&m, "reward")?, "reward")?,
        paddle: [paddle_v[0], paddle_v[1], paddle_v[2], paddle_v[3]],
        policy_name: as_text(get(&policy, "name")?, "policy.name")?,
        policy_kind,
        actions,
        actions_per_lane,
        init_set: as_int_vec(get(&m, "init_set")?, "init_set")?,
        horizon: as_int(get(&m, "horizon")?, "horizon")? as usize,
        claimed_returns: as_int_vec(get(&m, "claimed_returns")?, "claimed_returns")?,
        claimed_bound: as_int(get(&m, "claimed_bound")?, "claimed_bound")?,
        traces: witness_refs(get(&m, "traces")?, "traces")?,
        frames: match m.get("frames") {
            Some(v) => witness_refs(v, "frames")?,
            None => Vec::new(),
        },
    })
}

// ---------------------------------------------------------------- reward

/// Score decodes, mirroring ale-py 0.12.0's games/supported/*.cpp — the
/// independent recompute of the sim's own C++ decode (the producer ran the
/// same cross-check in python; three implementations now agree or refuse).
fn bcd2(b: u8) -> i64 {
    ((b >> 4) as i64) * 10 + (b & 0xF) as i64
}

fn decode_score(reward: &str, ram: &[u8]) -> Result<i64, String> {
    match reward {
        "pong" => Ok(ram[14] as i64 - ram[13] as i64),
        "freeway" => Ok(bcd2(ram[103])),
        "montezuma" => Ok(bcd2(ram[19]) * 10000 + bcd2(ram[20]) * 100 + bcd2(ram[21])),
        other => Err(format!("unknown reward decode `{other}`")),
    }
}

fn clamp(reward: &str, r: i64) -> i64 {
    if reward == "freeway" {
        r.clamp(0, 1)
    } else {
        r
    }
}

// ---------------------------------------------------------------- lanes

struct LaneOutcome {
    k: i64,
    ret: i64,
    trace_hash_ok: bool,
    frame_hash_ok: Option<bool>,
    sim_line: String,
    wall_s: f64,
}

struct LaneJob<'a> {
    claim: &'a Claim,
    ki: usize,
    sim: &'a Path,
    rom_hex: &'a Path,
    workdir: &'a Path,
}

fn parse_atrc(data: &[u8]) -> Result<(Vec<[u8; RAM]>, Vec<i32>), String> {
    if data.len() < 16 || &data[0..4] != b"ATRC" || data[4] != 1 || data[5] as usize != ATRC_REC {
        return Err("bad .atrc header".into());
    }
    let body = &data[16..];
    if body.len() % ATRC_REC != 0 {
        return Err("truncated .atrc record".into());
    }
    let n = body.len() / ATRC_REC;
    let mut frames = Vec::with_capacity(n);
    let mut rewards = Vec::with_capacity(n);
    for i in 0..n {
        let rec = &body[i * ATRC_REC..(i + 1) * ATRC_REC];
        let mut ram = [0u8; RAM];
        ram.copy_from_slice(&rec[..RAM]);
        frames.push(ram);
        rewards.push(i32::from_le_bytes(rec[RAM..].try_into().unwrap()));
    }
    Ok((frames, rewards))
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    let d = h.finalize();
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn run_lane(job: &LaneJob) -> Result<LaneOutcome, String> {
    let c = job.claim;
    let k = c.init_set[job.ki];
    let t0 = std::time::Instant::now();

    let actions: &[i64] = if c.actions_per_lane.is_empty() {
        &c.actions
    } else {
        &c.actions_per_lane[job.ki]
    };
    if actions.len() != c.horizon {
        return Err(format!(
            "lane k={k}: script length {} != horizon {}",
            actions.len(),
            c.horizon
        ));
    }

    let script_path = job.workdir.join(format!("k{k:02}.script"));
    {
        let mut f = fs::File::create(&script_path).map_err(|e| e.to_string())?;
        let mut s = String::new();
        for _ in 0..k {
            s.push_str(&format!("{NOOP}\n"));
        }
        for a in actions {
            s.push_str(&format!("{a}\n"));
        }
        f.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
    }

    let trace_path = job.workdir.join(format!("k{k:02}.atrc"));
    let afrm_path = job.workdir.join(format!("k{k:02}.afrm"));
    let want_frames = !c.frames.is_empty();

    let mut cmd = Command::new(job.sim);
    cmd.arg(format!("+ROMHEX={}", job.rom_hex.display()))
        .args(["--boot", "ale"])
        .arg("--script")
        .arg(&script_path)
        .arg("--trace")
        .arg(&trace_path)
        .args(["--reward", &c.reward]);
    for (i, p) in c.paddle.iter().enumerate() {
        cmd.arg(format!("--paddle{i}")).arg(p.to_string());
    }
    if want_frames {
        cmd.arg("--framedump").arg(&afrm_path);
    }
    let out = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("lane k={k}: sim spawn failed: {e}"))?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        return Err(format!("lane k={k}: sim failed:\n{stderr}"));
    }
    let sim_line = stderr.trim().lines().last().unwrap_or("").to_string();

    // -- trace witness --
    let data = fs::read(&trace_path).map_err(|e| e.to_string())?;
    let (frames, rewards) = parse_atrc(&data)?;
    let expect_recs = k as usize + c.horizon;
    if frames.len() != expect_recs || c.traces[job.ki].frames != expect_recs {
        return Err(format!(
            "trace-length: lane k={k}: {} records, claim declares {}, protocol wants {}",
            frames.len(),
            c.traces[job.ki].frames,
            expect_recs
        ));
    }

    // -- independent reward recompute (records 1.. — record 0's baseline is
    // primed on the post-boot state, invisible in the trace) --
    for i in 1..frames.len() {
        let r = clamp(
            &c.reward,
            decode_score(&c.reward, &frames[i])? - decode_score(&c.reward, &frames[i - 1])?,
        );
        if r != rewards[i] as i64 {
            return Err(format!(
                "reward-recompute: lane k={k} frame {i}: independent decode {r} != sim reward {}",
                rewards[i]
            ));
        }
    }
    let ret: i64 = rewards[k as usize..].iter().map(|r| *r as i64).sum();

    let trace_hash_ok = sha256_hex(&data) == c.traces[job.ki].sha256;

    // -- frame witness (pixel-witnessed species only) --
    let frame_hash_ok = if want_frames {
        let fdata = fs::read(&afrm_path).map_err(|e| e.to_string())?;
        if fdata.len() < 16 || &fdata[0..4] != b"AFRM" || fdata[4] != 1 {
            return Err(format!("lane k={k}: bad .afrm header"));
        }
        let w = u16::from_le_bytes([fdata[6], fdata[7]]) as usize;
        let h = u16::from_le_bytes([fdata[8], fdata[9]]) as usize;
        let y0 = u16::from_le_bytes([fdata[10], fdata[11]]);
        if (w, h, y0) != (AFRM_W, AFRM_H, AFRM_Y0) {
            return Err(format!(
                "lane k={k}: unexpected .afrm geometry {w}x{h}@y{y0}"
            ));
        }
        if (fdata.len() - 16) != expect_recs * w * h || c.frames[job.ki].frames != expect_recs {
            return Err(format!(
                "frame-length: lane k={k}: afrm record count mismatch"
            ));
        }
        Some(sha256_hex(&fdata) == c.frames[job.ki].sha256)
    } else {
        None
    };

    Ok(LaneOutcome {
        k,
        ret,
        trace_hash_ok,
        frame_hash_ok,
        sim_line,
        wall_s: t0.elapsed().as_secs_f64(),
    })
}

// ---------------------------------------------------------------- rom

fn rom_to_hex(rom: &Path, out: &Path) -> Result<(), String> {
    let mut data = fs::read(rom).map_err(|e| format!("{}: {e}", rom.display()))?;
    match data.len() {
        2048 => data = [data.as_slice(), data.as_slice()].concat(),
        4096 | 8192 => {}
        n => {
            return Err(format!(
                "unsupported ROM size {n} (want 2048, 4096, or 8192)"
            ))
        }
    }
    let mut s = String::with_capacity(data.len() * 3);
    for b in &data {
        s.push_str(&format!("{b:02x}\n"));
    }
    fs::write(out, s).map_err(|e| format!("{}: {e}", out.display()))
}

// ---------------------------------------------------------------- main

fn usage() -> String {
    "usage: simreplay <bundle-dir|claim.cbor> --rom <rom.bin|rom.hex> --sim <sim-binary>\n\
     \x20                [--jobs N] [--aig <netlist.aig>] [--keep <dir>] [--json <out.json>]\n\
     \x20      simreplay --rom2hex <rom.bin> <out.hex>"
        .into()
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn run() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--rom2hex") {
        if args.len() != 3 {
            return Err(usage());
        }
        rom_to_hex(Path::new(&args[1]), Path::new(&args[2]))?;
        println!("wrote {}", args[2]);
        return Ok(true);
    }

    let mut claim_arg = None;
    let mut rom = None;
    let mut sim = None;
    let mut aig = None;
    let mut keep = None;
    let mut json_out = None;
    let mut jobs: usize = std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4);
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let mut next = || -> Result<String, String> {
            i += 1;
            args.get(i).cloned().ok_or_else(usage)
        };
        match a.as_str() {
            "--rom" => rom = Some(PathBuf::from(next()?)),
            "--sim" => sim = Some(PathBuf::from(next()?)),
            "--aig" => aig = Some(PathBuf::from(next()?)),
            "--keep" => keep = Some(PathBuf::from(next()?)),
            "--json" => json_out = Some(PathBuf::from(next()?)),
            "--jobs" => jobs = next()?.parse().map_err(|_| usage())?,
            _ if claim_arg.is_none() && !a.starts_with('-') => {
                claim_arg = Some(PathBuf::from(a));
            }
            _ => return Err(usage()),
        }
        i += 1;
    }
    let (claim_arg, rom, sim) = match (claim_arg, rom, sim) {
        (Some(c), Some(r), Some(s)) => (c, r, s),
        _ => return Err(usage()),
    };
    let claim_path = if claim_arg.is_dir() {
        claim_arg.join("claim.cbor")
    } else {
        claim_arg
    };

    let claim = load_claim(&claim_path)?;
    let lanes = claim.init_set.len();
    if claim.claimed_returns.len() != lanes || claim.traces.len() != lanes {
        return Err("claim lane counts disagree (init_set vs returns vs traces)".into());
    }
    println!(
        "claim: {} policy {} ({}), T={}, {} lane(s), claimed bound {}",
        claim.env, claim.policy_name, claim.policy_kind, claim.horizon, lanes, claim.claimed_bound
    );

    // workdir + ROM hex
    let (workdir, ephemeral) = match keep {
        Some(d) => (d, false),
        None => (
            std::env::temp_dir().join(format!("simreplay-{}", std::process::id())),
            true,
        ),
    };
    fs::create_dir_all(&workdir).map_err(|e| e.to_string())?;
    let rom_hex = if rom.extension().is_some_and(|e| e == "hex") {
        rom.clone()
    } else {
        let p = workdir.join("rom.hex");
        rom_to_hex(&rom, &p)?;
        p
    };

    // lanes, in parallel
    let next_lane = AtomicUsize::new(0);
    let results: Mutex<Vec<Result<LaneOutcome, String>>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..jobs.max(1).min(lanes) {
            s.spawn(|| loop {
                let ki = next_lane.fetch_add(1, Ordering::Relaxed);
                if ki >= lanes {
                    break;
                }
                let r = run_lane(&LaneJob {
                    claim: &claim,
                    ki,
                    sim: &sim,
                    rom_hex: &rom_hex,
                    workdir: &workdir,
                });
                if let Ok(o) = &r {
                    println!(
                        "  k={:2}: return {:4}  trace {}  {}  ({:.1}s)",
                        o.k,
                        o.ret,
                        if o.trace_hash_ok {
                            "sha-match"
                        } else {
                            "SHA-MISMATCH"
                        },
                        match o.frame_hash_ok {
                            Some(true) => "frames sha-match",
                            Some(false) => "frames SHA-MISMATCH",
                            None => "",
                        },
                        o.wall_s
                    );
                }
                results.lock().unwrap().push(r);
            });
        }
    });

    let mut outcomes = Vec::new();
    for r in results.into_inner().unwrap() {
        outcomes.push(r?);
    }
    outcomes.sort_by_key(|o| o.k);

    // verdict
    let mut refusals: Vec<String> = Vec::new();
    for (ki, o) in outcomes.iter().enumerate() {
        let ki_claim = claim.init_set.iter().position(|k| *k == o.k).unwrap_or(ki);
        if o.ret != claim.claimed_returns[ki_claim] {
            refusals.push(format!(
                "return-mismatch: lane k={}: computed {}, claim says {}",
                o.k, o.ret, claim.claimed_returns[ki_claim]
            ));
        }
        if !o.trace_hash_ok {
            refusals.push(format!("trace-hash: lane k={}", o.k));
        }
        if o.frame_hash_ok == Some(false) {
            refusals.push(format!("frame-hash: lane k={}", o.k));
        }
    }
    let min_ret = outcomes.iter().map(|o| o.ret).min().unwrap_or(0);
    if min_ret != claim.claimed_bound {
        refusals.push(format!(
            "bound: computed min {} != claimed {}",
            min_ret, claim.claimed_bound
        ));
    }
    let aig_ok = match &aig {
        Some(p) => {
            let ok = sha256_hex(&fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?)
                == claim.aig_sha256;
            if !ok {
                refusals.push(format!(
                    "aig-hash: {} does not match the pinned netlist \
                     (byte-identity requires the pinned toolchain — see README)",
                    p.display()
                ));
            }
            Some(ok)
        }
        None => {
            println!(
                "note: no --aig given; pinned netlist sha256 is {}",
                claim.aig_sha256
            );
            None
        }
    };

    let verdict = refusals.is_empty();
    println!(
        "{}: min return {} over {} lane(s), bound {} — {}",
        if verdict { "ACCEPT" } else { "REFUSE" },
        min_ret,
        lanes,
        claim.claimed_bound,
        if verdict {
            "all witnesses re-derived bit-identical".to_string()
        } else {
            refusals.join("; ")
        }
    );

    if let Some(jp) = json_out {
        let lanes_json: Vec<String> = outcomes
            .iter()
            .map(|o| {
                format!(
                    "{{\"k\":{},\"return\":{},\"trace_sha_match\":{},\"frame_sha_match\":{},\"sim_line\":\"{}\",\"wall_s\":{:.2}}}",
                    o.k,
                    o.ret,
                    o.trace_hash_ok,
                    o.frame_hash_ok.map_or("null".into(), |b| b.to_string()),
                    json_escape(&o.sim_line),
                    o.wall_s
                )
            })
            .collect();
        let receipt = format!(
            "{{\n\"verdict\":\"{}\",\n\"refusals\":[{}],\n\"claim\":\"{}\",\n\"env\":\"{}\",\n\"policy\":\"{}\",\n\"horizon\":{},\n\"min_return\":{},\n\"claimed_bound\":{},\n\"aig_sha_match\":{},\n\"lanes\":[\n{}\n]\n}}\n",
            if verdict { "ACCEPT" } else { "REFUSE" },
            refusals
                .iter()
                .map(|r| format!("\"{}\"", json_escape(r)))
                .collect::<Vec<_>>()
                .join(","),
            json_escape(&claim_path.display().to_string()),
            json_escape(&claim.env),
            json_escape(&claim.policy_name),
            claim.horizon,
            min_ret,
            claim.claimed_bound,
            aig_ok.map_or("null".into(), |b| b.to_string()),
            lanes_json.join(",\n")
        );
        fs::write(&jp, receipt).map_err(|e| format!("{}: {e}", jp.display()))?;
        println!("receipt: {}", jp.display());
    }

    if ephemeral {
        let _ = fs::remove_dir_all(&workdir);
    }
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcd_decode_matches_ale_semantics() {
        assert_eq!(bcd2(0x00), 0);
        assert_eq!(bcd2(0x42), 42);
        assert_eq!(bcd2(0x99), 99);
        let mut ram = [0u8; RAM];
        ram[19] = 0x01; // higher pair: 1 * 10000
        ram[20] = 0x23;
        ram[21] = 0x45;
        assert_eq!(decode_score("montezuma", &ram).unwrap(), 12345);
        ram[13] = 3;
        ram[14] = 7;
        assert_eq!(decode_score("pong", &ram).unwrap(), 4);
        assert_eq!(clamp("freeway", 5), 1);
        assert_eq!(clamp("freeway", -2), 0);
        assert_eq!(clamp("montezuma", 400), 400);
    }

    #[test]
    fn atrc_roundtrip() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ATRC");
        data.push(1);
        data.push(ATRC_REC as u8);
        data.extend_from_slice(&[0u8; 10]);
        for i in 0..3u8 {
            let mut ram = [0u8; RAM];
            ram[0] = i;
            data.extend_from_slice(&ram);
            data.extend_from_slice(&(i as i32 - 1).to_le_bytes());
        }
        let (frames, rewards) = parse_atrc(&data).unwrap();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[2][0], 2);
        assert_eq!(rewards, vec![-1, 0, 1]);
        assert!(parse_atrc(&data[..data.len() - 1]).is_err()); // truncated record
        assert!(parse_atrc(b"XTRC............").is_err()); // bad magic
    }

    #[test]
    fn rom_to_hex_sizes() {
        let dir = std::env::temp_dir().join(format!("simreplay-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let rom2k = dir.join("r2k.bin");
        fs::write(&rom2k, vec![0xABu8; 2048]).unwrap();
        let hex = dir.join("r2k.hex");
        rom_to_hex(&rom2k, &hex).unwrap();
        let lines: Vec<String> = fs::read_to_string(&hex)
            .unwrap()
            .lines()
            .map(String::from)
            .collect();
        assert_eq!(lines.len(), 4096); // 2K doubled: mirror baked in
        assert!(lines.iter().all(|l| l == "ab"));
        let rom_bad = dir.join("bad.bin");
        fs::write(&rom_bad, vec![0u8; 3000]).unwrap();
        assert!(rom_to_hex(&rom_bad, &dir.join("bad.hex")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("simreplay: {e}");
            ExitCode::from(2)
        }
    }
}
