//! EV-bracket certificates for stored-row chains: the first moment of
//! the absorption time as an exact-rational bracket — unrolled exact
//! partial moment (survival form) + a weighted-transfer contraction
//! tail with a geometric template + the moment-assembly row.
//!
//! CHECKER STATUS: the generator-form seam SEALED for this (finite)
//! face at theory revision event 8 — the DtmcContraction species in
//! the baked dcc closure. Production certificates carry these rows
//! as container content (`contraction_block_json` below feeds the
//! spec path) and the fast-path checker owns their verdicts; this
//! module's checker remains as the library-level exact-rational
//! oracle (in-crate loci, differentials), verdict classes mirroring
//! the fast-path checker (ACCEPT / FAIL on claim discharge / REFUSE
//! on shape, side conditions, or binding). The pattern face (`ev3d`)
//! stays on the dev-oracle lane pending its own revision event. The
//! absorption/posterior rows of the SAME chain ride the fast-path
//! checker via the sealed container; where a spec-sha256 binding
//! appears, it names the exact spec bytes the container emitter
//! sealed.
//!
//! The mathematics being checked, in one paragraph: given weights
//! $w \ge 0$ on the chain's states (absorbing states weight 0) and a
//! rate $0 < \gamma < 1$ with the weighted transfer inequality
//! $\sum_y P(x,y)\,w(y) \le \gamma\,w(x)$ at every reachable alive
//! state $x$, the weighted alive mass $W(n)$ satisfies
//! $W(n) \le \gamma^{\,n-N} W(N)$ for $n \ge N$, so with
//! $\kappa = 1/\min w$ over reachable alive states the survival mass
//! obeys $P[T > n] \le \kappa\,W(N)\,\gamma^{\,n-N}$, and
//! $\mathbb{E}[T] = \sum_{n \ge 0} P[T > n]$ is bracketed by
//! $[L,\; L + \kappa\,W(N)/(1-\gamma)]$ with
//! $L = \sum_{n < N} P[T > n]$ exact from the rows. The geometric
//! template's two sign side-conditions ($0 \le M$, $0 < \gamma$) are
//! each load-bearing and both checked. The stored rows must be a
//! TOTAL law on the alive-reachable set (rows summing to one):
//! truncation-by-rowlessness soundly widens mass brackets but has no
//! sound first-moment reading, so a reachable rowless alive state is
//! a REFUSE here, not a widening.

use crate::{survival_masses, Chain, Q};
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::str::FromStr;

// ------------------------------------------------------------ shared

fn rq(x: &Q) -> String {
    format!("{}/{}", x.numer(), x.denom())
}

pub fn parse_q(s: &str) -> Option<Q> {
    let (n, d) = s.split_once('/')?;
    let n = BigInt::from_str(n.trim()).ok()?;
    let d = BigInt::from_str(d.trim()).ok()?;
    if d.is_zero() {
        return None;
    }
    Some(Q::new(n, d))
}

fn qpow(x: &Q, e: u32) -> Q {
    let mut acc = Q::one();
    for _ in 0..e {
        acc *= x.clone();
    }
    acc
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(64);
    for b in out {
        let _ = write!(s, "{b:02x}");
    }
    s
}

// ---------------------------------------------------------- template

/// Producer-side certificate ingredients for one program: state
/// weights (by wire name, `s<x>` in the homogeneous materialization),
/// the geometric rate, and the anchor offset (the number of
/// deterministic prologue transitions between the chain's anchor and
/// the source program's counter: the EV claim binds E[T - offset],
/// and the checker recomputes that the prologue really is
/// deterministic, P[T > offset - 1] = 1). Finding weights is
/// untrusted search — for the collector they are the closed-form left
/// eigenvector; for the ring protocol they are the eigen-tight
/// regional certificate with the unit floor (the alive-mass face);
/// nothing in the checker knows or cares where they came from.
pub struct EvTemplate {
    pub weights: Vec<(String, Q)>,
    pub gamma: Q,
    pub offset: u32,
}

pub fn ev_template(name: &str) -> Option<EvTemplate> {
    match name {
        // left eigenvector of the alive block at eigenvalue 4/5:
        // w_k = (5-k)/5 on collected-count k
        "coupon-collector5" => Some(EvTemplate {
            weights: (0..5u64)
                .map(|k| (format!("s{k}"), Q::new((5 - k as i64).into(), 5.into())))
                .collect(),
            gamma: Q::new(4.into(), 5.into()),
            offset: 0,
        }),
        // eigen-tight regional certificate with the unit floor
        // (min over alive = 1, so kappa = 1: the survival envelope IS
        // the bare alive-mass tail — the strengthened seam's
        // alive-mass face). The minimal per-state solution of the
        // transfer system at gamma = 1/2, the alive spectral radius:
        // 1 on the six 2-token configurations AND on {1,2,4}; 5/4 on
        // {1,2,3}; 3/2 on {1,3,4}; 21/16 on {2,3,4} and the full
        // configuration; 99/64 on the START prologue state. Every
        // transfer row holds with EQUALITY (the sequential block
        // order breaks the ring symmetry, so the weights are
        // per-state, not token-count-lumped). Offset 1: the
        // Bernoulli(1/2)^4 prologue is one deterministic transition,
        // T = count + 1.
        "israeli-jalfon4" => {
            let mut weights: Vec<(String, Q)> = (0..16u64)
                .filter(|c| (c & 0xF).count_ones() >= 2)
                .map(|c| {
                    let w = match c {
                        7 => Q::new(5.into(), 4.into()),
                        13 => Q::new(3.into(), 2.into()),
                        14 | 15 => Q::new(21.into(), 16.into()),
                        _ => Q::one(), // the 2-token six and {1,2,4}
                    };
                    (format!("s{c}"), w)
                })
                .collect();
            weights.push(("s16".into(), Q::new(99.into(), 64.into())));
            Some(EvTemplate {
                weights,
                gamma: Q::new(1.into(), 2.into()),
                offset: 1,
            })
        }
        // eigen-tight regional certificate with the unit floor for
        // the synchronous 3-ring (the alive-mass face, kappa = 1):
        // gamma = 1/2, the alive spectral radius; weights 1 on the
        // three two-one configurations, 3/2 on the uniform-resample
        // states 000 and 111, 3/2 on the START prologue state. Every
        // transfer row holds with EQUALITY. Offset 1: the
        // Bernoulli(1/2)^3 prologue is one deterministic transition,
        // T = count + 1. UNLIKE ij4, the above-floor states RECUR, so
        // the achieved tail sits strictly below the bound and the EV
        // upper endpoint exceeds the truth — by exactly
        // (1/6) 4^-(N-1) at horizon N.
        "herman3" => {
            let weights: Vec<(String, Q)> = [0u64, 3, 5, 6, 7, 8]
                .iter()
                .map(|&c| {
                    let w = match c {
                        0 | 7 | 8 => Q::new(3.into(), 2.into()),
                        _ => Q::one(), // the two-one configurations
                    };
                    (format!("s{c}"), w)
                })
                .collect();
            Some(EvTemplate {
                weights,
                gamma: Q::new(1.into(), 2.into()),
                offset: 1,
            })
        }
        _ => None,
    }
}

