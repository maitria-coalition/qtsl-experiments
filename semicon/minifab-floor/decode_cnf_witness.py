#!/usr/bin/env python3
"""decode a CaDiCaL 'v' model through cnf_floor's map and re-validate
the schedule against model R with z3_floor.validate (an explicit
round-by-round EDF tick assignment + machine counters)."""
import sys, json
from z3_floor import validate
mp = json.load(open(sys.argv[1])); sol = sys.argv[2]
true = set()
for line in open(sol):
    if line.startswith("v"):
        true.update(int(x) for x in line.split()[1:] if int(x) > 0)
def val(e):
    lo, hi, vs = e
    for t, v in zip(range(lo, hi), vs):
        if v in true: return t
    return hi
n, H, m = mp["n"], mp["H"], mp["map"]
rows = [{"rel": val(m["rel"][str(i)]), "steps": [[val(m["S"][f"{i},{k}"]), val(m["F"][f"{i},{k}"])] for k in range(5)]} for i in range(n)]
validate(rows, n, H, 0)
print(json.dumps({"n": n, "H": H, "witness_revalidated": True, "last_arrival": max(r["steps"][4][1] for r in rows), "witness": rows}))
