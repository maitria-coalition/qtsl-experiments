//! Exact-rational residual-mass posterior certificates for discrete
//! probabilistic programs — the untrusted producer.
//!
//! A source program is encoded as a finite-truncation, layered chain
//! over explicit states. The producer computes the exact T-step
//! distribution (integer-scaled dynamic programming over big integers;
//! one division at readout), the per-terminal-class absorbed masses,
//! the residual, and normalized posterior brackets by the
//! correlated-ratio form. Every derived quantity is re-derived by the
//! checker from the chain rows — certificates store claims, never
//! intermediate arithmetic.
//!
//! Soundness posture (design decision, stated once): states without
//! stored outgoing rows are treated as absorbing by the evaluation
//! semantics (self-loop completion). Under that convention a truncated
//! row set can only WIDEN brackets — mass held at an unexpanded state
//! counts toward the residual, never toward any class — so truncation
//! completeness is a tightness concern, not a soundness obligation.
//! Binding the stored rows to the source program is the translation
//! obligation, discharged by vendored sources + Monte-Carlo
//! differential + exact ground-truth containment where closed forms
//! exist.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use std::collections::BTreeMap;

pub type Q = BigRational;

pub fn q(n: i64, d: i64) -> Q {
    Q::new(BigInt::from(n), BigInt::from(d))
}

/// Destination of one weighted transition.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dst {
    /// Still running (chain-internal state id).
    Alive(u64),
    /// Absorbed into a terminal class (the query variable's value).
    Done(u32),
    /// Absorbed by a failed hard-conditioning observation: excluded
    /// from the posterior normalizer (rejected mass never enters the
    /// posterior; nonterminating mass likewise, by accruing to the
    /// residual instead of any class).
    Reject,
}

/// A layered program chain: successor rows may depend on the step
/// index, which keeps query variables that record absorption time
/// finite without the producer materializing a product space. The
/// certificate emitter materializes explicit timestamped states where
/// the wire species demands time-homogeneous rows.
pub trait Chain {
    fn init(&self) -> u64;
    /// Full outgoing row at alive state `x` when the transition taken
    /// is the one from step `t` to step `t+1` (t is 0-based). Weights
    /// are exact rationals summing to exactly one.
    fn succ(&self, x: u64, t: u32) -> Vec<(Dst, Q)>;
    fn name(&self) -> &'static str;
    /// True when `succ` ignores the step index, i.e. the chain is a
    /// genuine time-homogeneous stored-row chain. The certificate
    /// emitter then materializes states ONCE (not per layer): the wire
    /// species checks arbitrary stored-row chains, so the layered
    /// product is pure overhead wherever the query's classes are
    /// time-independent — the compact form buys orders of magnitude of
    /// horizon under the checker's work budget.
    fn time_homogeneous(&self) -> bool {
        false
    }
}

/// Result of the exact truncated evaluation.
#[derive(Clone, Debug)]
pub struct Truncated {
    pub horizon: u32,
    /// Exact absorbed mass per terminal class, within the horizon.
    pub class_mass: BTreeMap<u32, Q>,
    /// Exact rejected mass within the horizon.
    pub reject_mass: Q,
    /// Exact alive (unabsorbed, unrejected) mass at the horizon.
    pub residual: Q,
    /// Alive support at the horizon (state -> mass), diagnostics and
    /// tail-invariant feasibility fodder.
    pub alive_support: BTreeMap<u64, Q>,
}

impl Truncated {
    /// Sum of class masses (the truncated evidence mass).
    pub fn evidence(&self) -> Q {
        self.class_mass
            .values()
            .fold(Q::zero(), |a, b| a + b.clone())
    }

    /// Unnormalized bracket for one class: [N_c, N_c + R].
    pub fn absorb_bracket(&self, class: u32) -> (Q, Q) {
        let n = self.class_mass.get(&class).cloned().unwrap_or_else(Q::zero);
        (n.clone(), n + self.residual.clone())
    }

    /// Normalized (posterior) bracket for one class by the
    /// correlated-ratio form: [N_c/(S+R), (N_c+R)/(S+R)].
    /// None when S = 0 (no positive evidence witness).
    pub fn posterior_bracket(&self, class: u32) -> Option<(Q, Q)> {
        let s = self.evidence();
        if s.is_zero() {
            return None;
        }
        let n = self.class_mass.get(&class).cloned().unwrap_or_else(Q::zero);
        let den = s + self.residual.clone();
        Some((n.clone() / den.clone(), (n + self.residual.clone()) / den))
    }
}

fn gcd(a: &BigInt, b: &BigInt) -> BigInt {
    let mut a = a.abs();
    let mut b = b.abs();
    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }
    a
}

fn lcm(a: &BigInt, b: &BigInt) -> BigInt {
    let g = gcd(a, b);
    a / &g * b
}

/// Integer-scaled forward DP: masses are big-integer numerators over a
/// running denominator; each layer rescales by that layer's common row
/// denominator, so no rational reduction happens in the loop.
pub fn evaluate(chain: &dyn Chain, horizon: u32) -> Truncated {
    let mut alive: BTreeMap<u64, BigInt> = BTreeMap::new();
    alive.insert(chain.init(), BigInt::one());
    let mut denom = BigInt::one();
    let mut class_num: BTreeMap<u32, BigInt> = BTreeMap::new();
    let mut reject_num = BigInt::zero();

    for t in 0..horizon {
        let rows: Vec<(u64, Vec<(Dst, Q)>)> =
            alive.keys().map(|&x| (x, chain.succ(x, t))).collect();
        let mut layer_den = BigInt::one();
        for (_, row) in &rows {
            for (_, w) in row {
                layer_den = lcm(&layer_den, w.denom());
            }
        }
        // absorbed-so-far numerators advance to the new scale first
        for v in class_num.values_mut() {
            *v *= &layer_den;
        }
        reject_num *= &layer_den;

        let mut next: BTreeMap<u64, BigInt> = BTreeMap::new();
        for (x, row) in &rows {
            let m = alive.get(x).unwrap().clone();
            debug_assert!(
                row.iter()
                    .fold(Q::zero(), |a, (_, w)| a + w.clone())
                    .is_one(),
                "probability rows must sum to one"
            );
            for (dst, w) in row {
                let scaled = w * Q::new(layer_den.clone(), BigInt::one());
                debug_assert!(scaled.is_integer());
                let add = &m * scaled.to_integer();
                match dst {
                    Dst::Alive(y) => *next.entry(*y).or_insert_with(BigInt::zero) += add,
                    Dst::Done(c) => *class_num.entry(*c).or_insert_with(BigInt::zero) += add,
                    Dst::Reject => reject_num += add,
                }
            }
        }
        alive = next;
        denom *= &layer_den;
    }

    let scale = |n: &BigInt| Q::new(n.clone(), denom.clone());
    let class_mass: BTreeMap<u32, Q> = class_num.iter().map(|(c, n)| (*c, scale(n))).collect();
    let reject_mass = scale(&reject_num);
    let alive_support: BTreeMap<u64, Q> = alive.iter().map(|(x, n)| (*x, scale(n))).collect();
    let residual = alive_support.values().fold(Q::zero(), |a, b| a + b.clone());

    // conservation audit (probability chains): classes + reject +
    // alive account for exactly all the mass.
    let total = class_mass.values().fold(Q::zero(), |a, b| a + b.clone())
        + reject_mass.clone()
        + residual.clone();
    assert!(total.is_one(), "mass conservation violated: {total}");

    Truncated {
        horizon,
        class_mass,
        reject_mass,
        residual,
        alive_support,
    }
}

