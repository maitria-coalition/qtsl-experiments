//! Replay an explicit firing table against a net whose SEALED part (plates,
//! marking, pre-sets, read arcs, clocks, same-place tick resets and their
//! landing enclosures) is decoded from the container bytes, and whose
//! UNSEALED part (post-sets of cross-place rows, consumed-side guards of
//! finishes/ship, the start colour) is the transcribed residue below.
//!
//! Round semantics (carved clocks at the forced point, floor = ceil = 1):
//! a row is DUE at round r iff enabled at the end of round r-1; due rows
//! fire at most once per round, in phase order finish/ship -> release ->
//! start/unlock -> tick, each needing its token pick at its turn; after
//! the batch every due row left unfired and still enabled is a HOLD,
//! lawful only for the caller's allowed schemas. Independent
//! reimplementation of the semantics documented in mtk
//! carved_script.rs (whose crate cannot be built from public sources).
use crate::decode::{Clock, SealedNet};
use std::collections::BTreeMap;

/// Kempf process times, steps 1..6 (transcribed: mtk minifab.rs `T`).
pub const T: [i64; 6] = [225, 30, 55, 50, 255, 10];
/// Station per step (transcribed: mtk minifab.rs `STATION`).
pub const STATION: [usize; 6] = [0, 1, 2, 1, 0, 2];
pub const P_QPEND: usize = 0;
pub fn p_queue(k: usize) -> usize { k }
pub fn p_proc(k: usize) -> usize { 6 + k }
pub fn p_mach(s: usize) -> usize { 13 + s }
pub const P_GATE: usize = 16;
pub const P_KEY: usize = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role { Release, Start(usize), Tick(usize), Finish(usize), Ship, Unlock }

impl Role {
    pub fn name(&self) -> String {
        match self {
            Role::Release => "release".into(),
            Role::Start(k) => format!("start{k}"),
            Role::Tick(k) => format!("tick{k}"),
            Role::Finish(k) => format!("finish{k}"),
            Role::Ship => "ship".into(),
            Role::Unlock => "unlock-start6".into(),
        }
    }
    fn phase(&self) -> u8 {
        match self {
            Role::Finish(_) | Role::Ship => 0,
            Role::Release => 1,
            Role::Start(_) | Role::Unlock => 2,
            Role::Tick(_) => 3,
        }
    }
}

/// One compiled row: consumed patterns (plate, guard), produced tokens
/// (plate, colour rule).
#[derive(Debug, Clone)]
pub struct Row {
    pub role: Role,
    pub consumed: Vec<(usize, Option<(i64, i64)>)>,
    pub produced: Vec<(usize, Prod)>,
}
#[derive(Debug, Clone, Copy)]
pub enum Prod { Unit, Const(i64), Affine(i64, i64) /* c + a*w */ }

