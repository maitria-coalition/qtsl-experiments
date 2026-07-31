//! Deterministic-core DES for the Intel Five-Machine Six-Step Mini-Fab
//! (Kempf 1994). Spec provenance and every number: ../KEMPF-SPEC.md.
//!
//! SCOPE (deliberately small — this is a comprehension check of the
//! declarative System rendering in the workshop's declarative System rendering, not a production
//! simulator; see ../PATTERN-GAP-REPORT.md "reference DES" section):
//! machines + route + diffusion batching + litho sequence-dependent
//! setups + test-machine dedication + FIFO dispatch, with load/unload
//! occupying the machine. EXCLUDED, by design: operators/technician
//! (gap G1), transporter (G1), event fusion granularity (G2), implant
//! failures (G4) — simulating those would exercise vocabulary the
//! System patterns don't carry yet, which is the gap report's point.
//! Buffers are unbounded here (the TU/e thesis's own delta-max
//! measurement also neglected buffer caps).
//!
//! Time: integer ticks of 1/17 minute, so the type-A release interval
//! 10080/51 = 3360/17 min is EXACT (finding F1 in the gap report).
//!
//! Determinism: no RNG anywhere in this core. Event tie-break is
//! (time, insertion sequence) — an explicit W3 serialization (finding
//! F4): simultaneous events fire in the order they were scheduled.
//!
//! Single-threaded, with the reason the house norm asks for: a DES
//! event loop is a serial dependency chain (every event mutates the
//! shared fab state that enables the next), and this instance is tiny
//! (~1e5 events per simulated year, well under a core-minute).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

pub const TICKS_PER_MIN: u64 = 17;
pub const MIN_PER_WEEK: u64 = 10080;
pub const TICKS_PER_WEEK: u64 = MIN_PER_WEEK * TICKS_PER_MIN;

pub type Tick = u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PType {
    Test,
    A,
    B,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum St {
    Diff = 0,
    Impl = 1,
    Lith = 2,
}

impl St {
    pub const ALL: [St; 3] = [St::Diff, St::Impl, St::Lith];
    pub fn machines(self) -> usize {
        match self {
            St::Diff => 2,
            St::Impl => 2,
            St::Lith => 1,
        }
    }
    pub fn batch(self) -> usize {
        match self {
            St::Diff => 3,
            _ => 1,
        }
    }
    /// (load, unload) minutes — at diffusion these bind per BATCH
    /// (decision point 1, KEMPF-SPEC.md).
    pub fn handling(self) -> (u64, u64) {
        match self {
            St::Diff => (20, 40),
            St::Impl => (15, 15),
            St::Lith => (10, 10),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            St::Diff => "diffusion",
            St::Impl => "implantation",
            St::Lith => "lithography",
        }
    }
}

/// Route: step index 0..6 -> (station, recipe-is-high, process minutes).
pub const ROUTE: [(St, bool, u64); 6] = [
    (St::Diff, false, 225),
    (St::Impl, false, 30),
    (St::Lith, false, 55),
    (St::Impl, true, 50),
    (St::Diff, true, 255),
    (St::Lith, true, 10),
];

/// Litho setup minutes for (type_change, step_change).
pub fn setup_min(type_change: bool, step_change: bool) -> u64 {
    match (type_change, step_change) {
        (true, true) => 12,
        (false, true) => 10,
        (true, false) => 5,
        (false, false) => 0,
    }
}

#[derive(Clone, Debug)]
pub struct Lot {
    pub ptype: PType,
    pub step: usize, // 0..6 = next route position; 6 = done
    pub release: Tick,
    /// For test lots at two-machine stations: machine used on first visit.
    pub first_machine: [Option<usize>; 3],
}

#[derive(Clone, Copy, Debug)]
struct MachineKpi {
    setup: Tick,
    handling: Tick,
    processing: Tick,
}

#[derive(Clone, Debug)]
struct Machine {
    busy: bool,
    /// litho memory: (ptype, recipe_high) of last job
    last: Option<(PType, bool)>,
    kpi: MachineKpi,
}

enum Ev {
    Arrive(PType),
    Complete { st: St, m: usize, lots: Vec<usize> },
}

pub struct Config {
    /// lots per week per type: (Test, A, B). Kempf: (3, 51, 30).
    pub per_week: (u64, u64, u64),
    pub horizon: Tick,
}

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub released: u64,
    pub done: u64,
    pub done_by_type: [u64; 3],
    pub flow_sum_ticks: u128,
    pub flow_sum_by_type: [u128; 3],
    pub wip_integral: u128, // lot-ticks
    pub horizon: Tick,
    /// per (station, machine): (setup, handling, processing) ticks
    pub machine_kpis: Vec<(St, usize, Tick, Tick, Tick)>,
    pub end_wip: u64,
    pub max_buf: [usize; 3],
}

