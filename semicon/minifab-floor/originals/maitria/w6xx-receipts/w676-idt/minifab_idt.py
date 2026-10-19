#!/usr/bin/env python3
"""W676 IDT-optimality instrument suite (probe-class; z3 does the grinding).

Three sub-instruments over the carved-clock Mini-Fab (semantics S1-S6 of
hearth/maitria/prereg/W676-idt-optimality-prereg.md):

  search N T        decide: is a schedule with arrivals-makespan <= T feasible?
                    (z3 LIA; exact encoding, documented below)
  family N          construct the pi* schedule for N lots (closed form), run it
                    through the raw-semantics validator, print its makespan
  mutants           run the prereg mutant battery M1-M3, M5, M6

Encoding = the schedule view of the round semantics, exact by construction:
  per lot l, step k in 1..5 (step 6 WLOG-deleted for the arrivals objective:
  deleting step-6 firings from any feasible run stays feasible and leaves
  arrival rounds unchanged):
    rel_l < s_{l,1};  f_{l,k} >= s_{l,k} + T_k + 1;  s_{l,k+1} >= f_{l,k} + 1
      (S2 dwell: a token produced at round r is consumable from r+1)
  schema meters (S1, one clock per row => <=1 firing/round/schema):
    rel_* distinct; s_{*,k} distinct; f_{*,k} distinct;
    tick_k: each lot needs T_k tick-rounds inside its open window
    (s_{l,k}, f_{l,k}), all lots' tick-rounds pairwise distinct. Feasibility
    of such an assignment == Hall on interval windows == for every critical
    interval [a,b] (a = some s_i+1, b = some f_j-1):
        sum_l max(0, T_k - |W_l \\ [a,b]|)  <=  b - a + 1.
    (Unit-capacity preemptive scheduling with release/deadline windows; the
    critical-interval reduction is the standard EDF feasibility theorem.)
  machines: service intervals [s,f] at a station overlap <= CAPACITY
    pointwise; by Helly for 1-D intervals + perfection of interval graphs
    this is EXACTLY "no (CAP+1)-subset pairwise overlaps", and a witness
    respecting the cap is assignable to CAP concrete machines.
  WLOG symmetry reductions (proofs in FINDINGS): rel_l = l; f_{*,5} sorted.

Validator: independent of the encoding — takes explicit per-lot rounds +
tick-round sets, replays the RAW rules round by round (schema counts,
machine token counts, chain dwell, w-guards), refuses loudly otherwise.
"""

import sys
import itertools
from z3 import (Int, Solver, If, And, Or, Not, Distinct, Sum, sat, unsat)

T_PROC = [225, 30, 55, 50, 255, 10]
STATION = [0, 1, 2, 1, 0, 2]
CAP = [2, 2, 1]
STEPS = (1, 2, 3, 4, 5)  # step 6 excluded (arrivals objective)


def zmax(a, b):
    return If(a >= b, a, b)


def zmin(a, b):
    return If(a <= b, a, b)


