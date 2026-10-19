//! Mutants the decoded-container replay must refuse.
use minifab_glc_replay::{decode, replay::*};

const SUP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../minifab-dispatch/out/carved/minifab-supervised.glc");

fn sealed() -> decode::SealedNet {
    decode::open(&std::fs::read(SUP).unwrap()).unwrap().sealed_net(0).unwrap()
}

#[test]
fn clean_attains_21795() {
    let net = sealed();
    let rows = compile(&net).unwrap();
    let o = replay(&net, &rows, &table_of(&pi_star_lots(84)), 84, &["start1", "start5"], 30_000).unwrap();
    assert_eq!((o.last_arrival_round, o.pre_goal_firings), (21_795, 626 * 84));
}

#[test]
fn byte_flips_refuse_at_the_reader() {
    let bytes = std::fs::read(SUP).unwrap();
    for at in [9usize, bytes.len() / 2, bytes.len() - 1] {
        let mut b = bytes.clone();
        b[at] ^= 1;
        assert!(decode::open(&b).and_then(|c| c.sealed_net(0)).is_err(), "flip at {at} accepted");
    }
    assert!(decode::open(&bytes[..bytes.len() - 1]).is_err(), "truncation accepted");
}

#[test]
fn sealed_tick5_work_254_refuses() {
    let mut net = sealed();
    let t = net.trs.iter_mut().find(|t| t.sel == vec![(11, 1, 0)] && t.scale.is_some()).unwrap();
    t.scale.as_mut().unwrap().image[0].1 = (253, 1);
    let e = compile(&net).unwrap_err();
    assert!(e.contains("tick5"), "{e}");
}

#[test]
fn sealed_pending_count_mismatch_refuses() {
    let mut net = sealed();
    net.marking[P_QPEND] = 83;
    let rows = compile(&net).unwrap();
    assert!(replay(&net, &rows, &table_of(&pi_star_lots(84)), 84, &["start1", "start5"], 30_000).is_err());
}

#[test]
fn one_diffusion_machine_refuses() {
    let mut net = sealed();
    net.marking[p_mach(0)] = 1;
    let rows = compile(&net).unwrap();
    assert!(replay(&net, &rows, &table_of(&pi_star_lots(84)), 84, &["start1", "start5"], 30_000).is_err());
}

#[test]
fn shifted_tick_refuses() {
    let net = sealed();
    let rows = compile(&net).unwrap();
    let mut t = table_of(&pi_star_lots(84));
    let (&r, _) = t.iter().find(|(_, v)| v.iter().any(|f| f.role == Role::Tick(5))).unwrap();
    let v = t.get_mut(&r).unwrap();
    let i = v.iter().position(|f| f.role == Role::Tick(5)).unwrap();
    let f = v.remove(i);
    t.entry(r - 1).or_default().push(f);
    assert!(replay(&net, &rows, &t, 84, &["start1", "start5"], 30_000).is_err());
}

/// The same-round fresh-token tick is LAWFUL under the round semantics
/// (tick1 is due via the other lot's token; which token a firing takes is the scheduler's choice):
/// lot B starts step 1 at round 11 and takes tick1 in that same round.
/// The reached marking p1 = {217, 224} is unreachable if every lot's
/// ticks must lie strictly after its start (the MFSchedule `hwin` /
/// minifab_idt.py window reading) — so those models are not exact
/// projections of the net; see README.md, Findings 2.
#[test]
fn same_round_fresh_token_tick_is_lawful() {
    let mut net = sealed();
    net.marking[P_QPEND] = 2;
    let rows = compile(&net).unwrap();
    let mut t: std::collections::BTreeMap<u64, Vec<Firing>> = Default::default();
    let mut put = |r: u64, role: Role, w: Option<i64>| t.entry(r).or_default().push(Firing { role, pick_w: w });
    put(1, Role::Release, None);
    put(2, Role::Release, None);
    put(2, Role::Start(1), None);
    for j in 0..8 { put(3 + j, Role::Tick(1), Some(225 - j as i64)); }
    put(11, Role::Start(1), None);
    put(11, Role::Tick(1), Some(225)); // the fresh token, in its start round
    let m = replay_prefix(&net, &rows, &t, 2, &["start1", "release"], 11).unwrap();
    let mut p1 = m[p_proc(1)].clone();
    p1.sort();
    assert_eq!(p1, vec![217, 224]);
}