// ------------------------------------------------------------ programs

/// Geometric counter: `X := 0; while flip(1/2) { X += 1 }`.
/// At step t the single alive state carries X = t. Absorption at the
/// t-th transition lands in class X = t. Truth: P[X = n] = 2^-(n+1).
pub struct GeoCounter;

impl Chain for GeoCounter {
    fn init(&self) -> u64 {
        0
    }
    fn succ(&self, x: u64, t: u32) -> Vec<(Dst, Q)> {
        debug_assert_eq!(x as u32, t, "layered invariant");
        vec![(Dst::Alive(x + 1), q(1, 2)), (Dst::Done(t), q(1, 2))]
    }
    fn name(&self) -> &'static str {
        "geo-counter"
    }
}

/// Asymmetric random walk: `pos := 1; steps := 0; while pos > 0
/// { steps += 1; pos += 1 with prob r else pos -= 1 }`, bias r = 1/4.
/// Query variable = steps at termination = absorption time. Truth:
/// E[steps] = 1/(1-2r) = 2; absorption only at odd times.
pub struct AsymRw {
    pub r: Q,
}

impl Default for AsymRw {
    fn default() -> Self {
        AsymRw { r: q(1, 4) }
    }
}

impl Chain for AsymRw {
    fn init(&self) -> u64 {
        1
    }
    fn succ(&self, x: u64, t: u32) -> Vec<(Dst, Q)> {
        debug_assert!(x >= 1);
        let up = self.r.clone();
        let down = Q::one() - up.clone();
        let down_dst = if x == 1 {
            // reaching 0 at transition t -> t+1 records steps = t+1
            Dst::Done(t + 1)
        } else {
            Dst::Alive(x - 1)
        };
        vec![(Dst::Alive(x + 1), up), (down_dst, down)]
    }
    fn name(&self) -> &'static str {
        "asym-rw"
    }
}

/// Mossel's die paradox: throw a fair die until it shows 6, observe
/// (hard) that every throw is even. Query variable = number of throws.
/// Chain: one alive state; per throw: 6 with prob 1/6 -> accept with
/// throws = t+1; {2,4} with prob 2/6 -> continue; odd with prob 3/6 ->
/// reject. Truth: P[T = n | acc] = (2/3)(1/3)^(n-1), E = 3/2.
pub struct DieParadox;

impl Chain for DieParadox {
    fn init(&self) -> u64 {
        0
    }
    fn succ(&self, _x: u64, t: u32) -> Vec<(Dst, Q)> {
        vec![
            (Dst::Done(t + 1), q(1, 6)),
            (Dst::Alive(0), q(2, 6)),
            (Dst::Reject, q(3, 6)),
        ]
    }
    fn name(&self) -> &'static str {
        "die-paradox"
    }
}

/// PSI's burglarAlarm (vendored from the GuBPI benchmark suite, which
/// takes it from the PSI repository): a finite Bayes net with hard
/// conditioning. earthquake ~ flip(1/10000), burglary ~ flip(1/1000),
/// alarm = earthquake OR burglary, phoneWorking ~ flip(7/10 | 99/100 by
/// earthquake), maryWakes ~ flip(8/10 | 6/10 | 2/10 by alarm x
/// earthquake), observe(maryWakes AND phoneWorking), return burglary.
///
/// Layered encoding, one sampled variable per layer; the observe rides
/// the reject labeling (hard conditioning is a class, not a score).
/// The final layer folds the maryWakes sample with the observe test:
/// mw = 1 and pw = 1 absorbs into done_{burglary}, everything else
/// rejects. The program is finite, so horizon 4 is FULL unrolling: the
/// alive frontier is empty, the residual is exactly zero, and the
/// posterior brackets are exact points. PSI ground truth (from the
/// vendored source's own comment): P[burglary = 1 | called] =
/// 2969983/992160802.
///
/// State encoding per layer: t=0 start (state 0); t=1 e; t=2 2e+b;
/// t=3 4e+2b+pw.
pub struct BurglarAlarm;

impl Chain for BurglarAlarm {
    fn init(&self) -> u64 {
        0
    }
    fn succ(&self, x: u64, t: u32) -> Vec<(Dst, Q)> {
        match t {
            // sample earthquake
            0 => vec![
                (Dst::Alive(1), q(1, 10000)),
                (Dst::Alive(0), q(9999, 10000)),
            ],
            // sample burglary
            1 => {
                let e = x;
                vec![
                    (Dst::Alive(2 * e + 1), q(1, 1000)),
                    (Dst::Alive(2 * e), q(999, 1000)),
                ]
            }
            // sample phoneWorking (depends on earthquake)
            2 => {
                let e = x >> 1;
                let p_pw = if e == 1 { q(7, 10) } else { q(99, 100) };
                vec![
                    (Dst::Alive(2 * x + 1), p_pw.clone()),
                    (Dst::Alive(2 * x), Q::one() - p_pw),
                ]
            }
            // sample maryWakes, fold the observe: called = mw AND pw
            3 => {
                let e = x >> 2;
                let b = (x >> 1) & 1;
                let pw = x & 1;
                let alarm = e | b;
                let p_mw = match (alarm, e) {
                    (1, 1) => q(8, 10),
                    (1, 0) => q(6, 10),
                    _ => q(2, 10),
                };
                if pw == 1 {
                    vec![
                        (Dst::Done(b as u32), p_mw.clone()),
                        (Dst::Reject, Q::one() - p_mw),
                    ]
                } else {
                    // phone dead: called is false whatever mary does
                    vec![(Dst::Reject, Q::one())]
                }
            }
            _ => unreachable!("finite program: no layer past 3"),
        }
    }
    fn name(&self) -> &'static str {
        "burglar-alarm"
    }
}