struct Fab {
    lots: Vec<Lot>,
    bufs: [VecDeque<usize>; 3],
    machines: [Vec<Machine>; 3],
    heap: BinaryHeap<Reverse<(Tick, u64, usize)>>, // (time, seq, ev-slot)
    evs: Vec<Option<Ev>>,
    seq: u64,
    now: Tick,
    horizon: Tick,
    wip: u64,
    last_wip_t: Tick,
    rep: Report,
}

impl Fab {
    fn new() -> Self {
        Fab {
            lots: Vec::new(),
            bufs: [VecDeque::new(), VecDeque::new(), VecDeque::new()],
            machines: std::array::from_fn(|s| {
                let st = St::ALL[s];
                (0..st.machines())
                    .map(|_| Machine {
                        busy: false,
                        last: None,
                        kpi: MachineKpi {
                            setup: 0,
                            handling: 0,
                            processing: 0,
                        },
                    })
                    .collect()
            }),
            heap: BinaryHeap::new(),
            evs: Vec::new(),
            seq: 0,
            now: 0,
            horizon: 0,
            wip: 0,
            last_wip_t: 0,
            rep: Report::default(),
        }
    }

    fn schedule(&mut self, t: Tick, ev: Ev) {
        debug_assert!(t >= self.now, "event-time monotonicity");
        let slot = self.evs.len();
        self.evs.push(Some(ev));
        self.heap.push(Reverse((t, self.seq, slot)));
        self.seq += 1;
    }

    fn advance_wip_clock(&mut self, t: Tick) {
        self.rep.wip_integral += (t - self.last_wip_t) as u128 * self.wip as u128;
        self.last_wip_t = t;
    }

    /// FIFO-with-feasibility dispatch at station `st`: repeatedly start
    /// service while an idle machine and a feasible unit exist.
    fn dispatch(&mut self, st: St) {
        loop {
            let s = st as usize;
            if !self.machines[s].iter().any(|m| !m.busy) {
                return;
            }
            let Some((members, machine)) = self.pick_unit(st) else {
                return;
            };
            // remove members from buffer (by position, descending)
            let mut pos: Vec<usize> = members
                .iter()
                .map(|&lid| self.bufs[s].iter().position(|&x| x == lid).unwrap())
                .collect();
            pos.sort_unstable_by(|a, b| b.cmp(a));
            for p in pos {
                self.bufs[s].remove(p);
            }
            // durations
            let step = self.lots[members[0]].step;
            let (st2, high, proc) = ROUTE[step];
            debug_assert_eq!(st2, st);
            let (load, unload) = st.handling();
            let mut setup = 0;
            if st == St::Lith {
                let lot = &self.lots[members[0]];
                if let Some((lt, lh)) = self.machines[s][machine].last {
                    setup = setup_min(lt != lot.ptype, lh != high);
                }
                self.machines[s][machine].last = Some((lot.ptype, high));
            }
            // record test-lot first-visit machine (two-machine stations)
            if st != St::Lith {
                for &lid in &members {
                    let lot = &mut self.lots[lid];
                    if lot.ptype == PType::Test && lot.first_machine[s].is_none() {
                        lot.first_machine[s] = Some(machine);
                    }
                }
            }
            // KPI booking, clamped at the horizon: the service phases
            // run sequentially (setup, load, process, unload); a job
            // straddling the horizon books only its in-horizon part, so
            // occupancy never exceeds wall clock.
            let mut cursor = self.now;
            let mut book = |mins: u64, horizon: Tick| -> Tick {
                let start = cursor.min(horizon);
                cursor += mins * TICKS_PER_MIN;
                cursor.min(horizon) - start
            };
            let horizon = self.horizon;
            let b_setup = book(setup, horizon);
            let b_load = book(load, horizon);
            let b_proc = book(proc, horizon);
            let b_unload = book(unload, horizon);
            let m = &mut self.machines[s][machine];
            m.busy = true;
            m.kpi.setup += b_setup;
            m.kpi.handling += b_load + b_unload;
            m.kpi.processing += b_proc;
            let dur = (setup + load + proc + unload) * TICKS_PER_MIN;
            let t = self.now + dur;
            self.schedule(
                t,
                Ev::Complete {
                    st,
                    m: machine,
                    lots: members,
                },
            );
        }
    }