/// Classify a decoded row by its sealed pre-set and attach the residue.
/// Every sealed fact the residue relies on is CHECKED here against the
/// bytes; a mismatch refuses, naming the row.
pub fn compile(net: &SealedNet) -> Result<Vec<Row>, String> {
    let np = net.marking.len();
    if np != 16 && np != 17 && np != 18 {
        return Err(format!("{np} plates: not a Mini-Fab roster this replay knows"));
    }
    let gated_flat = np == 18;
    let mut tick_t = [None::<i64>; 7];
    let mut rows = Vec::new();
    for (i, t) in net.trs.iter().enumerate() {
        let unit = (1i64, 1i64);
        match &t.clock {
            Clock::Carved { floor, ceil: Some(c) } if *floor == unit && *c == unit => {}
            Clock::Forced(d) if *d == unit => {}
            other => return Err(format!("tr#{i}: clock {other:?} is not dwell-exactly-1")),
        }
        if t.sel.iter().any(|s| s.2 != 0) {
            return Err(format!("tr#{i}: native read arcs — this replay wants the flattened lane"));
        }
        let pre: Vec<(usize, u64)> = t.sel.iter().map(|s| (s.0, s.1)).collect();
        let role = match (pre.as_slice(), &t.scale) {
            ([(0, 1)], None) => Role::Release,
            ([(p, 1)], Some(_)) if (7..=12).contains(p) => Role::Tick(p - 6),
            ([(p, 1)], None) if (7..=11).contains(p) => Role::Finish(p - 6),
            ([(12, 1)], None) => Role::Ship,
            ([(q, 1), (m, 1)], None) if (1..=6).contains(q) && *m == p_mach(STATION[q - 1]) && !(gated_flat && *q == 6) => Role::Start(*q),
            ([(6, 1), (15, 1), (16, 1)], None) if gated_flat => Role::Start(6),
            ([(6, n), (17, 1)], None) if gated_flat => { let _ = n; Role::Unlock }
            _ => return Err(format!("tr#{i}: pre-set {pre:?} matches no Mini-Fab row")),
        };
        let row = match role {
            Role::Release => Row { role, consumed: vec![(P_QPEND, None)], produced: vec![(p_queue(1), Prod::Unit)] },
            Role::Tick(k) => {
                let sc = t.scale.as_ref().unwrap();
                // sealed: reset w -> w - 1 and landing image [0, T_k - 1]
                let want_reset = vec![((-1, 1), vec![0]), ((1, 1), vec![1])];
                let mut got = sc.reset.get(0).cloned().unwrap_or_default();
                got.sort();
                let mut w = want_reset.clone();
                w.sort();
                if sc.reset.len() != 1 || got != w {
                    return Err(format!("tr#{i} tick{k}: sealed reset is not w - 1"));
                }
                let (lo, hi) = sc.image[0];
                if lo != (0, 1) || hi.1 != 1 {
                    return Err(format!("tr#{i} tick{k}: image {:?} not [0, integer]", sc.image));
                }
                let tk = hi.0 + 1;
                if tk != T[k - 1] {
                    return Err(format!("tr#{i} tick{k}: sealed work {tk} != transcribed T = {}", T[k - 1]));
                }
                tick_t[k] = Some(tk);
                // guard = preimage of the sealed image under the sealed reset
                Row { role, consumed: vec![(p_proc(k), Some((1, tk)))], produced: vec![(p_proc(k), Prod::Affine(-1, 1))] }
            }
            Role::Finish(k) => Row {
                role,
                consumed: vec![(p_proc(k), Some((0, 0)))],
                produced: vec![(p_queue(k + 1), Prod::Unit), (p_mach(STATION[k - 1]), Prod::Unit)],
            },
            Role::Ship => Row { role, consumed: vec![(p_proc(6), Some((0, 0)))], produced: vec![(p_mach(STATION[5]), Prod::Unit)] },
            Role::Start(k) => {
                let mut consumed = vec![(p_queue(k), None), (p_mach(STATION[k - 1]), None)];
                let mut produced = vec![(p_proc(k), Prod::Const(T[k - 1]))];
                if gated_flat && k == 6 {
                    consumed.push((P_GATE, None));
                    produced.push((P_GATE, Prod::Unit)); // consume-and-return gate
                }
                Row { role, consumed, produced }
            }
            Role::Unlock => {
                let n = pre[0].1 as usize;
                let mut consumed = vec![(P_KEY, None)];
                consumed.extend(std::iter::repeat((p_queue(6), None)).take(n));
                let mut produced: Vec<(usize, Prod)> = std::iter::repeat((p_queue(6), Prod::Unit)).take(n).collect();
                produced.push((P_GATE, Prod::Unit));
                Row { role, consumed, produced }
            }
        };
        rows.push(row);
    }
    let mut seen: Vec<Role> = rows.iter().map(|r| r.role).collect();
    seen.sort();
    let n0 = seen.len();
    seen.dedup();
    if seen.len() != n0 {
        return Err("two decoded rows share a role".into());
    }
    if (1..=6).any(|k| tick_t[k].is_none()) {
        return Err("a tick row is missing from the sealed roster".into());
    }
    // canonical (model) order: matters for within-phase tie order
    rows.sort_by_key(|r| r.role);
    Ok(rows)
}