// ---------------------------------------------------------- emission

/// The contraction witness block for the container spec (the sealed
/// DtmcContraction species, revision event 8): the same template,
/// exact partial sum, weighted alive mass, and tail assembly the EV
/// certificate carried on the dev-oracle lane, rendered as the
/// emitter's ADDITIVE `contraction` spec block. Claim set mirrors the
/// retired `.evc` grammar exactly — `genform`, `partial_lo`, `ev`,
/// `tail`; no `z_one` (the dev-oracle certificates never claimed it).
/// `None` when the case has no registered template.
pub fn contraction_block_json(chain: &dyn Chain, horizon: u32) -> Option<String> {
    assert!(
        chain.time_homogeneous(),
        "contraction witnesses demand the time-homogeneous stored-row form"
    );
    let tpl = ev_template(chain.name())?;
    let n = horizon;
    assert!(
        tpl.offset <= n,
        "anchor offset {} beyond horizon {n}",
        tpl.offset
    );
    let surv = survival_masses(chain, n);
    if tpl.offset > 0 {
        assert!(
            surv[tpl.offset as usize - 1].is_one(),
            "anchor offset {} claims a deterministic prologue but P[T > {}] < 1",
            tpl.offset,
            tpl.offset - 1
        );
    }
    let l: Q = surv[tpl.offset as usize..n as usize]
        .iter()
        .fold(Q::zero(), |a, b| a + b.clone());
    let tr = crate::evaluate(chain, n);
    let wmap: BTreeMap<&str, &Q> = tpl.weights.iter().map(|(s, w)| (s.as_str(), w)).collect();
    let wn = tr.alive_support.iter().fold(Q::zero(), |a, (x, m)| {
        let name = format!("s{x}");
        a + m
            * (*wmap
                .get(name.as_str())
                .unwrap_or_else(|| panic!("no weight for reachable alive state {name}")))
            .clone()
    });
    let wmin = tpl
        .weights
        .iter()
        .map(|(_, w)| w.clone())
        .min()
        .expect("nonempty weights");
    let kappa = Q::one() / wmin;
    let tail = kappa.clone() * wn.clone() / (Q::one() - tpl.gamma.clone());
    let tail_coeff = kappa * wn / qpow(&tpl.gamma, n);

    let mut out = String::new();
    out.push_str("  \"contraction\": {\n");
    let _ = write!(out, "    \"anchor\": \"s{}\",\n", chain.init());
    let _ = write!(out, "    \"horizon\": {n},\n");
    if tpl.offset > 0 {
        let _ = write!(out, "    \"offset\": {},\n", tpl.offset);
    }
    let _ = write!(out, "    \"c\": \"{}\",\n", rq(&tpl.gamma));
    out.push_str("    \"weights\": [");
    for (i, (s, w)) in tpl.weights.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "[\"{s}\", \"{}\"]", rq(w));
    }
    out.push_str("],\n    \"claims\": {\n      \"genform\": true,\n");
    let _ = write!(out, "      \"partial_lo\": [\"{}\"],\n", rq(&l));
    let _ = write!(
        out,
        "      \"ev\": [[\"{}\", \"{}\"]],\n",
        rq(&l),
        rq(&(l.clone() + tail))
    );
    let _ = write!(
        out,
        "      \"tail\": [[\"{}\", \"{}\"]]\n",
        rq(&tail_coeff),
        rq(&tpl.gamma)
    );
    out.push_str("    }\n  }\n");
    Some(out)
}

