#!/usr/bin/env python3
"""Two-sided exhaustive makespan search for the carved-clock Mini-Fab
(discrete lane: mtk minifab.rs process times T, not the fluid lane's
occupancies), as an SMT problem over integer event rounds.

Model R (the relaxation; see README.md for the soundness argument):
lots i < N, steps k = 1..5 (step 6 is deleted: the arrivals goal is the
N-th finish5, and deleting start6/tick6/ship firings from any lawful run
keeps it lawful because those rows only take the litho token and give
it back). Per lot: release round rel_i; per step start s_ik, finish f_ik.
  * meter law: at most one firing per schema per round
      -> rel_i distinct; s_ik distinct over i; f_ik distinct over i;
         at most one tick_k per round (Hall condition below).
  * dueness (enabled at end of previous round), token flow:
      s_i1 >= rel_i + 1 ; s_ik >= f_i,k-1 + 1.
  * ticks: lot i's T_k ticks of step k lie in [s_ik + DELTA, f_ik - 1],
      one tick_k per round across lots.  DELTA = 0 is the relaxation
      (a token may be ticked in its start round); DELTA = 1 is the
      per-lot reading.  One-per-round unit work in windows is feasible
      iff Hall's condition holds on every [a_p, b_q] (convex bipartite).
  * machines (station sigma, capacity c): the step occupies one machine
      at the end of rounds s..f-1.  pickability: occupancy at end of the
      start phase of a start round <= c; dueness: occupancy at end of
      the previous round <= c - 1.
  * NO forcing: any due row may be left unfired (the hold axis is the
    whole roster). So R contains every run of the plated model under
    any supervision, and UNSAT(R, H) refutes every policy.
  * by default NO queue-order (FIFO) constraints: the only symmetry
    breaking is rel_0 < rel_1 < ..., a pure relabelling of lot indices
    (R is lot-permutation invariant). --fifo adds queue order, justified
    by the exchange lemma of README §R (tested by fifo_exchange_check.py).
usage: z3_floor.py N1,N2,... [DELTA] [--fifo]   (timings go to stderr only)
Goal: max_i f_i5 <= H (round of the N-th arrival into q6).
"""
import sys, json, hashlib, time
import z3

T = [225, 30, 55, 50, 255]
STATION = [0, 1, 2, 1, 0]
CAP = [2, 2, 1]
assert min(T) >= 2  # nonempty windows: Hall's guard below is otherwise vacuous

def t_opt(n):
    return 255 * n + 371 + (0 if n <= 2 else 2 if n == 3 else 4)


def build(n, horizon, delta, fifo=False):
    s = z3.Solver()
    rel = [z3.Int(f"rel_{i}") for i in range(n)]
    st = [[z3.Int(f"s_{i}_{k+1}") for k in range(5)] for i in range(n)]
    fi = [[z3.Int(f"f_{i}_{k+1}") for k in range(5)] for i in range(n)]
    for i in range(n):
        s.add(rel[i] >= 1)
        if i > 0:
            # symmetry breaking only: every other constraint of R is invariant
            # under permuting lot indices and the rel_i are distinct, so any
            # model can be relabelled to have increasing releases (README §R).
            s.add(rel[i] > rel[i - 1])
        s.add(st[i][0] >= rel[i] + 1)
        for k in range(5):
            if k > 0:
                s.add(st[i][k] >= fi[i][k - 1] + 1)
            s.add(fi[i][k] - st[i][k] >= T[k] + 1 - (1 if delta == 0 else 0))  # window length; Hall below is exact
        s.add(fi[i][4] <= horizon)
    if fifo:
        # queue order (start order follows arrival order): sound only by
        # the exchange lemma of README §R (R is closed under suffix
        # swaps), tested by fifo_exchange_check.py. Off by default.
        for k in range(1, 5):
            for i in range(n):
                for j in range(n):
                    if i != j:
                        s.add(z3.Implies(fi[i][k - 1] < fi[j][k - 1], st[i][k] < st[j][k]))
        for i in range(n - 1):
            s.add(st[i][0] < st[i + 1][0])
    for k in range(5):
        if n > 1:
            s.add(z3.Distinct([st[i][k] for i in range(n)]))
            s.add(z3.Distinct([fi[i][k] for i in range(n)]))
        if delta == 0:
            # a lot may take tick_k in its own start round only if tick_k is
            # DUE then. The net needs some other lot j in p_k with work >= 1 at
            # the end of the previous round; R asks only that j be in service
            # then (s_jk <= s_ik - 1 and s_ik <= f_jk - 1), which is weaker,
            # so R is larger (safe for the lower bound; README §R).
            early = [z3.Bool(f"early_{i}_{k+1}") for i in range(n)]
            for i in range(n):
                s.add(z3.Implies(early[i], z3.Or([z3.And(st[j][k] <= st[i][k] - 1, st[i][k] <= fi[j][k] - 1)
                                                   for j in range(n) if j != i] or [z3.BoolVal(False)])))
            a = [st[i][k] + z3.If(early[i], 0, 1) for i in range(n)]
        else:
            a = [st[i][k] + 1 for i in range(n)]
        b = [fi[i][k] - 1 for i in range(n)]
        for p in range(n):
            for q in range(n):
                inside = [z3.If(z3.And(a[l] >= a[p], b[l] <= b[q]), T[k], 0) for l in range(n)]
                s.add(z3.Implies(b[q] >= a[p], z3.Sum(inside) <= b[q] - a[p] + 1))
    for sig in range(3):
        ivs = [(st[i][k], fi[i][k]) for i in range(n) for k in range(5) if STATION[k] == sig]
        c = CAP[sig]
        for (si, _) in ivs:
            pick = z3.Sum([z3.If(z3.And(sj <= si, si < fj), 1, 0) for (sj, fj) in ivs])
            s.add(pick <= c)
            prev = z3.Sum([z3.If(z3.And(sj <= si - 1, si - 1 < fj), 1, 0) for (sj, fj) in ivs])
            s.add(prev <= c - 1)
    return s, rel, st, fi

