//! EV-bracket certificates for PATTERN-PRESENTED stored-row chains —
//! the d-dimensional uniform-axis ±1 walk family with monus at zero
//! and absorption at the origin (first instance: Diabolo's
//! `ours/3d-asym-rw`). This is the regional/template face of the same
//! draft strengthened generator-form seam as `ev.rs` (which carries
//! the finite-chain face): the alive-reachable set here is INFINITE,
//! so the weighted-transfer contraction cannot be discharged by
//! per-stored-row recomputation alone. The certificate presents:
//!
//!   - a finite stored-row WINDOW (every alive state with
//!     max-coordinate <= box+1 carries a stored row), checked
//!     per-row exactly, AND checked to REALIZE the declared pattern
//!     (stored rows = pattern rows, exact rationals);
//!   - a PRODUCT-PROFILE template for the weight function V outside
//!     the stored-V box: per-coordinate profile u with u(0) = 1,
//!     nondecreasing ratio list g(0..k), then a constant geometric
//!     growth ratio — V(x) = prod_i u(x_i);
//!   - the TEMPLATE COLLAPSE: writing r(v) for the per-coordinate
//!     transfer ratio (r(v) = p_up u(v+1)/u(v) + p_dn u(v-1)/u(v),
//!     r(0) = p_up u(1)/u(0) + p_dn by monus), every alive state x
//!     outside the box satisfies
//!       sum_y K(x,y) V(y) = (1/d) sum_i r(x_i) V(x)
//!     because exactly one coordinate moves and no absorption is
//!     reachable from outside the box; so the finitely many class
//!     inequalities  ((d-k) rmax + k rinf)/d <= c  (k = 1..d, rmax =
//!     max_v r(v), rinf the geometric-regime ratio) discharge the
//!     contraction at ALL of them at once. The collapse is sound
//!     exactly when no rowless alive state touches the stored-V box:
//!     the checker enforces window coverage at radius box+1, so every
//!     rowless state and all of its neighbors carry template V.
//!
//! Checker status: DEV ORACLE, same posture as `ev.rs` (the module
//! doc there states the seam's revision-event pendency; every
//! comparison row checked here is labeled pending that revision).
//! The window/pattern/collapse demand family is this case's addition
//! to the draft seam — its schema realization and the collapse
//! lemma's Lean twin ride the mint's needs list, recorded in the
//! campaign receipts.
//!
//! The claims and their mathematics (all exact rationals): with the
//! contraction verified at every alive state at rate c < 1 and
//! kappa = 1/min V over alive states, the survival mass obeys
//! P[T > n] <= kappa W(N) c^(n-N) for n >= N where W(N) is the
//! V-weighted alive mass at the horizon (exact from the DP), so
//!   E[T] = sum_(n>=0) P[T > n]  in  [L, L + kappa W(N)/(1-c)],
//! L the exact partial survival sum. c < 1 additionally certifies
//! P[T = infinity] = 0 (the claim `z-one` row: total row mass one and
//! geometric survival decay), which is what warrants reading the
//! survival-form bracket as the RAW FIRST MOMENT of the return-value
//! distribution — the baseline tool's EV objective.

use crate::ev::{parse_q, sha256_hex, Verdict};
use crate::Q;
use num_bigint::BigInt;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn rq(x: &Q) -> String {
    format!("{}/{}", x.numer(), x.denom())
}

fn qpow(x: &Q, e: u32) -> Q {
    let mut acc = Q::one();
    let mut base = x.clone();
    let mut e = e;
    while e > 0 {
        if e & 1 == 1 {
            acc *= base.clone();
        }
        base = base.clone() * base;
        e >>= 1;
    }
    acc
}

// ------------------------------------------------------- the pattern

/// The d = 3 instance's fixed parameters (the certificate carries
/// them; the checker recomputes everything from its own copy).
pub const P_UP: (i64, i64) = (1, 4);

/// Sorted-triple helpers (a <= b <= c), packed a | b<<12 | c<<24.
pub fn t3_pack(a: u64, b: u64, c: u64) -> u64 {
    debug_assert!(a <= b && b <= c && c < (1 << 12));
    a | (b << 12) | (c << 24)
}

pub fn t3_unpack(x: u64) -> (u64, u64, u64) {
    (x & 0xfff, (x >> 12) & 0xfff, (x >> 24) & 0xfff)
}

fn t3_sort(mut v: [u64; 3]) -> (u64, u64, u64) {
    v.sort_unstable();
    (v[0], v[1], v[2])
}

/// Pattern row at a sorted triple, MERGED by destination: up-moves at
/// p_up/3 per coordinate, monus down-moves at (1-p_up)/3, absorption
/// exactly at the origin. Weights are numerators over `scale` =
/// 3 * p_up_denominator (12 for the 1/4 walk). Returns (alive dsts,
/// absorbed numerator).
pub fn pattern_row(s: (u64, u64, u64), up_num: u64, scale: u64) -> (Vec<(u64, u64)>, u64) {
    let dn_num = scale / 3 - up_num; // per-coordinate: (1 - p_up)/3
    let coords = [s.0, s.1, s.2];
    let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
    let mut absorbed = 0u64;
    for k in 0..3 {
        let mut up = coords;
        up[k] += 1;
        let (a, b, c) = t3_sort(up);
        *acc.entry(t3_pack(a, b, c)).or_insert(0) += up_num;
        let mut dn = coords;
        dn[k] = dn[k].saturating_sub(1);
        let (a, b, c) = t3_sort(dn);
        if (a, b, c) == (0, 0, 0) {
            absorbed += dn_num;
        } else {
            *acc.entry(t3_pack(a, b, c)).or_insert(0) += dn_num;
        }
    }
    (acc.into_iter().collect(), absorbed)
}

// ------------------------------------------------------ the fast DP

/// Exact integer-scaled DP over the pattern chain from (1,1,1):
/// contiguous state interning, transitions compiled once (the chain
/// is time-homogeneous), masses as big-integer numerators over
/// scale^t. Returns per-step survival numerators, the horizon alive
/// support, and the absorbed-by-horizon numerator (single class).
pub struct Dp3d {
    pub scale: u64,
    /// survival numerators: P[T > t] = surv_num[t] / scale^t
    pub surv_num: Vec<BigInt>,
    /// alive support at the horizon: (packed state, numerator over
    /// scale^horizon)
    pub alive: Vec<(u64, BigInt)>,
    /// absorbed mass numerator over scale^horizon
    pub class_num: BigInt,
}