/// Emit the EV certificate (deterministic text form) for a
/// TIME-HOMOGENEOUS chain at `horizon`, binding it to the exact spec
/// bytes the container emitter seals. Canonical (sharpest admissible)
/// claims; the checker recomputes everything.
pub fn emit_evcert(chain: &dyn Chain, horizon: u32, spec_bytes: &[u8]) -> String {
    assert!(
        chain.time_homogeneous(),
        "EV certificates demand the time-homogeneous stored-row form \
         (the tail premise quantifies over the total stored law)"
    );
    let tpl = ev_template(chain.name())
        .unwrap_or_else(|| panic!("no EV template registered for {}", chain.name()));
    let n = horizon;
    assert!(
        tpl.offset <= n,
        "anchor offset {} beyond horizon {n}",
        tpl.offset
    );

    let surv = survival_masses(chain, n);
    if tpl.offset > 0 {
        assert!(
            surv[tpl.offset as usize - 1].is_one(),
            "anchor offset {} claims a deterministic prologue but \
             P[T > {}] < 1",
            tpl.offset,
            tpl.offset - 1
        );
    }
    let l: Q = surv[tpl.offset as usize..n as usize]
        .iter()
        .fold(Q::zero(), |a, b| a + b.clone());

    // weighted alive mass at the horizon, on wire names
    let tr = crate::evaluate(chain, n);
    let wmap: BTreeMap<&str, &Q> = tpl.weights.iter().map(|(s, w)| (s.as_str(), w)).collect();
    let wn = tr.alive_support.iter().fold(Q::zero(), |a, (x, m)| {
        let name = format!("s{x}");
        a + m
            * (*wmap
                .get(name.as_str())
                .unwrap_or_else(|| panic!("no weight for reachable alive state {name}")))
            .clone()
    });

    let wmin = tpl
        .weights
        .iter()
        .map(|(_, w)| w.clone())
        .min()
        .expect("nonempty weights");
    let kappa = Q::one() / wmin;
    let one_minus = Q::one() - tpl.gamma.clone();
    let tail = kappa.clone() * wn.clone() / one_minus;
    let tail_coeff = kappa * wn.clone() / qpow(&tpl.gamma, n); // A: P[T>n] <= A*gamma^n, n >= N

    let mut out = String::new();
    let _ = writeln!(out, "evcert v0");
    let _ = writeln!(out, "program {}", chain.name());
    let _ = writeln!(out, "spec-sha256 {}", sha256_hex(spec_bytes));
    let _ = writeln!(out, "horizon {n}");
    if tpl.offset > 0 {
        // the EV claim binds E[T - offset]; absent line = offset 0
        // (committed artifacts predating the offset row are unchanged)
        let _ = writeln!(out, "offset {}", tpl.offset);
    }
    let _ = writeln!(out, "gamma {}", rq(&tpl.gamma));
    let _ = writeln!(out, "M {}", rq(&wn));
    for (s, w) in &tpl.weights {
        let _ = writeln!(out, "weight {s} {}", rq(w));
    }
    let _ = writeln!(out, "claim partial-lo {}", rq(&l));
    let _ = writeln!(out, "claim ev {} {}", rq(&l), rq(&(l.clone() + tail)));
    let _ = writeln!(out, "claim tail {} {}", rq(&tail_coeff), rq(&tpl.gamma));
    out
}

// ------------------------------------------------------------ verdict

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// All claims discharged; carries (partial, ev, tail) claim counts.
    Accept { claims: (u32, u32, u32) },
    /// A claim failed its recomputation.
    Fail(String),
    /// Shape, side condition, or binding refused before any claim.
    Refuse(String),
}

// ----------------------------------------------------- spec parsing
// Minimal JSON reader for the emitter's spec schema (objects, arrays,
// strings, integers, booleans — exactly what render_spec emits). The
// checker parses the SAME bytes the container was sealed from; any
// deviation is a REFUSE, never a guess.

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Jv {
    S(String),
    N(i64),
    B(bool),
    A(Vec<Jv>),
    O(Vec<(String, Jv)>),
}

struct Jp<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Jp<'a> {
    fn ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.ws();
        self.b.get(self.i).copied()
    }
    fn eat(&mut self, c: u8) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", c as char, self.i))
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut s = String::new();
        loop {
            match self.b.get(self.i).copied() {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(s);
                }
                Some(b'\\') => {
                    self.i += 1;
                    match self.b.get(self.i).copied() {
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        other => return Err(format!("unsupported escape {other:?}")),
                    }
                    self.i += 1;
                }
                Some(c) => {
                    s.push(c as char);
                    self.i += 1;
                }
                None => return Err("unterminated string".into()),
            }
        }
    }
    fn value(&mut self) -> Result<Jv, String> {
        match self.peek() {
            Some(b'"') => Ok(Jv::S(self.string()?)),
            Some(b'{') => {
                self.eat(b'{')?;
                let mut o = Vec::new();
                if self.peek() == Some(b'}') {
                    self.i += 1;
                    return Ok(Jv::O(o));
                }
                loop {
                    let k = self.string()?;
                    self.eat(b':')?;
                    o.push((k, self.value()?));
                    match self.peek() {
                        Some(b',') => {
                            self.i += 1;
                        }
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Jv::O(o));
                        }
                        other => return Err(format!("object: unexpected {other:?}")),
                    }
                }
            }
            Some(b'[') => {
                self.eat(b'[')?;
                let mut a = Vec::new();
                if self.peek() == Some(b']') {
                    self.i += 1;
                    return Ok(Jv::A(a));
                }
                loop {
                    a.push(self.value()?);
                    match self.peek() {
                        Some(b',') => {
                            self.i += 1;
                        }
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Jv::A(a));
                        }
                        other => return Err(format!("array: unexpected {other:?}")),
                    }
                }
            }
            Some(b't') => {
                if self.b[self.i..].starts_with(b"true") {
                    self.i += 4;
                    Ok(Jv::B(true))
                } else {
                    Err("bad literal".into())
                }
            }
            Some(b'f') => {
                if self.b[self.i..].starts_with(b"false") {
                    self.i += 5;
                    Ok(Jv::B(false))
                } else {
                    Err("bad literal".into())
                }
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let start = self.i;
                if c == b'-' {
                    self.i += 1;
                }
                while self.b.get(self.i).is_some_and(|d| d.is_ascii_digit()) {
                    self.i += 1;
                }
                let s = std::str::from_utf8(&self.b[start..self.i]).unwrap();
                s.parse::<i64>()
                    .map(Jv::N)
                    .map_err(|e| format!("number: {e}"))
            }
            other => Err(format!("unexpected {other:?} at byte {}", self.i)),
        }
    }
}

/// Parse spec bytes with the minimal reader (shared with the pattern
/// face in `ev3d`).
pub(crate) fn parse_spec_json(bytes: &[u8]) -> Result<Jv, String> {
    let mut jp = Jp { b: bytes, i: 0 };
    jp.value()
}

pub(crate) type SpecTables = (
    BTreeSet<String>,                   // states
    BTreeMap<String, Vec<(String, Q)>>, // rows by source
    BTreeMap<String, Q>,                // init measure
    BTreeSet<String>,                   // class state names
    BTreeSet<String>,                   // reject state names
    u32,                                // horizon
);

