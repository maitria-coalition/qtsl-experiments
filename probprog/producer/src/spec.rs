//! Certificate-spec emission: materialize a layered program chain into
//! an explicit finite stored-row chain (states time-stamped where the
//! query variable needs it) plus canonical bracket claims, as the JSON
//! the emitter binary seals into a certificate container.
//!
//! Materialization discipline: alive states are (state, layer) pairs
//! reachable within the horizon; the layer-`horizon` frontier is left
//! ROWLESS — under the species' rowless-is-absorbing evaluation
//! convention that is exactly the truncation, sound by construction.
//! Class states are one absorbing rowless state per terminal class;
//! rejection is one rowless state. Claims are the canonical (tight)
//! brackets from the exact evaluation; the checker recomputes and
//! admits any claim containing the canonical interval, so canonical
//! claims are the sharpest admissible.

use crate::{evaluate, Chain, Dst, Q};
use num_traits::One;
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn rq(x: &Q) -> String {
    format!("{}/{}", x.numer(), x.denom())
}

fn jstr(s: &str) -> String {
    // states are machine-named [a-z0-9_-]; keep the escape honest anyway
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Emit the certificate spec JSON for `chain` at `horizon`, claiming
/// posterior + absorb brackets for every absorbed class. Dispatches on
/// the chain's own declared shape: time-homogeneous chains materialize
/// states once (the wire species checks arbitrary stored-row chains);
/// layered chains get the timestamped product.
pub fn emit_spec(chain: &dyn Chain, horizon: u32) -> String {
    if chain.time_homogeneous() {
        emit_spec_homogeneous(chain, horizon)
    } else {
        emit_spec_layered(chain, horizon)
    }
}

/// Compact emission for time-homogeneous chains: one wire state per
/// reachable alive state (named `s<x>`), rows stored once. Sound for
/// exactly the chains whose `succ` ignores the step index — guarded by
/// a row-identity check at materialization, not by trust in the flag.
pub fn emit_spec_homogeneous(chain: &dyn Chain, horizon: u32) -> String {
    let tr = evaluate(chain, horizon);

    let mut names: BTreeMap<u64, String> = BTreeMap::new();
    let mut rows: Vec<(String, String, Q)> = Vec::new();
    let mut class_states: BTreeMap<u32, String> = BTreeMap::new();
    let mut used_reject = false;

    let mut queue: Vec<u64> = vec![chain.init()];
    names.insert(chain.init(), format!("s{}", chain.init()));
    while let Some(x) = queue.pop() {
        let src = names[&x].clone();
        let row = chain.succ(x, 0);
        debug_assert_eq!(
            row,
            chain.succ(x, 1),
            "declared time-homogeneous but succ({x}, 0) != succ({x}, 1)"
        );
        for (dst, w) in row {
            match dst {
                Dst::Alive(y) => {
                    let name = names
                        .entry(y)
                        .or_insert_with(|| {
                            queue.push(y);
                            format!("s{y}")
                        })
                        .clone();
                    rows.push((src.clone(), name, w));
                }
                Dst::Done(c) => {
                    let name = class_states
                        .entry(c)
                        .or_insert_with(|| format!("done_{c}"))
                        .clone();
                    rows.push((src.clone(), name, w));
                }
                Dst::Reject => {
                    used_reject = true;
                    rows.push((src.clone(), "reject".into(), w));
                }
            }
        }
    }

    let mut states: Vec<String> = names.values().cloned().collect();
    states.extend(class_states.values().cloned());
    if used_reject {
        states.push("reject".into());
    }
    states.sort();
    states.dedup();

    render_spec(
        &states,
        &names[&chain.init()],
        rows,
        &class_states,
        used_reject,
        horizon,
        &tr,
    )
}

/// Layered emission (states timestamped per layer) — the form for
/// chains whose rows depend on the step index, e.g. where the class
/// records absorption time.
pub fn emit_spec_layered(chain: &dyn Chain, horizon: u32) -> String {
    let tr = evaluate(chain, horizon);

    // ---- materialize states ----
    // alive layer states: reachable (x, t); frontier layer rowless.
    let mut alive_names: BTreeMap<(u64, u32), String> = BTreeMap::new();
    let mut frontier: Vec<(u64, u32)> = Vec::new();
    let mut layer: Vec<u64> = vec![chain.init()];
    alive_names.insert((chain.init(), 0), format!("a{}_{}", 0, chain.init()));
    let mut rows: Vec<(String, String, Q)> = Vec::new();
    let mut class_states: BTreeMap<u32, String> = BTreeMap::new();
    let mut used_reject = false;

    for t in 0..horizon {
        let mut next: Vec<u64> = Vec::new();
        for &x in &layer {
            let src = alive_names[&(x, t)].clone();
            for (dst, w) in chain.succ(x, t) {
                match dst {
                    Dst::Alive(y) => {
                        let key = (y, t + 1);
                        let name = alive_names
                            .entry(key)
                            .or_insert_with(|| {
                                next.push(y);
                                format!("a{}_{}", t + 1, y)
                            })
                            .clone();
                        rows.push((src.clone(), name, w));
                    }
                    Dst::Done(c) => {
                        let name = class_states
                            .entry(c)
                            .or_insert_with(|| format!("done_{c}"))
                            .clone();
                        rows.push((src.clone(), name, w));
                    }
                    Dst::Reject => {
                        used_reject = true;
                        rows.push((src.clone(), "reject".into(), w));
                    }
                }
            }
        }
        // dedupe next layer (entry() above pushes once per new key)
        next.sort_unstable();
        next.dedup();
        layer = next;
    }
    for &x in &layer {
        frontier.push((x, horizon));
    }

    // ---- states roster ----
    let mut states: Vec<String> = alive_names.values().cloned().collect();
    states.extend(class_states.values().cloned());
    if used_reject {
        states.push("reject".into());
    }
    states.sort();
    states.dedup();
    let _ = &frontier; // frontier states are in alive_names already

    render_spec(
        &states,
        &alive_names[&(chain.init(), 0)],
        rows,
        &class_states,
        used_reject,
        horizon,
        &tr,
    )
}

/// Shared JSON assembly: coalesce parallel edges (the stored grain
/// holds at most one probability per ordered state pair), then render
/// the emitter's stable spec schema.
fn render_spec(
    states: &[String],
    anchor: &str,
    rows: Vec<(String, String, Q)>,
    class_states: &BTreeMap<u32, String>,
    used_reject: bool,
    horizon: u32,
    tr: &crate::Truncated,
) -> String {
    let mut coalesced: BTreeMap<(String, String), Q> = BTreeMap::new();
    for (s, d, w) in rows {
        *coalesced
            .entry((s, d))
            .or_insert_with(|| Q::new(0.into(), 1.into())) += w;
    }

    let mut out = String::new();
    out.push_str("{\n  \"chain\": {\n    \"states\": [");
    for (i, s) in states.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&jstr(s));
    }
    out.push_str("],\n    \"init\": [[");
    let _ = write!(out, "{}, {}", jstr(anchor), jstr(&rq(&Q::one())));
    out.push_str("]],\n    \"rows\": [\n");
    let mut first = true;
    for ((s, d), w) in &coalesced {
        if !first {
            out.push_str(",\n");
        }
        first = false;
        let _ = write!(out, "      [{}, {}, {}]", jstr(s), jstr(d), jstr(&rq(w)));
    }
    out.push_str("\n    ]\n  },\n  \"query\": {\n");
    let _ = write!(
        out,
        "    \"anchor\": {},\n    \"horizon\": {},\n",
        jstr(anchor),
        horizon
    );
    out.push_str("    \"classes\": [");
    let mut first = true;
    for (c, name) in class_states {
        if !first {
            out.push_str(", ");
        }
        first = false;
        let _ = write!(out, "[{}, {}]", jstr(name), c);
    }
    out.push_str("],\n    \"rejects\": [");
    if used_reject {
        out.push_str(&jstr("reject"));
    }
    out.push_str("]\n  },\n  \"claims\": {\n    \"wf\": true,\n    \"absorb\": [\n");
    let mut first = true;
    for c in tr.class_mass.keys() {
        let (lo, hi) = tr.absorb_bracket(*c);
        if !first {
            out.push_str(",\n");
        }
        first = false;
        let _ = write!(out, "      [{}, {}, {}]", c, jstr(&rq(&lo)), jstr(&rq(&hi)));
    }
    out.push_str("\n    ],\n    \"posterior\": [\n");
    let mut first = true;
    for c in tr.class_mass.keys() {
        if let Some((lo, hi)) = tr.posterior_bracket(*c) {
            if !first {
                out.push_str(",\n");
            }
            first = false;
            let _ = write!(out, "      [{}, {}, {}]", c, jstr(&rq(&lo)), jstr(&rq(&hi)));
        }
    }
    out.push_str("\n    ]\n  }\n}\n");
    out
}

