//! Checker-side decoding: container grammar, envelope verification and the
//! derived theory vocabulary all come from the public qtsl provider (the
//! `glc-check` reader), never from the producer.
use geolog_envelope::body::Op;
use geolog_envelope::cell::{ElemRef, Value};
use qtsl_geolog_provider::container::{parse_container, verify_commits};
use qtsl_geolog_provider::reprovocab::baked::{baked_vocab_from_container, resolve_baked_rev, BakedVocab};

pub struct Opened {
    pub vocab: BakedVocab,
    pub commits: Vec<Vec<Op>>,
    pub deps: Vec<Vec<[u8; 32]>>,
}

pub fn open(bytes: &[u8]) -> Result<Opened, String> {
    let c = parse_container(bytes)?;
    let pin = c
        .prereqs
        .iter()
        .map(|p| p.chid)
        .find(|chid| resolve_baked_rev(chid).is_some())
        .ok_or("no baked roster revision among the prerequisites")?;
    let entry = resolve_baked_rev(&pin).unwrap();
    let vocab = baked_vocab_from_container((entry.container)(), pin)?;
    let roster: Vec<[u8; 32]> = c.prereqs.iter().map(|p| p.chid).collect();
    let vc = verify_commits(&c, &roster)?;
    Ok(Opened {
        vocab,
        deps: vc.iter().map(|v| v.deps.clone()).collect(),
        commits: vc.into_iter().map(|v| v.ops).collect(),
    })
}

impl Opened {
    /// Declared name of an external reference (sort or function), if any.
    pub fn name_of(&self, ci: usize, r: &ElemRef) -> Option<String> {
        match r {
            ElemRef::External { dep_index, counter } => {
                let rev = *self.deps[ci].get(*dep_index as usize)?;
                let chid = (rev, *counter);
                let ix = &self.vocab.index;
                ix.fun_by_chid(chid)
                    .map(|f| f.name.clone())
                    .or_else(|| ix.sorts_by_chid.get(&chid).cloned())
            }
            ElemRef::Local(_) => None,
        }
    }
}

pub fn show_val(c: &Opened, v: &Value) -> String {
    match v {
        Value::Ref(ElemRef::Local(n)) => format!("@{n}"),
        Value::Ref(r) => c.name_of(0, r).unwrap_or_else(|| format!("{r:?}")),
        other => format!("{other:?}"),
    }
}

pub fn show_op(c: &Opened, op: &Op) -> String {
    let nm = |r: &ElemRef| c.name_of(0, r).unwrap_or_else(|| format!("{r:?}"));
    match op {
        Op::Elta { sort } => format!("elt {}", nm(sort)),
        Op::Rela { rel, args } => format!(
            "rel {}({})",
            nm(rel),
            args.iter().map(|a| show_val(c, a)).collect::<Vec<_>>().join(", ")
        ),
        Op::Funa { fun, args, value } => format!(
            "fun {}({}) = {}",
            nm(fun),
            args.iter().map(|a| show_val(c, a)).collect::<Vec<_>>().join(", "),
            show_val(c, value)
        ),
        other => format!("{other:?}"),
    }
}

// ---------------------------------------------------------------------
// Sealed net: exactly what the container bytes carry about the plated
// System, read through the declared names (no producer code involved).
// ---------------------------------------------------------------------

use geolog_envelope::cell::LiteralView;
use num_bigint::{BigInt, BigUint};
use std::collections::BTreeMap;

/// Exact rational as (num, den), den > 0; the Mini-Fab cells are small.
pub type Q = (i64, i64);

