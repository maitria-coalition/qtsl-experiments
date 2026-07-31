//! Proptest invariants for the deterministic Mini-Fab core.
//! (The event-time monotonicity invariant is a debug_assert inside the
//! event loop — these tests run debug, so every run below checks it.)

use minifab_sim::{run, Config, TICKS_PER_WEEK};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Lot conservation: released = completed + in-system, for any mix
    /// and horizon.
    #[test]
    fn conservation(
        t in 0u64..6, a in 0u64..70, b in 0u64..45, weeks in 1u64..6
    ) {
        prop_assume!(t + a + b > 0);
        let r = run(&Config { per_week: (t, a, b), horizon: weeks * TICKS_PER_WEEK });
        prop_assert_eq!(r.released, r.done + r.end_wip);
    }

    /// Determinism: identical configs give identical reports (the core
    /// has no RNG; this pins the W3 tie-break as reproducible).
    #[test]
    fn deterministic(a in 0u64..70, b in 0u64..45) {
        prop_assume!(a + b > 0);
        let cfg = || Config { per_week: (3, a, b), horizon: 3 * TICKS_PER_WEEK };
        let r1 = run(&cfg());
        let r2 = run(&cfg());
        prop_assert_eq!(format!("{r1:?}"), format!("{r2:?}"));
    }

    /// Machine occupancy never exceeds wall clock (no double-booking).
    #[test]
    fn occupancy_bounded(a in 0u64..70, b in 0u64..45, weeks in 1u64..5) {
        prop_assume!(a + b > 0);
        let r = run(&Config { per_week: (3, a, b), horizon: weeks * TICKS_PER_WEEK });
        for (_, _, su, ha, pr) in &r.machine_kpis {
            prop_assert!(su + ha + pr <= r.horizon);
        }
    }
}

/// Kempf-mix regression pins (exact expectations from the capacity
/// arithmetic in ../KEMPF-SPEC.md; failure = the data tables or the
/// dispatch reading drifted).
#[test]
fn kempf_mix_stable_and_litho_bound() {
    let r = run(&Config {
        per_week: (3, 51, 30),
        horizon: 52 * TICKS_PER_WEEK,
    });
    // deterministic core is stable at 84/week: throughput tracks release
    let thru = r.throughput_per_week();
    assert!(thru > 82.0 && thru <= 84.5, "throughput {thru}");
    // WIP does not grow without bound
    assert!(r.end_wip < 60, "end wip {}", r.end_wip);
    // litho (single machine) is the bottleneck: occupied fraction near
    // 84*105/10080 = 87.5% plus setups, and strictly the max
    let mut occ: Vec<(String, f64)> = r
        .machine_kpis
        .iter()
        .map(|(st, i, su, ha, pr)| {
            (
                format!("{}{}", st.name(), i),
                (*su + *ha + *pr) as f64 / r.horizon as f64,
            )
        })
        .collect();
    occ.sort_by(|x, y| y.1.total_cmp(&x.1));
    assert!(occ[0].0.starts_with("lithography"), "bottleneck {occ:?}");
    // 87.5% handling+processing (capacity arithmetic) + FIFO's setup
    // burden (~8%; the thesis's optimized schedule gets this to ~3%)
    assert!(occ[0].1 > 0.85 && occ[0].1 < 0.97, "litho occ {}", occ[0].1);
}