def build(N, Tq, *, machines=True, hall5=True, t5=255):
    """z3 solver for: exists schedule, all lots arrive in q6 by round Tq."""
    Tk = {1: 225, 2: 30, 3: 55, 4: 50, 5: t5}
    s = {(l, k): Int(f"s_{l}_{k}") for l in range(N) for k in STEPS}
    f = {(l, k): Int(f"f_{l}_{k}") for l in range(N) for k in STEPS}
    cs = []
    for l in range(N):
        # rel_l = l+1 (WLOG), q1 token usable from rel+1
        cs.append(s[(l, 1)] >= l + 2)
        for k in STEPS:
            cs.append(f[(l, k)] >= s[(l, k)] + Tk[k] + 1)
            if k < 5:
                cs.append(s[(l, k + 1)] >= f[(l, k)] + 1)
        cs.append(f[(l, 5)] <= Tq)
        cs.append(s[(l, 1)] >= 1)
    # WLOG: final arrivals sorted; other schema meters as Distinct
    for l in range(N - 1):
        cs.append(f[(l, 5)] < f[(l + 1, 5)])
    for k in STEPS:
        if N > 1:
            cs.append(Distinct(*[s[(l, k)] for l in range(N)]))
            if k < 5:
                cs.append(Distinct(*[f[(l, k)] for l in range(N)]))
    # tick meters: Hall on interval windows
    for k in STEPS:
        if k == 5 and not hall5:
            continue  # M3 mutant
        for i in range(N):
            for j in range(N):
                a = s[(i, k)] + 1
                b = f[(j, k)] - 1
                demand = []
                for l in range(N):
                    wl = s[(l, k)] + 1
                    wr = f[(l, k)] - 1
                    left = zmax(0, zmin(wr, a - 1) - wl + 1)
                    right = zmax(0, wr - zmax(wl, b + 1) + 1)
                    demand.append(zmax(0, Tk[k] - (left + right)))
                cs.append(Or(a > b, Sum(demand) <= b - a + 1))
    # machine capacities
    if machines:
        def ov(x, y):
            (sx, fx), (sy, fy) = x, y
            return And(sx <= fy, sy <= fx)
        for st in range(3):
            ivs = [(s[(l, k)], f[(l, k)])
                   for l in range(N) for k in STEPS if STATION[k - 1] == st]
            if CAP[st] == 1:
                for x, y in itertools.combinations(range(len(ivs)), 2):
                    cs.append(Not(ov(ivs[x], ivs[y])))
            else:
                for x, y, z in itertools.combinations(range(len(ivs)), 3):
                    cs.append(Not(And(ov(ivs[x], ivs[y]),
                                      ov(ivs[x], ivs[z]),
                                      ov(ivs[y], ivs[z]))))
    sol = Solver()
    sol.add(cs)
    return sol, s, f


def extract(model, s, f, N, t5=255):
    """Model -> explicit schedule dict; ticks via EDF within windows."""
    Tk = {1: 225, 2: 30, 3: 55, 4: 50, 5: t5}
    sched = {"rel": {l: l + 1 for l in range(N)}, "s": {}, "f": {},
             "ticks": {}}
    for l in range(N):
        for k in STEPS:
            sched["s"][(l, k)] = model.eval(s[(l, k)]).as_long()
            sched["f"][(l, k)] = model.eval(f[(l, k)]).as_long()
    for k in STEPS:
        # EDF: walk rounds; give the round to the pending lot w/ earliest f
        need = {l: Tk[k] for l in range(N)}
        wins = {l: (sched["s"][(l, k)] + 1, sched["f"][(l, k)] - 1)
                for l in range(N)}
        ticks = {l: [] for l in range(N)}
        lo = min(w[0] for w in wins.values())
        hi = max(w[1] for w in wins.values())
        for r in range(lo, hi + 1):
            live = [l for l in range(N)
                    if need[l] > 0 and wins[l][0] <= r <= wins[l][1]]
            if not live:
                continue
            pick = min(live, key=lambda l: wins[l][1])
            ticks[pick].append(r)
            need[pick] -= 1
        if any(v > 0 for v in need.values()):
            raise SystemExit(
                f"EDF failed at step {k}: encoding bug (Hall said feasible)")
        for l in range(N):
            sched["ticks"][(l, k)] = ticks[l]
    return sched


