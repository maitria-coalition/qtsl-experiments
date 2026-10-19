#!/usr/bin/env python3
"""Executable check of the FIFO exchange lemma (README.md §R, "Queue order").

Lemma: R is closed under suffix swaps. If an R-schedule has lots i, j
with f_i,k-1 < f_j,k-1 (rel_i < rel_j for k = 1) but s_ik > s_jk, giving
lot i lot j's steps k..5 and vice versa is again an R-schedule; sorting
suffixes stage by stage (k = 1..5) yields a FIFO R-schedule with the same
last arrival. So adding FIFO to R does not change satisfiability.

This script does not prove the lemma; it tests it. It samples R-witnesses
from the FIFO-free encoding (z3_floor.build, which has no queue-order
constraints) over several N, horizons and solver seeds, applies the
normalisation, and re-validates with the independent round-by-round
validator. It also reports how many raw witnesses violated FIFO, so the
test is visibly non-vacuous. Exit 0 iff every normalised witness
validates, is FIFO, and keeps its last arrival.
"""
import sys, json
import z3
from z3_floor import build, model_rows, validate, t_opt

def normalise(rows):
    rows = [{"rel": r["rel"], "steps": [list(x) for x in r["steps"]]} for r in rows]
    n = len(rows)
    for k in range(5):
        key = (lambda i: rows[i]["rel"]) if k == 0 else (lambda i: rows[i]["steps"][k - 1][1])
        order = sorted(range(n), key=key)                      # lots by arrival into q_k
        suffixes = sorted((rows[i]["steps"][k:] for i in range(n)), key=lambda s: s[0][0])  # by start_k
        for i, suf in zip(order, suffixes):
            rows[i]["steps"][k:] = [list(x) for x in suf]
    return rows

def is_fifo(rows):
    n = len(rows)
    for k in range(5):
        prev = (lambda i: rows[i]["rel"]) if k == 0 else (lambda i: rows[i]["steps"][k - 1][1])
        for i in range(n):
            for j in range(n):
                if prev(i) < prev(j) and not rows[i]["steps"][k][0] < rows[j]["steps"][k][0]:
                    return False
    return True

def main():
    cases, raw_violations = 0, 0
    for n in (2, 3, 4):
        for slack in (0, 3, 17, 60):
            h = t_opt(n) + slack
            for seed in range(6 if n < 4 else 3):
                z3.set_param("smt.random_seed", seed)
                s, rel, st, fi = build(n, h, 0)
                assert s.check() == z3.sat, (n, h)
                w = model_rows(s.model(), n, rel, st, fi)
                validate(w, n, h, 0)
                raw_violations += not is_fifo(w)
                v = normalise(w)
                validate(v, n, h, 0)
                assert is_fifo(v), (n, h, seed)
                assert max(x["steps"][4][1] for x in v) == max(x["steps"][4][1] for x in w)
                cases += 1
    print(json.dumps({"fifo_exchange_check": "ok", "witnesses": cases, "raw_non_fifo": raw_violations}))
    if raw_violations == 0:
        print("warning: no raw witness violated FIFO; the check was vacuous", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