    /// Choose the next service unit at `st` under FIFO-with-feasibility,
    /// and an idle machine compatible with its dedication constraints.
    /// Returns (member lot ids, machine index).
    fn pick_unit(&self, st: St) -> Option<(Vec<usize>, usize)> {
        let s = st as usize;
        let idle: Vec<usize> = (0..self.machines[s].len())
            .filter(|&i| !self.machines[s][i].busy)
            .collect();
        if st.batch() == 1 {
            // oldest lot whose dedication admits an idle machine
            for &lid in &self.bufs[s] {
                if let Some(&m) = idle.iter().find(|&&m| self.machine_ok(lid, s, m)) {
                    return Some((vec![lid], m));
                }
            }
            return None;
        }
        // diffusion: batch of 3, same step, <=1 test, high batches
        // commercial-pure. FIFO reading: consider steps in order of
        // their OLDEST waiting lot; within a step take the earliest
        // feasible triple (skip surplus test lots); the batch's
        // dedication constraints must admit a common idle machine.
        let mut steps_seen: Vec<usize> = Vec::new();
        for &lid in &self.bufs[s] {
            let step = self.lots[lid].step;
            if !steps_seen.contains(&step) {
                steps_seen.push(step);
            }
        }
        for step in steps_seen {
            let high = ROUTE[step].1;
            let group: Vec<usize> = self.bufs[s]
                .iter()
                .copied()
                .filter(|&lid| self.lots[lid].step == step)
                .collect();
            if group.len() < 3 {
                continue;
            }
            let candidates = self.batch_candidates(&group, high);
            for cand in candidates {
                if let Some(&m) = idle
                    .iter()
                    .find(|&&m| cand.iter().all(|&lid| self.machine_ok(lid, s, m)))
                {
                    return Some((cand, m));
                }
            }
        }
        None
    }

    /// Earliest-feasible triples from a same-step FIFO group, best first.
    /// Low step: any mix, at most one test. High step: additionally the
    /// commercial members must be type-pure (KEMPF-SPEC.md).
    fn batch_candidates(&self, group: &[usize], high: bool) -> Vec<Vec<usize>> {
        let mut out: Vec<Vec<usize>> = Vec::new();
        if !high {
            // earliest 3 with <=1 test: take in order, skipping tests
            // beyond the first
            let mut picked = Vec::new();
            let mut tests = 0;
            for &lid in group {
                let is_t = self.lots[lid].ptype == PType::Test;
                if is_t && tests >= 1 {
                    continue;
                }
                if is_t {
                    tests += 1;
                }
                picked.push(lid);
                if picked.len() == 3 {
                    break;
                }
            }
            if picked.len() == 3 {
                out.push(picked);
            }
            return out;
        }
        // high step: candidate compositions, ranked by the arrival
        // position of their latest member (earliest-completing-triple
        // first) — a documented greedy, not spec text.
        let firsts = |ty: PType, n: usize| -> Option<Vec<usize>> {
            let v: Vec<usize> = group
                .iter()
                .copied()
                .filter(|&l| self.lots[l].ptype == ty)
                .take(n)
                .collect();
            (v.len() == n).then_some(v)
        };
        let mut cands: Vec<Vec<usize>> = Vec::new();
        for ty in [PType::A, PType::B] {
            if let Some(c) = firsts(ty, 3) {
                cands.push(c);
            }
            if let (Some(mut c2), Some(t1)) = (firsts(ty, 2), firsts(PType::Test, 1)) {
                c2.extend(t1);
                cands.push(c2);
            }
        }
        let posn = |lid: usize| group.iter().position(|&x| x == lid).unwrap();
        cands.sort_by_key(|c| c.iter().map(|&l| posn(l)).max());
        out.extend(cands);
        out
    }

    /// Test-machine dedication: a test lot's second visit to a
    /// two-machine station must use the other machine.
    fn machine_ok(&self, lid: usize, s: usize, m: usize) -> bool {
        let lot = &self.lots[lid];
        if lot.ptype != PType::Test {
            return true;
        }
        match lot.first_machine[s] {
            Some(fm) => fm != m || St::ALL[s].machines() == 1,
            None => true,
        }
    }