/// Beauquier–Gradinariu–Johnen-style self-stabilizing token ring, 3
/// processes (`beauquier-etal-3`, vendored from the GuBPI benchmark
/// suite's Sankaranarayanan-18 adaptation,
/// `benchmarks/ProbEstimation/beauquier-etal-3/beauquier-etal-3-Q1.spcf`).
/// Each process i carries two bits (d_i, p_i); all six are sampled
/// iid fair at start. One synchronous round, every guard reading OLD
/// values (ring predecessor of 1 is 3):
///
/// - d_i flips iff d_i = d_{i-1};
/// - p_i resamples uniformly over {0, 1} iff d_i = d_{i-1} AND
///   p_i = p_{i-1} (the source's "keep w.p. 1/2 else flip" coin IS a
///   uniform resample);
/// - otherwise both bits keep.
///
/// The program loops while p_1 + p_2 + p_3 >= 2, incrementing `count`
/// each call including the exit call, and enters the loop only when
/// the INITIAL p-sum is >= 2. Query Q1: P[count < 1] — count = 0 iff
/// the initial p-sum is <= 1, an event of exactly binomial mass
/// C(3,0)/8 + C(3,1)/8 = 1/2 on the three iid p-bits.
///
/// Chain encoding, TIME-HOMOGENEOUS (the classes are
/// time-independent, so no layer product): state 64 = start; states
/// 0..63 = bit-packed configs (d1 p1 d2 p2 d3 p3, high to low). The
/// start row is the 64-outcome initial sample, classifying p-sum <= 1
/// into class 0 (count < 1) immediately; alive configs (p-sum >= 2,
/// 32 of them) take the synchronous round with one weight per joint
/// coin outcome (2^-k, k = enabled resamples); a round landing at
/// p-sum <= 1 absorbs into class 1 (count >= 1). Score-free: no
/// rejection, evidence + residual = 1 exactly, so the posterior
/// brackets coincide with the unnormalized ones.
pub struct Beauquier3;

/// Packed-bit helpers for the 6-bit ring configs (d1 p1 d2 p2 d3 p3,
/// high to low).
fn bq3_bits(x: u64) -> (u64, u64, u64, u64, u64, u64) {
    (
        (x >> 5) & 1,
        (x >> 4) & 1,
        (x >> 3) & 1,
        (x >> 2) & 1,
        (x >> 1) & 1,
        x & 1,
    )
}

fn bq3_pack(d1: u64, p1: u64, d2: u64, p2: u64, d3: u64, p3: u64) -> u64 {
    (d1 << 5) | (p1 << 4) | (d2 << 3) | (p2 << 2) | (d3 << 1) | p3
}

fn bq3_psum(x: u64) -> u64 {
    ((x >> 4) & 1) + ((x >> 2) & 1) + (x & 1)
}

pub const BQ3_START: u64 = 64;

impl Chain for Beauquier3 {
    fn init(&self) -> u64 {
        BQ3_START
    }
    fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
        if x == BQ3_START {
            // initial sample: six iid fair bits; p-sum <= 1 is count = 0.
            return (0..64u64)
                .map(|cfg| {
                    let dst = if bq3_psum(cfg) <= 1 {
                        Dst::Done(0)
                    } else {
                        Dst::Alive(cfg)
                    };
                    (dst, q(1, 64))
                })
                .collect();
        }
        debug_assert!(x < 64 && bq3_psum(x) >= 2, "alive configs have p-sum >= 2");
        let (d1, p1, d2, p2, d3, p3) = bq3_bits(x);
        // deterministic d-round, all guards on OLD values
        let nd1 = if d1 == d3 { 1 - d1 } else { d1 };
        let nd2 = if d2 == d1 { 1 - d2 } else { d2 };
        let nd3 = if d3 == d2 { 1 - d3 } else { d3 };
        // enabled p-resamples (uniform over {0,1} where enabled)
        let e1 = d1 == d3 && p1 == p3;
        let e2 = d2 == d1 && p2 == p1;
        let e3 = d3 == d2 && p3 == p2;
        let k = e1 as u32 + e2 as u32 + e3 as u32;
        let w = q(1, 1 << k);
        let mut out = Vec::with_capacity(1 << k);
        for c in 0..(1u64 << k) {
            let mut bit = 0;
            let mut draw = |enabled: bool, old: u64| {
                if enabled {
                    let v = (c >> bit) & 1;
                    bit += 1;
                    v
                } else {
                    old
                }
            };
            let np1 = draw(e1, p1);
            let np2 = draw(e2, p2);
            let np3 = draw(e3, p3);
            let ncfg = bq3_pack(nd1, np1, nd2, np2, nd3, np3);
            let dst = if np1 + np2 + np3 <= 1 {
                Dst::Done(1)
            } else {
                Dst::Alive(ncfg)
            };
            out.push((dst, w.clone()));
        }
        out
    }
    fn name(&self) -> &'static str {
        "beauquier-3"
    }
    fn time_homogeneous(&self) -> bool {
        true
    }
}

/// Coupon collector, five coupons (`coupon-collector5`, vendored from
/// the Diabolo benchmark suite, `benchmarks/ours/coupon-collector5.sgcl`):
/// draw uniformly among five coupons until every coupon has been seen,
/// returning the number of draws. The compared query is the first
/// moment of the return value (the baseline's EV objective); truth
/// $\mathbb{E} = 5 H_5 = 137/12$, tail rate exactly $4/5$ (the alive
/// substochastic block is triangular with diagonal $k/5$).
///
/// Chain encoding, TIME-HOMOGENEOUS: the five coupon flags are
/// exchangeable, so the chain state is the count $k$ of distinct
/// coupons collected ($k \in 0..4$ alive; collecting the fifth is the
/// single terminal class). From $k$: a new coupon w.p. $(5-k)/5$
/// ($k{+}1$, or class 0 at $k = 4$), a repeat w.p. $k/5$ (self-loop;
/// at $k = 0$ the repeat weight is zero and no row is stored). The
/// exchangeability lump is a translation step, discharged by the
/// source-semantics differential (five explicit flags) and the
/// inclusion-exclusion oracle. Absorption time = the program's count.
///
/// The time-homogeneous form is LOAD-BEARING for the EV certificate,
/// not an optimization: the contraction-tail premise quantifies over
/// the chain's law at every future step, so the stored rows must BE
/// the total law on the alive-reachable set. A per-layer truncated
/// row set has no sound EV reading (rowless frontier states hold
/// their mass forever, so survival would not decay) — mass brackets
/// tolerate truncation, first moments do not.
pub struct CouponCollector5;

