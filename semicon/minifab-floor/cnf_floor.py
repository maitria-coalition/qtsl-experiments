#!/usr/bin/env python3
"""Independent CNF (order-encoding) of model R — the same relaxation
z3_floor.py states over integers, re-encoded from scratch so the two
lanes share the model text (README.md §R) and nothing else.

usage: cnf_floor.py N H out.cnf [--fifo]   -> writes DIMACS; the
       variable map goes to out.cnf.map.json for witness decoding.

Encoding: each event round x in [lo,hi] gets order literals
[x<=t] for lo<=t<hi ([x<=hi] = TRUE, [x<=lo-1] = FALSE) with the ladder
[x<=t] -> [x<=t+1]. All constraints of R become clauses over these:
  precedence x >= y + c        : [x<=t] -> [y<=t-c]
  distinct (one firing / round): not(x=t and y=t)
  machines                     : containment indicators g forced TRUE
                                 by containment, at-most-(cap-1) over g
  ticks (Hall on intervals)    : indicators forced TRUE by window
                                 inclusion, unary counter, counter
                                 output m -> f_q - a_p >= m*T_k
  early first tick             : e_i -> OR_j z_ij, z_ij -> (s_j<=s_i-1
                                 and s_i<=f_j-1)
By default NO queue-order (FIFO) clauses: the only symmetry breaking
is rel_0 < rel_1 < ..., a relabelling of lot indices under which R is
invariant. --fifo adds queue order (start order follows arrival order
at q1..q5), justified only by the exchange lemma of README §R (tested
by fifo_exchange_check.py), not by net semantics. Only the 'forced TRUE' direction of each indicator is emitted: every
indicator sits only on the side of a constraint that it tightens, so a
spurious TRUE can only hurt the solver; equisatisfiability with R is
by that monotonicity (README.md §CNF).
"""
import sys, json

T = [225, 30, 55, 50, 255]
STATION = [0, 1, 2, 1, 0]
CAP = [2, 2, 1]
# The window/Hall encoding assumes every window is nonempty (T_k >= 2;
# with T_k = 1 an empty early window would skip Hall vacuously).
assert min(T) >= 2

class CNF:
    def __init__(self):
        self.nv = 0
        self.cl = []
    def new(self):
        self.nv += 1
        return self.nv
    def add(self, *lits):
        lits = [l for l in lits if l is not False]
        if any(l is True for l in lits):
            return
        self.cl.append(lits)

class IntVar:
    """order-encoded integer on [lo, hi]."""
    def __init__(self, c, name, lo, hi):
        assert lo <= hi, (name, lo, hi)
        self.c, self.name, self.lo, self.hi = c, name, lo, hi
        self.v = {t: c.new() for t in range(lo, hi)}
        for t in range(lo, hi - 1):
            c.add(-self.v[t], self.v[t + 1])
    def le(self, t):  # literal for [x <= t], or True/False
        if t >= self.hi: return True
        if t < self.lo: return False
        return self.v[t]
    def gt(self, t):  # literal for [x > t]
        l = self.le(t)
        return (not l) if isinstance(l, bool) else -l
    def ts(self):
        return range(self.lo - 1, self.hi + 1)

def neg(l):
    return (not l) if isinstance(l, bool) else -l

def geq(c, x, y, k, guard=()):
    """guard -> x >= y + k."""
    for t in range(x.lo - 1, x.hi + 1):
        # x <= t -> y <= t - k
        c.add(*[neg(g) for g in guard], neg(x.le(t)), y.le(t - k))

def distinct(c, x, y):
    for t in range(max(x.lo, y.lo), min(x.hi, y.hi) + 1):
        c.add(neg(x.le(t)), x.le(t - 1), neg(y.le(t)), y.le(t - 1))

def force_lt(c, x, y, ind):
    """(x < y) -> ind     [x<y iff exists t: x<=t and y>t]"""
    for t in range(x.lo - 1, x.hi + 1):
        c.add(neg(x.le(t)), y.le(t), ind)

def at_most(c, lits, k):
    from itertools import combinations
    for sub in combinations(lits, k + 1):
        c.add(*[-l for l in sub])