#[derive(Debug, Clone, Copy)]
pub struct Firing { pub role: Role, pub pick_w: Option<i64> }

#[derive(Debug, Clone)]
pub struct Outcome {
    pub n: u64,
    pub last_arrival_round: u64,
    pub pre_goal_firings: u64,
    pub holds_by_schema: BTreeMap<String, u64>,
    pub max_batch: usize,
}

fn pick(marking: &[Vec<i64>], row: &Row, want: Option<i64>) -> Option<Vec<(usize, usize)>> {
    let mut picks: Vec<(usize, usize)> = Vec::new();
    for &(plate, guard) in &row.consumed {
        let j = marking[plate].iter().enumerate().position(|(j, &w)| {
            !picks.contains(&(plate, j))
                && match guard {
                    None => true,
                    Some((lo, hi)) => lo <= w && w <= hi && want.map_or(true, |x| x == w),
                }
        })?;
        picks.push((plate, j));
    }
    Some(picks)
}

/// Replay `table` (round -> firings) on the compiled rows from the sealed
/// marking, with the pending count overridden only if `n_override`.
pub fn replay(
    net: &SealedNet,
    rows: &[Row],
    table: &BTreeMap<u64, Vec<Firing>>,
    n: u64,
    hold_allowed: &[&str],
    max_rounds: u64,
) -> Result<Outcome, String> {
    replay_inner(net, rows, table, n, hold_allowed, max_rounds, None).map(|r| r.0)
}

/// Lawfulness of a table PREFIX up to `upto` rounds (no goal demanded);
/// returns the marking at the end of round `upto`.
pub fn replay_prefix(
    net: &SealedNet,
    rows: &[Row],
    table: &BTreeMap<u64, Vec<Firing>>,
    n: u64,
    hold_allowed: &[&str],
    upto: u64,
) -> Result<Vec<Vec<i64>>, String> {
    replay_inner(net, rows, table, n, hold_allowed, upto, Some(())).map(|r| r.1)
}

fn replay_inner(
    net: &SealedNet,
    rows: &[Row],
    table: &BTreeMap<u64, Vec<Firing>>,
    n: u64,
    hold_allowed: &[&str],
    max_rounds: u64,
    prefix: Option<()>,
) -> Result<(Outcome, Vec<Vec<i64>>), String> {
    let mut marking: Vec<Vec<i64>> = net.marking.iter().map(|&m| vec![0; m as usize]).collect();
    if marking[P_QPEND].len() as u64 != n {
        return Err(format!("sealed marking has {} pending lots, table is for {n}", marking[P_QPEND].len()));
    }
    let ix: BTreeMap<Role, usize> = rows.iter().enumerate().map(|(i, r)| (r.role, i)).collect();
    let (mut arrivals, mut firings, mut max_batch) = (0u64, 0u64, 0usize);
    let mut holds: BTreeMap<String, u64> = BTreeMap::new();
    let empty = Vec::new();
    for round in 1..=max_rounds {
        let due: Vec<usize> = (0..rows.len()).filter(|&i| pick(&marking, &rows[i], None).is_some()).collect();
        let mut batch: Vec<(u8, usize, Option<i64>)> = Vec::new();
        for f in table.get(&round).unwrap_or(&empty) {
            let &i = ix.get(&f.role).ok_or_else(|| format!("round {round}: {} not in the sealed roster", f.role.name()))?;
            batch.push((rows[i].role.phase(), i, f.pick_w));
        }
        batch.sort();
        let mut fired: Vec<usize> = Vec::new();
        for &(_, i, want) in &batch {
            let nm = rows[i].role.name();
            if !due.contains(&i) {
                return Err(format!("round {round}: scripted {nm} is not due"));
            }
            if fired.contains(&i) {
                return Err(format!("round {round}: scripted {nm} twice (meter law)"));
            }
            let picks = pick(&marking, &rows[i], want).ok_or_else(|| format!("round {round}: {nm} pick {want:?} unavailable"))?;
            let var = picks.iter().find(|(p, _)| rows[i].consumed.iter().any(|c| c.0 == *p && c.1.is_some())).map(|&(p, j)| marking[p][j]);
            let mut sorted = picks.clone();
            sorted.sort_by(|a, b| b.cmp(a));
            for (p, j) in sorted {
                marking[p].remove(j);
            }
            for &(p, pr) in &rows[i].produced {
                let w = match pr {
                    Prod::Unit => 0,
                    Prod::Const(c) => c,
                    Prod::Affine(c, a) => c + a * var.expect("affine reset needs a coloured pick"),
                };
                marking[p].push(w);
            }
            fired.push(i);
            firings += 1;
            if rows[i].role == Role::Finish(5) {
                arrivals += 1;
                if arrivals == n {
                    return Ok((Outcome { n, last_arrival_round: round, pre_goal_firings: firings, holds_by_schema: holds, max_batch: max_batch.max(fired.len()) }, marking));
                }
            }
        }
        max_batch = max_batch.max(fired.len());
        for &i in &due {
            if !fired.contains(&i) && pick(&marking, &rows[i], None).is_some() {
                let nm = rows[i].role.name();
                if !hold_allowed.contains(&nm.as_str()) {
                    return Err(format!("round {round}: {nm} due, unfired, still enabled — hold outside {hold_allowed:?}"));
                }
                *holds.entry(nm).or_insert(0) += 1;
            }
        }
    }
    if prefix.is_some() {
        let o = Outcome { n, last_arrival_round: 0, pre_goal_firings: firings, holds_by_schema: holds, max_batch };
        return Ok((o, marking));
    }
    Err(format!("goal not reached within {max_rounds} rounds ({arrivals}/{n})"))
}

