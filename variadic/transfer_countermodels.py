#!/usr/bin/env python3
"""Transfer-theorem countermodels + positive checks, exact rationals.

The plated-certificate transfer theorem (one footprint row per
transition schema bounds the whole marking at every extent) holds only
under specific hypotheses: a pinned positive gap, coverage, a floor
row, and a translated-sum (non-idempotent) fold scale. This battery
verifies each countermodel move by move — the rows the would-be
certificate carries all HOLD while the transfer conclusion FAILS — and
checks the positive theorems (fold locality, safety transfer, gap
transfer) on randomized exact instances, so theorems and countermodels
are witnessed by the same machinery.

Kernel-checked twins: the qtsl repository mechanizes the theorems and
the countermodels marked below in `lean/qtsl/Qtsl/Plate/TTransfer.lean`
(safety/ranking transfer, the max carousel, the halving run, the
signed-birth descent, verdict transport, the factorized modulus). The
cells WITHOUT Lean twins — the mean's arity-dependent modulus (CM2) and
the l2 aggregate-scale vanishing (CM4) — are this battery's unique
cargo, alongside the randomized positive trials.

Fixture, not grinder: sub-second, `fractions` only, no numerics.

Marking = multiset of Fraction token values, represented as sorted tuple.
Footprint firing: m' = (m \\ X) + Y, X a sub-multiset of m.
"""
from fractions import Fraction as Q
from math import isqrt
import random

random.seed(20260728)

FAILURES = []


def check(name, cond):
    tag = "PASS" if cond else "FAIL"
    if not cond:
        FAILURES.append(name)
    print(f"  [{tag}] {name}")


def fold_sum(m):
    return sum(m, Q(0))


def fold_max(m):
    return max(m) if m else None  # -inf as None; countermodels keep m nonempty


def fire(m, X, Y):
    m = list(m)
    for x in X:
        m.remove(x)  # raises if X not a sub-multiset: footprint discipline
    return tuple(sorted(m + list(Y)))


# ---------------------------------------------------------------- positive: L-LOC / T1 / T2 over (Q,+)
print("POSITIVE TRIALS — L-LOC, T1, T2 at (Q, +, <=), randomized exact instances")
ok_loc = ok_t1 = ok_t2 = True
for _ in range(500):
    n = random.randint(1, 8)
    m = tuple(sorted(Q(random.randint(-20, 20), random.randint(1, 9)) for _ in range(n)))
    kx = random.randint(1, n)
    X = random.sample(list(m), kx)
    rest = list(m)
    for x in X:
        rest.remove(x)
    # L-LOC: V(m) = A(X) + V(m\X)
    ok_loc &= fold_sum(m) == fold_sum(X) + fold_sum(rest)
    # T1: any Y with sum(Y) <= sum(X)  =>  V' <= V
    Y = [Q(random.randint(-20, 20), random.randint(1, 9)) for _ in range(random.randint(0, 3))]
    slack = fold_sum(X) - fold_sum(Y)
    if slack >= 0:
        ok_t1 &= fold_sum(fire(m, X, Y)) <= fold_sum(m)
    # T2: enforce gap delta, check aggregate drop >= delta
    delta = Q(random.randint(1, 5), random.randint(1, 4))
    Y2 = list(Y)
    excess = fold_sum(X) - delta - fold_sum(Y2)
    if Y2:
        Y2[0] += excess  # retune so sum(Y2) = sum(X) - delta exactly
    else:
        Y2 = [fold_sum(X) - delta]
    ok_t2 &= fold_sum(m) - fold_sum(fire(m, X, Y2)) >= delta
check("L-LOC fold locality, 500 sampled instances", ok_loc)
check("T1 safety transfer at sum, all APPLICABLE sampled instances (~half the draws)", ok_t1)
check("T2 gap transfer at sum: aggregate drop >= delta, 500 sampled instances", ok_t2)

# ---------------------------------------------------------------- L-IDEM at max
print("L-IDEM — gap-preservation fails at max, for every delta > 0 (r := b)")
ok = True
for _ in range(200):
    a = Q(random.randint(-9, 9), random.randint(1, 5))
    delta = Q(random.randint(1, 9), random.randint(1, 5))
    b = a + delta + Q(random.randint(0, 4))  # a <= b - delta
    r = b
    ok &= not (max(a, r) <= max(b, r) - delta)  # translation destroys the gap
check("max: a <= b-delta but max(a,r) > max(b,r)-delta at r=b, 200 instances", ok)

# ---------------------------------------------------------------- CM1
print("CM1 — strict drops, no gap: infinite run, V bounded below (v_n = 2^-n)")
v = Q(1)
rows = True
for _ in range(60):
    v2 = v / 2
    rows &= (v2 < v) and (v > 0)  # strict per-instance drop + guard
    v = v2
check("60 firings, every row strict, guard never disables", rows)
check("V after 60 firings still > 0 (no progress to the {V<=0} target)", v > 0)
check("total drop < 1 at the 60-firing prefix (closed form v_n=2^-n gives every n)", Q(1) - v < 1)