impl Chain for CouponCollector5 {
    fn init(&self) -> u64 {
        0
    }
    fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
        debug_assert!(x <= 4, "alive states are counts 0..4");
        let k = x as i64;
        let advance = if x == 4 {
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
        "coupon-collector5"
    }
    fn time_homogeneous(&self) -> bool {
        true
    }
}

/// Israeli–Jalfon randomized self-stabilization on the ring
/// $\mathbb{Z}_4$ (`israeli-jalfon4`, vendored from the Diabolo
/// benchmark suite, `benchmarks/ours/israeli-jalfon4.sgcl`):
/// four token flags sampled iid fair; while more than one token
/// remains, one loop iteration snapshots the flags and walks the four
/// if-blocks IN ORDER — a block whose OLD flag is set clears its own
/// flag (current state) and sets a uniformly-chosen neighbor's flag
/// (predecessor on heads, successor on tails, per the source's
/// branches); `count` increments once per iteration; return count.
/// The compared query is the first moment of the return value (the
/// expected self-stabilization time).
///
/// The sequential-update semantics is load-bearing: a later block
/// clears its own bit even if an earlier block's move just set it
/// (the pass-through merge), and sets survive only if no later block
/// clears them. The one-step direct-transcription oracle and the
/// cross-program synchronous-slip tooth in `differential.rs` pin
/// exactly this.
///
/// Chain encoding, TIME-HOMOGENEOUS: states 0..15 are the flag
/// bit-vectors (bit $i$ = process $i{+}1$), state 16 = START (the
/// Bernoulli$(1/2)^4$ prologue as one 16-outcome row); configurations
/// with token count $\le 1$ (five) classify into the single terminal
/// class `done_0` (stabilized); the 11 multi-token configurations
/// carry their exact one-iteration rows ($2^{|s|}$ coin outcomes,
/// coalesced by the emitter). Token count never increases and the
/// last-firing block always leaves a token, so the stabilized class
/// is closed (no re-entry) and the alive region loses no mass to
/// truncation surprises. With the prologue, the chain's absorption
/// time is $T = \mathit{count} + 1$ on every path — the EV
/// certificate carries anchor-offset 1 (see `ev.rs`).
///
/// Ground truth (exact linear-solve oracle over the 11 alive
/// configurations, `israeli_jalfon4_linear_solve_oracle` below):
/// $\mathbb{E}[\mathit{count}] = 373/256 = 1.45703125$; alive spectral
/// radius exactly $1/2$. NOTE the sequential block order BREAKS the
/// ring's rotational symmetry — the 3-token expectations differ by
/// configuration ($E_{\{1,2,4\}} = 2$ but $E_{\{1,3,4\}} = 5/2$), so
/// any class-lumped analysis over token counts is wrong (a
/// pre-registered hand-lumped derivation of this leg died exactly
/// there; the mechanical oracles are the record).
pub struct IsraeliJalfon4;

pub const IJ4_START: u64 = 16; // one past the 0..15 configs

/// One loop iteration of the source program: `old` is the snapshot,
/// `coins` supplies one bit per firing block in block order (bit 0 =
/// the lowest-numbered old token). Heads (bit 1) sends to the
/// predecessor (process1 -> process4), tails to the successor
/// (process1 -> process2), matching the source's branch arms.
pub fn ij4_step(old: u64, coins: u64) -> u64 {
    debug_assert!(old < 16);
    let mut cur = old;
    let mut bit = 0u32;
    for i in 0..4u64 {
        if (old >> i) & 1 == 1 {
            cur &= !(1 << i);
            let heads = (coins >> bit) & 1 == 1;
            bit += 1;
            let dst = if heads { (i + 3) % 4 } else { (i + 1) % 4 };
            cur |= 1 << dst;
        }
    }
    cur
}

pub fn ij4_tokens(cfg: u64) -> u32 {
    (cfg & 0xF).count_ones()
}

impl Chain for IsraeliJalfon4 {
    fn init(&self) -> u64 {
        IJ4_START
    }
    fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
        if x == IJ4_START {
            // Bernoulli(1/2)^4 prologue: sixteen configurations at
            // 1/16 each; already-stabilized ones are count = 0.
            return (0..16u64)
                .map(|cfg| {
                    let dst = if ij4_tokens(cfg) <= 1 {
                        Dst::Done(0)
                    } else {
                        Dst::Alive(cfg)
                    };
                    (dst, q(1, 16))
                })
                .collect();
        }
        debug_assert!(
            x < 16 && ij4_tokens(x) >= 2,
            "alive configs hold >= 2 tokens"
        );
        let k = ij4_tokens(x);
        let w = q(1, 1 << k);
        (0..(1u64 << k))
            .map(|coins| {
                let ncfg = ij4_step(x, coins);
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
        "israeli-jalfon4"
    }
    fn time_homogeneous(&self) -> bool {
        true
    }
}

/// 3-D asymmetric random walk (`3d-asym-rw`, vendored from the
/// Diabolo benchmark suite, `benchmarks/ours/3d-asym-rw.sgcl`):
/// $(x,y,z) := (1,1,1)$; while $(x,y,z) \ne (0,0,0)$: choose an axis
/// uniformly ($1/3$ each — the source spells it flip(1/3) then
/// flip(1/2)), move it $+1$ w.p. $1/4$ else $-1$ with MONUS at zero
/// (SGCL variables are naturals: the vendored tool's decrement is
/// `saturating_sub` in its support transform and a sum-into-index-0
/// `shift_left` on its mass bounds), count the step. Return the count
/// = the absorption time. The alive-reachable set is INFINITE (no
/// closed-form truth; the pattern face in `ev3d` carries the
/// certificate machinery — this Chain impl is the generic ORACLE the
/// fast DP is differentially tested against).
///
/// Chain encoding, TIME-HOMOGENEOUS: coordinates are exchangeable
/// (the axis choice is uniform and the move law identical), so the
/// state is the SORTED triple $a \le b \le c$, packed
/// $a\,|\,b{\ll}12\,|\,c{\ll}24$. The exchangeability lump is a
/// translation step, discharged by the source-semantics differential
/// (three explicit coordinates, literal flips) and the one-step
/// tuple-vs-lump oracle test.
pub struct AsymRw3d;

impl Chain for AsymRw3d {
    fn init(&self) -> u64 {
        ev3d::t3_pack(1, 1, 1)
    }
    fn succ(&self, x: u64, _t: u32) -> Vec<(Dst, Q)> {
        let (alive, absorbed) = ev3d::pattern_row(ev3d::t3_unpack(x), 1, 12);
        let mut out: Vec<(Dst, Q)> = alive
            .into_iter()
            .map(|(d, w)| (Dst::Alive(d), q(w as i64, 12)))
            .collect();
        if absorbed > 0 {
            out.push((Dst::Done(0), q(absorbed as i64, 12)));
        }
        out
    }
    fn name(&self) -> &'static str {
        "3d-asym-rw"
    }
    fn time_homogeneous(&self) -> bool {
        true
    }
}