/// Window spec for the 3-D walk's container lane: stored rows for
/// every alive sorted triple with max coordinate <= `rstore`; the
/// max = rstore+1 shell is listed rowless (the container evaluation's
/// rowless-is-absorbing convention parks boundary mass in the
/// residual — the truncation widens, never lies). Claims are the
/// window-materialized chain's canonical absorb/posterior brackets,
/// which the container checker recomputes under the same convention.
/// State names carry the triple legibly (`s<a>_<b>_<c>`) — the EV
/// certificate's pattern checker decodes them.
pub fn emit_spec_3d_window(horizon: u32, rstore: u64) -> String {
    use crate::ev3d::{pattern_row, t3_unpack};
    use crate::q;
    let name3 = |t: (u64, u64, u64)| format!("s{}_{}_{}", t.0, t.1, t.2);

    let mut states: Vec<String> = Vec::new();
    let mut rows: Vec<(String, String, Q)> = Vec::new();
    let mut boundary: std::collections::BTreeSet<(u64, u64, u64)> = Default::default();
    for a in 0..=rstore {
        for b in a..=rstore {
            for c in b..=rstore {
                if (a, b, c) == (0, 0, 0) {
                    continue;
                }
                let src = name3((a, b, c));
                states.push(src.clone());
                let (alive, absorbed) = pattern_row((a, b, c), 1, 12);
                for (d, w) in alive {
                    let t = t3_unpack(d);
                    if t.2 > rstore {
                        boundary.insert(t);
                    }
                    rows.push((src.clone(), name3(t), q(w as i64, 12)));
                }
                if absorbed > 0 {
                    rows.push((src.clone(), "done_0".into(), q(absorbed as i64, 12)));
                }
            }
        }
    }
    states.extend(boundary.into_iter().map(name3));
    states.push("done_0".into());
    states.sort();
    states.dedup();

    let tr = evaluate(&crate::AsymRw3dWindow { rstore }, horizon);
    let mut class_states = BTreeMap::new();
    class_states.insert(0u32, "done_0".to_string());
    render_spec(
        &states,
        &name3((1, 1, 1)),
        rows,
        &class_states,
        false,
        horizon,
        &tr,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DieParadox, GeoCounter};

    #[test]
    fn die_spec_shape() {
        let s = emit_spec(&DieParadox, 6);
        assert!(s.contains("\"done_1\""));
        assert!(s.contains("\"reject\""));
        // one alive state per layer, rowless frontier at t=6
        assert!(s.contains("\"a6_0\""));
        // spot value: absorb lower for class 1 is 1/6
        assert!(s.contains("[1, \"1/6\","));
    }

    #[test]
    fn geo_spec_no_reject() {
        let s = emit_spec(&GeoCounter, 5);
        // the "rejects" KEY is always present; the reject STATE must not be
        assert!(!s.contains("\"reject\""));
        assert!(s.contains("\"done_0\""));
    }

    #[test]
    fn beauquier3_spec_is_homogeneous() {
        use crate::Beauquier3;
        let s51 = emit_spec(&Beauquier3, 51);
        let s1000 = emit_spec(&Beauquier3, 1000);
        // states materialized once: anchor s64, no timestamped names
        assert!(s51.contains("\"s64\""));
        assert!(!s51.contains("\"a1_"));
        assert!(s51.contains("\"done_0\"") && s51.contains("\"done_1\""));
        assert!(!s51.contains("\"reject\""));
        // the stored chain is horizon-independent; only the query's
        // horizon and the claims move
        let chain_of = |s: &str| s.split("\"query\"").next().unwrap().to_string();
        assert_eq!(chain_of(&s51), chain_of(&s1000));
        // start row coalesced: class-0 mass 1/2 as a single stored row
        assert!(s51.contains("[\"s64\", \"done_0\", \"1/2\"]"));
    }
}