// ---------------- the two schedule families (closed form, transcribed
// from mtk carved_script.rs pi_star_lots / wc_lots / table_of) ---------

pub type Lot = (u64, [(u64, u64, u64); 5]); // rel, per step (s, f, first tick)

fn chain_from(steps: &mut [(u64, u64, u64); 5], start1: u64) -> u64 {
    let t1 = T[0] as u64;
    steps[0] = (start1, start1 + t1 + 1, start1 + 1);
    let mut t = start1 + t1 + 1;
    for k in 2..=4usize {
        let tk = T[k - 1] as u64;
        steps[k - 1] = (t + 1, t + 2 + tk, t + 2);
        t += 2 + tk;
    }
    t
}

pub fn pi_star_lots(n: u64) -> Vec<Lot> {
    let t5 = T[4] as u64;
    let s1 = |j: u64| if j < n.min(4) { 2 + 227 * j } else if j == 4 { 910 } else if j == 5 { 1141 } else { 1141 + 255 * (j - 5) };
    let slot = |j: u64| if j == 1 { 371 } else if n <= 2 { 626 } else if j == 2 { 628 } else if n == 3 { 883 } else if j == 3 { 885 } else { 885 + 255 * (j - 3) };
    (0..n)
        .map(|l| {
            let mut st = [(0, 0, 0); 5];
            let q5 = chain_from(&mut st, s1(l));
            let first = slot(l + 1);
            let sv = (q5 + 1).max(first - 1);
            assert!(sv <= first - 1);
            st[4] = (sv, first + t5, first);
            (l + 1, st)
        })
        .collect()
}

pub fn wc_lots(n: u64) -> Vec<Lot> {
    assert!(n >= 3);
    let t5 = T[4] as u64;
    let s1 = |j: u64| match j { 0 => (2, 228, 3), 1 => (3, 453, 228), 2 => (229, 678, 453), _ => { let s = 255 * (j + 1) - 309; (s, s + 226, s + 1) } };
    let mut lots: Vec<Lot> = Vec::new();
    let (mut f1v, mut f4v) = (Vec::new(), Vec::new());
    for l in 0..n {
        let (s, f, t0) = s1(l);
        let mut st = [(0, 0, 0); 5];
        st[0] = (s, f, t0);
        let mut t = f;
        for k in 2..=4usize {
            let tk = T[k - 1] as u64;
            st[k - 1] = (t + 1, t + 2 + tk, t + 2);
            t += 2 + tk;
        }
        f1v.push(f);
        f4v.push(t);
        lots.push((l + 1, st));
    }
    for l in 0..n as usize {
        let first = 455 + t5 * l as u64;
        let sv = if (l as u64) < n - 1 { f1v[l + 1] + 1 } else { f4v[l] + 1 };
        let sv = sv.max(f4v[l] + 1);
        assert!(sv <= first - 1);
        lots[l].1[4] = (sv, first + t5, first);
    }
    lots
}

