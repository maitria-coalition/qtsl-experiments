//! `probprog-evcheck` — the dev-oracle checker for EV-bracket
//! certificates. The generator-form seam sealed for the FINITE face
//! at theory revision event 8 (the dcc closure): those rows are
//! container content now and glc-check owns their verdicts. What
//! checks here is the 3-D walk's PATTERN face (finitely presented
//! infinite chain), descoped from that event to its own design leg —
//! plus the retained v0 finite-face checker as a library oracle.
//! Status, mathematics, and verdict classes: the
//! `probprog_producer::ev` and `ev3d` module docs. Until the pattern
//! face's own revision event, comparison rows checked here are
//! labeled as dev-oracle verdicts; the same chain's absorption rows
//! ride the fast-path checker via the sealed container, and the
//! `spec-sha256` line binds the two artifacts to the same stored
//! rows.
//!
//! usage: probprog-evcheck --spec <spec.json> <cert.evc>
//! exit: 0 ACCEPT · 3 FAIL (claim discharge) · 4 REFUSE (shape, side
//! condition, or binding) · 2 usage

use probprog_producer::ev::{check_evcert, Verdict};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: probprog-evcheck --spec <spec.json> <cert.evc>";
    let mut spec_path: Option<String> = None;
    let mut cert_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--spec" {
            if i + 1 >= args.len() || spec_path.is_some() {
                eprintln!("{usage}");
                std::process::exit(2);
            }
            spec_path = Some(args[i + 1].clone());
            i += 2;
        } else {
            if cert_path.is_some() {
                eprintln!("{usage}");
                std::process::exit(2);
            }
            cert_path = Some(args[i].clone());
            i += 1;
        }
    }
    let (spec_path, cert_path) = match (spec_path, cert_path) {
        (Some(s), Some(c)) => (s, c),
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };

    let spec = std::fs::read(&spec_path).unwrap_or_else(|e| {
        eprintln!("evcheck: cannot read {spec_path}: {e}");
        std::process::exit(2);
    });
    let cert = std::fs::read_to_string(&cert_path).unwrap_or_else(|e| {
        eprintln!("evcheck: cannot read {cert_path}: {e}");
        std::process::exit(2);
    });

    // dispatch on the certificate's own header line: v0 = finite
    // stored-row face, v1 = pattern/window face (ev3d)
    let verdict = if cert.starts_with("evcert v1") {
        probprog_producer::ev3d::check_evcert_3d(&spec, &cert)
    } else {
        check_evcert(&spec, &cert)
    };
    match verdict {
        Verdict::Accept { claims: (p, e, t) } => {
            println!(
                "{cert_path} ACCEPT claims partial={p} ev={e} tail={t} \
                 (dev-oracle, pattern face pending its own revision event)"
            );
        }
        Verdict::Fail(m) => {
            println!("{cert_path} FAIL {m}");
            std::process::exit(3);
        }
        Verdict::Refuse(m) => {
            println!("{cert_path} REFUSE {m}");
            std::process::exit(4);
        }
    }
}