def validate(sched, N, t5=255):
    """Raw-semantics replay. Returns makespan; raises on any violation."""
    Tk = {1: 225, 2: 30, 3: 55, 4: 50, 5: t5}
    # chain + window checks
    for l in range(N):
        rel = sched["rel"][l]
        assert rel >= 1, f"lot {l}: rel<1"
        prev_done = rel  # token usable from prev_done+1
        for k in STEPS:
            sk, fk = sched["s"][(l, k)], sched["f"][(l, k)]
            assert sk >= prev_done + 1, f"lot {l} step {k}: start before token"
            tks = sched["ticks"][(l, k)]
            assert len(tks) == Tk[k], f"lot {l} step {k}: tick count"
            assert all(sk + 1 <= r <= fk - 1 for r in tks), \
                f"lot {l} step {k}: tick outside window"
            assert len(set(tks)) == len(tks), f"lot {l} step {k}: dup tick"
            assert fk >= max(tks) + 1, f"lot {l} step {k}: finish before w=0"
            prev_done = fk
    # schema meters: <=1 firing per round per schema
    for name, rounds in [("release", [sched["rel"][l] for l in range(N)])] + \
            [(f"start{k}", [sched["s"][(l, k)] for l in range(N)])
             for k in STEPS] + \
            [(f"finish{k}", [sched["f"][(l, k)] for l in range(N)])
             for k in STEPS] + \
            [(f"tick{k}",
              [r for l in range(N) for r in sched["ticks"][(l, k)]])
             for k in STEPS]:
        assert len(set(rounds)) == len(rounds), f"schema meter violated: {name}"
    # machine capacities, pointwise
    events = {0: [], 1: [], 2: []}
    for l in range(N):
        for k in STEPS:
            st = STATION[k - 1]
            events[st].append((sched["s"][(l, k)], +1))
            events[st].append((sched["f"][(l, k)] + 1, -1))
    for st in range(3):
        busy = 0
        for r, d in sorted(events[st]):
            busy += d
            assert busy <= CAP[st], f"station {st} over capacity at round {r}"
    return max(sched["f"][(l, 5)] for l in range(N))


def family(N):
    """The pi* schedule family, closed form (E4). Returns sched dict.

    Slots (step-5 services) 1..1+min(N-2,2) consecutively on machine B with
    +2 handovers; thereafter alternating pre-positioned (0-gap). Step-1
    services 1..min(N,4) back-to-back on A; service 5 on A, 6 on B, then
    alternating into the inter-slot windows. Constants derived in FINDINGS.
    """
    Tk = {1: 225, 2: 30, 3: 55, 4: 50, 5: 255}
    sched = {"rel": {l: l + 1 for l in range(N)}, "s": {}, "f": {},
             "ticks": {}}

    def chain_from(l, start1):
        """Steps 1..4 back-to-back from start1; returns q5-ready round f4."""
        sched["s"][(l, 1)] = start1
        sched["ticks"][(l, 1)] = list(range(start1 + 1, start1 + 226))
        sched["f"][(l, 1)] = start1 + 226
        t = start1 + 226
        for k in (2, 3, 4):
            sched["s"][(l, k)] = t + 1
            sched["ticks"][(l, k)] = list(range(t + 2, t + 2 + Tk[k]))
            sched["f"][(l, k)] = t + 2 + Tk[k]
            t = t + 2 + Tk[k]
        return t

    # step-1 service start rounds (metered: 225 ticks + handover apart)
    s1 = {}
    for j in range(min(N, 4)):
        s1[j] = 2 + 227 * j            # A back-to-back: 2, 229, 456, 683
    if N >= 5:
        s1[4] = 910                    # A, right after service 4
    if N >= 6:
        s1[5] = 1141                   # B, after slot-3's finish frees B
    for j in range(6, N):
        s1[j] = 1141 + 255 * (j - 5)   # alternating windows, one per cycle
    q5ready = {l: chain_from(l, s1[l]) for l in range(N)}

    # slot tick blocks
    def slot_block(j):  # j = 1-based slot index
        if j == 1:
            return 371
        if N <= 2:
            return 626          # slot 2 pre-positioned, 0-gap
        if j == 2:
            return 628          # B->B handover, +2
        if N == 3:
            return 883          # slot 3 pre-positioned on A, 0-gap
        if j == 3:
            return 885          # second B->B handover, +2
        return 885 + 255 * (j - 3)  # alternating, 0-gap

    for l in range(N):
        j = l + 1
        first = slot_block(j)
        # in-service: late-hold at first-1 (token usable at `first`, machine
        # freed by the prior same-machine occupant no later than first-1)
        sv = max(q5ready[l] + 1, first - 1)
        assert sv <= first - 1, f"lot {l}: q5 not ready for slot {j}"
        sched["s"][(l, 5)] = sv
        sched["ticks"][(l, 5)] = list(range(first, first + 255))
        sched["f"][(l, 5)] = first + 255
    return sched