/// Window materialization of the 3-D walk for the CONTAINER lane:
/// true rows for states with max coordinate <= `rstore`, a mass-
/// holding self-loop everywhere beyond (the stored-row spec leaves
/// those states rowless; the container checker's rowless-is-absorbing
/// evaluation and this self-loop agree exactly, and the parked mass
/// accrues to the residual — truncation soundly WIDENS the absorb
/// bracket, per the module-level soundness posture).
pub struct AsymRw3dWindow {
    pub rstore: u64,
}

impl Chain for AsymRw3dWindow {
    fn init(&self) -> u64 {
        AsymRw3d.init()
    }
    fn succ(&self, x: u64, t: u32) -> Vec<(Dst, Q)> {
        let (_, _, c) = ev3d::t3_unpack(x);
        if c <= self.rstore {
            AsymRw3d.succ(x, t)
        } else {
            vec![(Dst::Alive(x), Q::one())]
        }
    }
    fn name(&self) -> &'static str {
        "3d-asym-rw-window"
    }
    fn time_homogeneous(&self) -> bool {
        true
    }
}

/// Exact survival masses $P[T > n]$ for $n = 0..=\mathrm{horizon}$,
/// where $T$ is the time of leaving the alive set (absorption into any
/// class or rejection). This is the unrolled exact partial-moment
/// ingredient of the EV certificates: the first moment in survival
/// form is $\mathbb{E}[T] = \sum_{n \ge 0} P[T > n]$. Same forward DP
/// as `evaluate`, recording the alive total per step.
pub fn survival_masses(chain: &dyn Chain, horizon: u32) -> Vec<Q> {
    let mut alive: BTreeMap<u64, Q> = BTreeMap::new();
    alive.insert(chain.init(), Q::one());
    let mut out = Vec::with_capacity(horizon as usize + 1);
    out.push(alive.values().fold(Q::zero(), |a, b| a + b.clone()));
    for t in 0..horizon {
        let mut next: BTreeMap<u64, Q> = BTreeMap::new();
        for (&x, m) in &alive {
            for (dst, w) in chain.succ(x, t) {
                if let Dst::Alive(y) = dst {
                    *next.entry(y).or_insert_with(Q::zero) += m * w;
                }
            }
        }
        alive = next;
        out.push(alive.values().fold(Q::zero(), |a, b| a + b.clone()));
    }
    out
}

// ---------------------------------------------------------- truths

/// Exact closed-form posterior mass per class, for containment tests.
pub fn truth(name: &str, n: u32) -> Q {
    match name {
        // P[X = n] = (1/2)^(n+1)
        "geo-counter" => q(1, 2).pow((n + 1) as i32),
        // P[T = n | acc] = (2/3)(1/3)^(n-1), n >= 1
        "die-paradox" => {
            if n == 0 {
                Q::zero()
            } else {
                q(2, 3) * q(1, 3).pow((n - 1) as i32)
            }
        }
        // P[burglary = n | called] — PSI's exact result, vendored from
        // the benchmark source's own comment.
        "burglar-alarm" => match n {
            1 => q(2969983, 992160802),
            0 => q(989190819, 992160802),
            _ => Q::zero(),
        },
        // P[count < 1] = P[initial p-sum <= 1] = (C(3,0) + C(3,1))/8;
        // the complement is absorbed a.s. (limit mass), score-free so
        // posterior = unnormalized.
        "beauquier-3" => match n {
            0 | 1 => q(1, 2),
            _ => Q::zero(),
        },
        // single terminal class (all five coupons collected), reached
        // almost surely: limit mass 1.
        "coupon-collector5" => match n {
            0 => Q::one(),
            _ => Q::zero(),
        },
        // single terminal class (stabilized), reached almost surely.
        "israeli-jalfon4" => match n {
            0 => Q::one(),
            _ => Q::zero(),
        },
        _ => panic!("no closed form registered for {name}"),
    }
}

/// Inclusion–exclusion closed form for the five-coupon collector:
/// $P[T \le n] = \sum_{j=0}^{5} (-1)^j \binom{5}{j} ((5-j)/5)^n$ —
/// the independent analytic oracle for the chain DP.
pub fn cc5_absorbed_by(n: u32) -> Q {
    let binom = [1i64, 5, 10, 10, 5, 1];
    let mut acc = Q::zero();
    for (j, b) in binom.iter().enumerate() {
        let term = q(*b, 1) * q(5 - j as i64, 5).pow(n as i32);
        if j % 2 == 0 {
            acc += term;
        } else {
            acc -= term;
        }
    }
    acc
}

trait QPow {
    fn pow(&self, e: i32) -> Q;
}