def model_rows(m, n, rel, st, fi):
    ev = lambda x: m.eval(x).as_long()
    return [{"rel": ev(rel[i]), "steps": [[ev(st[i][k]), ev(fi[i][k])] for k in range(5)]} for i in range(n)]

def validate(rows, n, horizon, delta):
    """Independent re-validation of a witness against R, round by round
    (explicit tick assignment by EDF, explicit machine counters)."""
    # witness shape and integer domains: n lots, five steps, integral rounds,
    # every service starts before it finishes and lies within [1, horizon]
    assert isinstance(rows, list) and len(rows) == n, "lot count"
    for r in rows:
        assert isinstance(r.get("rel"), int) and len(r.get("steps", [])) == 5, "lot shape"
        for st_fi in r["steps"]:
            assert len(st_fi) == 2 and all(isinstance(x, int) for x in st_fi), "step shape"
            s_, f_ = st_fi
            assert 1 <= s_ < f_ <= horizon, f"service ({s_},{f_}) not inside [1,{horizon}] with start < finish"
    rels = [r["rel"] for r in rows]
    assert len(set(rels)) == n and min(rels) >= 1
    for k in range(5):
        assert len({r["steps"][k][0] for r in rows}) == n
        assert len({r["steps"][k][1] for r in rows}) == n
    for r in rows:
        prev = r["rel"]
        for k in range(5):
            s, f = r["steps"][k]
            assert s >= prev + 1
            prev = f
        assert r["steps"][4][1] <= horizon
    for k in range(5):  # EDF unit-tick assignment, one per round
        def a_of(i):
            s_i = rows[i]["steps"][k][0]
            if delta == 0 and any(j != i and rows[j]["steps"][k][0] <= s_i - 1 and s_i <= rows[j]["steps"][k][1] - 1
                                  for j in range(n)):
                return s_i
            return s_i + 1
        jobs = sorted(((a_of(i), rows[i]["steps"][k][1] - 1, T[k]) for i in range(n)))
        rem = {j: jobs[j][2] for j in range(n)}
        for t in range(1, horizon + 1):
            avail = [j for j in range(n) if jobs[j][0] <= t <= jobs[j][1] and rem[j] > 0]
            if avail:
                j = min(avail, key=lambda j: jobs[j][1])
                rem[j] -= 1
            for j in range(n):
                assert not (jobs[j][1] == t and rem[j] > 0), f"step {k+1}: tick deadline missed"
        assert all(rem[j] == 0 for j in range(n)), f"step {k+1}: work left after the horizon"
    for sig in range(3):
        ivs = [tuple(r["steps"][k]) for r in rows for k in range(5) if STATION[k] == sig]
        occ = lambda t: sum(1 for (s, f) in ivs if s <= t < f)
        for (s, f) in ivs:
            assert occ(s) <= CAP[sig] and occ(s - 1) <= CAP[sig] - 1, f"station {sig} at {s}"
    return True

def selftest():
    """negative tests: validate() must REJECT malformed or impossible witnesses."""
    good = [{"rel": 1, "steps": [[2, 228], [229, 260], [261, 317], [318, 369], [370, 626]]}]
    validate(good, 1, 626, 0)
    bad = {
        "backwards services (astra re-review C1)": [{"rel": 1, "steps": [[2, 0] for _ in range(5)]}],
        "service too short for its work": [{"rel": 1, "steps": [[2, 228], [229, 260], [261, 317], [318, 369], [370, 500]]}],
        "missing lot": [],
        "four steps": [{"rel": 1, "steps": good[0]["steps"][:4]}],
        "beyond horizon": [{"rel": 1, "steps": good[0]["steps"][:4] + [[370, 627]]}],
    }
    for name, w in bad.items():
        try:
            validate(w, 1, 626, 0)
        except (AssertionError, KeyError, TypeError):
            continue
        sys.exit(f"selftest: validate accepted {name}")
    print(json.dumps({"validate_selftest": "ok", "rejected": sorted(bad)}))

def main():
    if sys.argv[1:] == ["--selftest"]:
        return selftest()
    args = [a for a in sys.argv[1:] if a != "--fifo"]
    fifo = len(args) != len(sys.argv) - 1
    ns = [int(x) for x in args[0].split(",")] if args else list(range(1, 8))
    delta = int(args[1]) if len(args) > 1 else 0
    out = {"z3": z3.get_version_string(), "delta": delta, "fifo": fifo, "rows": []}
    for n in ns:
        top = t_opt(n)
        row = {"n": n, "t_opt_claimed": top}
        for h in (top - 1, top):
            s, rel, st, fi = build(n, h, delta, fifo)
            t0 = time.time()
            r = s.check()
            dt = time.time() - t0
            row[f"H={h}"] = str(r)
            row[f"secs@{h}"] = round(dt, 2)
            if r == z3.sat:
                w = model_rows(s.model(), n, rel, st, fi)
                validate(w, n, h, delta)
                row["witness_revalidated"] = True
                row["witness_last_arrival"] = max(x["steps"][4][1] for x in w)
                row["witness"] = w
            print(json.dumps({k: v for k, v in row.items() if k != "witness"}), file=sys.stderr, flush=True)
        out["rows"].append(row)
    # receipt carries results only; wall times are in the stderr log
    for row in out["rows"]:
        for key in [k for k in row if k.startswith("secs@")]:
            del row[key]
    txt = json.dumps(out, indent=1, sort_keys=True)
    print(txt)

if __name__ == "__main__":
    main()