// Flat fixed-width little-endian limb arithmetic for the DP's hot
// loop: masses at layer t are integers < scale^t, so a width sized to
// scale^horizon holds every intermediate exactly, and the whole layer
// lives in two contiguous buffers — no per-edge allocation. The
// generic layered evaluator (`crate::evaluate`) is the independent
// oracle this arithmetic is tested against, exactly, in the module
// tests.
#[inline]
fn limb_add_mul(dst: &mut [u64], src: &[u64], w: u64) {
    // dst += src * w; src may be narrower than dst (the carry
    // propagates into dst's remaining limbs; falling off dst's end is
    // an exactness bug and panics in every build)
    let mut carry: u128 = 0;
    for (d, s) in dst.iter_mut().zip(src) {
        let cur = (*s as u128) * (w as u128) + (*d as u128) + carry;
        *d = cur as u64;
        carry = cur >> 64;
    }
    let mut i = src.len();
    while carry > 0 {
        assert!(i < dst.len(), "limb overflow");
        let cur = (dst[i] as u128) + carry;
        dst[i] = cur as u64;
        carry = cur >> 64;
        i += 1;
    }
}

#[inline]
fn limb_add(dst: &mut [u64], src: &[u64]) {
    let mut carry: u128 = 0;
    for (d, s) in dst.iter_mut().zip(src) {
        let cur = (*s as u128) + (*d as u128) + carry;
        *d = cur as u64;
        carry = cur >> 64;
    }
    let mut i = src.len();
    while carry > 0 {
        assert!(i < dst.len(), "limb overflow");
        let cur = (dst[i] as u128) + carry;
        dst[i] = cur as u64;
        carry = cur >> 64;
        i += 1;
    }
}

#[inline]
fn limb_mul_small(dst: &mut [u64], w: u64) {
    let mut carry: u128 = 0;
    for d in dst.iter_mut() {
        let cur = (*d as u128) * (w as u128) + carry;
        *d = cur as u64;
        carry = cur >> 64;
    }
    assert_eq!(carry, 0, "limb overflow");
}

fn limb_to_bigint(x: &[u64]) -> BigInt {
    let mut bytes = Vec::with_capacity(x.len() * 8);
    for l in x {
        bytes.extend_from_slice(&l.to_le_bytes());
    }
    BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes)
}

pub fn dp3d(horizon: u32, up_num: u64, scale: u64) -> Dp3d {
    use std::collections::HashMap;
    // width: scale^horizon < 2^(horizon * log2(scale)) — size with a
    // one-limb margin
    let bits = (horizon as f64 + 1.0) * (scale as f64).log2();
    let limbs = (bits / 64.0).ceil() as usize + 1;

    let init = t3_pack(1, 1, 1);
    let mut index: HashMap<u64, u32> = HashMap::new();
    let mut packed: Vec<u64> = Vec::new();
    // transitions[i] = (alive dsts as (index, num), absorbed num)
    let mut trans: Vec<Option<(Vec<(u32, u64)>, u64)>> = Vec::new();
    let intern = |x: u64,
                  index: &mut HashMap<u64, u32>,
                  packed: &mut Vec<u64>,
                  trans: &mut Vec<Option<(Vec<(u32, u64)>, u64)>>|
     -> u32 {
        *index.entry(x).or_insert_with(|| {
            packed.push(x);
            trans.push(None);
            (packed.len() - 1) as u32
        })
    };
    let i0 = intern(init, &mut index, &mut packed, &mut trans);
    debug_assert_eq!(i0, 0);

    // compact per-occupied mass rows at the current layer's ACTIVE
    // width (masses at layer t are < scale^t — the stride grows with
    // t); full-width next buffer indexed by global state id, zeroed
    // only where touched
    let mut mass: Vec<u64> = vec![1];
    let mut stride: usize = 1;
    let mut occupied: Vec<u32> = vec![i0];
    let mut next: Vec<u64> = Vec::new();
    let mut surv_num: Vec<BigInt> = vec![BigInt::one()];
    let mut class_acc: Vec<u64> = vec![0; limbs];

    let log2_scale = (scale as f64).log2();
    for t in 0..horizon {
        for &i in &occupied {
            if trans[i as usize].is_none() {
                let (alive_dsts, absorbed) =
                    pattern_row(t3_unpack(packed[i as usize]), up_num, scale);
                let compiled: Vec<(u32, u64)> = alive_dsts
                    .into_iter()
                    .map(|(d, w)| (intern(d, &mut index, &mut packed, &mut trans), w))
                    .collect();
                trans[i as usize] = Some((compiled, absorbed));
            }
        }
        if next.len() < packed.len() * limbs {
            next.resize(packed.len() * limbs, 0);
        }
        // masses at layer t+1 are < scale^(t+1): only this many limbs
        // can be nonzero, so the hot loop runs at the ACTIVE width
        let active = ((((t + 1) as f64 * log2_scale) / 64.0).ceil() as usize + 1).min(limbs);
        let mut next_occ: Vec<u32> = Vec::with_capacity(occupied.len() * 2);
        let mut touched: Vec<bool> = vec![false; packed.len()];
        limb_mul_small(&mut class_acc, scale);
        for (pos, &i) in occupied.iter().enumerate() {
            let m = &mass[pos * stride..(pos + 1) * stride];
            let (dsts, absorbed) = trans[i as usize].as_ref().unwrap();
            for &(j, w) in dsts {
                let ju = j as usize;
                if !touched[ju] {
                    touched[ju] = true;
                    next_occ.push(j);
                    next[ju * limbs..ju * limbs + active].fill(0);
                }
                limb_add_mul(&mut next[ju * limbs..ju * limbs + active], m, w);
            }
            if *absorbed > 0 {
                limb_add_mul(&mut class_acc, m, *absorbed);
            }
        }
        next_occ.sort_unstable();
        let mut new_mass: Vec<u64> = Vec::with_capacity(next_occ.len() * active);
        let mut total: Vec<u64> = vec![0; active];
        for &j in &next_occ {
            let row = &next[j as usize * limbs..j as usize * limbs + active];
            limb_add(&mut total, row);
            new_mass.extend_from_slice(row);
        }
        occupied = next_occ;
        mass = new_mass;
        stride = active;
        surv_num.push(limb_to_bigint(&total));
    }

    let alive: Vec<(u64, BigInt)> = occupied
        .iter()
        .enumerate()
        .map(|(pos, &i)| {
            (
                packed[i as usize],
                limb_to_bigint(&mass[pos * stride..(pos + 1) * stride]),
            )
        })
        .collect();
    Dp3d {
        scale,
        surv_num,
        alive,
        class_num: limb_to_bigint(&class_acc),
    }
}