#[derive(Debug, Clone, PartialEq)]
pub enum Clock {
    Forced(Q),
    Carved { floor: Q, ceil: Option<Q> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    /// landing enclosure of the produced colour, per coordinate (lo, hi)
    pub image: Vec<(Q, Q)>,
    /// reset polynomial per coordinate: terms (coef, exponents)
    pub reset: Vec<Vec<(Q, Vec<u64>)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SealedTr {
    /// (plate, consume count, read count), plates ascending
    pub sel: Vec<(usize, u64, u64)>,
    pub clock: Clock,
    pub scale: Option<Scale>,
}

#[derive(Debug, Clone)]
pub struct SealedNet {
    pub mech: String,
    pub marking: Vec<u64>,
    pub trs: Vec<SealedTr>,
}

fn lit_q(v: &Value) -> Result<Q, String> {
    let Value::Lit(l) = v else { return Err(format!("expected Q literal, got {v:?}")) };
    match l.view() {
        LiteralView::Q { num, den } => {
            let n = BigInt::from_signed_bytes_le(num);
            let d = BigUint::from_bytes_le(den);
            let n: i64 = n.try_into().map_err(|_| "Q num too large")?;
            let d: i64 = i64::try_from(d).map_err(|_| "Q den too large")?;
            Ok((n, d))
        }
        other => Err(format!("expected Q, got {other:?}")),
    }
}
fn lit_nat(v: &Value) -> Result<u64, String> {
    let Value::Lit(l) = v else { return Err(format!("expected Nat literal, got {v:?}")) };
    match l.view() {
        LiteralView::Nat(n) => Ok(n),
        other => Err(format!("expected Nat, got {other:?}")),
    }
}
fn lit_str(v: &Value) -> Result<String, String> {
    let Value::Lit(l) = v else { return Err(format!("expected Str literal, got {v:?}")) };
    match l.view() {
        LiteralView::Str(s) => Ok(s.to_string()),
        other => Err(format!("expected Str, got {other:?}")),
    }
}
fn lit_shape(v: &Value) -> Result<Vec<u64>, String> {
    let Value::Lit(l) = v else { return Err(format!("expected Shape literal, got {v:?}")) };
    match l.view() {
        LiteralView::Shape(s) => Ok(s.iter().collect()),
        other => Err(format!("expected Shape, got {other:?}")),
    }
}
fn loc(v: &Value) -> Result<u32, String> {
    match v {
        Value::Ref(ElemRef::Local(n)) => Ok(*n),
        other => Err(format!("expected a same-body element, got {other:?}")),
    }
}

impl Opened {
    /// Read the plated System section of commit `ci` into a [`SealedNet`].
    /// Refuses (naming the row) on anything outside the expected grammar.
    pub fn sealed_net(&self, ci: usize) -> Result<SealedNet, String> {
        let ops = &self.commits[ci];
        // constructor table: element -> (ctor name, args); field table.
        let mut ctor: BTreeMap<u32, (String, Vec<Value>)> = BTreeMap::new();
        let mut field: BTreeMap<(String, u32), Value> = BTreeMap::new();
        let mut rels: Vec<(String, Vec<Value>)> = Vec::new();
        for op in ops {
            match op {
                Op::Funa { fun, args, value } => {
                    let Some(name) = self.name_of(ci, fun) else { continue };
                    if args.len() == 1 && name.contains('/') {
                        field.insert((name, loc(&args[0])?), value.clone());
                    } else if let Value::Ref(ElemRef::Local(x)) = value {
                        ctor.insert(*x, (name, args.clone()));
                    }
                }
                Op::Rela { rel, args } => {
                    if let Some(name) = self.name_of(ci, rel) {
                        rels.push((name, args.clone()));
                    }
                }
                _ => {}
            }
        }
        let c = |x: u32, want: &str| -> Result<&Vec<Value>, String> {
            match ctor.get(&x) {
                Some((n, a)) if n == want => Ok(a),
                Some((n, _)) => Err(format!("@{x}: expected {want}, found {n}")),
                None => Err(format!("@{x}: no constructor row (expected {want})")),
            }
        };
        // the subject mechanism = the one carrying mech_bag_marking rows
        let mut by_mech: BTreeMap<u32, Vec<(u64, u32)>> = BTreeMap::new();
        let mut wire: BTreeMap<(u32, u32), u64> = BTreeMap::new();
        for (n, a) in &rels {
            match n.as_str() {
                "mech_bag_marking" => by_mech.entry(loc(&a[1])?).or_default().push((lit_nat(&a[0])?, loc(&a[2])?)),
                "mech_bag_wire" => {
                    wire.insert((loc(&a[0])?, loc(&a[1])?), lit_nat(&a[2])?);
                }
                _ => {}
            }
        }
        if by_mech.len() != 1 {
            return Err(format!("expected exactly one marked mechanism, found {}", by_mech.len()));
        }
        let (&mech, toks) = by_mech.iter().next().unwrap();
        let mut marking = vec![None; toks.len()];
        for &(count, tok) in toks {
            let ix = *wire.get(&(mech, tok)).ok_or("marking token without a wire index")? as usize;
            if ix >= marking.len() || marking[ix].is_some() {
                return Err(format!("wire index {ix} out of range or duplicated"));
            }
            marking[ix] = Some(count);
        }
        let marking: Vec<u64> = marking.into_iter().map(|m| m.ok_or("gap in wire indices")).collect::<Result<_, _>>()?;
        let mech_name = lit_str(field.get(&("Mech/name".into(), mech)).ok_or("subject mechanism has no name")?)?;
        // the subject System: SysMech(mech)
        let sys = ctor
            .iter()
            .find(|(_, (n, a))| n == "SysMech" && a.len() == 1 && loc(&a[0]).ok() == Some(mech))
            .map(|(x, _)| *x)
            .ok_or("no SysMech row for the subject mechanism")?;
        let mut tr_ids: Vec<u32> = rels
            .iter()
            .filter(|(n, a)| n == "sys_transition" && loc(&a[0]).ok() == Some(sys))
            .map(|(_, a)| loc(&a[1]))
            .collect::<Result<_, _>>()?;
        tr_ids.sort();
        let mut scale_of: BTreeMap<u32, u32> = BTreeMap::new();
        for (n, a) in &rels {
            if n == "tr_scale" {
                scale_of.insert(loc(&a[1])?, loc(&a[0])?);
            }
        }
        let poly = |mut x: u32| -> Result<Vec<(Q, Vec<u64>)>, String> {
            let mut terms = Vec::new();
            loop {
                match ctor.get(&x) {
                    Some((n, _)) if n == "pZero" => return Ok(terms),
                    Some((n, a)) if n == "pTerm" => {
                        terms.push((lit_q(&a[0])?, lit_shape(&a[1])?));
                        x = loc(&a[2])?;
                    }
                    other => return Err(format!("@{x}: not a polynomial row: {other:?}")),
                }
            }
        };
        let mut trs = Vec::new();
        for &t in &tr_ids {
            // grain: grTuple(sellist)
            let g = loc(field.get(&("Transition/grain".into(), t)).ok_or("transition without grain")?)?;
            let mut l = loc(&c(g, "grTuple")?[0])?;
            let mut sel = Vec::new();
            loop {
                match ctor.get(&l) {
                    Some((n, _)) if n == "sslNil" => break,
                    Some((n, a)) if n == "sslCons" => {
                        let s = c(loc(&a[0])?, "selMk")?;
                        sel.push((lit_nat(&s[1])? as usize, lit_nat(&s[0])?, lit_nat(&s[2])?));
                        l = loc(&a[1])?;
                    }
                    other => return Err(format!("@{l}: not a selection list row: {other:?}")),
                }
            }
            sel.sort();
            let k = loc(field.get(&("Transition/clock".into(), t)).ok_or("transition without clock")?)?;
            let clock = match ctor.get(&k) {
                Some((n, a)) if n == "tkForced" => Clock::Forced(lit_q(&a[0])?),
                Some((n, a)) if n == "tkCarved" => {
                    let e = loc(&a[0])?;
                    let ceil = match ctor.get(&e) {
                        Some((n, a)) if n == "enFin" => Some(lit_q(&a[0])?),
                        Some((n, _)) if n == "enInf" => None,
                        other => return Err(format!("@{e}: carved ceil not an EnnRat ctor: {other:?}")),
                    };
                    Clock::Carved { floor: lit_q(&a[1])?, ceil }
                }
                other => return Err(format!("@{k}: clock wing outside this lane: {other:?}")),
            };
            let scale = match scale_of.get(&t) {
                None => None,
                Some(&sv) => {
                    let a = c(sv, "mkScaleView")?;
                    let mut bl = loc(&a[0])?;
                    let mut branches = Vec::new();
                    loop {
                        match ctor.get(&bl) {
                            Some((n, _)) if n == "sbNil" => break,
                            Some((n, b)) if n == "sbCons" => {
                                branches.push((loc(&b[0])?, lit_q(&b[2])?));
                                bl = loc(&b[1])?;
                            }
                            other => return Err(format!("@{bl}: not a branch list row: {other:?}")),
                        }
                    }
                    if branches.len() != 1 || branches[0].1 != (1, 1) {
                        return Err("scale view is not a single Dirac branch".into());
                    }
                    let sn = c(branches[0].0, "srNext")?;
                    let mut image = Vec::new();
                    let mut bx = loc(&sn[0])?;
                    loop {
                        match ctor.get(&bx) {
                            Some((n, _)) if n == "boxNil" => break,
                            Some((n, b)) if n == "boxCons" => {
                                let iv = c(loc(&b[0])?, "mkInterval")?;
                                let (x, y) = (lit_q(&iv[0])?, lit_q(&iv[1])?);
                                // cross-multiplied order: (lo, hi)
                                image.push(if x.0 * y.1 <= y.0 * x.1 { (x, y) } else { (y, x) });
                                bx = loc(&b[1])?;
                            }
                            other => return Err(format!("@{bx}: not a box row: {other:?}")),
                        }
                    }
                    let mut reset = Vec::new();
                    let mut pv = loc(&sn[1])?;
                    loop {
                        match ctor.get(&pv) {
                            Some((n, _)) if n == "pvNil" => break,
                            Some((n, b)) if n == "pvCons" => {
                                reset.push(poly(loc(&b[0])?)?);
                                pv = loc(&b[1])?;
                            }
                            other => return Err(format!("@{pv}: not a polyvec row: {other:?}")),
                        }
                    }
                    Some(Scale { image, reset })
                }
            };
            trs.push(SealedTr { sel, clock, scale });
        }
        Ok(SealedNet { mech: mech_name, marking, trs })
    }
}