/// Extract the spec's chain/query tables with the same refusal
/// semantics as the finite-face checker (shape errors are Err strings
/// the caller renders as REFUSE).
pub(crate) fn spec_tables(spec: &Jv) -> Result<SpecTables, String> {
    let chain = spec.get("chain").ok_or("no chain")?;
    let states: BTreeSet<String> = chain
        .get("states")
        .and_then(Jv::arr)
        .ok_or("no states")?
        .iter()
        .filter_map(|v| v.s().map(String::from))
        .collect();
    let query = spec.get("query").ok_or("no query")?;
    let horizon = match query.get("horizon").and_then(Jv::n) {
        Some(n) if n >= 0 => n as u32,
        _ => return Err("bad horizon".into()),
    };
    let mut classes: BTreeSet<String> = BTreeSet::new();
    for c in query.get("classes").and_then(Jv::arr).unwrap_or(&[]).iter() {
        match c.arr().and_then(|p| p.first()).and_then(Jv::s) {
            Some(s) => {
                classes.insert(s.to_string());
            }
            None => return Err("bad class row".into()),
        }
    }
    let mut rejects: BTreeSet<String> = BTreeSet::new();
    for r in query.get("rejects").and_then(Jv::arr).unwrap_or(&[]).iter() {
        match r.s() {
            Some(s) => {
                rejects.insert(s.to_string());
            }
            None => return Err("bad reject row".into()),
        }
    }
    let mut rows: BTreeMap<String, Vec<(String, Q)>> = BTreeMap::new();
    for r in chain.get("rows").and_then(Jv::arr).unwrap_or(&[]).iter() {
        let (src, dst, w) = match r.arr() {
            Some([s, d, w]) => match (s.s(), d.s(), w.s().and_then(parse_q)) {
                (Some(s), Some(d), Some(w)) => (s.to_string(), d.to_string(), w),
                _ => return Err("bad row".into()),
            },
            _ => return Err("bad row shape".into()),
        };
        if !states.contains(&src) || !states.contains(&dst) {
            return Err(format!("row names unknown state {src} -> {dst}"));
        }
        if w.is_negative() {
            return Err(format!("negative row weight at {src}"));
        }
        rows.entry(src).or_default().push((dst, w));
    }
    let mut init: BTreeMap<String, Q> = BTreeMap::new();
    for r in chain.get("init").and_then(Jv::arr).unwrap_or(&[]).iter() {
        match r.arr() {
            Some([s, m]) => match (s.s(), m.s().and_then(parse_q)) {
                (Some(s), Some(m)) if states.contains(s) && !m.is_negative() => {
                    *init.entry(s.to_string()).or_insert_with(Q::zero) += m;
                }
                _ => return Err("bad init row".into()),
            },
            _ => return Err("bad init shape".into()),
        }
    }
    Ok((states, rows, init, classes, rejects, horizon))
}

impl Jv {
    fn get<'a>(&'a self, k: &str) -> Option<&'a Jv> {
        match self {
            Jv::O(o) => o.iter().find(|(n, _)| n == k).map(|(_, v)| v),
            _ => None,
        }
    }
    fn arr(&self) -> Option<&[Jv]> {
        match self {
            Jv::A(a) => Some(a),
            _ => None,
        }
    }
    fn s(&self) -> Option<&str> {
        match self {
            Jv::S(s) => Some(s),
            _ => None,
        }
    }
    fn n(&self) -> Option<i64> {
        match self {
            Jv::N(n) => Some(*n),
            _ => None,
        }
    }
}

// ------------------------------------------------------------ checker

struct ParsedCert {
    program: String,
    spec_sha256: String,
    horizon: u32,
    offset: u32,
    gamma: Q,
    m: Q,
    weights: BTreeMap<String, Q>,
    partial_lo: Q,
    ev: (Q, Q),
    tail: (Q, Q),
}