impl Dp3d {
    pub fn survivals(&self) -> Vec<Q> {
        let mut den = BigInt::one();
        let mut out = Vec::with_capacity(self.surv_num.len());
        for n in &self.surv_num {
            out.push(Q::new(n.clone(), den.clone()));
            den *= self.scale;
        }
        out
    }
    pub fn partial_survival_sum(&self) -> Q {
        // L = sum_{t < horizon} P[T > t] = Horner over the survival
        // numerators (one big-integer pass, one reduction at the end —
        // the naive rational fold re-normalized at every add)
        let n = self.surv_num.len() - 1;
        let mut acc = BigInt::zero();
        for k in &self.surv_num[..n] {
            acc = acc * self.scale + k;
        }
        let mut den = BigInt::one();
        for _ in 0..n.saturating_sub(1) {
            den *= self.scale;
        }
        Q::new(acc, den)
    }

    /// V-weighted alive mass at the horizon, exactly. The closure
    /// supplies each state's weight as an UNREDUCED numerator/
    /// denominator pair (no per-state gcd normalization); terms are
    /// grouped by denominator so the accumulation is big-integer
    /// multiply-adds, with one rational reduction per distinct
    /// denominator instead of one per state.
    pub fn weighted_alive<F: FnMut(u64) -> (BigInt, BigInt)>(&self, mut v_of: F) -> Q {
        use std::collections::HashMap;
        let mut den = BigInt::one();
        for _ in 0..self.surv_num.len() - 1 {
            den *= self.scale;
        }
        let mut acc: HashMap<BigInt, BigInt> = HashMap::new();
        for (x, num) in &self.alive {
            let (vn, vd) = v_of(*x);
            *acc.entry(vd).or_insert_with(BigInt::zero) += num * vn;
        }
        let mut out = Q::zero();
        for (vd, sum) in acc {
            out += Q::new(sum, &den * vd);
        }
        out
    }
}

// ------------------------------------------------------ the profile

/// Per-coordinate template profile: u(0) = 1, u(v+1) = u(v) g(v) for
/// v < ratios.len(), then u(v+1) = u(v) * growth.
pub struct Profile {
    pub ratios: Vec<Q>,
    pub growth: Q,
}

impl Profile {
    pub fn u(&self, v: u64) -> Q {
        let mut x = Q::one();
        for k in 0..v {
            x *= if (k as usize) < self.ratios.len() {
                self.ratios[k as usize].clone()
            } else {
                self.growth.clone()
            };
        }
        x
    }
    /// u-values 0..=vmax as a table (incremental products).
    pub fn u_table(&self, vmax: u64) -> Vec<Q> {
        let mut out = Vec::with_capacity(vmax as usize + 1);
        let mut x = Q::one();
        out.push(x.clone());
        for k in 0..vmax {
            x *= if (k as usize) < self.ratios.len() {
                self.ratios[k as usize].clone()
            } else {
                self.growth.clone()
            };
            out.push(x.clone());
        }
        out
    }
    /// Per-coordinate transfer ratios r(0..=ratios.len()) — beyond
    /// that index every r(v) equals `rinf(p)` exactly (both u-ratios
    /// sit in the geometric regime).
    pub fn r_values(&self, p_up: &Q) -> Vec<Q> {
        let p_dn = Q::one() - p_up.clone();
        let vmax = self.ratios.len() as u64 + 1;
        let u = self.u_table(vmax + 1);
        let mut out = Vec::new();
        // r(0): up to u(1)/u(0), monus stay at weight p_dn
        out.push(p_up.clone() * u[1].clone() / u[0].clone() + p_dn.clone());
        for v in 1..=self.ratios.len() as u64 {
            out.push(
                p_up.clone() * u[(v + 1) as usize].clone() / u[v as usize].clone()
                    + p_dn.clone() * u[(v - 1) as usize].clone() / u[v as usize].clone(),
            );
        }
        out
    }
    pub fn rinf(&self, p_up: &Q) -> Q {
        let p_dn = Q::one() - p_up.clone();
        p_up.clone() * self.growth.clone() + p_dn / self.growth.clone()
    }
}

/// The registered template for the 3d walk: ramp ratios chosen so the
/// class floor (2 rmax + rinf)/3 sits under the claimed c = 193/200,
/// with the near-origin stored V solved at rate 24/25 against the
/// template boundary. Untrusted search, like every producer
/// ingredient: the checker recomputes all of it.
pub fn profile_3d() -> Profile {
    let f = |n: i64, d: i64| Q::new(BigInt::from(n), BigInt::from(d));
    Profile {
        ratios: vec![f(1, 1), f(21, 20), f(23, 20), f(27, 20), f(31, 20), f(7, 4)],
        growth: f(7, 4),
    }
}

pub const EV3D_C: (i64, i64) = (193, 200);
pub const EV3D_BOX: u64 = 10;
pub const EV3D_SOLVE_C: f64 = 0.96;

// ------------------------------------------------- producer emission

/// Solve the stored-V box against the template boundary at the solve
/// rate (floats), then round to dyadic rationals. Deterministic:
/// fixed iteration count, sorted state order.
fn solve_vbox(profile: &Profile, boxb: u64, cprime: f64) -> Vec<(u64, Q)> {
    let mut states: Vec<(u64, u64, u64)> = Vec::new();
    for a in 0..=boxb {
        for b in a..=boxb {
            for c in b..=boxb {
                if (a, b, c) != (0, 0, 0) {
                    states.push((a, b, c));
                }
            }
        }
    }
    let idx: BTreeMap<(u64, u64, u64), usize> =
        states.iter().enumerate().map(|(i, s)| (*s, i)).collect();
    let ut = profile.u_table(boxb + 2);
    let uf: Vec<f64> = ut.iter().map(|q| q.to_f64().unwrap()).collect();
    let tmpl = |s: (u64, u64, u64)| uf[s.0 as usize] * uf[s.1 as usize] * uf[s.2 as usize];

    // compiled transitions: (inside index | template value, weight/12)
    enum Dst {
        In(usize),
        Out(f64),
    }
    let mut trans: Vec<Vec<(Dst, f64)>> = Vec::with_capacity(states.len());
    for &s in &states {
        let (alive, _absorbed) = pattern_row(s, 1, 12);
        let mut outs = Vec::new();
        for (d, w) in alive {
            let t = t3_unpack(d);
            let w = w as f64 / 12.0;
            match idx.get(&t) {
                Some(&j) => outs.push((Dst::In(j), w)),
                None => outs.push((Dst::Out(tmpl(t)), w)),
            }
        }
        trans.push(outs);
    }
    let mut v: Vec<f64> = states.iter().map(|&s| tmpl(s)).collect();
    for _ in 0..8000 {
        let mut nv = vec![0.0f64; v.len()];
        for (i, outs) in trans.iter().enumerate() {
            let mut acc = 0.0;
            for (d, w) in outs {
                acc += w * match d {
                    Dst::In(j) => v[*j],
                    Dst::Out(t) => *t,
                };
            }
            nv[i] = acc / cprime;
        }
        v = nv;
    }
    // dyadic rounding at 2^-40
    let scale = (1u64 << 40) as f64;
    states
        .iter()
        .zip(v)
        .map(|(&s, x)| {
            let num = (x * scale).round() as i128;
            (
                t3_pack(s.0, s.1, s.2),
                Q::new(BigInt::from(num.max(1)), BigInt::from(1u64 << 40)),
            )
        })
        .collect()
}

