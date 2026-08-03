//! Monte-Carlo translation differential: simulate the SOURCE program's
//! published semantics directly (not the chain encoding) and compare
//! empirical terminal-class frequencies against the chain evaluation's
//! exact masses. Agreement bands are 4-sigma binomial around the exact
//! (or, for classes beyond the horizon, bracketed) masses. This
//! discharges the translation obligation's statistical leg; the exact
//! closed-form containment tests discharge the analytic leg.
//!
//! Determinism: a fixed-seed SplitMix64 generator — reproducible runs,
//! no external dependency; statistical quality is ample for frequency
//! agreement at 10^6 paths.

use crate::{Truncated, Q};
use num_traits::ToPrimitive;
use std::collections::BTreeMap;

pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    /// Uniform in 0..n by rejection (unbiased).
    pub fn below(&mut self, n: u64) -> u64 {
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let v = self.next_u64();
            if v < zone {
                return v % n;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    Class(u32),
    Rejected,
    AliveAtCap,
}

/// One simulated run of a source program's published semantics.
pub type Simulator = fn(&mut SplitMix64, u32) -> Outcome;

/// Geometric counter, source semantics: X := 0; while flip(1/2) X += 1.
pub fn sim_geo_counter(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let mut x = 0u32;
    for _ in 0..cap {
        if rng.below(2) == 0 {
            return Outcome::Class(x);
        }
        x += 1;
    }
    Outcome::AliveAtCap
}

/// Biased walk, source semantics: pos := 1; steps := 0;
/// while pos > 0 { steps += 1; pos += 1 w.p. 1/4 else pos -= 1 }.
pub fn sim_asym_rw(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let mut pos = 1u64;
    let mut steps = 0u32;
    while pos > 0 {
        if steps >= cap {
            return Outcome::AliveAtCap;
        }
        steps += 1;
        if rng.below(4) == 0 {
            pos += 1;
        } else {
            pos -= 1;
        }
    }
    Outcome::Class(steps)
}

/// Die paradox, source semantics with rejection sampling for the hard
/// observation: throw a fair die until 6, observing every non-final
/// throw even; an odd throw rejects the whole run.
pub fn sim_die_paradox(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let mut throws = 0u32;
    loop {
        if throws >= cap {
            return Outcome::AliveAtCap;
        }
        throws += 1;
        let d = rng.below(6) + 1;
        if d == 6 {
            return Outcome::Class(throws);
        }
        if d % 2 == 1 {
            return Outcome::Rejected;
        }
    }
}

/// burglarAlarm, source semantics (the vendored PSI/GuBPI program):
/// four Bernoulli draws (exact integer draws, equivalent to the
/// source's `uniform(0,1) <= p` flips), observe(called) as rejection.
/// Always terminates within the cap (finite program; cap >= 4 by the
/// shared-horizon protocol).
pub fn sim_burglar_alarm(rng: &mut SplitMix64, _cap: u32) -> Outcome {
    let e = rng.below(10000) < 1;
    let b = rng.below(1000) < 1;
    let alarm = e || b;
    let pw = if e {
        rng.below(10) < 7
    } else {
        rng.below(100) < 99
    };
    let mw = if alarm {
        if e {
            rng.below(10) < 8
        } else {
            rng.below(10) < 6
        }
    } else {
        rng.below(10) < 2
    };
    if mw && pw {
        Outcome::Class(if b { 1 } else { 0 })
    } else {
        Outcome::Rejected
    }
}

/// beauquier-etal-3 Q1, source semantics: a direct transcription of the
/// vendored `.spcf` (sequential lets over OLD values, one flip per
/// enabled probabilistic update — `uniform(0,1) <= 0.5` simulated as an
/// exact fair bit). Cap counts CHAIN transitions: the initial
/// six-bit sample is transition 1 (p-sum <= 1 there is count = 0,
/// class 0); each synchronous round is one further transition (a round
/// landing at p-sum <= 1 is count >= 1, class 1).
pub fn sim_beauquier3(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let mut used = 0u32;
    if used >= cap {
        return Outcome::AliveAtCap;
    }
    let mut d = [0u8; 3];
    let mut p = [0u8; 3];
    for i in 0..3 {
        d[i] = rng.below(2) as u8;
        p[i] = rng.below(2) as u8;
    }
    used += 1;
    if p[0] + p[1] + p[2] <= 1 {
        return Outcome::Class(0);
    }
    loop {
        if used >= cap {
            return Outcome::AliveAtCap;
        }
        // one synchronous round, guards on OLD values; predecessor of
        // process 1 is process 3 (the .spcf's comparison pairs)
        let (od, op) = (d, p);
        for i in 0..3 {
            let j = (i + 2) % 3; // predecessor in the ring
            if od[i] == od[j] {
                d[i] = 1 - od[i];
                if op[i] == op[j] {
                    // keep w.p. 1/2 else flip == fresh fair bit
                    p[i] = if rng.below(2) == 0 { op[i] } else { 1 - op[i] };
                }
            }
        }
        used += 1;
        if p[0] + p[1] + p[2] <= 1 {
            return Outcome::Class(1);
        }
    }
}

/// coupon-collector5, source semantics: five explicit coupon flags
/// (no exchangeability lump on this side — that is the point), one
/// uniform draw among five per iteration, count incremented every
/// iteration, loop while any flag is unset. Returns the count as the
/// class, so the absorption-TIME differential can compare the full
/// return distribution against the chain DP's survival increments.
pub fn sim_coupon_collector5(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let mut have = [false; 5];
    let mut count = 0u32;
    while !(have[0] && have[1] && have[2] && have[3] && have[4]) {
        if count >= cap {
            return Outcome::AliveAtCap;
        }
        have[rng.below(5) as usize] = true;
        count += 1;
    }
    Outcome::Class(count)
}

/// israeli-jalfon4, source semantics: a direct transcription of the
/// vendored `.sgcl` — four named flags, one loop iteration snapshots
/// them and walks the four if-blocks IN ORDER (guards on the
/// snapshot, assignments on current state, fresh fair flip per firing
/// block; heads sends process i's token to its predecessor, tails to
/// its successor, per the source's branch arms). Cap counts CHAIN
/// transitions: the initial four-bit sample is transition 1 (token
/// count <= 1 there is count = 0), each loop iteration one further
/// transition — so the returned Class is the chain absorption time
/// T = count + 1 and the absorption-time differential compares it
/// against the chain DP's survival increments directly.
pub fn sim_israeli_jalfon4(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let mut used = 0u32;
    if used >= cap {
        return Outcome::AliveAtCap;
    }
    let mut p = [0u8; 4];
    for f in p.iter_mut() {
        *f = rng.below(2) as u8;
    }
    used += 1;
    loop {
        if p.iter().map(|&b| b as u32).sum::<u32>() <= 1 {
            return Outcome::Class(used);
        }
        if used >= cap {
            return Outcome::AliveAtCap;
        }
        let old = p;
        for i in 0..4usize {
            if old[i] == 1 {
                p[i] = 0;
                if rng.below(2) == 1 {
                    p[(i + 3) % 4] = 1; // predecessor (process1 -> process4)
                } else {
                    p[(i + 1) % 4] = 1; // successor (process1 -> process2)
                }
            }
        }
        used += 1;
    }
}

/// 3d-asym-rw, source semantics: a LITERAL transcription of the
/// vendored SGCL text — `if flip(1/3) {x} else if flip(1/2) {y} else
/// {z}` as two exact Bernoulli draws, the move as `flip(1/4)` up else
/// down, decrement with MONUS (SGCL naturals: the vendored tool's own
/// decrement saturates at zero), loop while any coordinate is
/// nonzero, count every iteration. Returns the count (= absorption
/// time) as the class, for the absorption-time differential.
pub fn sim_asym_rw_3d(rng: &mut SplitMix64, cap: u32) -> Outcome {
    let (mut x, mut y, mut z) = (1u64, 1u64, 1u64);
    let mut n = 0u32;
    while x != 0 || y != 0 || z != 0 {
        if n >= cap {
            return Outcome::AliveAtCap;
        }
        let axis: &mut u64 = if rng.below(3) < 1 {
            &mut x
        } else if rng.below(2) < 1 {
            &mut y
        } else {
            &mut z
        };
        if rng.below(4) < 1 {
            *axis += 1;
        } else {
            *axis = axis.saturating_sub(1);
        }
        n += 1;
    }
    Outcome::Class(n)
}

pub struct DiffReport {
    pub paths: u64,
    pub worst_sigma: f64,
    pub rows: Vec<(String, f64, f64, f64)>, // label, expected, observed, sigma
}

/// Compare empirical frequencies against the exact truncated masses.
/// Classes are compared unnormalized (frequency among ALL runs vs the
/// exact chain mass); rejection and residual likewise. All comparisons
/// use the same horizon for simulator cap and chain evaluation, so the
/// AliveAtCap frequency matches the residual exactly in expectation.
pub fn differential(sim: Simulator, tr: &Truncated, paths: u64, seed: u64) -> DiffReport {
    let mut rng = SplitMix64(seed);
    let mut counts: BTreeMap<Outcome, u64> = BTreeMap::new();
    for _ in 0..paths {
        *counts.entry(sim(&mut rng, tr.horizon)).or_insert(0) += 1;
    }
    let n = paths as f64;
    let mut rows = Vec::new();
    let mut worst: f64 = 0.0;
    let mut push = |label: String, p: f64, k: u64| {
        let obs = k as f64 / n;
        let sd = (p * (1.0 - p) / n).sqrt();
        let sigma = if sd > 0.0 {
            (obs - p).abs() / sd
        } else if obs == p {
            0.0
        } else {
            f64::INFINITY
        };
        if sigma > worst {
            worst = sigma;
        }
        rows.push((label, p, obs, sigma));
    };
    for (c, mass) in &tr.class_mass {
        let p = qf(mass);
        // only compare classes with enough expected hits for the
        // normal band to make sense
        if p * n >= 16.0 {
            let k = counts.get(&Outcome::Class(*c)).copied().unwrap_or(0);
            push(format!("class {c}"), p, k);
        }
    }
    let pr = qf(&tr.reject_mass);
    if pr * n >= 16.0 {
        push(
            "rejected".into(),
            pr,
            counts.get(&Outcome::Rejected).copied().unwrap_or(0),
        );
    }
    let pa = qf(&tr.residual);
    if pa * n >= 16.0 {
        push(
            "alive-at-cap".into(),
            pa,
            counts.get(&Outcome::AliveAtCap).copied().unwrap_or(0),
        );
    }
    DiffReport {
        paths,
        worst_sigma: worst,
        rows,
    }
}

/// Absorption-time differential: compare empirical frequencies of the
/// simulator's Class(n) — the source program's return value, where
/// that value IS the absorption time — against the chain DP's exact
/// survival increments P[T = n] = P[T > n-1] - P[T > n]. The
/// class-mass differential above is vacuous for single-class chains
/// (everything absorbs into class 0); this is the translation
/// instrument for the EV-query cases. Same 4-sigma bands, same
/// minimum-expected-hits guard, AliveAtCap compared to the survival
/// mass at the cap.
pub fn differential_times(sim: Simulator, survivals: &[Q], paths: u64, seed: u64) -> DiffReport {
    let horizon = (survivals.len() - 1) as u32;
    let mut rng = SplitMix64(seed);
    let mut counts: BTreeMap<Outcome, u64> = BTreeMap::new();
    for _ in 0..paths {
        *counts.entry(sim(&mut rng, horizon)).or_insert(0) += 1;
    }
    let n = paths as f64;
    let mut rows = Vec::new();
    let mut worst: f64 = 0.0;
    let mut push = |label: String, p: f64, k: u64| {
        let obs = k as f64 / n;
        let sd = (p * (1.0 - p) / n).sqrt();
        let sigma = if sd > 0.0 {
            (obs - p).abs() / sd
        } else if obs == p {
            0.0
        } else {
            f64::INFINITY
        };
        if sigma > worst {
            worst = sigma;
        }
        rows.push((label, p, obs, sigma));
    };
    for t in 1..=horizon as usize {
        let p = qf(&(survivals[t - 1].clone() - survivals[t].clone()));
        if p * n >= 16.0 {
            let k = counts.get(&Outcome::Class(t as u32)).copied().unwrap_or(0);
            push(format!("time {t}"), p, k);
        }
    }
    let pa = qf(&survivals[horizon as usize]);
    if pa * n >= 16.0 {
        push(
            "alive-at-cap".into(),
            pa,
            counts.get(&Outcome::AliveAtCap).copied().unwrap_or(0),
        );
    }
    DiffReport {
        paths,
        worst_sigma: worst,
        rows,
    }
}

fn qf(x: &Q) -> f64 {
    x.to_f64().unwrap_or(f64::NAN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn run(sim: Simulator, chain: &dyn Chain, horizon: u32) -> DiffReport {
        let tr = evaluate(chain, horizon);
        differential(sim, &tr, 1_000_000, 0xD1CE_5EED)
    }

    #[test]
    fn geo_counter_differential() {
        let r = run(sim_geo_counter, &GeoCounter, 30);
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn asym_rw_differential() {
        let r = run(sim_asym_rw, &AsymRw::default(), 70);
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn burglar_alarm_differential() {
        let r = run(sim_burglar_alarm, &BurglarAlarm, 4);
        // class 1 expects ~594 hits at 1e6 paths — inside the >=16 gate
        assert!(
            r.rows.iter().any(|(l, ..)| l == "class 1"),
            "class 1 must be compared, rows {:?}",
            r.rows
        );
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn die_paradox_differential() {
        let r = run(sim_die_paradox, &DieParadox, 40);
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn beauquier3_differential() {
        // matched-depth horizon; class 0 (= exactly 1/2), class 1 and
        // the residual are all compared (residual at 51 is far below
        // the >=16 gate, so effectively classes 0 and 1)
        let r = run(sim_beauquier3, &Beauquier3, 51);
        assert!(
            r.rows.iter().any(|(l, ..)| l == "class 0")
                && r.rows.iter().any(|(l, ..)| l == "class 1"),
            "both classes must be compared, rows {:?}",
            r.rows
        );
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    /// The cross-program tooth (the battery's x1 locus): a chain
    /// rebuilt self-consistently for a VARIANT protocol — the
    /// realistic transcription slip of reading NEW neighbor values in
    /// the guards (sequential update) where the source reads OLD ones
    /// (synchronous) — emits an internally-green certificate; the
    /// translation differential is the instrument that refuses it.
    struct Beauquier3SeqSlip;

    impl Chain for Beauquier3SeqSlip {
        fn init(&self) -> u64 {
            BQ3_START
        }
        fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
            if x == BQ3_START {
                return Beauquier3.succ(x, 0);
            }
            // sequential slip: process i's guards read the ALREADY
            // UPDATED predecessor bits
            let mut out: Vec<(u64, Q)> = vec![(x, Q::one())];
            // process 1 vs OLD process 3 (first in source order — same
            // either way), then 2 vs NEW 1, then 3 vs NEW 2.
            let steps: [(usize, usize); 3] = [(0, 2), (1, 0), (2, 1)];
            for (i, j) in steps {
                let mut next: Vec<(u64, Q)> = Vec::new();
                for (cfg, w) in out {
                    let db = |c: u64, k: usize| (c >> (5 - 2 * k)) & 1;
                    let pb = |c: u64, k: usize| (c >> (4 - 2 * k)) & 1;
                    let (di, dj) = (db(cfg, i), db(cfg, j));
                    let (pi, pj) = (pb(cfg, i), pb(cfg, j));
                    if di == dj {
                        let flipped = cfg ^ (1 << (5 - 2 * i));
                        if pi == pj {
                            let a = flipped & !(1 << (4 - 2 * i));
                            let b = flipped | (1 << (4 - 2 * i));
                            next.push((a, w.clone() * q(1, 2)));
                            next.push((b, w * q(1, 2)));
                        } else {
                            next.push((flipped, w));
                        }
                    } else {
                        next.push((cfg, w));
                    }
                }
                out = next;
            }
            out.into_iter()
                .map(|(cfg, w)| {
                    let psum = ((cfg >> 4) & 1) + ((cfg >> 2) & 1) + (cfg & 1);
                    let dst = if psum <= 1 {
                        Dst::Done(1)
                    } else {
                        Dst::Alive(cfg)
                    };
                    (dst, w)
                })
                .collect()
        }
        fn name(&self) -> &'static str {
            "beauquier-3-seq-slip"
        }
        fn time_homogeneous(&self) -> bool {
            true
        }
    }

    #[test]
    fn asym_rw_3d_time_differential() {
        // horizon 100 keeps the alive-at-cap row inside the >=16 gate
        // while comparing the whole bulk of the return distribution
        let surv = survival_masses(&AsymRw3d, 100);
        let r = differential_times(sim_asym_rw_3d, &surv, 1_000_000, 0xD1CE_5EED);
        assert!(
            r.rows.iter().any(|(l, ..)| l == "time 3")
                && r.rows.iter().any(|(l, ..)| l == "time 20"),
            "return-distribution rows must be compared, rows {:?}",
            r.rows
        );
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    /// The cross-program tooth for the 3-D walk: the realistic
    /// transcription slip of reading the loop guard conjunctively
    /// (absorb as soon as ANY coordinate hits zero). The variant chain
    /// is internally green — probability rows, clean DP, and it even
    /// admits a certificate of its own — the translation differential
    /// is the instrument that refuses it against the source.
    struct AsymRw3dGuardSlip;

    impl Chain for AsymRw3dGuardSlip {
        fn init(&self) -> u64 {
            crate::ev3d::t3_pack(1, 1, 1)
        }
        fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
            let (alive, absorbed) = crate::ev3d::pattern_row(crate::ev3d::t3_unpack(x), 1, 12);
            let mut out: Vec<(Dst, Q)> = Vec::new();
            let mut done = absorbed;
            for (d, w) in alive {
                let t = crate::ev3d::t3_unpack(d);
                if t.0 == 0 {
                    // any zero coordinate absorbs under the slipped guard
                    done += w;
                } else {
                    out.push((Dst::Alive(d), q(w as i64, 12)));
                }
            }
            if done > 0 {
                out.push((Dst::Done(0), q(done as i64, 12)));
            }
            out
        }
        fn name(&self) -> &'static str {
            "3d-asym-rw-guard-slip"
        }
        fn time_homogeneous(&self) -> bool {
            true
        }
    }

    #[test]
    fn asym_rw_3d_cross_program_refused_by_differential() {
        let surv = survival_masses(&AsymRw3dGuardSlip, 100);
        let r = differential_times(sim_asym_rw_3d, &surv, 1_000_000, 0xD1CE_5EED);
        assert!(
            r.worst_sigma > 4.0,
            "the slipped chain must be REFUSED by the translation \
             differential; worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn coupon_collector5_time_differential() {
        let surv = survival_masses(&CouponCollector5, 80);
        let r = differential_times(sim_coupon_collector5, &surv, 1_000_000, 0xD1CE_5EED);
        // the bulk of the return distribution sits well inside the
        // >=16-hits gate: times 5..~40 are all compared
        assert!(
            r.rows.iter().any(|(l, ..)| l == "time 5")
                && r.rows.iter().any(|(l, ..)| l == "time 20"),
            "return-distribution rows must be compared, rows {:?}",
            r.rows
        );
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    /// The cross-program tooth for the collector: the realistic
    /// transcription slip of absorbing one coupon early (a loop-guard
    /// misreading — four distinct coupons already "collects them
    /// all"). The variant chain is internally green (probability rows,
    /// clean DP); the absorption-time differential is the instrument
    /// that refuses it.
    struct CouponCollector4Slip;

    impl Chain for CouponCollector4Slip {
        fn init(&self) -> u64 {
            0
        }
        fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
            let k = x as i64;
            let advance = if x == 3 {
                Dst::Done(0)
            } else {
                Dst::Alive(x + 1)
            };
            if x == 0 {
                vec![(advance, Q::one())]
            } else {
                vec![(advance, q(5 - k, 5)), (Dst::Alive(x), q(k, 5))]
            }
        }
        fn name(&self) -> &'static str {
            "coupon-collector4-slip"
        }
        fn time_homogeneous(&self) -> bool {
            true
        }
    }

    #[test]
    fn coupon_collector5_cross_program_refused_by_differential() {
        let surv = survival_masses(&CouponCollector4Slip, 80);
        let r = differential_times(sim_coupon_collector5, &surv, 1_000_000, 0xD1CE_5EED);
        assert!(
            r.worst_sigma > 4.0,
            "the slipped chain must be REFUSED by the translation \
             differential; worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn israeli_jalfon4_time_differential() {
        let surv = survival_masses(&IsraeliJalfon4, 31);
        let r = differential_times(sim_israeli_jalfon4, &surv, 1_000_000, 0xD1CE_5EED);
        // T = 1 (count 0, mass 5/16) through the geometric bulk are
        // all inside the >=16-hits gate
        assert!(
            r.rows.iter().any(|(l, ..)| l == "time 1")
                && r.rows.iter().any(|(l, ..)| l == "time 10"),
            "return-distribution rows must be compared, rows {:?}",
            r.rows
        );
        assert!(
            r.worst_sigma < 4.0,
            "worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    /// One-step direct-transcription oracle: for every alive
    /// configuration, walk the four if-blocks over NAMED variables
    /// (p1..p4, oldp1..oldp4 — no bit packing, no shared helper) for
    /// all coin assignments and compare the exact successor
    /// distribution against the chain rows.
    #[test]
    fn israeli_jalfon4_one_step_matches_direct_transcription() {
        type Cfg = (u8, u8, u8, u8);
        // successor targets per the source text's branch arms:
        // block1: heads p4 else p2; block2: heads p1 else p3;
        // block3: heads p2 else p4; block4: heads p3 else p1.
        let step_direct = |old: Cfg| -> Vec<(Cfg, Q)> {
            let (o1, o2, o3, o4) = old;
            let outs: Vec<(Cfg, Q)> = vec![(old, Q::one())];
            let apply = |outs: Vec<(Cfg, Q)>,
                         fires: bool,
                         clear: usize,
                         heads_dst: usize,
                         tails_dst: usize|
             -> Vec<(Cfg, Q)> {
                if !fires {
                    return outs;
                }
                outs.into_iter()
                    .flat_map(|(c, w)| {
                        let mut base = [c.0, c.1, c.2, c.3];
                        base[clear] = 0;
                        let mut h = base;
                        h[heads_dst] = 1;
                        let mut t = base;
                        t[tails_dst] = 1;
                        [
                            ((h[0], h[1], h[2], h[3]), w.clone() * q(1, 2)),
                            ((t[0], t[1], t[2], t[3]), w * q(1, 2)),
                        ]
                    })
                    .collect()
            };
            let outs = apply(outs, o1 == 1, 0, 3, 1);
            let outs = apply(outs, o2 == 1, 1, 0, 2);
            let outs = apply(outs, o3 == 1, 2, 1, 3);
            apply(outs, o4 == 1, 3, 2, 0)
        };
        for x in (0..16u64).filter(|&x| ij4_tokens(x) >= 2) {
            let old = (
                (x & 1) as u8,
                ((x >> 1) & 1) as u8,
                ((x >> 2) & 1) as u8,
                ((x >> 3) & 1) as u8,
            );
            let mut direct: BTreeMap<Option<u64>, Q> = BTreeMap::new();
            for (c, w) in step_direct(old) {
                let packed =
                    (c.0 as u64) | ((c.1 as u64) << 1) | ((c.2 as u64) << 2) | ((c.3 as u64) << 3);
                let key = if ij4_tokens(packed) <= 1 {
                    None
                } else {
                    Some(packed)
                };
                *direct.entry(key).or_insert_with(Q::zero) += w;
            }
            let mut chain: BTreeMap<Option<u64>, Q> = BTreeMap::new();
            for (dst, w) in IsraeliJalfon4.succ(x, 0) {
                let key = match dst {
                    Dst::Alive(y) => Some(y),
                    Dst::Done(0) => None,
                    other => panic!("unexpected dst {other:?}"),
                };
                *chain.entry(key).or_insert_with(Q::zero) += w;
            }
            assert_eq!(direct, chain, "one-step divergence at config {x:04b}");
        }
    }

    /// The cross-program tooth for the ring protocol: the realistic
    /// transcription slip of implementing FULLY-SYNCHRONOUS token
    /// movement (all destinations computed from the OLD state, new
    /// configuration = the union of destinations) where the source's
    /// sequential in-place blocks differ — e.g. from an adjacent pair,
    /// the sequential pass-through merge absorbs with probability 1/2
    /// while the synchronous reading keeps two tokens with
    /// probability 3/4. The variant chain is internally green
    /// (probability rows, clean DP, its own contraction certificate
    /// would even exist); the translation differential is the
    /// instrument that refuses it.
    struct IsraeliJalfon4SyncSlip;

    impl Chain for IsraeliJalfon4SyncSlip {
        fn init(&self) -> u64 {
            IJ4_START
        }
        fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
            if x == IJ4_START {
                return IsraeliJalfon4.succ(x, 0);
            }
            let k = ij4_tokens(x);
            let w = q(1, 1 << k);
            (0..(1u64 << k))
                .map(|coins| {
                    // synchronous: union of destinations from OLD state
                    let mut ncfg = 0u64;
                    let mut bit = 0u32;
                    for i in 0..4u64 {
                        if (x >> i) & 1 == 1 {
                            let heads = (coins >> bit) & 1 == 1;
                            bit += 1;
                            let dst = if heads { (i + 3) % 4 } else { (i + 1) % 4 };
                            ncfg |= 1 << dst;
                        }
                    }
                    let dst = if ij4_tokens(ncfg) <= 1 {
                        Dst::Done(0)
                    } else {
                        Dst::Alive(ncfg)
                    };
                    (dst, w.clone())
                })
                .collect()
        }
        fn name(&self) -> &'static str {
            "israeli-jalfon4-sync-slip"
        }
        fn time_homogeneous(&self) -> bool {
            true
        }
    }

    #[test]
    fn israeli_jalfon4_cross_program_refused_by_differential() {
        let surv = survival_masses(&IsraeliJalfon4SyncSlip, 31);
        let r = differential_times(sim_israeli_jalfon4, &surv, 1_000_000, 0xD1CE_5EED);
        assert!(
            r.worst_sigma > 4.0,
            "the synchronous-slip chain must be REFUSED by the \
             translation differential; worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
    }

    #[test]
    fn beauquier3_cross_program_refused_by_differential() {
        // short horizon: the sequential slip changes the absorption
        // CDF where the mass is still moving
        let tr = evaluate(&Beauquier3SeqSlip, 4);
        let r = differential(sim_beauquier3, &tr, 1_000_000, 0xD1CE_5EED);
        assert!(
            r.worst_sigma > 4.0,
            "the slipped chain must be REFUSED by the translation \
             differential; worst sigma {} rows {:?}",
            r.worst_sigma,
            r.rows
        );
        // and the true chain at the same horizon passes the same test
        let ok = run(sim_beauquier3, &Beauquier3, 4);
        assert!(
            ok.worst_sigma < 4.0,
            "true chain must pass at the same horizon; {} rows {:?}",
            ok.worst_sigma,
            ok.rows
        );
    }
}