pub fn table_of(lots: &[Lot]) -> BTreeMap<u64, Vec<Firing>> {
    let mut by: BTreeMap<u64, Vec<Firing>> = BTreeMap::new();
    for (rel, steps) in lots {
        by.entry(*rel).or_default().push(Firing { role: Role::Release, pick_w: None });
        for (i, &(s, f, t0)) in steps.iter().enumerate() {
            let k = i + 1;
            by.entry(s).or_default().push(Firing { role: Role::Start(k), pick_w: None });
            for j in 0..T[k - 1] {
                by.entry(t0 + j as u64).or_default().push(Firing { role: Role::Tick(k), pick_w: Some(T[k - 1] - j) });
            }
            by.entry(f).or_default().push(Firing { role: Role::Finish(k), pick_w: Some(0) });
        }
    }
    by
}

/// Same text format as carved_script.rs canonical_of_lots (differential
/// surface against the vendored original dump
/// originals/maitria/w6xx-receipts/w676-idt-replay/rust-tables.txt).
pub fn canonical(label: &str, lots: &[Lot]) -> String {
    let n = lots.len();
    let mut out = String::new();
    for (l, (rel, steps)) in lots.iter().enumerate() {
        out.push_str(&format!("{label} N={n} lot={l} rel={rel}\n"));
        for (i, &(s, f, t0)) in steps.iter().enumerate() {
            out.push_str(&format!("{label} N={n} lot={l} k={} s={s} f={f} ticks={t0}..{}\n", i + 1, t0 + T[i] as u64 - 1));
        }
    }
    out
}

/// The book's claimed optimum, T_opt(N) = 255N + 371 + d(N).
pub fn t_opt(n: u64) -> u64 { 255 * n + 371 + if n <= 2 { 0 } else if n == 3 { 2 } else { 4 } }

/// INSTANCE ADAPTATION (not byte-faithful): the sealed container is the
/// N = 84 instance. To replay at another N, exactly two sealed facts are
/// substituted in the decoded net, and nothing else: the pending-lot count
/// (initial marking of q_pend, sealed 84) and the unlock threshold (the
/// q6 multiplicity in the unlock row's pre-set, sealed 84; present only in
/// the 18-plate supervised lane). Returns the substituted net plus the
/// (sealed, substituted) values for the receipt. Refuses if either sealed
/// value is not the expected 84, so a different container is not silently
/// adapted.
pub fn adapt_instance(net: &SealedNet, n: u64) -> Result<(SealedNet, u64, Option<u64>), String> {
    let mut a = net.clone();
    let sealed_pend = a.marking[P_QPEND];
    if sealed_pend != 84 {
        return Err(format!("sealed pending count {sealed_pend} != 84: not the instance this adaptation is written for"));
    }
    a.marking[P_QPEND] = n;
    let mut sealed_unlock = None;
    for t in a.trs.iter_mut() {
        if t.sel.len() == 2 && t.sel[0].0 == p_queue(6) && t.sel[1].0 == P_KEY {
            if t.sel[0].1 != 84 {
                return Err(format!("sealed unlock threshold {} != 84", t.sel[0].1));
            }
            sealed_unlock = Some(t.sel[0].1);
            t.sel[0].1 = n;
        }
    }
    if a.marking.len() == 18 && sealed_unlock.is_none() {
        return Err("18-plate supervised lane without an unlock row (q6 x 84, key): refusing to adapt".into());
    }
    Ok((a, sealed_pend, sealed_unlock))
}