fn state_name(x: u64) -> String {
    let (a, b, c) = t3_unpack(x);
    format!("s{a}_{b}_{c}")
}

/// Emit the EV certificate (deterministic text form) for the 3d walk
/// at `horizon`, binding it to the exact spec bytes the container
/// emitter seals. The emitter self-checks: it runs the full checker
/// on its own output and panics on anything but ACCEPT (an untrusted
/// producer, but not a knowingly-broken one).
pub fn emit_evcert_3d(horizon: u32, spec_bytes: &[u8]) -> String {
    let t0 = std::time::Instant::now();
    let profile = profile_3d();
    let c = Q::new(BigInt::from(EV3D_C.0), BigInt::from(EV3D_C.1));
    let weights = solve_vbox(&profile, EV3D_BOX, EV3D_SOLVE_C);
    eprintln!("emit: solve_vbox {:.3}s", t0.elapsed().as_secs_f64());

    let t1 = std::time::Instant::now();
    let dp = dp3d(horizon, 1, 12);
    eprintln!("emit: dp3d {:.3}s", t1.elapsed().as_secs_f64());
    let t2 = std::time::Instant::now();
    let l = dp.partial_survival_sum();
    eprintln!("emit: partial {:.3}s", t2.elapsed().as_secs_f64());

    // W(N) = <m_N, V> exactly; V = stored on the box, template beyond
    let wmap: BTreeMap<u64, &Q> = weights.iter().map(|(x, w)| (*x, w)).collect();
    let maxc = dp
        .alive
        .iter()
        .map(|(x, _)| t3_unpack(*x).2)
        .max()
        .unwrap_or(0);
    let ut = profile.u_table(maxc + 1);
    let un: Vec<BigInt> = ut.iter().map(|q| q.numer().clone()).collect();
    let ud: Vec<BigInt> = ut.iter().map(|q| q.denom().clone()).collect();
    let t3 = std::time::Instant::now();
    let wn = dp.weighted_alive(|x| match wmap.get(&x) {
        Some(w) => (w.numer().clone(), w.denom().clone()),
        None => {
            let (a, b, cc) = t3_unpack(x);
            (
                &un[a as usize] * &un[b as usize] * &un[cc as usize],
                &ud[a as usize] * &ud[b as usize] * &ud[cc as usize],
            )
        }
    });
    eprintln!("emit: wdot {:.3}s", t3.elapsed().as_secs_f64());

    let wmin = weights
        .iter()
        .map(|(_, w)| w.clone())
        .min()
        .expect("nonempty box");
    let wmin = wmin.min(Q::one()); // template values are >= 1
    let kappa = Q::one() / wmin;
    let one_minus = Q::one() - c.clone();
    let tail = kappa.clone() * wn.clone() / one_minus;
    let tail_coeff = kappa * wn.clone() / qpow(&c, horizon);

    let mut out = String::new();
    let _ = writeln!(out, "evcert v1");
    let _ = writeln!(out, "program 3d-asym-rw");
    let _ = writeln!(out, "spec-sha256 {}", sha256_hex(spec_bytes));
    let _ = writeln!(out, "horizon {horizon}");
    let _ = writeln!(out, "pattern walk3d-monus 3 {}/{}", P_UP.0, P_UP.1);
    let _ = writeln!(out, "c {}", rq(&c));
    let _ = writeln!(out, "box {}", EV3D_BOX);
    let _ = writeln!(out, "growth {}", rq(&profile.growth));
    for (k, g) in profile.ratios.iter().enumerate() {
        let _ = writeln!(out, "profile-ratio {k} {}", rq(g));
    }
    for (x, w) in &weights {
        let _ = writeln!(out, "weight {} {}", state_name(*x), rq(w));
    }
    let _ = writeln!(out, "claim partial-lo {}", rq(&l));
    let _ = writeln!(out, "claim ev {} {}", rq(&l), rq(&(l.clone() + tail)));
    let _ = writeln!(out, "claim tail {} {}", rq(&tail_coeff), rq(&c));
    let _ = writeln!(out, "claim z-one");

    out
}

// ------------------------------------------------------- the checker

struct Cert3d {
    program: String,
    spec_sha256: String,
    horizon: u32,
    dims: u32,
    p_up: Q,
    c: Q,
    boxb: u64,
    growth: Q,
    ratios: Vec<(u64, Q)>,
    weights: BTreeMap<(u64, u64, u64), Q>,
    partial_lo: Q,
    ev: (Q, Q),
    tail: (Q, Q),
    z_one: bool,
}

fn parse_triple(name: &str) -> Option<(u64, u64, u64)> {
    let rest = name.strip_prefix('s')?;
    let mut it = rest.split('_');
    let a: u64 = it.next()?.parse().ok()?;
    let b: u64 = it.next()?.parse().ok()?;
    let c: u64 = it.next()?.parse().ok()?;
    if it.next().is_some() || a > b || b > c {
        return None;
    }
    Some((a, b, c))
}