# ---------------------------------------------------------------- CM2
print("CM2 — mean's modulus at arity n is delta/n (witness family)")
ok = True
for n in [1, 2, 10, 100, 1000]:
    delta = Q(1)
    x = [Q(0)] * n
    y = [delta] + [Q(0)] * (n - 1)
    drop = fold_sum(y) / n - fold_sum(x) / n
    ok &= drop == delta / n
check("mean drop == delta/n exactly at n in {1,2,10,100,1000} -> no uniform modulus", ok)

# ---------------------------------------------------------------- CM3
print("CM3 — max carousel: every row verified, D fires forever, V constant; sum refuses B")
m = (Q(1),)
V0 = fold_max(m)
rows_ok = True
d_fires = 0
for step in range(40):
    if step % 2 == 0:  # B: consume {a}, produce {a, a}; safety row at max
        a = max(m)
        rows_ok &= a >= 1
        rows_ok &= fold_max([a, a]) <= fold_max([a])  # max-footprint safety row
        m = fire(m, [a], [a, a])
    else:  # D: consume {a}, a>=1, produce {a-1}; ranking row gap 1 by schema shape
        a = max(m)
        rows_ok &= a >= 1
        rows_ok &= a - (a - 1) >= 1  # gap 1 (holds by the schema's definition)
        rows_ok &= fold_max([a - 1]) <= fold_max([a]) - 1  # even the footprint max-row
        m = fire(m, [a], [a - 1])
        d_fires += 1
    rows_ok &= fold_max(m) == V0  # V never moves
check(f"40-step prefix (marking invariant {{1}}+k*{{0}} gives every k), {d_fires} ranking "
      f"firings, all rows hold, V = max = 1 throughout", rows_ok)
check("sum-footprint row for B is FALSE (2a <= a fails at a=1): sum correctly refuses",
      not (Q(2) <= Q(1)))
print("  minimal 2-token exhibit: m={1,1}, D on one token: per-instance gap 1, dV=0")
check("2-token exhibit: max unchanged under gap-1 per-instance drop",
      fold_max(fire((Q(1), Q(1)), [Q(1)], [Q(0)])) == Q(1))

# ---------------------------------------------------------------- CM4
print("CM4 — l2 A-space modulus vanishes: sqrt(n)-sqrt(n-1) < eta once n >= 1/(4 eta^2)+2")
ok = True
for meta in [2, 10, 100, 1000]:
    eta = Q(1, meta)
    n = (1 / (4 * eta * eta)).__ceil__() + 2
    # exact check via squared comparison, no roots computed:
    # sqrt(n) - sqrt(n-1) < eta  <==  1 < eta*(sqrt(n)+sqrt(n-1))  <==  2*eta*sqrt(n-1) > 1...
    # certified by (n-1) > 1/(4 eta^2), an exact rational comparison:
    ok &= (n - 1) > 1 / (4 * eta * eta)
check("A-space drop < eta for eta in {1/2,1/10,1/100,1/1000} at certified n (squared cmp)", ok)

# ---------------------------------------------------------------- CM5
print("CM5 — no floor row: all rows hold, gap firings forever, V -> -inf")
m = (Q(0),)
rows_ok = True
d_fires = 0
V_trace = [fold_sum(m)]
for step in range(40):
    if step % 2 == 0:  # B: consume {0}, produce {-1, +1}; sum-footprint 0 <= 0 OK
        rows_ok &= fold_sum([Q(-1), Q(1)]) <= fold_sum([Q(0)])
        m = fire(m, [Q(0)], [Q(-1), Q(1)])
    else:  # D: consume {1}, produce {0}; gap-1 ranking row
        rows_ok &= fold_sum([Q(0)]) <= fold_sum([Q(1)]) - 1
        m = fire(m, [Q(1)], [Q(0)])
        d_fires += 1
    V_trace.append(fold_sum(m))
check(f"40 steps, {d_fires} gap firings, every footprint row holds", rows_ok)
check("V = -k after the k-th D firing, EVERY k (prefix; induction gives all k)",
      all(V_trace[2 * i + 2] == -Q(i + 1) for i in range(d_fires)))
check("floor row V >= 0 is violated en route (the refusal the floor buys)",
      any(V < 0 for V in V_trace))

# ---------------------------------------------------------------- T3 phi-space
print("T3 — phi-space certification at p=2: S = sum v^2 gap-transfers; verdicts transport")
ok = True
applicable = 0
for _ in range(200):
    n = random.randint(1, 6)
    m = tuple(Q(random.randint(0, 9), random.randint(1, 3)) for _ in range(n))
    x = random.choice(m)
    # footprint: consume {x}, produce {y} with y^2 <= x^2 - delta_phi (phi-space gap row)
    delta_phi = Q(1, 7)
    if x * x >= delta_phi:
        applicable += 1
        target = x * x - delta_phi
        # any rational y with y^2 <= target; take y = floor-ish rational approx from below
        num, den = target.numerator, target.denominator
        y = Q(isqrt(num * den), den)  # y = floor(sqrt(num*den))/den <= sqrt(target)
        assert y * y <= target
        S_before = sum(t * t for t in m)
        S_after = sum(t * t for t in fire(m, [x], [y]))
        ok &= S_before - S_after >= delta_phi