    fn handle(&mut self, ev: Ev) {
        match ev {
            Ev::Arrive(pt) => {
                let lid = self.lots.len();
                self.lots.push(Lot {
                    ptype: pt,
                    step: 0,
                    release: self.now,
                    first_machine: [None; 3],
                });
                self.rep.released += 1;
                self.wip += 1;
                let st = ROUTE[0].0;
                self.bufs[st as usize].push_back(lid);
                self.rep.max_buf[st as usize] =
                    self.rep.max_buf[st as usize].max(self.bufs[st as usize].len());
                self.dispatch(st);
            }
            Ev::Complete { st, m, lots } => {
                self.machines[st as usize][m].busy = false;
                let mut next_sts: Vec<St> = Vec::new();
                for lid in lots {
                    let step = {
                        let lot = &mut self.lots[lid];
                        lot.step += 1;
                        lot.step
                    };
                    if step == 6 {
                        let lot = &self.lots[lid];
                        let flow = (self.now - lot.release) as u128;
                        self.rep.done += 1;
                        self.rep.flow_sum_ticks += flow;
                        let ti = match lot.ptype {
                            PType::Test => 0,
                            PType::A => 1,
                            PType::B => 2,
                        };
                        self.rep.done_by_type[ti] += 1;
                        self.rep.flow_sum_by_type[ti] += flow;
                        self.wip -= 1;
                    } else {
                        let nst = ROUTE[step].0;
                        self.bufs[nst as usize].push_back(lid);
                        self.rep.max_buf[nst as usize] =
                            self.rep.max_buf[nst as usize].max(self.bufs[nst as usize].len());
                        if !next_sts.contains(&nst) {
                            next_sts.push(nst);
                        }
                    }
                }
                for nst in next_sts {
                    self.dispatch(nst);
                }
                self.dispatch(st);
            }
        }
    }
}

pub fn run(cfg: &Config) -> Report {
    let mut fab = Fab::new();
    fab.horizon = cfg.horizon;
    // arrival timers: first release at t=0, constant intervals
    // (interval = week / per-week count, exact in 1/17-min ticks for
    // the Kempf mix; rounded otherwise).
    for (pt, n) in [
        (PType::Test, cfg.per_week.0),
        (PType::A, cfg.per_week.1),
        (PType::B, cfg.per_week.2),
    ] {
        if n > 0 {
            fab.schedule(0, Ev::Arrive(pt));
        }
    }
    let interval = |n: u64| -> Tick { TICKS_PER_WEEK / n };
    let mut last: Tick;
    while let Some(&Reverse((t, _, _))) = fab.heap.peek() {
        if t > cfg.horizon {
            break;
        }
        let Reverse((t, _, slot)) = fab.heap.pop().unwrap();
        last = fab.now;
        debug_assert!(t >= last, "event-time monotonicity");
        fab.advance_wip_clock(t);
        fab.now = t;
        let ev = fab.evs[slot].take().unwrap();
        // re-arm arrival timer before handling (deterministic order)
        if let Ev::Arrive(pt) = ev {
            let n = match pt {
                PType::Test => cfg.per_week.0,
                PType::A => cfg.per_week.1,
                PType::B => cfg.per_week.2,
            };
            fab.schedule(t + interval(n), Ev::Arrive(pt));
        }
        fab.handle(ev);
    }
    fab.advance_wip_clock(cfg.horizon);
    fab.rep.horizon = cfg.horizon;
    fab.rep.end_wip = fab.wip;
    for (s, st) in St::ALL.iter().enumerate() {
        for (i, m) in fab.machines[s].iter().enumerate() {
            fab.rep
                .machine_kpis
                .push((*st, i, m.kpi.setup, m.kpi.handling, m.kpi.processing));
        }
    }
    fab.rep
}

impl Report {
    pub fn throughput_per_week(&self) -> f64 {
        self.done as f64 * TICKS_PER_WEEK as f64 / self.horizon as f64
    }
    pub fn mean_flow_min(&self) -> f64 {
        if self.done == 0 {
            return f64::NAN;
        }
        self.flow_sum_ticks as f64 / self.done as f64 / TICKS_PER_MIN as f64
    }
    pub fn mean_wip(&self) -> f64 {
        self.wip_integral as f64 / self.horizon as f64
    }
}
