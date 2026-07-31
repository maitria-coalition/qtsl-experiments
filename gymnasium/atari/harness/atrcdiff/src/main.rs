//! differential comparator for .atrc traces.
//!
//! Format (little-endian): 16-byte header "ATRC" ver=1 rec=132, then
//! per-frame records of 128 RAM bytes + i32 reward.
//!
//! Reports: exact-frame match rate, first divergence (frame + addresses),
//! per-address mismatch counts, agreement excluding persistently-divergent
//! addresses, and reward-stream agreement. Human table on stdout; `--json`
//! emits a machine-readable summary beside it.

use std::env;
use std::fs;
use std::process::ExitCode;

const REC: usize = 132;
const RAM: usize = 128;

struct Trace {
    frames: Vec<[u8; RAM]>,
    rewards: Vec<i32>,
}

fn load(path: &str) -> Result<Trace, String> {
    let data = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if data.len() < 16 || &data[0..4] != b"ATRC" {
        return Err(format!("{path}: not an ATRC file"));
    }
    if data[4] != 1 || data[5] as usize != REC {
        return Err(format!("{path}: unsupported version/record size"));
    }
    let body = &data[16..];
    let n = body.len() / REC;
    if body.len() % REC != 0 {
        eprintln!(
            "warning: {path}: {} trailing bytes ignored",
            body.len() % REC
        );
    }
    let mut frames = Vec::with_capacity(n);
    let mut rewards = Vec::with_capacity(n);
    for i in 0..n {
        let rec = &body[i * REC..(i + 1) * REC];
        let mut ram = [0u8; RAM];
        ram.copy_from_slice(&rec[..RAM]);
        frames.push(ram);
        rewards.push(i32::from_le_bytes([rec[128], rec[129], rec[130], rec[131]]));
    }
    Ok(Trace { frames, rewards })
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut paths = Vec::new();
    let mut json_out: Option<String> = None;
    let mut labels = ("A".to_string(), "B".to_string());
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                i += 1;
                json_out = Some(args[i].clone());
            }
            "--labels" => {
                i += 1;
                let mut it = args[i].splitn(2, ',');
                labels = (
                    it.next().unwrap_or("A").to_string(),
                    it.next().unwrap_or("B").to_string(),
                );
            }
            p => paths.push(p.to_string()),
        }
        i += 1;
    }
    if paths.len() != 2 {
        eprintln!("usage: atrcdiff <a.atrc> <b.atrc> [--json out.json] [--labels A,B]");
        return ExitCode::from(2);
    }

    let (ta, tb) = match (load(&paths[0]), load(&paths[1])) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    let n = ta.frames.len().min(tb.frames.len());
    println!(
        "frames: {}={} {}={} compared={}",
        labels.0,
        ta.frames.len(),
        labels.1,
        tb.frames.len(),
        n
    );

    let mut exact = 0usize;
    let mut first_div: Option<usize> = None;
    let mut addr_miss = [0usize; RAM];
    let mut reward_miss = 0usize;
    let mut first_reward_div: Option<usize> = None;

    for f in 0..n {
        let (a, b) = (&ta.frames[f], &tb.frames[f]);
        let mut same = true;
        for k in 0..RAM {
            if a[k] != b[k] {
                addr_miss[k] += 1;
                same = false;
            }
        }
        if same {
            exact += 1;
        } else if first_div.is_none() {
            first_div = Some(f);
        }
        if ta.rewards[f] != tb.rewards[f] {
            reward_miss += 1;
            if first_reward_div.is_none() {
                first_reward_div = Some(f);
            }
        }
    }

    let exact_rate = exact as f64 / n.max(1) as f64;
    println!(
        "exact-frame matches: {exact}/{n} ({:.2}%)",
        100.0 * exact_rate
    );

    match first_div {
        None => println!("first divergence: NONE (traces identical over compared range)"),
        Some(f) => {
            println!("first divergence: frame {f}");
            let (a, b) = (&ta.frames[f], &tb.frames[f]);
            let diffs: Vec<String> = (0..RAM)
                .filter(|&k| a[k] != b[k])
                .map(|k| {
                    format!(
                        "ram[{k}]: {}={:#04x} {}={:#04x}",
                        labels.0, a[k], labels.1, b[k]
                    )
                })
                .collect();
            for d in diffs.iter().take(16) {
                println!("  {d}");
            }
            if diffs.len() > 16 {
                println!("  ... and {} more addresses", diffs.len() - 16);
            }
        }
    }

    // per-address profile
    let mut ranked: Vec<(usize, usize)> = addr_miss
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(k, &c)| (k, c))
        .collect();
    ranked.sort_by(|x, y| y.1.cmp(&x.1));
    println!("divergent addresses: {}/{RAM}", ranked.len());
    for (k, c) in ranked.iter().take(20) {
        println!(
            "  ram[{k:3}]: {c} frames ({:.1}%)",
            100.0 * *c as f64 / n as f64
        );
    }
    if ranked.len() > 20 {
        println!("  ... and {} more", ranked.len() - 20);
    }

    // agreement excluding persistently-divergent addresses (>50% of frames)
    let mask: Vec<usize> = (0..RAM).filter(|&k| addr_miss[k] * 2 > n).collect();
    if !mask.is_empty() {
        let mut masked_exact = 0usize;
        for f in 0..n {
            let (a, b) = (&ta.frames[f], &tb.frames[f]);
            if (0..RAM).all(|k| mask.contains(&k) || a[k] == b[k]) {
                masked_exact += 1;
            }
        }
        println!(
            "agreement excluding {} persistently-divergent addrs {:?}: {}/{} ({:.2}%)",
            mask.len(),
            mask,
            masked_exact,
            n,
            100.0 * masked_exact as f64 / n as f64
        );
    }

    println!(
        "reward stream: {} mismatched frames ({}), first at {:?}; totals {}={} {}={}",
        reward_miss,
        if reward_miss == 0 {
            "IDENTICAL"
        } else {
            "DIVERGENT"
        },
        first_reward_div,
        labels.0,
        ta.rewards[..n].iter().sum::<i32>(),
        labels.1,
        tb.rewards[..n].iter().sum::<i32>()
    );

    if let Some(jp) = json_out {
        let ranked_json: Vec<String> = ranked
            .iter()
            .map(|(k, c)| format!("{{\"addr\":{k},\"frames\":{c}}}"))
            .collect();
        let j = format!(
            "{{\"frames_a\":{},\"frames_b\":{},\"compared\":{},\"exact\":{},\"exact_rate\":{:.6},\"first_divergence\":{},\"divergent_addrs\":[{}],\"reward_mismatch_frames\":{},\"reward_total_a\":{},\"reward_total_b\":{}}}",
            ta.frames.len(),
            tb.frames.len(),
            n,
            exact,
            exact_rate,
            first_div.map(|f| f.to_string()).unwrap_or("null".into()),
            ranked_json.join(","),
            reward_miss,
            ta.rewards[..n].iter().sum::<i32>(),
            tb.rewards[..n].iter().sum::<i32>()
        );
        if let Err(e) = fs::write(&jp, j) {
            eprintln!("warning: json write failed: {e}");
        }
    }

    ExitCode::SUCCESS
}
