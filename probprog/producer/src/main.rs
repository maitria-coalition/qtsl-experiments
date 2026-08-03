//! CLI: evaluate one program at one horizon and print the certificate
//! ingredients (exact rationals) plus decimal renderings. The sealed
//! container emission lives behind the `emit` subcommand once the
//! species' wire lane is wired; `eval` is the timed producer core.

use probprog_producer::*;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: probprog-produce <eval|spec|evcert> <geo-counter|asym-rw|die-paradox|burglar-alarm|beauquier-3|coupon-collector5|israeli-jalfon4|3d-asym-rw> <horizon> [--classes N] [--spec-file F]\n  eval: print brackets + producer wall\n  spec: print the certificate spec JSON (for the emitter binary) on stdout\n  evcert: print the EV-bracket certificate (dev-oracle lane; requires --spec-file, the exact spec bytes the container was sealed from)";
    if args.len() < 4 || (args[1] != "eval" && args[1] != "spec" && args[1] != "evcert") {
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
        "coupon-collector5" => Box::new(CouponCollector5),
        "israeli-jalfon4" => Box::new(IsraeliJalfon4),
        "3d-asym-rw" => Box::new(AsymRw3d),
        other => {
            eprintln!("unknown program {other}; {usage}");
            std::process::exit(2);
        }
    };

    if args[1] == "spec" {
        let t0 = Instant::now();
        // the 3d walk's alive set is infinite: its spec is the stored
        // WINDOW (rows to max coordinate box+1); everything else takes
        // the reachable-closure emitter
        let json = if args[2] == "3d-asym-rw" {
            probprog_producer::spec::emit_spec_3d_window(
                horizon,
                probprog_producer::ev3d::EV3D_BOX + 1,
            )
        } else {
            probprog_producer::spec::emit_spec(chain.as_ref(), horizon)
        };
        let dt = t0.elapsed();
        print!("{json}");
        eprintln!("spec_wall_s={:.6}", dt.as_secs_f64());
        return;
    }

    if args[1] == "evcert" {
        let spec_path = args
            .iter()
            .position(|a| a == "--spec-file")
            .map(|i| args[i + 1].clone())
            .unwrap_or_else(|| {
                eprintln!("evcert requires --spec-file <sealed spec.json>; {usage}");
                std::process::exit(2);
            });
        let spec_bytes = std::fs::read(&spec_path).unwrap_or_else(|e| {
            eprintln!("evcert: cannot read {spec_path}: {e}");
            std::process::exit(2);
        });
        let t0 = Instant::now();
        let cert = if args[2] == "3d-asym-rw" {
            probprog_producer::ev3d::emit_evcert_3d(horizon, &spec_bytes)
        } else {
            probprog_producer::ev::emit_evcert(chain.as_ref(), horizon, &spec_bytes)
        };
        let dt = t0.elapsed();
        print!("{cert}");
        eprintln!("evcert_wall_s={:.6}", dt.as_secs_f64());
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