def build(n, H, fifo=False):
    c = CNF()
    # domains: ASAP from rel_i >= i+1 (rels strictly increasing), ALAP from H
    lo_s, lo_f, hi_s, hi_f = {}, {}, {}, {}
    for i in range(n):
        t = i + 1
        for k in range(5):
            lo_s[i, k] = t + 1
            lo_f[i, k] = lo_s[i, k] + T[k]
            t = lo_f[i, k]
        t = H
        for k in reversed(range(5)):
            hi_f[i, k] = t
            hi_s[i, k] = t - T[k]
            t = hi_s[i, k] - 1
    rel = [IntVar(c, f"rel{i}", i + 1, hi_s[i, 0] - 1) for i in range(n)]
    S = {(i, k): IntVar(c, f"s{i}_{k}", lo_s[i, k], hi_s[i, k]) for i in range(n) for k in range(5)}
    F = {(i, k): IntVar(c, f"f{i}_{k}", lo_f[i, k], hi_f[i, k]) for i in range(n) for k in range(5)}
    for i in range(n):
        if i > 0:  # lot-relabelling symmetry break only (README §R)
            geq(c, rel[i], rel[i - 1], 1)
        geq(c, S[i, 0], rel[i], 1)
        for k in range(1, 5):
            geq(c, S[i, k], F[i, k - 1], 1)
    for k in range(5):
        for i in range(n):
            for j in range(i + 1, n):
                distinct(c, S[i, k], S[j, k])
                distinct(c, F[i, k], F[j, k])
    if fifo:  # queue order: only under the exchange lemma (README §R)
        for i in range(n - 1):
            geq(c, S[i + 1, 0], S[i, 0], 1)
        for k in range(1, 5):
            for i in range(n):
                for j in range(n):
                    if i == j: continue
                    b = c.new()
                    force_lt(c, F[i, k - 1], F[j, k - 1], b)
                    geq(c, S[j, k], S[i, k], 1, guard=(b,))
    # machines
    for sig in range(3):
        ivs = [(i, k) for i in range(n) for k in range(5) if STATION[k] == sig]
        for (i, k) in ivs:
            x = S[i, k]
            for shift in (0, 1):  # 0: pickability at s; 1: dueness at s-1
                gs = []
                for (j, m) in ivs:
                    if (j, m) == (i, k): continue
                    g = c.new(); gs.append(g)
                    # contained(X): S_jm <= X < F_jm, X = s_ik - shift
                    for t in range(x.lo, x.hi + 1):
                        X = t - shift
                        c.add(neg(x.le(t)), x.le(t - 1), neg(S[j, m].le(X)), F[j, m].le(X), g)
                at_most(c, gs, CAP[sig] - 1)
    # early first tick, window starts a_ik = s_ik + 1 - e_ik
    A = {}
    for k in range(5):
        for i in range(n):
            e = c.new()
            zs = []
            for j in range(n):
                if j == i: continue
                z = c.new(); zs.append(z)
                geq(c, S[i, k], S[j, k], 1, guard=(z,))   # s_j <= s_i - 1
                geq(c, F[j, k], S[i, k], 1, guard=(z,))   # s_i <= f_j - 1
            c.add(-e, *zs)
            a = IntVar(c, f"a{i}_{k}", S[i, k].lo, S[i, k].hi + 1)
            # e -> a = s ; -e -> a = s + 1
            geq(c, a, S[i, k], 0, guard=(e,)); geq(c, S[i, k], a, 0, guard=(e,))
            geq(c, a, S[i, k], 1, guard=(-e,)); geq(c, S[i, k], a, -1, guard=(-e,))
            A[i, k] = a
    # Hall on every [a_p, f_q - 1]
    for k in range(5):
        for p in range(n):
            for q in range(n):
                hs = []
                for l in range(n):
                    u = c.new()  # forced by a_l >= a_p   (i.e. not a_l < a_p)
                    for t in range(A[p, k].lo - 1, A[p, k].hi + 1):
                        c.add(u, neg(A[p, k].le(t)), A[l, k].le(t - 1))
                    v = c.new()  # forced by f_l <= f_q   (i.e. not f_q < f_l)
                    for t in range(F[l, k].lo - 1, F[l, k].hi + 1):
                        c.add(v, neg(F[l, k].le(t)), F[q, k].le(t - 1))
                    h = c.new()
                    c.add(-u, -v, h)
                    hs.append(h)
                # unary counter R[l][m]: at least m of hs[0..l]
                prev = {}
                for l, h in enumerate(hs):
                    cur = {}
                    for m in range(1, l + 2):
                        r = c.new(); cur[m] = r
                        if m in prev: c.add(-prev[m], r)
                        if m == 1: c.add(-h, r)
                        else: c.add(-prev[m - 1], -h, r)
                    prev = cur
                for m, o in prev.items():
                    # o -> (f_q - 1) - a_p + 1 >= m T  i.e. f_q >= a_p + m T
                    geq(c, F[q, k], A[p, k], m * T[k], guard=(o,))
    vmap = {"rel": {str(i): [rel[i].lo, rel[i].hi, [rel[i].v[t] for t in range(rel[i].lo, rel[i].hi)]] for i in range(n)},
            "S": {f"{i},{k}": [S[i, k].lo, S[i, k].hi, [S[i, k].v[t] for t in range(S[i, k].lo, S[i, k].hi)]] for (i, k) in S},
            "F": {f"{i},{k}": [F[i, k].lo, F[i, k].hi, [F[i, k].v[t] for t in range(F[i, k].lo, F[i, k].hi)]] for (i, k) in F}}
    return c, vmap

def main():
    n, H, out = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    fifo = "--fifo" in sys.argv[4:]
    if any(a != "--fifo" for a in sys.argv[4:]) or len(sys.argv) < 4:
        sys.exit("usage: cnf_floor.py N H out.cnf [--fifo]")
    c, vmap = build(n, H, fifo)
    with open(out, "w") as f:
        f.write(f"c minifab model R n={n} H={H} fifo={fifo}\n")
        f.write(f"p cnf {c.nv} {len(c.cl)}\n")
        for cl in c.cl:
            f.write(" ".join(map(str, cl)) + " 0\n")
    with open(out + ".map.json", "w") as f:
        json.dump({"n": n, "H": H, "map": vmap}, f)
    print(f"n={n} H={H} vars={c.nv} clauses={len(c.cl)}", file=sys.stderr)

if __name__ == "__main__":
    main()
