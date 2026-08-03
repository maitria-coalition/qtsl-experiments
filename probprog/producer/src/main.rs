//! CLI: evaluate one program at one horizon and print the certificate
//! ingredients (exact rationals) plus decimal renderings. The sealed
//! container emission lives behind the `emit` subcommand once the
//! species' wire lane is wired; `eval` is the timed producer core.

use probprog_producer::*;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: probprog-produce <eval|spec> <geo-counter|asym-rw|die-paradox|burglar-alarm|beauquier-3> <horizon> [--classes N]\n  eval: print brackets + producer wall\n  spec: print the certificate spec JSON (for the emitter binary) on stdout";
    if args.len() < 4 || (args[1] != "eval" && args[1] != "spec") {
        eprintln!("{usage}");
        std::process::exit(2);
    }
    let horizon: u32 = args[3].parse().expect("horizon must be a number");
    let n_classes: u32 = args
        .iter()
        .position(|a| a == "--classes")
        .map(|i| args[i + 1].parse().expect("--classes N"))
        .unwrap_or(10);

    let chain: Box<dyn Chain> = match args[2].as_str() {
        "geo-counter" => Box::new(GeoCounter),
        "asym-rw" => Box::new(AsymRw::default()),
        "die-paradox" => Box::new(DieParadox),
        "burglar-alarm" => Box::new(BurglarAlarm),
        "beauquier-3" => Box::new(Beauquier3),
        other => {
            eprintln!("unknown program {other}; {usage}");
            std::process::exit(2);
        }
    };

    if args[1] == "spec" {
        let t0 = Instant::now();
        let json = probprog_producer::spec::emit_spec(chain.as_ref(), horizon);
        let dt = t0.elapsed();
        print!("{json}");
        eprintln!("spec_wall_s={:.6}", dt.as_secs_f64());
        return;
    }

    let t0 = Instant::now();
    let tr = evaluate(chain.as_ref(), horizon);
    let dt = t0.elapsed();

    println!(
        "program={} horizon={} classes_absorbed={} residual={} ({:.3e})",
        chain.name(),
        horizon,
        tr.class_mass.len(),
        tr.residual,
        rat_f64(&tr.residual),
    );
    println!("evidence={} reject={}", tr.evidence(), tr.reject_mass);
    for c in 0..n_classes {
        if let Some((lo, hi)) = tr.posterior_bracket(c) {
            println!(
                "class {c}: posterior [{}, {}] ~ [{:.9}, {:.9}] width {:.3e}",
                lo,
                hi,
                rat_f64(&lo),
                rat_f64(&hi),
                rat_f64(&(hi.clone() - lo.clone())),
            );
        }
    }
    println!("producer_wall_s={:.6}", dt.as_secs_f64());
}

fn rat_f64(x: &Q) -> f64 {
    use num_traits::ToPrimitive;
    x.to_f64().unwrap_or(f64::NAN)
}