fn parse_cert_3d(text: &str) -> Result<Cert3d, String> {
    let mut program = None;
    let mut sha = None;
    let mut horizon = None;
    let mut dims = None;
    let mut p_up = None;
    let mut c = None;
    let mut boxb = None;
    let mut growth = None;
    let mut ratios: Vec<(u64, Q)> = Vec::new();
    let mut weights: BTreeMap<(u64, u64, u64), Q> = BTreeMap::new();
    let mut partial = None;
    let mut ev = None;
    let mut tail = None;
    let mut z_one = false;
    let mut lines = text.lines();
    if lines.next() != Some("evcert v1") {
        return Err("missing 'evcert v1' header".into());
    }
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        match toks.as_slice() {
            ["program", p] => program = Some(p.to_string()),
            ["spec-sha256", h] => sha = Some(h.to_string()),
            ["horizon", n] => {
                horizon = Some(n.parse::<u32>().map_err(|e| format!("horizon: {e}"))?)
            }
            ["pattern", "walk3d-monus", d, p] => {
                dims = Some(d.parse::<u32>().map_err(|e| format!("dims: {e}"))?);
                p_up = Some(parse_q(p).ok_or("p_up: bad fraction")?);
            }
            ["c", v] => c = Some(parse_q(v).ok_or("c: bad fraction")?),
            ["box", v] => boxb = Some(v.parse::<u64>().map_err(|e| format!("box: {e}"))?),
            ["growth", v] => growth = Some(parse_q(v).ok_or("growth: bad fraction")?),
            ["profile-ratio", k, v] => ratios.push((
                k.parse::<u64>().map_err(|e| format!("ratio index: {e}"))?,
                parse_q(v).ok_or("ratio: bad fraction")?,
            )),
            ["weight", s, w] => {
                let t = parse_triple(s).ok_or_else(|| format!("weight: bad state name {s}"))?;
                let w = parse_q(w).ok_or("weight: bad fraction")?;
                if weights.insert(t, w).is_some() {
                    return Err(format!("duplicate weight line for {s}"));
                }
            }
            ["claim", "partial-lo", v] => {
                partial = Some(parse_q(v).ok_or("partial-lo: bad fraction")?)
            }
            ["claim", "ev", lo, hi] => {
                ev = Some((
                    parse_q(lo).ok_or("ev lo: bad fraction")?,
                    parse_q(hi).ok_or("ev hi: bad fraction")?,
                ))
            }
            ["claim", "tail", a, g] => {
                tail = Some((
                    parse_q(a).ok_or("tail coeff: bad fraction")?,
                    parse_q(g).ok_or("tail rate: bad fraction")?,
                ))
            }
            ["claim", "z-one"] => z_one = true,
            other => return Err(format!("unrecognized line: {other:?}")),
        }
    }
    // ratio indices must be exactly 0..n in order
    for (k, (i, _)) in ratios.iter().enumerate() {
        if *i != k as u64 {
            return Err(format!(
                "profile-ratio indices must be 0..n; saw {i} at {k}"
            ));
        }
    }
    Ok(Cert3d {
        program: program.ok_or("missing program")?,
        spec_sha256: sha.ok_or("missing spec-sha256")?,
        horizon: horizon.ok_or("missing horizon")?,
        dims: dims.ok_or("missing pattern")?,
        p_up: p_up.ok_or("missing pattern")?,
        c: c.ok_or("missing c")?,
        boxb: boxb.ok_or("missing box")?,
        growth: growth.ok_or("missing growth")?,
        ratios,
        weights,
        partial_lo: partial.ok_or("missing claim partial-lo")?,
        ev: ev.ok_or("missing claim ev")?,
        tail: tail.ok_or("missing claim tail")?,
        z_one,
    })
}

