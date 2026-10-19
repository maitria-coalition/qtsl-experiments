//! The attainment receipt: decode the sealed carved Mini-Fab containers
//! and replay the pi* and work-conserving firing tables against them.
use minifab_glc_replay::{decode, replay::*};
use sha2::{Digest, Sha256};

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

fn run(path: &str, label: &str, lots: &[Lot], holds: &[&str]) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let c = decode::open(&bytes)?;
    let net = c.sealed_net(0)?;
    let rows = compile(&net)?;
    let n = lots.len() as u64;
    let o = replay(&net, &rows, &table_of(lots), n, holds, 300 * n + 2000)?;
    Ok(format!(
        "{{\"container\":\"{path}\",\"sha256\":\"{}\",\"mech\":\"{}\",\"label\":\"{label}\",\"n\":{},\"last_arrival_round\":{},\"pre_goal_firings\":{},\"holds_by_schema\":{:?},\"max_batch\":{}}}",
        hex(&Sha256::digest(&bytes)), net.mech, o.n, o.last_arrival_round, o.pre_goal_firings, o.holds_by_schema, o.max_batch
    ))
}

/// pi* at N = 1..12 on the supervised container, with the two sealed
/// N = 84 facts substituted (adapt_instance): "sealed net with n pending",
/// not the sealed bytes themselves.
fn run_instance(path: &str, n: u64) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let net = decode::open(&bytes)?.sealed_net(0)?;
    let (net, sealed_pend, sealed_unlock) = adapt_instance(&net, n)?;
    let rows = compile(&net)?;
    let lots = pi_star_lots(n);
    let o = replay(&net, &rows, &table_of(&lots), n, &["start1", "start5"], 300 * n + 2000)?;
    if o.last_arrival_round != t_opt(n) {
        return Err(format!("N={n}: pi* arrival {} != T_opt {}", o.last_arrival_round, t_opt(n)));
    }
    Ok(format!(
        "{{\"container\":\"{path}\",\"sha256\":\"{}\",\"mech\":\"{}\",\"label\":\"pi-star-instance\",\"instance_adaptation\":{{\"pending_lots\":[{sealed_pend},{n}],\"unlock_threshold\":[{},{n}],\"note\":\"sealed N=84 value -> substituted; all other sealed facts unchanged\"}},\"n\":{n},\"t_opt\":{},\"last_arrival_round\":{},\"pre_goal_firings\":{},\"holds_by_schema\":{:?},\"max_batch\":{}}}",
        hex(&Sha256::digest(&bytes)), net.mech, sealed_unlock.map_or("null".into(), |u| u.to_string()), t_opt(n), o.last_arrival_round, o.pre_goal_firings, o.holds_by_schema, o.max_batch
    ))
}

const USAGE: &str = "usage: glc-replay <carved-container-dir> | glc-replay --dump-tables\n  replays pi* / work-conserving tables against the decoded containers in <dir>\n  (N = 84 sealed instance, then pi* at N = 1..12 with the documented instance adaptation);\n  --dump-tables prints the canonical pi*/WC tables instead.";

fn usage_exit(msg: Option<String>) -> ! {
    if let Some(m) = msg { eprintln!("glc-replay: {m}"); }
    eprintln!("{USAGE}");
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = match args.as_slice() {
        [a] if a == "--dump-tables" => None,
        [d, a] if a == "--dump-tables" && !d.starts_with('-') => None,
        [a] if a == "-h" || a == "--help" => usage_exit(None),
        [d] if !d.starts_with('-') => Some(d.clone()),
        [] => usage_exit(None),
        other => usage_exit(Some(format!("unknown arguments {other:?}"))),
    };
    let Some(dir) = dir else {
        for n in [1u64, 2, 3, 4, 5, 6, 8, 12, 30, 84] { print!("{}", canonical("pistar", &pi_star_lots(n))); }
        for n in [3u64, 4, 5, 8, 84] { print!("{}", canonical("wcfamily", &wc_lots(n))); }
        return;
    };
    let sup = format!("{dir}/minifab-supervised.glc");
    let open = format!("{dir}/minifab.glc");
    let gated_native = format!("{dir}/minifab-gated.glc");
    let n = 84;
    // expected: None = must replay; Some(why) = must refuse, and the refusal
    // message must contain `why` (so a missing file or an unrelated error is
    // not counted as the intended semantic refusal).
    let cases: Vec<(&str, &str, Vec<Lot>, Vec<&str>, Option<&str>)> = vec![
        (sup.as_str(), "pi-star", pi_star_lots(n), vec!["start1", "start5"], None),
        (sup.as_str(), "wc-family", wc_lots(n), vec![], None),
        (open.as_str(), "pi-star", pi_star_lots(n), vec!["start1", "start5", "start6"], None),
        (open.as_str(), "wc-family", wc_lots(n), vec!["start6"], None),
        // must REFUSE: pi* is not work-conserving (a hold outside the allowed set)
        (sup.as_str(), "pi-star-under-forced-law", pi_star_lots(n), vec![], Some("hold outside")),
        // must REFUSE: the native-read gated lane is not this replay's lane
        (gated_native.as_str(), "pi-star", pi_star_lots(n), vec!["start1", "start5"], Some("native read arcs")),
    ];
    let mut ok = true;
    for (p, label, lots, holds, expect) in cases {
        match (run(p, label, &lots, &holds), expect) {
            (Ok(s), None) => println!("{s}"),
            (Err(e), Some(why)) if e.contains(why) => {
                let sha = std::fs::read(p).map(|b| hex(&Sha256::digest(&b))).unwrap_or_default();
                println!("{{\"container\":\"{p}\",\"refused_sha256\":\"{sha}\",\"label\":\"{label}\",\"refused_as_expected\":{e:?}}}")
            }
            (Err(e), Some(why)) => { ok = false; println!("WRONG REFUSAL {p} {label}: wanted {why:?}, got {e}") }
            (Ok(s), Some(_)) => { ok = false; println!("UNEXPECTED ACCEPT {s}") }
            (Err(e), None) => { ok = false; println!("UNEXPECTED REFUSAL {p} {label}: {e}") }
        }
    }
    for n in 1..=12u64 {
        match run_instance(&sup, n) {
            Ok(s) => println!("{s}"),
            Err(e) => { ok = false; println!("UNEXPECTED REFUSAL {sup} pi-star-instance N={n}: {e}") }
        }
    }
    if !ok { std::process::exit(1) }
}
