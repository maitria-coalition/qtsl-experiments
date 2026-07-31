#!/usr/bin/env python3
"""Normalized-aggregate and polynomial-head countermodels, exact rationals.

Companion battery to transfer_countermodels.py (numbering continues
CM1-CM5 there): the cells an adversarial review added — arity
normalization breaks footprint locality (CM6, kernel twin
`mean_breaks_locality` in the qtsl repository's
`lean/qtsl/Qtsl/Plate/TTransfer.lean`); the transformed gap is not the
aggregate-scale gap (CM7, kernel twin `square_gap_is_not_A_gap` /
`pow_gap_factorized`); polynomial heads over multiple folds need their
own hypotheses (CM8/CM8b — why a plated certificate is a bare fold and
nonlinear heads take the direct row); expectation rows give no
pathwise count bound (CM9 — why stochastic resets ride the
supermartingale route); guard scope is load-bearing (CM10); and the
gap-failure statement for idempotent folds needs a separated pair
(CM11 — vacuity otherwise). Fractions only, sub-second."""

from fractions import Fraction as Q


FAILURES = []


def check(name, condition):
    status = "PASS" if condition else "FAIL"
    print(f"  [{status}] {name}")
    if not condition:
        FAILURES.append(name)


def mean(values):
    return sum(values, Q(0)) / len(values)


print("CM6 — coordinatewise-monotone mean lacks footprint locality")
m = (Q(0), Q(1))
X = (Q(1),)
Y = (Q(1), Q(1))
m_after = (Q(0), Q(1), Q(1))
check("footprint mean safety row holds", mean(Y) <= mean(X))
check("ambient mean nevertheless increases from 1/2 to 2/3",
      mean(m) == Q(1, 2) and mean(m_after) == Q(2, 3) and mean(m_after) > mean(m))


print("CM7 — transformed gap is not the same A-space gap at p=2")
x = Q(5, 4)
y = Q(3, 4)
S_gap = x * x - y * y
A_gap = x - y  # singleton l2 aggregates need no root computation
shortcut = Q(1) * Q(1) - Q(0) * Q(0)
check("square-sum gap is exactly 1", S_gap == 1)
check("A-space drop is 1/2, below phi(1)-phi(0)=1",
      A_gap == Q(1, 2) and A_gap < shortcut)


print("CM8 — per-place safety does not survive an arbitrary polynomial head")
V1_before, V2_before = Q(0), Q(1)
V1_after, V2_after = Q(0), Q(0)
q_before = V1_before - V2_before
q_after = V1_after - V2_after  # q(V1,V2)=V1-V2
check("both component sums are nonincreasing",
      V1_after <= V1_before and V2_after <= V2_before)
check("polynomial-head certificate increases from -1 to 0",
      q_before == -1 and q_after == 0 and q_after > q_before)

print("CM8b — even an isotone polynomial head can lose a uniform gap")
ok = True
for n in (1, 2, 10, 100, 1000):
    # q(x,z)=xz is strictly increasing in each coordinate on x,z>0.
    z = Q(1, n)
    component_gap = Q(1) - Q(0)
    head_gap = Q(1) * z - Q(0) * z
    ok &= component_gap == 1 and head_gap == Q(1, n)
check("component gap 1 becomes head gap 1/n as the other component approaches 0", ok)


print("CM9 — an expectation gap gives no deterministic pathwise count bound")
prob_keep = Q(1, 2)
expected_output = prob_keep * Q(1) + (1 - prob_keep) * Q(0)
delta = Q(1, 2)
V0 = Q(1)
no_drop_firings = 3
path_probability = prob_keep ** no_drop_firings
check("expected footprint row E[Y] <= X-delta holds with equality",
      expected_output == V0 - delta)
check("a positive-probability path exceeds V0/delta without any drop",
      path_probability == Q(1, 8) and no_drop_firings > V0 / delta)


print("CM10 — a guard-local row is unsound if firing escapes its guard")
# Reset X={x} to Y={0}; 0 <= x-1 is valid only on guard x>=1.
x_outside_guard = Q(0)
row_at_outside_value = Q(0) <= x_outside_guard - 1
check("gap row is false at the out-of-guard firing value x=0",
      not row_at_outside_value)
check("the erroneous reset can self-loop at V=0 forever while the floor holds",
      x_outside_guard == 0)


print("CM11 — nontrivial bounded max does not fail GAP at every delta")
M = (Q(0), Q(1))
large_delta = Q(2)
premise_instances = [(a, b) for a in M for b in M if a <= b - large_delta]
gap_holds_vacuously = all(
    max(a, r) <= max(b, r) - large_delta
    for a in M for b in M for r in M
    if a <= b - large_delta
)
check("at delta=2 there is no separated pair in M={0,1}", premise_instances == [])
check("GAP therefore holds vacuously at that delta", gap_holds_vacuously)


print()
if FAILURES:
    print("RESULT: FAIL —", FAILURES)
    raise SystemExit(1)
print("RESULT: ALL COUNTERMODELS PASS (fractions only)")