def wcfamily(N):
    """Work-conserving optimum schedule, N >= 3: 255N + 455 (E7).

    The forced-window class (sigma_pull, sigma_tick — no holds): machines are
    grabbed for step-1 at rounds 2-3, the third release re-grabs the first
    freed machine at 229, so the first tick5 is provably >= 455; this
    constructor attains 455 + gapless. Every start below sits at a forced
    moment (machine-free AND queue-nonempty the round before), so the
    schedule is realizable by a (sigma_pull, sigma_tick) pair with no holds
    — checked structurally by wc_forced_check below.
    """
    assert N >= 3
    sched = {"rel": {l: l + 1 for l in range(N)}, "s": {}, "f": {},
             "ticks": {}}

    def steps234(l, f1):
        sched["f"][(l, 1)] = f1
        t = f1
        for k in (2, 3, 4):
            Tk = {2: 30, 3: 55, 4: 50}[k]
            sched["s"][(l, k)] = t + 1
            sched["ticks"][(l, k)] = list(range(t + 2, t + 2 + Tk))
            sched["f"][(l, k)] = t + 2 + Tk
            t = t + 2 + Tk
        return t  # q5-ready round (f4)

    # step-1: l1 fast-tracked; l2 in service early, ticks after l1's;
    # l3 grabbed at 229 (forced), ticks after l2's; l_j (j>=4) starts when
    # finish5(l_{j-3}) frees a machine (forced: q1 nonempty, q5 empty then).
    s1 = {0: 2, 1: 3, 2: 229}
    tick1 = {0: list(range(3, 228)),
             1: list(range(228, 453)),
             2: list(range(453, 678))}
    f1 = {0: 228, 1: 453, 2: 678}
    for j in range(3, N):
        s1[j] = 255 * (j + 1) - 309          # = finish5(l_{j-3}) + 1
        tick1[j] = list(range(s1[j] + 1, s1[j] + 226))
        f1[j] = s1[j] + 226
    f4 = {}
    for l in range(N):
        sched["s"][(l, 1)] = s1[l]
        sched["ticks"][(l, 1)] = tick1[l]
        f4[l] = steps234(l, f1[l])
    # step-5: slot j ticks [455+255(j-1), 454+255j]; in-service at the
    # forced moment: s5_j = f1_{j+1}+1 (the machine freed by the next lot's
    # step-1 finish), last lot at its own q5 arrival + 1.
    for l in range(N):
        first = 455 + 255 * l
        if l < N - 1:
            sv = f1[l + 1] + 1
        else:
            sv = f4[l] + 1
        sv = max(sv, f4[l] + 1)
        assert sv <= first - 1, f"wc lot {l}: in-service too late"
        sched["s"][(l, 5)] = sv
        sched["ticks"][(l, 5)] = list(range(first, first + 255))
        sched["f"][(l, 5)] = first + 255
    return sched