fn parse_cert(text: &str) -> Result<ParsedCert, String> {
    let mut program = None;
    let mut sha = None;
    let mut horizon = None;
    let mut offset = 0u32;
    let mut gamma = None;
    let mut m = None;
    let mut weights: BTreeMap<String, Q> = BTreeMap::new();
    let mut partial = None;
    let mut ev = None;
    let mut tail = None;
    let mut lines = text.lines();
    if lines.next() != Some("evcert v0") {
        return Err("missing 'evcert v0' header".into());
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
            ["offset", k] => offset = k.parse::<u32>().map_err(|e| format!("offset: {e}"))?,
            ["gamma", g] => gamma = Some(parse_q(g).ok_or("gamma: bad fraction")?),
            ["M", v] => m = Some(parse_q(v).ok_or("M: bad fraction")?),
            ["weight", s, w] => {
                let w = parse_q(w).ok_or("weight: bad fraction")?;
                if weights.insert(s.to_string(), w).is_some() {
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
            other => return Err(format!("unrecognized line: {other:?}")),
        }
    }
    Ok(ParsedCert {
        program: program.ok_or("missing program")?,
        spec_sha256: sha.ok_or("missing spec-sha256")?,
        horizon: horizon.ok_or("missing horizon")?,
        offset,
        gamma: gamma.ok_or("missing gamma")?,
        m: m.ok_or("missing M")?,
        weights,
        partial_lo: partial.ok_or("missing claim partial-lo")?,
        ev: ev.ok_or("missing claim ev")?,
        tail: tail.ok_or("missing claim tail")?,
    })
}

/// The dev-oracle check: recompute every quantity from the stored
/// chain rows in the spec bytes and discharge the EV certificate's
/// claims. See the module doc for the mathematics and the verdict
/// classes.
pub fn check_evcert(spec_bytes: &[u8], cert_text: &str) -> Verdict {
    let cert = match parse_cert(cert_text) {
        Ok(c) => c,
        Err(e) => return Verdict::Refuse(format!("cert shape: {e}")),
    };

    // binding: the cert names the exact spec bytes the container
    // emitter sealed
    let actual = sha256_hex(spec_bytes);
    if actual != cert.spec_sha256 {
        return Verdict::Refuse(format!(
            "spec binding: {} cert names {} but the spec bytes hash to {actual}",
            cert.program, cert.spec_sha256
        ));
    }

    // parse the spec
    let mut jp = Jp {
        b: spec_bytes,
        i: 0,
    };
    let spec = match jp.value() {
        Ok(v) => v,
        Err(e) => return Verdict::Refuse(format!("spec parse: {e}")),
    };
    let chain = match spec.get("chain") {
        Some(c) => c,
        None => return Verdict::Refuse("spec: no chain".into()),
    };
    let states: BTreeSet<String> = match chain.get("states").and_then(Jv::arr) {
        Some(a) => a.iter().filter_map(|v| v.s().map(String::from)).collect(),
        None => return Verdict::Refuse("spec: no states".into()),
    };
    let query = match spec.get("query") {
        Some(v) => v,
        None => return Verdict::Refuse("spec: no query".into()),
    };
    let horizon = match query.get("horizon").and_then(Jv::n) {
        Some(n) if n >= 0 => n as u32,
        _ => return Verdict::Refuse("spec: bad horizon".into()),
    };
    if horizon != cert.horizon {
        return Verdict::Refuse(format!(
            "horizon mismatch: cert {} vs spec {horizon}",
            cert.horizon
        ));
    }
    let mut terminal: BTreeSet<String> = BTreeSet::new();
    for c in query.get("classes").and_then(Jv::arr).unwrap_or(&[]).iter() {
        match c.arr().and_then(|p| p.first()).and_then(Jv::s) {
            Some(s) => {
                terminal.insert(s.to_string());
            }
            None => return Verdict::Refuse("spec: bad class row".into()),
        }
    }
    for r in query.get("rejects").and_then(Jv::arr).unwrap_or(&[]).iter() {
        match r.s() {
            Some(s) => {
                terminal.insert(s.to_string());
            }
            None => return Verdict::Refuse("spec: bad reject row".into()),
        }
    }

    // stored rows
    let mut rows: BTreeMap<String, Vec<(String, Q)>> = BTreeMap::new();
    for r in chain.get("rows").and_then(Jv::arr).unwrap_or(&[]).iter() {
        let (src, dst, w) = match r.arr() {
            Some([s, d, w]) => match (s.s(), d.s(), w.s().and_then(parse_q)) {
                (Some(s), Some(d), Some(w)) => (s.to_string(), d.to_string(), w),
                _ => return Verdict::Refuse("spec: bad row".into()),
            },
            _ => return Verdict::Refuse("spec: bad row shape".into()),
        };
        if !states.contains(&src) || !states.contains(&dst) {
            return Verdict::Refuse(format!("spec: row names unknown state {src} -> {dst}"));
        }
        if w.is_negative() {
            return Verdict::Refuse(format!("spec: negative row weight at {src}"));
        }
        rows.entry(src).or_default().push((dst, w));
    }
    for t in &terminal {
        if rows.contains_key(t) {
            return Verdict::Refuse(format!("terminal state {t} has outgoing rows"));
        }
    }

    // initial measure
    let mut init: BTreeMap<String, Q> = BTreeMap::new();
    for r in chain.get("init").and_then(Jv::arr).unwrap_or(&[]).iter() {
        match r.arr() {
            Some([s, m]) => match (s.s(), m.s().and_then(parse_q)) {
                (Some(s), Some(m)) if states.contains(s) && !m.is_negative() => {
                    *init.entry(s.to_string()).or_insert_with(Q::zero) += m;
                }
                _ => return Verdict::Refuse("spec: bad init row".into()),
            },
            _ => return Verdict::Refuse("spec: bad init shape".into()),
        }
    }
    let init_total = init.values().fold(Q::zero(), |a, b| a + b.clone());
    if !init_total.is_one() {
        return Verdict::Refuse(format!("init mass {init_total} != 1"));
    }

    // weights: named states exist, nonnegative; terminal weights zero
    for (s, w) in &cert.weights {
        if !states.contains(s) {
            return Verdict::Refuse(format!("weight names unknown state {s}"));
        }
        if w.is_negative() {
            return Verdict::Refuse(format!("negative weight at {s}"));
        }
        if terminal.contains(s) && !w.is_zero() {
            return Verdict::Refuse(format!("terminal state {s} carries nonzero weight"));
        }
    }
    let w_of = |s: &str| -> Q {
        if terminal.contains(s) {
            Q::zero()
        } else {
            cert.weights.get(s).cloned().unwrap_or_else(Q::zero)
        }
    };

    // side conditions (the geometric template's two sign checks, plus
    // the assembly's summability)
    if cert.gamma <= Q::zero() {
        return Verdict::Refuse("side condition 0 < gamma violated".into());
    }
    if cert.gamma >= Q::one() {
        return Verdict::Refuse("assembly needs gamma < 1 (geometric sum)".into());
    }
    if cert.m.is_negative() {
        return Verdict::Refuse("side condition 0 <= M violated".into());
    }

    // reachable alive set (init support + row closure, minus terminals)
    let mut reach: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = init.keys().cloned().collect();
    while let Some(x) = stack.pop() {
        if terminal.contains(&x) || !reach.insert(x.clone()) {
            continue;
        }
        if let Some(rs) = rows.get(&x) {
            for (d, _) in rs {
                if !terminal.contains(d) && !reach.contains(d) {
                    stack.push(d.clone());
                }
            }
        }
    }
    // the total-law demand + positive weights on the reachable alive set
    for x in &reach {
        let total = rows
            .get(x)
            .map(|rs| rs.iter().fold(Q::zero(), |a, (_, w)| a + w.clone()))
            .unwrap_or_else(Q::zero);
        if !total.is_one() {
            return Verdict::Refuse(format!(
                "reachable alive state {x} is not row-total (sum {total}): \
                 truncation has no sound first-moment reading"
            ));
        }
        if w_of(x) <= Q::zero() {
            return Verdict::Refuse(format!("reachable alive state {x} needs a positive weight"));
        }
    }
    let wmin = reach.iter().map(|x| w_of(x)).min().unwrap_or_else(Q::one);
    let kappa = Q::one() / wmin;

    // weighted transfer inequality at every reachable alive state
    for x in &reach {
        let lhs = rows[x].iter().fold(Q::zero(), |a, (d, w)| a + w * w_of(d));
        let rhs = cert.gamma.clone() * w_of(x);
        if lhs > rhs {
            return Verdict::Fail(format!("transfer inequality refuted at {x}: {lhs} > {rhs}"));
        }
    }

    // exact DP from the stored rows: survival masses + weighted alive
    // mass at the horizon
    let mut alive: BTreeMap<String, Q> = init;
    let mut survivals: Vec<Q> = Vec::with_capacity(horizon as usize + 1);
    survivals.push(alive.values().fold(Q::zero(), |a, b| a + b.clone()));
    for _ in 0..horizon {
        let mut next: BTreeMap<String, Q> = BTreeMap::new();
        for (x, m) in &alive {
            if let Some(rs) = rows.get(x) {
                for (d, w) in rs {
                    if !terminal.contains(d) {
                        *next.entry(d.clone()).or_insert_with(Q::zero) += m * w;
                    }
                }
            }
        }
        alive = next;
        survivals.push(alive.values().fold(Q::zero(), |a, b| a + b.clone()));
    }

    // anchor offset: the EV claim binds E[T - offset]. The offset is a
    // recomputed property of the chain, never an out-of-band shift:
    // the claimed prologue must be deterministic (P[T > offset-1] = 1
    // from the stored rows) and inside the horizon.
    if cert.offset > horizon {
        return Verdict::Refuse(format!(
            "anchor offset {} beyond horizon {horizon}",
            cert.offset
        ));
    }
    if cert.offset > 0 && !survivals[cert.offset as usize - 1].is_one() {
        return Verdict::Refuse(format!(
            "anchor offset {} claims a deterministic prologue but \
             P[T > {}] = {} != 1",
            cert.offset,
            cert.offset - 1,
            survivals[cert.offset as usize - 1]
        ));
    }

    let l_exact: Q = survivals[cert.offset as usize..horizon as usize]
        .iter()
        .fold(Q::zero(), |a, b| a + b.clone());
    let w_exact = alive.iter().fold(Q::zero(), |a, (x, m)| a + m * w_of(x));

    // template magnitude: claimed M must dominate the recomputed
    // weighted alive mass
    if cert.m < w_exact {
        return Verdict::Fail(format!(
            "template magnitude: claimed M {} below recomputed weighted \
             alive mass {}",
            cert.m, w_exact
        ));
    }

    // claims
    if cert.partial_lo > l_exact {
        return Verdict::Fail(format!(
            "partial-lo {} above the exact partial survival sum {}",
            cert.partial_lo, l_exact
        ));
    }
    let tail_bound = kappa.clone() * w_exact.clone() / (Q::one() - cert.gamma.clone());
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
    if *tail_g < cert.gamma || *tail_g >= Q::one() {
        return Verdict::Fail(format!(
            "tail rate {tail_g} not in [gamma, 1) = [{}, 1)",
            cert.gamma
        ));
    }
    if tail_a.clone() * qpow(tail_g, horizon) < kappa * w_exact {
        return Verdict::Fail(
            "tail coefficient below the warranted survival envelope at n = horizon".into(),
        );
    }

    Verdict::Accept { claims: (1, 1, 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::emit_spec;
    use crate::CouponCollector5;
    use num_traits::ToPrimitive;

    fn spec_and_cert() -> (Vec<u8>, String) {
        let spec = emit_spec(&CouponCollector5, 80).into_bytes();
        let cert = emit_evcert(&CouponCollector5, 80, &spec);
        (spec, cert)
    }

    #[test]
    fn round_trip_accepts() {
        let (spec, cert) = spec_and_cert();
        assert_eq!(
            check_evcert(&spec, &cert),
            Verdict::Accept { claims: (1, 1, 1) }
        );
    }

    #[test]
    fn canonical_bracket_contains_truth_and_is_tight() {
        let (_, cert) = spec_and_cert();
        let parsed = parse_cert(&cert).unwrap();
        let truth = Q::new(137.into(), 12.into());
        assert!(parsed.ev.0 <= truth && truth <= parsed.ev.1);
        let width = (parsed.ev.1.clone() - parsed.ev.0.clone())
            .to_f64()
            .unwrap();
        assert!(width < 5e-7, "width {width}");
    }

    /// The in-crate faces of the artifact battery's designed loci —
    /// each mutant must land in its class (the artifact battery reruns
    /// the same loci through the committed files).
    #[test]
    fn mutant_loci_land_in_class() {
        let (spec, cert) = spec_and_cert();
        let refused = |c: &str| matches!(check_evcert(&spec, c), Verdict::Refuse(_));
        let failed = |c: &str| matches!(check_evcert(&spec, c), Verdict::Fail(_));

        // sign side conditions (the template addendum's two loci)
        let gamma_neg = cert.replace("gamma 4/5", "gamma -4/5");
        assert!(refused(&gamma_neg), "gamma-neg must REFUSE");
        let m_line = cert
            .lines()
            .find(|l| l.starts_with("M "))
            .unwrap()
            .to_string();
        let m_neg = cert.replace(&m_line, &format!("M -{}", &m_line[2..]));
        assert!(refused(&m_neg), "m-neg must REFUSE");

        // contraction rate tightened past the spectral radius
        let c_tight = cert.replace("gamma 4/5", "gamma 3/5");
        assert!(failed(&c_tight), "c-tighter must FAIL");

        // template magnitude tightened below the recomputed mass
        let m_tight = cert.replace(&m_line, "M 1/100000000000");
        assert!(failed(&m_tight), "m-tighter must FAIL");

        // partial sum inflated
        let p_line = cert
            .lines()
            .find(|l| l.starts_with("claim partial-lo "))
            .unwrap()
            .to_string();
        let p_inflated = cert.replace(&p_line, "claim partial-lo 12/1");
        assert!(failed(&p_inflated), "partial-inflated must FAIL");

        // EV hi tightened below the warranted upper
        let ev_line = cert
            .lines()
            .find(|l| l.starts_with("claim ev "))
            .unwrap()
            .to_string();
        let toks: Vec<&str> = ev_line.split_whitespace().collect();
        let ev_tight = cert.replace(&ev_line, &format!("claim ev {} {}", toks[2], toks[2]));
        assert!(failed(&ev_tight), "ev-tighter must FAIL");

        // a reachable alive state's weight zeroed
        let w_zero = cert.replace("weight s4 1/5", "weight s4 0/5");
        assert!(refused(&w_zero), "w-zero must REFUSE");

        // binding: a different spec (one row weight changed)
        let spec2 = String::from_utf8(spec.clone())
            .unwrap()
            .replace("\"4/5\"", "\"3/5\"")
            .into_bytes();
        assert!(
            matches!(check_evcert(&spec2, &cert), Verdict::Refuse(_)),
            "chain-mismatch must REFUSE at the binding"
        );

        // widened EV bracket stays admissible (positive control)
        let widened = cert.replace(&ev_line, &format!("claim ev {} 12/1", toks[2]));
        assert_eq!(
            check_evcert(&spec, &widened),
            Verdict::Accept { claims: (1, 1, 1) },
            "widened must ACCEPT"
        );
    }

    // ------------------------------------------------ israeli-jalfon4

    fn ij4_spec_and_cert() -> (Vec<u8>, String) {
        let spec = emit_spec(&crate::IsraeliJalfon4, 31).into_bytes();
        let cert = emit_evcert(&crate::IsraeliJalfon4, 31, &spec);
        (spec, cert)
    }

    #[test]
    fn ij4_round_trip_accepts() {
        let (spec, cert) = ij4_spec_and_cert();
        assert!(cert.contains("offset 1"), "offset line must be emitted");
        assert_eq!(
            check_evcert(&spec, &cert),
            Verdict::Accept { claims: (1, 1, 1) }
        );
    }

    #[test]
    fn ij4_bracket_contains_truth_and_is_tight() {
        let (_, cert) = ij4_spec_and_cert();
        let parsed = parse_cert(&cert).unwrap();
        let truth = Q::new(373.into(), 256.into());
        assert!(parsed.ev.0 <= truth && truth <= parsed.ev.1);
        let width = (parsed.ev.1.clone() - parsed.ev.0.clone())
            .to_f64()
            .unwrap();
        assert!(width < 2e-9, "width {width}");
        // kappa = 1 exactly (the alive-mass face): the emitted M is
        // the eigen-tight (19/12) 2^-31 and the ev upper minus the
        // partial equals 2 M (1/(1-gamma) = 2, kappa = 1)
        let two_m = Q::new(2.into(), 1.into()) * parsed.m.clone();
        assert_eq!(parsed.ev.1.clone() - parsed.partial_lo.clone(), two_m);
    }

    /// The offset row's designed loci (preregistered with the row):
    /// offset-early REFUSEs at the prologue-determinism recompute,
    /// offset-dropped FAILs at the assembly (the offset is
    /// load-bearing, not annotation), and the scale-doubled positive
    /// control ACCEPTs (the kappa-normalized floor face: the assembly
    /// is scale-invariant, so the unit floor's content is exactly
    /// kappa = 1).
    #[test]
    fn ij4_mutant_loci_land_in_class() {
        let (spec, cert) = ij4_spec_and_cert();
        let refused = |c: &str| matches!(check_evcert(&spec, c), Verdict::Refuse(_));
        let failed = |c: &str| matches!(check_evcert(&spec, c), Verdict::Fail(_));

        // offset-early: k = 2 claims a 2-step deterministic prologue,
        // but P[T > 1] = 11/16
        let early = cert.replace("offset 1", "offset 2");
        assert!(refused(&early), "offset-early must REFUSE");

        // offset beyond horizon
        let beyond = cert.replace("offset 1", "offset 99");
        assert!(refused(&beyond), "offset-beyond-horizon must REFUSE");

        // offset-dropped: k = 0 with the k = 1 claims kept — the
        // warranted assembly gains the prologue's survival mass 1, so
        // the kept ev-hi sits below it
        let dropped = cert.replace("offset 1\n", "");
        assert!(failed(&dropped), "offset-dropped must FAIL");

        // template sign side-conditions, case face
        let gamma_neg = cert.replace("gamma 1/2", "gamma -1/2");
        assert!(refused(&gamma_neg), "gamma-neg must REFUSE");

        // contraction rate tightened past the spectral radius
        let c_tight = cert.replace("gamma 1/2", "gamma 1/3");
        assert!(failed(&c_tight), "c-tighter must FAIL");

        // a reachable alive state's weight zeroed
        let w_zero = cert.replace("weight s3 1/1", "weight s3 0/1");
        assert!(refused(&w_zero), "w-zero must REFUSE");

        // no-re-entry face (the alive-mass face's closed-complement
        // hypothesis, IJ-m2's checker face): a stored row from the
        // stabilized class back into the alive region, certificate
        // re-bound to the mutated bytes so the binding passes and the
        // shape check is what refuses
        let spec_txt = String::from_utf8(spec.clone()).unwrap();
        let marker = "\"rows\": [\n";
        let reentry = spec_txt
            .replacen(
                marker,
                &format!("{marker}      [\"done_0\", \"s3\", \"1/1\"],\n"),
                1,
            )
            .into_bytes();
        assert_ne!(reentry, spec, "mutant must change the spec bytes");
        let rebound = emit_evcert(&crate::IsraeliJalfon4, 31, &reentry);
        match check_evcert(&reentry, &rebound) {
            Verdict::Refuse(msg) => {
                assert!(
                    msg.contains("outgoing rows"),
                    "re-entry must refuse at the terminal-rows recompute, got: {msg}"
                );
            }
            other => panic!("re-entry must REFUSE, got {other:?}"),
        }

        // scale-doubled positive control: all weights doubled, claims
        // unchanged — kappa halves the doubled mass back, ACCEPT
        let mut scaled = String::new();
        for line in cert.lines() {
            let toks: Vec<&str> = line.split_whitespace().collect();
            match toks.as_slice() {
                ["weight", s, w] => {
                    let w = parse_q(w).unwrap() * Q::new(2.into(), 1.into());
                    scaled.push_str(&format!("weight {s} {}/{}\n", w.numer(), w.denom()));
                }
                ["M", v] => {
                    let v = parse_q(v).unwrap() * Q::new(2.into(), 1.into());
                    scaled.push_str(&format!("M {}/{}\n", v.numer(), v.denom()));
                }
                _ => {
                    scaled.push_str(line);
                    scaled.push('\n');
                }
            }
        }
        assert_eq!(
            check_evcert(&spec, &scaled),
            Verdict::Accept { claims: (1, 1, 1) },
            "scale-doubled must ACCEPT (kappa-normalized floor face)"
        );
    }

    // ------------------------------------------------------- herman3

    fn h3_spec_and_cert() -> (Vec<u8>, String) {
        let spec = emit_spec(&crate::Herman3, 31).into_bytes();
        let cert = emit_evcert(&crate::Herman3, 31, &spec);
        (spec, cert)
    }

    #[test]
    fn herman3_round_trip_accepts() {
        let (spec, cert) = h3_spec_and_cert();
        assert!(cert.contains("offset 1"), "offset line must be emitted");
        assert!(
            cert.contains("gamma 1/2"),
            "gamma is the alive spectral radius"
        );
        assert_eq!(
            check_evcert(&spec, &cert),
            Verdict::Accept { claims: (1, 1, 1) }
        );
    }

    #[test]
    fn herman3_bracket_contains_truth_and_is_tight() {
        let (_, cert) = h3_spec_and_cert();
        let parsed = parse_cert(&cert).unwrap();
        let truth = Q::new(4.into(), 3.into());
        assert!(parsed.ev.0 <= truth && truth <= parsed.ev.1);
        let width = (parsed.ev.1.clone() - parsed.ev.0.clone())
            .to_f64()
            .unwrap();
        assert!(width < 1.4e-9, "width {width}");
        // kappa = 1 (alive-mass face): M is the eigen-tight
        // (3/2) 2^-31 and ev-upper minus partial equals 2 M
        let two_m = Q::new(2.into(), 1.into()) * parsed.m.clone();
        assert_eq!(parsed.ev.1.clone() - parsed.partial_lo.clone(), two_m);
        // the DISANALOGY with ij4, preregistered: the upper endpoint
        // sits STRICTLY above the truth (recurrent above-floor
        // states), by exactly (1/6) 4^-30
        let mut excess = Q::new(1.into(), 6.into());
        for _ in 0..30 {
            excess = excess * Q::new(1.into(), 4.into());
        }
        assert_eq!(parsed.ev.1.clone() - truth, excess);
    }

    /// The herman3 faces of the preregistered loci (the artifact
    /// battery reruns the same loci through the committed files).
    #[test]
    fn herman3_mutant_loci_land_in_class() {
        let (spec, cert) = h3_spec_and_cert();
        let refused = |c: &str| matches!(check_evcert(&spec, c), Verdict::Refuse(_));
        let failed = |c: &str| matches!(check_evcert(&spec, c), Verdict::Fail(_));

        // offset-early: k = 2 claims a 2-step deterministic prologue,
        // but P[T > 1] = 5/8
        let early = cert.replace("offset 1", "offset 2");
        assert!(refused(&early), "offset-early must REFUSE");

        // offset-dropped: the warranted assembly gains the prologue's
        // survival mass 1, so the kept ev-hi sits below it
        let dropped = cert.replace("offset 1\n", "");
        assert!(failed(&dropped), "offset-dropped must FAIL");

        // template sign side-conditions
        let gamma_neg = cert.replace("gamma 1/2", "gamma -1/2");
        assert!(refused(&gamma_neg), "gamma-neg must REFUSE");

        // contraction rate tightened past the spectral radius
        let c_tight = cert.replace("gamma 1/2", "gamma 1/3");
        assert!(failed(&c_tight), "c-tighter must FAIL");

        // a reachable alive state's weight zeroed
        let w_zero = cert.replace("weight s3 1/1", "weight s3 0/1");
        assert!(refused(&w_zero), "w-zero must REFUSE");

        // no-re-entry face: a stored row from the count >= 1 class
        // back into the alive region, certificate re-bound so the
        // shape check is what refuses
        let spec_txt = String::from_utf8(spec.clone()).unwrap();
        let marker = "\"rows\": [\n";
        let reentry = spec_txt
            .replacen(
                marker,
                &format!("{marker}      [\"done_1\", \"s3\", \"1/1\"],\n"),
                1,
            )
            .into_bytes();
        assert_ne!(reentry, spec, "mutant must change the spec bytes");
        let rebound = emit_evcert(&crate::Herman3, 31, &reentry);
        match check_evcert(&reentry, &rebound) {
            Verdict::Refuse(msg) => {
                assert!(
                    msg.contains("outgoing rows"),
                    "re-entry must refuse at the terminal-rows recompute, got: {msg}"
                );
            }
            other => panic!("re-entry must REFUSE, got {other:?}"),
        }

        // scale-doubled positive control (kappa-normalized floor face)
        let mut scaled = String::new();
        for line in cert.lines() {
            let toks: Vec<&str> = line.split_whitespace().collect();
            match toks.as_slice() {
                ["weight", s, w] => {
                    let w = parse_q(w).unwrap() * Q::new(2.into(), 1.into());
                    scaled.push_str(&format!("weight {s} {}/{}\n", w.numer(), w.denom()));
                }
                ["M", v] => {
                    let v = parse_q(v).unwrap() * Q::new(2.into(), 1.into());
                    scaled.push_str(&format!("M {}/{}\n", v.numer(), v.denom()));
                }
                _ => {
                    scaled.push_str(line);
                    scaled.push('\n');
                }
            }
        }
        assert_eq!(
            check_evcert(&spec, &scaled),
            Verdict::Accept { claims: (1, 1, 1) },
            "scale-doubled must ACCEPT (kappa-normalized floor face)"
        );
    }
}