ok &= applicable >= 150
check(f"phi-space sum drops >= delta_phi per firing, {applicable}/200 applicable sampled "
      "firings (never computing roots)", ok)
# Non-tautological transport check (adversarial-review repair): on the subfamily where A is
# exactly known (single token: S = v^2, A = |v| = v since v >= 0), compare the
# INDEPENDENTLY computed A >= t against the phi-space S >= t^2, for t >= 0.
ok = True
for _ in range(200):
    v = Q(random.randint(0, 9), random.randint(1, 3))
    t = Q(random.randint(0, 12), random.randint(1, 3))
    ok &= ((v >= t) == (v * v >= t * t))  # A-space vs phi-space, computed separately
check("verdict transport at t>=0, checked on the exact-root subfamily (A known = v)", ok)
# The domain clause is load-bearing (adversarial-review counterexample, p=2, S=0, t=-1):
check("even-p transport FAILS without t>=0: A=0 >= -1 true but S=0 >= (-1)^2 false",
      (Q(0) >= Q(-1)) and not (Q(0) >= Q(1)))

# ---------------------------------------------------------------- TOY (Minifab-shaped)
# Two stations A, B as plates; lot color = remaining work w; features are the
# shaped potentials vA(w) = w + WB + 2, vB(w) = w + 1; cert = sum fold; every
# schema carries a gap-1 footprint row (procA, moveAB, procB, shipB). This is
# the expressiveness walk of the staged HPlate fragment (hplate-rows.geolog):
# every row the toy needs is one of the fragment's declared shapes, and the
# forbidden shape (gap row on a max wire) is its load-reject.
print("TOY — two-station Minifab shape: shaped-potential sum cert, gap 1 per schema")
WB = Q(3)  # work per lot at station B
vA = lambda w: w + WB + 2
vB = lambda w: w + 1
# marking = (multiset at A, multiset at B), values = remaining work
def toyV(mA, mB):
    return sum((vA(w) for w in mA), Q(0)) + sum((vB(w) for w in mB), Q(0))
# footprint rows, checked as universally-shaped algebra on the schema data:
w = Q(7)  # symbolic stand-in checks ride the fixture's exact instances below
rows = True
rows &= vA(w) - vA(w - 1) >= 1            # procA: gap 1 at every w >= 1
rows &= vA(Q(0)) - vB(WB) >= 1            # moveAB: vA(0)=WB+2, vB(WB)=WB+1
rows &= vB(w) - vB(w - 1) >= 1            # procB: gap 1
rows &= vB(Q(0)) - Q(0) >= 1              # shipB: death pays vB(0)=1
check("all four schemas carry gap-1 footprint rows (shaped potential)", rows)
# exact run at 3 lots: A={2,1,0}, B={} — every firing charges >= 1 against V0
mA, mB = [Q(2), Q(1), Q(0)], []
V0 = toyV(mA, mB)
fires = 0
trace_ok = True
while mA or mB:
    Vpre = toyV(mA, mB)
    if any(x >= 1 for x in mA):           # procA
        i = next(j for j, x in enumerate(mA) if x >= 1); mA[i] -= 1
    elif any(x == 0 for x in mA):         # moveAB
        mA.remove(Q(0)); mB.append(WB)
    elif any(x >= 1 for x in mB):         # procB
        i = next(j for j, x in enumerate(mB) if x >= 1); mB[i] -= 1
    else:                                 # shipB
        mB.remove(Q(0))
    fires += 1
    trace_ok &= toyV(mA, mB) <= Vpre - 1  # every firing drops V by >= 1
check(f"exact 3-lot run: {fires} firings, each dropping V by >= 1", trace_ok)
check(f"firing bound holds: {fires} <= V0 = {V0} (T2 at delta = 1, extent-uniform)",
      Q(fires) <= V0)
check("floor: V = 0 at the empty marking, >= 0 throughout (nonneg features)", toyV(mA, mB) == 0)
# the negative cell: the SAME per-instance drops under a max fold do not move the
# aggregate when a sibling lot dominates — the fragment's load-reject is semantic:
mA2 = (Q(2), Q(2))
after = fire(tuple(mA2), [Q(2)], [Q(1)])  # procA on one lot, gap 1 per instance
check("negative cell: max fold unchanged under a gap-1 firing (the load-rejected shape)",
      max(vA(x) for x in after) == max(vA(x) for x in mA2))

print()
if FAILURES:
    print("RESULT: FAIL —", FAILURES)
    raise SystemExit(1)
print("RESULT: ALL PASS (exact rationals throughout; no floats, no roots)")