def wc_forced_check(sched, N):
    """Structural no-holds check: every start_k fires at the first round it
    could (machine token + queue token both available and, for k=1/5, no
    earlier free-machine-with-nonempty-queue moment passed it by). We check
    the necessary condition that is cheap and damning: at every round r
    where a station has a free machine AND a nonempty feeding queue, some
    start at that station fires at r+1 or the free machine was consumed at
    r+1 by the other queue's start."""
    horizon = max(sched["f"][(l, 5)] for l in range(N)) + 2
    # per-round machine-free counts and queue contents
    for st, ks in [(0, (1, 5)), (1, (2, 4)), (2, (3,))]:
        busy_delta = {}
        for l in range(N):
            for k in ks:
                busy_delta[sched["s"][(l, k)]] = \
                    busy_delta.get(sched["s"][(l, k)], 0) + 1
                busy_delta[sched["f"][(l, k)] + 1] = \
                    busy_delta.get(sched["f"][(l, k)] + 1, 0) - 1
        # queue k: entered at (rel if k==1 else f_{k-1}) + 1 usable; left at s_k
        qin = {k: sorted((sched["rel"][l] if k == 1 else
                          sched["f"][(l, k - 1)]) for l in range(N))
               for k in ks}
        qout = {k: sorted(sched["s"][(l, k)] for l in range(N)) for k in ks}
        starts = {k: set(sched["s"][(l, k)] for l in range(N)) for k in ks}
        busy = 0
        idle_viol = []
        for r in range(1, horizon):
            busy += busy_delta.get(r, 0)
            free = CAP[st] - busy
            for k in ks:
                waiting = sum(1 for x in qin[k] if x < r) - \
                          sum(1 for x in qout[k] if x <= r)
                if free > 0 and waiting > 0:
                    # a start of SOME step at this station must fire at r+1
                    if not any((r + 1) in starts[kk] for kk in ks):
                        idle_viol.append((st, k, r))
        if idle_viol:
            raise SystemExit(f"WC check failed (deliberate idle): "
                             f"{idle_viol[:5]}")
    return True


def search(N, Tq, **kw):
    sol, s, f = build(N, Tq, **kw)
    r = sol.check()
    return r, (sol, s, f)


def main():
    cmd = sys.argv[1]
    if cmd == "search":
        N, Tq = int(sys.argv[2]), int(sys.argv[3])
        r, (sol, s, f) = search(N, Tq)
        print(f"N={N} T<={Tq}: {r}")
        if r == sat:
            sched = extract(sol.model(), s, f, N)
            mk = validate(sched, N)
            print(f"  witness validated, makespan={mk}")
    elif cmd == "family":
        N = int(sys.argv[2])
        sched = family(N)
        mk = validate(sched, N)
        d = mk - (255 * N + 371)
        print(f"family N={N}: makespan={mk} = 255N+371+{d}")
    elif cmd == "familytable":
        for N in [int(x) for x in sys.argv[2:]]:
            sched = family(N)
            mk = validate(sched, N)
            d = mk - (255 * N + 371)
            print(f"family N={N}: makespan={mk} = 255N+371+{d}")
    elif cmd == "wcfamilytable":
        for N in [int(x) for x in sys.argv[2:]]:
            sched = wcfamily(N)
            mk = validate(sched, N)
            wc_forced_check(sched, N)
            d = mk - (255 * N + 371)
            print(f"wcfamily N={N}: makespan={mk} = 255N+371+{d} "
                  f"(no-holds check passed)")
    elif cmd == "mutants":
        run_mutants()
    else:
        raise SystemExit(f"unknown subcommand {cmd}")


def run_mutants():
    # M1 handled by the receipts runner (SAT/UNSAT pairs per N).
    # M2: t5=254 => optima 254N+371+D(N) at N in {1,2,3}
    for N, D in [(1, 0), (2, 0), (3, 2)]:
        want = 254 * N + 371 + D
        r1, _ = search(N, want, t5=254)
        r0, _ = search(N, want - 1, t5=254)
        ok = (r1 == sat and r0 == unsat)
        print(f"M2 t5=254 N={N}: opt={want}? sat@{want}={r1} "
              f"unsat@{want-1}={r0} -> {'PASS' if ok else 'FAIL'}")
    # M3: drop step-5 Hall at N=2 => optimum strictly below 881
    r, _ = search(2, 880, hall5=False)
    print(f"M3 no-tick5-meter N=2 sat@880={r} -> "
          f"{'PASS' if r == sat else 'FAIL'} (meter law is load-bearing)")
    # M6: machines OFF, meters+chains only: still UNSAT at 255N+370
    for N in (1, 2):
        r, _ = search(N, 255 * N + 370, machines=False)
        print(f"M6 no-machines N={N} unsat@{255*N+370}={r} -> "
              f"{'PASS' if r == unsat else 'FAIL'} (B0 rides meters+chains)")
    # M5: family vs z3 witness agreement is asserted by the receipts runner.


if __name__ == "__main__":
    main()