/// The dev-oracle check for the pattern face: recompute everything —
/// stored rows against the declared pattern, window coverage at
/// radius box+1, per-row transfer inequalities with the mixed
/// stored/template V, the template-collapse class inequalities, the
/// exact pattern DP, and the claim discharges. Verdict classes as in
/// `ev.rs`.
pub fn check_evcert_3d(spec_bytes: &[u8], cert_text: &str) -> Verdict {
    let cert = match parse_cert_3d(cert_text) {
        Ok(c) => c,
        Err(e) => return Verdict::Refuse(format!("cert shape: {e}")),
    };
    if cert.program != "3d-asym-rw" {
        return Verdict::Refuse(format!("unknown pattern program {}", cert.program));
    }
    if cert.dims != 3 {
        return Verdict::Refuse("pattern walk3d-monus demands dims = 3".into());
    }

    // binding
    let actual = sha256_hex(spec_bytes);
    if actual != cert.spec_sha256 {
        return Verdict::Refuse(format!(
            "spec binding: cert names {} but the spec bytes hash to {actual}",
            cert.spec_sha256
        ));
    }

    // side conditions on the template and rate
    if cert.p_up <= Q::zero() || cert.p_up >= Q::one() {
        return Verdict::Refuse("pattern demands 0 < p_up < 1".into());
    }
    if cert.c <= Q::zero() || cert.c >= Q::one() {
        return Verdict::Refuse("side condition 0 < c < 1 violated".into());
    }
    if cert.growth < Q::one() {
        return Verdict::Refuse("template growth must be >= 1 (unit floor)".into());
    }
    for (_, g) in &cert.ratios {
        if *g < Q::one() {
            return Verdict::Refuse(
                "profile ratios must be >= 1 (unit floor over the template region)".into(),
            );
        }
    }
    if cert.boxb < 1 {
        return Verdict::Refuse("box must be >= 1 (the origin's neighbors need stored V)".into());
    }
    // Reach demand (kernel-checked necessity): the template-collapse
    // class inequalities cover exterior states only where every
    // coordinate is past the ramp (r(v) = rinf exactly); a profile
    // longer than box + 1 would leave ramp-resident exterior states
    // (some coordinate still inside the ramp, max coordinate beyond
    // the box) covered by NEITHER the per-row window checks NOR the
    // class checks. The shipped certificate satisfies it with room
    // (6 ratios, box 10).
    if cert.ratios.len() as u64 > cert.boxb + 1 {
        return Verdict::Refuse(format!(
            "profile ratios beyond the stored box: {} ratios with box {} — \
             ramp-resident exterior states would escape the class checks \
             (need ratios <= box + 1)",
            cert.ratios.len(),
            cert.boxb
        ));
    }

    // scale: rows are numerators over 3 * denom(p_up); demand exact
    let pd = cert.p_up.denom().to_u64().unwrap_or(0);
    let pn = cert.p_up.numer().to_u64().unwrap_or(0);
    if pd == 0 || pn == 0 {
        return Verdict::Refuse("p_up out of range".into());
    }
    let scale = 3 * pd;

    // ---- spec parse (same minimal reader as the finite face) ----
    let spec = match crate::ev::parse_spec_json(spec_bytes) {
        Ok(v) => v,
        Err(e) => return Verdict::Refuse(format!("spec parse: {e}")),
    };
    let (states, rows, init, classes, rejects, horizon) = match crate::ev::spec_tables(&spec) {
        Ok(t) => t,
        Err(e) => return Verdict::Refuse(format!("spec: {e}")),
    };
    // NOTE deliberate decoupling: the spec's `horizon` parameterizes
    // the CONTAINER's absorb-bracket claims (the fast-path checker's
    // exact evaluation cost grows with it); the EV certificate's
    // pattern DP runs at the certificate's own horizon. The sha
    // binding ties the two artifacts to the same CHAIN bytes — the
    // horizon is a per-query parameter, not chain content.
    let _ = horizon;
    if !rejects.is_empty() {
        return Verdict::Refuse("pattern face is score-free: rejects must be empty".into());
    }
    if classes.len() != 1 {
        return Verdict::Refuse("pattern face expects exactly one terminal class".into());
    }
    let done = classes.iter().next().unwrap().clone();

    // decode state names; class state exempt
    let mut triples: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
    for s in &states {
        if *s == done {
            continue;
        }
        match parse_triple(s) {
            Some(t) => {
                if t == (0, 0, 0) {
                    return Verdict::Refuse(
                        "origin listed as an alive state (region/absorption inconsistency)".into(),
                    );
                }
                triples.insert(s.clone(), t);
            }
            None => return Verdict::Refuse(format!("state {s} is not a sorted triple")),
        }
    }

    // init must be exactly (1,1,1) with mass one
    if init.len() != 1 {
        return Verdict::Refuse("init must be a single anchor".into());
    }
    let (init_name, init_mass) = init.iter().next().unwrap();
    if triples.get(init_name) != Some(&(1, 1, 1)) || !init_mass.is_one() {
        return Verdict::Refuse("init must be s1_1_1 with mass 1".into());
    }

    // ---- stored rows realize the pattern, exactly ----
    let scale_q = Q::new(BigInt::from(scale), BigInt::one());
    for (src, row) in &rows {
        let t = match triples.get(src) {
            Some(t) => *t,
            None => {
                return Verdict::Refuse(format!("stored row at non-alive or unknown state {src}"))
            }
        };
        let (alive_dsts, absorbed) = pattern_row(t, pn, scale);
        let mut expect: BTreeMap<String, Q> = alive_dsts
            .into_iter()
            .map(|(d, w)| {
                (
                    state_name(d),
                    Q::new(BigInt::from(w), BigInt::one()) / &scale_q,
                )
            })
            .collect();
        if absorbed > 0 {
            expect.insert(
                done.clone(),
                Q::new(BigInt::from(absorbed), BigInt::one()) / &scale_q,
            );
        }
        let mut got: BTreeMap<String, Q> = BTreeMap::new();
        for (d, w) in row {
            *got.entry(d.clone()).or_insert_with(Q::zero) += w.clone();
        }
        if got != expect {
            return Verdict::Refuse(format!(
                "stored row at {src} does not realize the declared pattern"
            ));
        }
    }

    // ---- window coverage: every alive triple with max <= box+1 has
    // a stored row (so rowless alive states and all their neighbors
    // sit in the pure-template region) ----
    let rowful: std::collections::BTreeSet<(u64, u64, u64)> =
        rows.keys().map(|s| triples[s]).collect();
    let bb = cert.boxb + 1;
    for a in 0..=bb {
        for b in a..=bb {
            for c in b..=bb {
                if (a, b, c) == (0, 0, 0) {
                    continue;
                }
                if !rowful.contains(&(a, b, c)) {
                    return Verdict::Refuse(format!(
                        "window coverage: alive state ({a},{b},{c}) inside radius box+1 \
                         has no stored row"
                    ));
                }
            }
        }
    }

    // ---- weights: exactly the box states, all positive ----
    for a in 0..=cert.boxb {
        for b in a..=cert.boxb {
            for c in b..=cert.boxb {
                if (a, b, c) == (0, 0, 0) {
                    continue;
                }
                match cert.weights.get(&(a, b, c)) {
                    Some(w) if *w > Q::zero() => {}
                    Some(_) => {
                        return Verdict::Refuse(format!(
                            "stored weight at ({a},{b},{c}) must be positive"
                        ))
                    }
                    None => {
                        return Verdict::Refuse(format!(
                            "stored weight missing at box state ({a},{b},{c})"
                        ))
                    }
                }
            }
        }
    }
    for (t, _) in &cert.weights {
        if t.2 > cert.boxb {
            return Verdict::Refuse(format!(
                "weight at ({},{},{}) outside the declared box",
                t.0, t.1, t.2
            ));
        }
    }

    // ---- template machinery ----
    let profile = Profile {
        ratios: cert.ratios.iter().map(|(_, g)| g.clone()).collect(),
        growth: cert.growth.clone(),
    };
    let rvals = profile.r_values(&cert.p_up);
    let rinf = profile.rinf(&cert.p_up);
    let rmax = rvals
        .iter()
        .cloned()
        .fold(rinf.clone(), |a, b| if b > a { b } else { a });
    // collapse classes: k in 1..=3 coordinates in the geometric
    // far-field, the rest anywhere on the profile
    let three_c = Q::new(BigInt::from(3), BigInt::one()) * cert.c.clone();
    for k in 1..=3u32 {
        let lhs = Q::new(BigInt::from(3 - k), BigInt::one()) * rmax.clone()
            + Q::new(BigInt::from(k), BigInt::one()) * rinf.clone();
        if lhs > three_c {
            return Verdict::Fail(format!(
                "template collapse refuted at class k={k}: {lhs} > 3c"
            ));
        }
    }

    // ---- per-row transfer inequalities on the stored window ----
    let maxc_needed = bb + 2;
    let ut = profile.u_table(maxc_needed + 1);
    let v_of = |t: (u64, u64, u64)| -> Q {
        if t.2 <= cert.boxb {
            cert.weights[&t].clone()
        } else {
            ut[t.0 as usize].clone() * ut[t.1 as usize].clone() * ut[t.2 as usize].clone()
        }
    };
    for (src, _) in &rows {
        let t = triples[src];
        let (alive_dsts, _absorbed) = pattern_row(t, pn, scale);
        let lhs = alive_dsts.iter().fold(Q::zero(), |a, (d, w)| {
            a + Q::new(BigInt::from(*w), BigInt::from(scale)) * v_of(t3_unpack(*d))
        });
        let rhs = cert.c.clone() * v_of(t);
        if lhs > rhs {
            return Verdict::Fail(format!(
                "transfer inequality refuted at {src}: {lhs} > {rhs}"
            ));
        }
    }

    // ---- exact pattern DP ----
    let dp = dp3d(cert.horizon, pn, scale);
    let l_exact = dp.partial_survival_sum();
    let maxc = dp
        .alive
        .iter()
        .map(|(x, _)| t3_unpack(*x).2)
        .max()
        .unwrap_or(0);
    let ut_far = profile.u_table(maxc + 1);
    let un_far: Vec<BigInt> = ut_far.iter().map(|q| q.numer().clone()).collect();
    let ud_far: Vec<BigInt> = ut_far.iter().map(|q| q.denom().clone()).collect();
    let w_exact = dp.weighted_alive(|x| {
        let t = t3_unpack(x);
        if t.2 <= cert.boxb {
            let w = &cert.weights[&t];
            (w.numer().clone(), w.denom().clone())
        } else {
            (
                &un_far[t.0 as usize] * &un_far[t.1 as usize] * &un_far[t.2 as usize],
                &ud_far[t.0 as usize] * &ud_far[t.1 as usize] * &ud_far[t.2 as usize],
            )
        }
    });

    let wmin = cert
        .weights
        .values()
        .cloned()
        .min()
        .unwrap_or_else(Q::one)
        .min(Q::one());
    let kappa = Q::one() / wmin;

    // ---- claims ----
    if cert.partial_lo > l_exact {
        return Verdict::Fail(format!(
            "partial-lo {} above the exact partial survival sum {}",
            cert.partial_lo, l_exact
        ));
    }
    let tail_bound = kappa.clone() * w_exact.clone() / (Q::one() - cert.c.clone());
    let (ev_lo, ev_hi) = &cert.ev;
    if *ev_lo > l_exact {
        return Verdict::Fail(format!("ev lo {ev_lo} above warranted lower {l_exact}"));
    }
    let warranted_hi = l_exact.clone() + tail_bound;
    if *ev_hi < warranted_hi {
        return Verdict::Fail(format!(
            "ev hi {ev_hi} below warranted upper {warranted_hi}"
        ));
    }
    let (tail_a, tail_g) = &cert.tail;
    if *tail_g < cert.c || *tail_g >= Q::one() {
        return Verdict::Fail(format!(
            "tail rate {tail_g} not in [c, 1) = [{}, 1)",
            cert.c
        ));
    }
    if tail_a.clone() * qpow(tail_g, cert.horizon) < kappa * w_exact {
        return Verdict::Fail(
            "tail coefficient below the warranted survival envelope at n = horizon".into(),
        );
    }
    if !cert.z_one {
        return Verdict::Refuse(
            "missing claim z-one (the raw-moment reading of the survival bracket \
             rides the certified termination row)"
                .into(),
        );
    }
    // z-one warrant: score-free (checked), stochastic pattern rows
    // (by construction: up + down = 1), c < 1 (checked) — discharged.

    Verdict::Accept { claims: (1, 1, 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::emit_spec_3d_window;
    use crate::{evaluate, q, survival_masses, AsymRw3d, Chain};

    /// The fast compiled DP against the generic layered oracle,
    /// exactly, at a horizon where both are cheap.
    #[test]
    fn dp3d_matches_generic_oracle() {
        let h = 40u32;
        let dp = dp3d(h, 1, 12);
        let surv_fast = dp.survivals();
        let surv_slow = survival_masses(&AsymRw3d, h);
        assert_eq!(surv_fast, surv_slow);
        let tr = evaluate(&AsymRw3d, h);
        let mut den = num_bigint::BigInt::one();
        for _ in 0..h {
            den *= 12u64;
        }
        assert_eq!(Q::new(dp.class_num.clone(), den), tr.class_mass[&0]);
        assert_eq!(dp.alive.len(), tr.alive_support.len());
    }

    /// Exact small-time absorption structure: the walk needs three
    /// steps to die, and P[T = 3] = 3! * (1/4)^3 = 3/32 exactly.
    #[test]
    fn dp3d_small_time_masses_exact() {
        let dp = dp3d(4, 1, 12);
        let surv = dp.survivals();
        assert!(surv[0].is_one() && surv[1].is_one() && surv[2].is_one());
        assert_eq!(surv[3], Q::one() - q(3, 32));
    }

    /// One-step tuple-vs-lump oracle: evolve FULL ordered triples by a
    /// direct transcription of the source's move law (axis, up/monus)
    /// and compare against the sorted-lumped pattern rows.
    #[test]
    fn pattern_row_matches_tuple_transcription() {
        for &(x, y, z) in &[
            (1u64, 1, 1),
            (0, 0, 1),
            (0, 1, 1),
            (0, 0, 5),
            (2, 3, 5),
            (0, 2, 2),
            (1, 1, 7),
        ] {
            // direct: 6 outcomes over ordered tuples, mass 1/12 up and
            // 3/12 down per axis
            let mut direct: std::collections::BTreeMap<Option<u64>, u64> = Default::default();
            for k in 0..3 {
                let mut up = [x, y, z];
                up[k] += 1;
                let (a, b, c) = {
                    let mut v = up;
                    v.sort_unstable();
                    (v[0], v[1], v[2])
                };
                *direct.entry(Some(t3_pack(a, b, c))).or_insert(0) += 1;
                let mut dn = [x, y, z];
                dn[k] = dn[k].saturating_sub(1);
                let mut v = dn;
                v.sort_unstable();
                let key = if (v[0], v[1], v[2]) == (0, 0, 0) {
                    None
                } else {
                    Some(t3_pack(v[0], v[1], v[2]))
                };
                *direct.entry(key).or_insert(0) += 3;
            }
            let (alive, absorbed) = pattern_row(
                {
                    let mut v = [x, y, z];
                    v.sort_unstable();
                    (v[0], v[1], v[2])
                },
                1,
                12,
            );
            let mut lumped: std::collections::BTreeMap<Option<u64>, u64> =
                alive.into_iter().map(|(d, w)| (Some(d), w)).collect();
            if absorbed > 0 {
                lumped.insert(None, absorbed);
            }
            assert_eq!(direct, lumped, "one-step divergence at ({x},{y},{z})");
        }
    }

    /// The generic Chain face and the pattern rows agree (the oracle
    /// chain is built FROM pattern_row, so this is a wiring check).
    #[test]
    fn chain_rows_are_probability_rows() {
        for &s in &[(0u64, 0, 1), (1, 1, 1), (0, 3, 9), (2, 2, 2)] {
            let x = t3_pack(s.0, s.1, s.2);
            let total = AsymRw3d
                .succ(x, 0)
                .iter()
                .fold(Q::zero(), |a, (_, w)| a + w.clone());
            assert!(total.is_one(), "row at {s:?} sums to {total}");
        }
    }

    fn spec_and_cert(h: u32) -> (Vec<u8>, String) {
        let spec = emit_spec_3d_window(h, EV3D_BOX + 1).into_bytes();
        let cert = emit_evcert_3d(h, &spec);
        (spec, cert)
    }

    #[test]
    fn round_trip_accepts() {
        let (spec, cert) = spec_and_cert(60);
        assert_eq!(
            check_evcert_3d(&spec, &cert),
            Verdict::Accept { claims: (1, 1, 1) }
        );
    }

    /// Brackets refine monotonically in the horizon; the certified
    /// width at 140 sits under 2e-1 (the flagship artifact rides
    /// horizon 200, width ~1e-2 — the survival tail decays at the
    /// true spectral rate ~0.956 per step while kappa/(1-c) is a
    /// fixed ~270x multiplier).
    #[test]
    fn brackets_nest_and_tighten() {
        let (_, c100) = spec_and_cert(100);
        let (_, c140) = spec_and_cert(140);
        let parse = |t: &str| parse_cert_3d(t).unwrap();
        let (a, b) = (parse(&c100), parse(&c140));
        assert!(b.ev.0 >= a.ev.0, "lower bounds must be monotone");
        assert!(b.ev.1 <= a.ev.1, "upper bounds must refine");
        let width = (b.ev.1.clone() - b.ev.0.clone()).to_f64().unwrap();
        assert!(width < 2e-1, "width {width}");
        // and the rate row beats the baseline's published/measured
        // tail rate 0.98148 by construction
        assert!(b.tail.1 < q(19630, 20000));
    }

    /// The prereg's designed mutant loci (3D-m1..m6 + the positive
    /// control), in-crate faces; the artifact battery reruns the same
    /// loci through committed files.
    #[test]
    fn mutant_loci_land_in_class() {
        let (spec, cert) = spec_and_cert(60);
        let refused = |c: &str| matches!(check_evcert_3d(&spec, c), Verdict::Refuse(_));
        let failed = |c: &str| matches!(check_evcert_3d(&spec, c), Verdict::Fail(_));

        // 3D-m1: contraction violated at one boundary state — inflate
        // a stored weight so a NEIGHBOR's transfer inequality breaks
        // (the row recompute is the kill site)
        let w_line = cert
            .lines()
            .find(|l| l.starts_with("weight s9_9_9 "))
            .unwrap()
            .to_string();
        let m1 = cert.replace(&w_line, "weight s9_9_9 1000000/1");
        assert!(failed(&m1), "3D-m1 must FAIL at the transfer recompute");

        // 3D-m2: the exit/floor face — a box weight zeroed (kappa's
        // positive-weight demand)
        let w2 = cert
            .lines()
            .find(|l| l.starts_with("weight s0_0_1 "))
            .unwrap()
            .to_string();
        let m2 = cert.replace(&w2, "weight s0_0_1 0/1");
        assert!(
            refused(&m2),
            "3D-m2 must REFUSE at the positive-weight demand"
        );

        // 3D-m3: tail claimed at c = 1
        let m3 = cert.replace("c 193/200", "c 1/1");
        assert!(
            refused(&m3),
            "3D-m3 must REFUSE at the c < 1 side condition"
        );

        // 3D-m4: origin included in the region (an alive row at the
        // origin) — spec-side mutation, cert re-bound to the bytes
        let spec4 = String::from_utf8(spec.clone())
            .unwrap()
            .replace("\"s0_0_1\", \"done_0\"", "\"s0_0_0\", \"done_0\"")
            .replace("[\"s0_0_1\",", "[\"s0_0_0\",");
        let spec4 = spec4.replacen("\"s0_0_1\"", "\"s0_0_0\", \"s0_0_1\"", 1);
        let cert4 = {
            let sha = sha256_hex(spec4.as_bytes());
            let old = cert
                .lines()
                .find(|l| l.starts_with("spec-sha256 "))
                .unwrap()
                .to_string();
            cert.replace(&old, &format!("spec-sha256 {sha}"))
        };
        assert!(
            matches!(
                check_evcert_3d(spec4.as_bytes(), &cert4),
                Verdict::Refuse(_)
            ),
            "3D-m4 must REFUSE at region/absorption or pattern consistency"
        );

        // 3D-m5: the score-one face — one stored row rescaled (K
        // stored where s*K claimed); pattern realization refuses
        let spec5 = String::from_utf8(spec.clone()).unwrap().replace(
            "[\"s1_1_1\", \"s1_1_2\", \"1/4\"]",
            "[\"s1_1_1\", \"s1_1_2\", \"9/40\"]",
        );
        let cert5 = {
            let sha = sha256_hex(spec5.as_bytes());
            let old = cert
                .lines()
                .find(|l| l.starts_with("spec-sha256 "))
                .unwrap()
                .to_string();
            cert.replace(&old, &format!("spec-sha256 {sha}"))
        };
        assert!(
            matches!(
                check_evcert_3d(spec5.as_bytes(), &cert5),
                Verdict::Refuse(_)
            ),
            "3D-m5 must REFUSE at the pattern realization"
        );

        // 3D-m6: off-by-one in the survival window (partial-lo claimed
        // with one extra term)
        let p_line = cert
            .lines()
            .find(|l| l.starts_with("claim partial-lo "))
            .unwrap()
            .to_string();
        let claimed = parse_q(p_line.trim_start_matches("claim partial-lo ").trim()).unwrap();
        let dp = dp3d(60, 1, 12);
        let surv = dp.survivals();
        let shifted = claimed + surv[60].clone();
        let m6 = cert.replace(&p_line, &format!("claim partial-lo {}", rq(&shifted)));
        assert!(failed(&m6), "3D-m6 must FAIL at the partial recompute");

        // template class locus: growth pushed past the collapse floor
        // (rinf grows until (2 rmax + rinf)/3 > c)
        let mg = cert.replace("growth 7/4", "growth 3/1");
        assert!(
            failed(&mg),
            "growth-past-floor must FAIL at the collapse class"
        );

        // binding: different spec bytes
        let spec_b = String::from_utf8(spec.clone())
            .unwrap()
            .replace("\"horizon\": 60", "\"horizon\": 61");
        assert!(
            matches!(
                check_evcert_3d(spec_b.as_bytes(), &cert),
                Verdict::Refuse(_)
            ),
            "chain-mismatch must REFUSE at the binding"
        );

        // positive control: widened EV bracket stays admissible
        let ev_line = cert
            .lines()
            .find(|l| l.starts_with("claim ev "))
            .unwrap()
            .to_string();
        let toks: Vec<&str> = ev_line.split_whitespace().collect();
        let widened = cert.replace(&ev_line, &format!("claim ev {} 100/1", toks[2]));
        assert_eq!(
            check_evcert_3d(&spec, &widened),
            Verdict::Accept { claims: (1, 1, 1) },
            "widened must ACCEPT"
        );
    }

    /// The reach demand (kernel-checked necessity): the class checks
    /// cover exterior states only past the ramp, so a profile longer
    /// than box + 1 would leave ramp-resident exterior states covered
    /// by neither the window rows nor the class inequalities — the
    /// checker REFUSES it outright. The shipped certificate satisfies
    /// the bound with room (6 ratios <= box + 1 = 11) and still
    /// accepts (control).
    #[test]
    fn reach_demand_bounds_profile_length() {
        let (spec, cert) = spec_and_cert(60);

        // too-long profile: extend the ramp to box + 2 = 12 ratios
        // (each >= 1, indices contiguous, so every earlier gate is
        // passed and the reach demand is what refuses)
        let last = cert
            .lines()
            .find(|l| l.starts_with("profile-ratio 5 "))
            .unwrap()
            .to_string();
        let mut extended = last.clone();
        for k in 6..12 {
            extended.push_str(&format!("\nprofile-ratio {k} 7/4"));
        }
        let too_long = cert.replace(&last, &extended);
        match check_evcert_3d(&spec, &too_long) {
            Verdict::Refuse(msg) => assert!(
                msg.contains("ramp-resident exterior states"),
                "the reach demand must be the refusing gate, got: {msg}"
            ),
            other => panic!("too-long profile must REFUSE, got {other:?}"),
        }

        // control: the shipped certificate accepts unchanged
        assert_eq!(
            check_evcert_3d(&spec, &cert),
            Verdict::Accept { claims: (1, 1, 1) },
            "shipped cert must still ACCEPT under the reach demand"
        );
    }
}