impl QPow for Q {
    fn pow(&self, e: i32) -> Q {
        let mut acc = Q::one();
        for _ in 0..e {
            acc *= self.clone();
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geo_counter_masses_exact() {
        let tr = evaluate(&GeoCounter, 30);
        for n in 0..30u32 {
            assert_eq!(tr.class_mass[&n], truth("geo-counter", n));
        }
        assert_eq!(tr.residual, q(1, 2).pow(30));
        assert!(tr.reject_mass.is_zero());
    }

    #[test]
    fn die_paradox_posterior_contains_truth() {
        let tr = evaluate(&DieParadox, 40);
        // Z_T = (1/6)(1-(1/3)^40)/(2/3); residual = (1/3)^40
        for n in 1..=10u32 {
            let (lo, hi) = tr.posterior_bracket(n).unwrap();
            let t = truth("die-paradox", n);
            assert!(lo <= t && t <= hi, "class {n}: [{lo}, {hi}] vs {t}");
        }
        // width = R/(S+R), identical across classes
        let (lo1, hi1) = tr.posterior_bracket(1).unwrap();
        let (lo2, hi2) = tr.posterior_bracket(2).unwrap();
        assert_eq!(hi1.clone() - lo1.clone(), hi2.clone() - lo2.clone());
        assert_eq!(
            hi1 - lo1,
            tr.residual.clone() / (tr.evidence() + tr.residual)
        );
    }

    /// Independent analytic oracle: enumerate the 16 joint assignments
    /// of (e, b, pw, mw) directly — no Chain machinery — and check the
    /// DP's class/reject masses against it, plus the vendored PSI
    /// ground truth at the posterior.
    #[test]
    fn burglar_alarm_exact_at_full_unrolling() {
        let tr = evaluate(&BurglarAlarm, 4);

        // full unrolling of a finite program: nothing alive
        assert!(tr.residual.is_zero(), "residual {}", tr.residual);
        assert!(tr.alive_support.is_empty());

        // independent enumeration
        let mut joint_called = [Q::zero(), Q::zero()]; // by burglary
        let mut rejected = Q::zero();
        for e in 0..2u32 {
            for b in 0..2u32 {
                for pw in 0..2u32 {
                    for mw in 0..2u32 {
                        let pe = if e == 1 { q(1, 10000) } else { q(9999, 10000) };
                        let pb = if b == 1 { q(1, 1000) } else { q(999, 1000) };
                        let ppw1 = if e == 1 { q(7, 10) } else { q(99, 100) };
                        let ppw = if pw == 1 { ppw1 } else { Q::one() - ppw1 };
                        let alarm = e | b;
                        let pmw1 = match (alarm, e) {
                            (1, 1) => q(8, 10),
                            (1, 0) => q(6, 10),
                            _ => q(2, 10),
                        };
                        let pmw = if mw == 1 { pmw1 } else { Q::one() - pmw1 };
                        let p = pe * pb * ppw * pmw;
                        if mw == 1 && pw == 1 {
                            joint_called[b as usize] += p;
                        } else {
                            rejected += p;
                        }
                    }
                }
            }
        }
        assert_eq!(tr.class_mass[&0], joint_called[0]);
        assert_eq!(tr.class_mass[&1], joint_called[1]);
        assert_eq!(tr.reject_mass, rejected);

        // posterior brackets are exact points equal to the PSI truth
        for c in 0..2u32 {
            let (lo, hi) = tr.posterior_bracket(c).unwrap();
            assert_eq!(lo, hi, "width must be exactly 0 at full unrolling");
            assert_eq!(lo, truth("burglar-alarm", c));
        }
        // and the two posterior masses sum to one
        let (p0, _) = tr.posterior_bracket(0).unwrap();
        let (p1, _) = tr.posterior_bracket(1).unwrap();
        assert!((p0 + p1).is_one());
    }

    #[test]
    fn asym_rw_expectation_sanity() {
        // absorbed mass concentrates toward E[steps] = 2: exact masses
        // at small odd times: P[T=1] = 3/4, P[T=3] = (1/4)(3/4)^2 = 9/64.
        let tr = evaluate(&AsymRw::default(), 70);
        assert_eq!(tr.class_mass[&1], q(3, 4));
        assert_eq!(tr.class_mass[&3], q(9, 64));
        assert!(tr.class_mass.get(&2).is_none(), "even times unreachable");
        // truncated masses + residual account for everything
        let s: Q = tr.evidence();
        assert!(s + tr.residual.clone() == Q::one());
    }

    /// The count < 1 mass is EXACTLY 1/2 from the first transition on
    /// (it depends only on the initial p-bits), the chain is
    /// score-free (no rejection), alive support stays inside the
    /// p-sum >= 2 configs, and the residual is monotone in the
    /// horizon.
    #[test]
    fn beauquier3_class0_exact_half() {
        let tr = evaluate(&Beauquier3, 51);
        assert_eq!(tr.class_mass[&0], q(1, 2));
        assert!(tr.reject_mass.is_zero());
        for x in tr.alive_support.keys() {
            assert!(*x < 64 && bq3_psum(*x) >= 2, "alive support {x}");
        }
        let earlier = evaluate(&Beauquier3, 10);
        assert!(tr.residual < earlier.residual);
        // score-free: evidence + residual = 1, so posterior = absorb
        assert!(tr.evidence() + tr.residual.clone() == Q::one());
        let (alo, ahi) = tr.absorb_bracket(0);
        let (plo, phi) = tr.posterior_bracket(0).unwrap();
        assert_eq!((alo, ahi), (plo, phi));
    }

    /// Truth containment at both emitted horizons, and the tight row
    /// genuinely crushes the baseline width scale (GuBPI's published
    /// face at this query is ~2e-4).
    #[test]
    fn beauquier3_bracket_contains_truth() {
        use num_traits::ToPrimitive;
        for horizon in [51u32, 1000] {
            let tr = evaluate(&Beauquier3, horizon);
            for c in 0..2u32 {
                let (lo, hi) = tr.posterior_bracket(c).unwrap();
                let t = truth("beauquier-3", c);
                assert!(
                    lo <= t && t <= hi,
                    "class {c} h {horizon}: [{lo}, {hi}] vs {t}"
                );
            }
        }
        let tight = evaluate(&Beauquier3, 1000);
        let width = tight.residual.to_f64().unwrap();
        assert!(width < 1e-40, "tight-row residual {width:e}");
    }

    /// Independent single-step oracle: evolve the FULL 64-config
    /// distribution one synchronous round by direct transcription of
    /// the source text (per-process lets over old values, tuple state,
    /// no bit packing) and compare against the chain rows' one-step
    /// action on the same distribution.
    #[test]
    fn beauquier3_one_step_matches_direct_transcription() {
        // direct transcription over (d1,p1,d2,p2,d3,p3) tuples
        type Cfg = (u64, u64, u64, u64, u64, u64);
        let step_direct = |old: Cfg| -> Vec<(Cfg, Q)> {
            let (od1, op1, od2, op2, od3, op3) = old;
            let nd1 = if od1 == od3 { 1 - od1 } else { od1 };
            let nd2 = if od2 == od1 { 1 - od2 } else { od2 };
            let nd3 = if od3 == od2 { 1 - od3 } else { od3 };
            let mut outs: Vec<(Cfg, Q)> = vec![((nd1, op1, nd2, op2, nd3, op3), Q::one())];
            // process 1 coin
            if od1 == od3 && op1 == op3 {
                outs = outs
                    .into_iter()
                    .flat_map(|(c, w)| {
                        [(0u64, w.clone()), (1u64, w)]
                            .into_iter()
                            .map(move |(v, w)| ((c.0, v, c.2, c.3, c.4, c.5), w * q(1, 2)))
                    })
                    .collect();
            }
            if od2 == od1 && op2 == op1 {
                outs = outs
                    .into_iter()
                    .flat_map(|(c, w)| {
                        [(0u64, w.clone()), (1u64, w)]
                            .into_iter()
                            .map(move |(v, w)| ((c.0, c.1, c.2, v, c.4, c.5), w * q(1, 2)))
                    })
                    .collect();
            }
            if od3 == od2 && op3 == op2 {
                outs = outs
                    .into_iter()
                    .flat_map(|(c, w)| {
                        [(0u64, w.clone()), (1u64, w)]
                            .into_iter()
                            .map(move |(v, w)| ((c.0, c.1, c.2, c.3, c.4, v), w * q(1, 2)))
                    })
                    .collect();
            }
            outs
        };
        for x in 0..64u64 {
            if bq3_psum(x) < 2 {
                continue;
            }
            let (d1, p1, d2, p2, d3, p3) = bq3_bits(x);
            // direct: mass per packed successor (alive and done together)
            let mut direct: BTreeMap<Option<u64>, Q> = BTreeMap::new();
            for (c, w) in step_direct((d1, p1, d2, p2, d3, p3)) {
                let key = if c.1 + c.3 + c.5 <= 1 {
                    None // absorbed (count >= 1)
                } else {
                    Some(bq3_pack(c.0, c.1, c.2, c.3, c.4, c.5))
                };
                *direct.entry(key).or_insert_with(Q::zero) += w;
            }
            // chain rows
            let mut chain: BTreeMap<Option<u64>, Q> = BTreeMap::new();
            for (dst, w) in Beauquier3.succ(x, 0) {
                let key = match dst {
                    Dst::Alive(y) => Some(y),
                    Dst::Done(1) => None,
                    other => panic!("unexpected dst {other:?}"),
                };
                *chain.entry(key).or_insert_with(Q::zero) += w;
            }
            assert_eq!(direct, chain, "one-step divergence at config {x:06b}");
        }
    }

    /// Independent analytic oracle: the chain DP's absorbed-by-n mass
    /// equals the inclusion–exclusion closed form, exactly in Q.
    #[test]
    fn coupon_collector5_inclusion_exclusion_oracle() {
        let surv = survival_masses(&CouponCollector5, 80);
        for n in [0u32, 1, 4, 5, 6, 10, 20, 40, 80] {
            let absorbed = Q::one() - surv[n as usize].clone();
            assert_eq!(
                absorbed,
                cc5_absorbed_by(n),
                "inclusion-exclusion divergence at n = {n}"
            );
        }
        // survival and evaluate agree on the residual at the horizon
        let tr = evaluate(&CouponCollector5, 80);
        assert_eq!(tr.residual, surv[80]);
        assert!(tr.reject_mass.is_zero());
        assert_eq!(tr.class_mass.len(), 1);
    }

    /// The certificate weights (1, 4/5, 3/5, 2/5, 1/5) are the exact
    /// left eigenvector of the alive block at eigenvalue 4/5: every
    /// weighted transfer inequality holds with equality, and the
    /// weighted alive mass at N is exactly (4/5)^N.
    #[test]
    fn coupon_collector5_weights_eigenvector() {
        let w = |x: u64| q(5 - x as i64, 5);
        let c = q(4, 5);
        for x in 0..5u64 {
            let lhs = CouponCollector5
                .succ(x, 0)
                .iter()
                .fold(Q::zero(), |a, (dst, p)| match dst {
                    Dst::Alive(y) => a + p * w(*y),
                    _ => a, // absorbed weight 0
                });
            assert_eq!(lhs, c.clone() * w(x), "transfer inequality at k = {x}");
        }
        let tr = evaluate(&CouponCollector5, 80);
        let wn = tr
            .alive_support
            .iter()
            .fold(Q::zero(), |a, (x, m)| a + m * w(*x));
        assert_eq!(wn, c.pow(80), "weighted alive mass at 80");
    }

    /// The certificate weights for israeli-jalfon4: the minimal
    /// eigen-tight solution of the transfer system at gamma = 1/2
    /// (every row an equality), with the unit floor on the alive
    /// region (min over alive = 1 — the alive-mass face, kappa = 1).
    /// Per-state, NOT class-lumped: the sequential block order breaks
    /// the ring symmetry.
    fn ij4_weight(x: u64) -> Q {
        match x {
            IJ4_START => q(99, 64),
            7 => q(5, 4),    // {1,2,3}
            11 => Q::one(),  // {1,2,4}
            13 => q(3, 2),   // {1,3,4}
            14 => q(21, 16), // {2,3,4}
            15 => q(21, 16), // {1,2,3,4}
            _ => Q::one(),   // the six 2-token configurations
        }
    }

    /// Independent exact oracle: solve (I - Q_A) E = 1 over the 11
    /// alive configurations by rational Gaussian elimination (no DP,
    /// no survival machinery) and check the per-state values and the
    /// initial-distribution expectation E[count] = 373/256.
    #[test]
    fn israeli_jalfon4_linear_solve_oracle() {
        let alive: Vec<u64> = (0..16u64).filter(|&c| ij4_tokens(c) >= 2).collect();
        assert_eq!(alive.len(), 11);
        let idx = |c: u64| alive.iter().position(|&a| a == c).unwrap();
        // build A = I - Q_A and rhs = 1
        let n = alive.len();
        let mut m: Vec<Vec<Q>> = vec![vec![Q::zero(); n + 1]; n];
        for (i, &x) in alive.iter().enumerate() {
            m[i][i] += Q::one();
            for (dst, w) in IsraeliJalfon4.succ(x, 0) {
                if let Dst::Alive(y) = dst {
                    let j = idx(y);
                    m[i][j] -= w;
                }
            }
            m[i][n] = Q::one();
        }
        // Gaussian elimination, exact
        for col in 0..n {
            let piv = (col..n)
                .find(|&r| !m[r][col].is_zero())
                .expect("singular system");
            m.swap(col, piv);
            let p = m[col][col].clone();
            for k in col..=n {
                m[col][k] = m[col][k].clone() / p.clone();
            }
            for r in 0..n {
                if r != col && !m[r][col].is_zero() {
                    let f = m[r][col].clone();
                    for k in col..=n {
                        m[r][k] = m[r][k].clone() - f.clone() * m[col][k].clone();
                    }
                }
            }
        }
        let e = |c: u64| m[idx(c)][n].clone();
        // per-state values (symmetry-broken by the sequential blocks):
        // 2 on every 2-token configuration AND on {1,2,4}; 9/4 on
        // {1,2,3}; 5/2 on {1,3,4}; 73/32 on {2,3,4} and the full
        // configuration.
        for &x in &alive {
            let expected = match x {
                7 => q(9, 4),
                11 => q(2, 1),
                13 => q(5, 2),
                14 | 15 => q(73, 32),
                _ => q(2, 1),
            };
            assert_eq!(e(x), expected, "E at config {x:04b}");
        }
        // E[count] over the Bernoulli(1/2)^4 initial distribution
        let mut ev = Q::zero();
        for cfg in 0..16u64 {
            if ij4_tokens(cfg) >= 2 {
                ev += q(1, 16) * e(cfg);
            }
        }
        assert_eq!(ev, q(373, 256), "E[count] = 373/256");
    }

    /// Every weighted transfer row holds with EQUALITY at gamma = 1/2
    /// (the weights are eigen-tight on every row, so 1/2 is exactly
    /// the alive spectral radius and W(n) = (99/64) 2^-n exactly).
    #[test]
    fn israeli_jalfon4_weights_all_rows_tight() {
        let c = q(1, 2);
        let mut states: Vec<u64> = (0..16u64).filter(|&x| ij4_tokens(x) >= 2).collect();
        states.push(IJ4_START);
        for &x in &states {
            let lhs = IsraeliJalfon4
                .succ(x, 0)
                .iter()
                .fold(Q::zero(), |a, (dst, p)| match dst {
                    Dst::Alive(y) => a + p * ij4_weight(*y),
                    _ => a, // absorbed weight 0
                });
            assert_eq!(lhs, c.clone() * ij4_weight(x), "transfer row at {x}");
        }
        // eigen-equality of the weighted alive mass at the horizon
        let tr = evaluate(&IsraeliJalfon4, 31);
        let wn = tr
            .alive_support
            .iter()
            .fold(Q::zero(), |a, (x, m)| a + m * ij4_weight(*x));
        let mut expect = q(99, 64);
        for _ in 0..31 {
            expect = expect * q(1, 2);
        }
        assert_eq!(wn, expect, "W(31) = (99/64) 2^-31");
    }

    /// Structural teeth: token count never increases (the stabilized
    /// class is closed — the R18 no-re-entry recompute's ground), the
    /// last-firing block always leaves a token (alive rows never reach
    /// the empty configuration), and alive rows are total probability
    /// rows.
    #[test]
    fn israeli_jalfon4_no_reentry_structure() {
        for x in (0..16u64).filter(|&x| ij4_tokens(x) >= 2) {
            let row = IsraeliJalfon4.succ(x, 0);
            let total = row.iter().fold(Q::zero(), |a, (_, w)| a + w.clone());
            assert!(total.is_one(), "row at {x} sums to {total}");
            for (dst, _) in &row {
                match dst {
                    Dst::Alive(y) => {
                        assert!(ij4_tokens(*y) <= ij4_tokens(x), "{x:04b} -> {y:04b}");
                        assert!(ij4_tokens(*y) >= 2);
                    }
                    Dst::Done(0) => {}
                    other => panic!("unexpected dst {other:?}"),
                }
            }
            // never the empty configuration among successors
            for coins in 0..(1u64 << ij4_tokens(x)) {
                assert!(ij4_step(x, coins) != 0, "empty config from {x:04b}");
            }
        }
    }

    /// EV bracket containment at the matched-depth horizon: with the
    /// offset-1 partial survival sum L (the prologue transition is
    /// deterministic overhead: T = count + 1) and the alive-mass-face
    /// tail (kappa = 1, gamma = 1/2), the bracket contains the exact
    /// truth E[count] = 373/256 and its width is ~1.44e-9 — and the
    /// upper endpoint EQUALS the truth exactly: the states with
    /// weight above the unit floor are all transient within the
    /// first three iterations (the >= 3-token subgraph is
    /// {1,2,3,4} -> {2,3,4}/{1,2,3} -> {1,2,4} -> only 2-token), so
    /// P[T > t] = W(t) exactly for every t past the transient and the
    /// geometric tail bound is achieved, not just sound.
    #[test]
    fn israeli_jalfon4_ev_bracket_contains_truth() {
        let n = 31u32;
        let surv = survival_masses(&IsraeliJalfon4, n);
        assert!(surv[0].is_one(), "deterministic prologue: P[T > 0] = 1");
        assert_eq!(surv[1], q(11, 16), "P[T > 1] = P[count > 0] = 11/16");
        let l = surv[1..n as usize]
            .iter()
            .fold(Q::zero(), |a, b| a + b.clone());
        let mut wn = q(99, 64);
        for _ in 0..n {
            wn = wn * q(1, 2);
        }
        let tail = q(2, 1) * wn; // kappa = 1, 1/(1 - gamma) = 2
        let truth_ev = q(373, 256);
        assert!(l <= truth_ev, "lower {l} vs 373/256");
        assert_eq!(
            l.clone() + tail.clone(),
            truth_ev,
            "the upper endpoint is exactly the truth (achieved tail)"
        );
        use num_traits::ToPrimitive;
        let width = tail.to_f64().unwrap();
        assert!(width < 2e-9, "width {width}");
    }

    /// EV bracket containment: with L the exact partial survival sum
    /// at N = 80 and the tail bounded by kappa * W(N) / (1 - gamma)
    /// (kappa = 5, gamma = 4/5), the bracket contains the exact truth
    /// E[T] = 5 * H_5 = 137/12.
    #[test]
    fn coupon_collector5_ev_bracket_contains_truth() {
        let n = 80u32;
        let surv = survival_masses(&CouponCollector5, n);
        let l = surv[..n as usize]
            .iter()
            .fold(Q::zero(), |a, b| a + b.clone());
        let wn = q(4, 5).pow(n as i32); // eigenvector equality, tested above
        let tail = q(5, 1) * wn * q(5, 1); // kappa * W(N) * 1/(1 - gamma)
        let truth_ev = q(137, 12);
        assert!(l <= truth_ev, "lower {l} vs 137/12");
        assert!(truth_ev <= l.clone() + tail.clone(), "upper");
        // the bracket is tight: width = tail is ~4.4e-7
        use num_traits::ToPrimitive;
        let width = tail.to_f64().unwrap();
        assert!(width < 5e-7, "width {width}");
    }
}

pub mod differential;
pub mod ev;
pub mod ev3d;
pub mod spec;