/// Second projection gap (review M1, astra + opus): SAME-ROUND MACHINE
/// HANDOVER of a diffusion machine. At the end of round 452 lot A holds
/// one diffusion machine with zero work, the other is free, and lots B
/// (at q5) and C (at q1) are queued, so finish1, start1 and start5 are
/// all due at 453. Finish A (phase 0) and then start C and B (phase 2):
/// one of them must take the machine A freed in that same round. The
/// three closed diffusion intervals [3,453], [453,679], [453,709]
/// pairwise intersect, so no Bool machine labelling meets MFSchedule's
/// strict same-machine separation (finish < start). Lawful in the net.
#[test]
fn same_round_machine_handover_is_lawful() {
    let mut net = sealed();
    net.marking[P_QPEND] = 3;
    let rows = compile(&net).unwrap();
    let b: Lot = (1, [(2, 228, 3), (229, 260, 230), (261, 317, 262), (318, 369, 319), (453, 709, 454)]);
    let a: Lot = (2, [(3, 453, 228), (454, 485, 455), (486, 542, 487), (543, 594, 544), (680, 964, 709)]);
    let c: Lot = (3, [(453, 679, 454), (680, 711, 681), (712, 768, 713), (769, 820, 770), (821, 1219, 964)]);
    let lots = vec![b, a, c];
    let o = replay(&net, &rows, &table_of(&lots), 3, &["start1", "start5"], 3_000).unwrap();
    assert_eq!(o.last_arrival_round, 1219);
    // the strict-separation (Lean hcap11/hcap15/hcap55) labelling does not exist
    let ivs = [(3u64, 453u64), (453, 679), (453, 709)];
    let ok = (0..8u32).any(|m| {
        (0..3).all(|i| (0..3).all(|j| {
            i == j || ((m >> i) & 1) != ((m >> j) & 1) || ivs[i].1 < ivs[j].0 || ivs[j].1 < ivs[i].0
        }))
    });
    assert!(!ok, "a strict-separation 2-machine labelling exists");
}

/// The instance adaptation used by the N = 1..12 receipt: substitutes only
/// the pending count and the unlock threshold, and refuses a container
/// whose sealed values are not the N = 84 ones.
#[test]
fn instance_adaptation_is_narrow() {
    let net = sealed();
    let (a, pend, unlock) = adapt_instance(&net, 10).unwrap();
    assert_eq!((pend, unlock), (84, Some(84)));
    let changed_marking: Vec<usize> = (0..net.marking.len()).filter(|&i| net.marking[i] != a.marking[i]).collect();
    assert_eq!(changed_marking, vec![P_QPEND]);
    let changed_trs = net.trs.iter().zip(&a.trs).filter(|(x, y)| x != y).count();
    assert_eq!(changed_trs, 1);
    let rows = compile(&a).unwrap();
    let o = replay(&a, &rows, &table_of(&pi_star_lots(10)), 10, &["start1", "start5"], 5_000).unwrap();
    assert_eq!(o.last_arrival_round, 2925);
    let mut bad = net.clone();
    bad.marking[P_QPEND] = 83;
    assert!(adapt_instance(&bad, 10).is_err());
    // supervised lane with its unlock row removed: must refuse, not report null
    let mut nounlock = net.clone();
    nounlock.trs.retain(|t| !(t.sel.len() == 2 && t.sel[0].0 == p_queue(6) && t.sel[1].0 == P_KEY));
    assert!(adapt_instance(&nounlock, 10).is_err());
}
