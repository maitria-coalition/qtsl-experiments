# Lean-route admission gate (generated — `make lean-route` re-derives; do not hand-edit)

Fitted law (compiled reflection route, 9 accepted rungs,
919–24,686 container bytes, one box):

    t(seconds) = exp(-30.207) * bytes^3.577
    exponent 95% CI [3.24, 3.92], r^2 = 0.989
    box: AMD EPYC 7B13, 16 threads, 126 GiB (epyc-lean-dedicated)

**Admission threshold at the 10-minute budget: ~28 KB
of sealed container.** A species attempts a Lean-reflection timing only if
(a) its certificates sit under the threshold and (b) it is in the route's
accepted fragment (first-order pinned-vocabulary rule set; a
primitive-application certificate is out of fragment, and the route
declines it by design -- a completeness boundary, never a soundness
one). The kernel (`decide`) route admits **nothing**: its
smallest measured full acceptance anywhere (919 B) exceeded both the
budget and this box's memory; see the chapter section.

Byte basis: the fitted regressor is sealed-container bytes. Each census
row declares its `bytes_definition`: `artifact_proxy` rows (committed
artifacts' sizes) get the measured ~420 B container overhead added;
`sealed_container` rows (campaign-banked real container sizes) are used
as-is — one definition per row, never blended. When receipts land,
update the row's stats and flip its definition together, then recompute
(`make lean-route`). Predictions
beyond the fitted domain (24,686 B) are extrapolations of the
power law and are marked by being, frankly, enormous.

| species | section | n | med. bytes | max bytes | fragment | pred. (med.) | pred. (max) | verdict |
|---|---|---:|---:|---:|---|---|---|---|
| hardware liveness (circuits) | circuits | 245 | 829 | 2,627 | untested | 9.1 ms | 0.22 s | **PASS** |
| Atari certified replay | atari | 15 | 1,189 | 5,113 | untested | 22 ms | 1.9 s | **PASS** |
| FrozenLake 4x4 (det + slippery + sr34) | gymnasium | 20 | 9,400 | 52,015 | expected-in | 15 s | 97 min | **PARTIAL (median passes, max does not)** |
| CliffWalking | gymnasium | 10 | 30,000 | 71,872 | expected-in | 14 min | 5.1 h | **FAIL** |
| FrozenLake 8x8 | gymnasium | 8 | 73,345 | 500,289 | expected-in | 5.5 h | 0.59 yr | **FAIL** |
| Blackjack | gymnasium | 4 | 87,549 | 115,779 | expected-in | 10 h | 28 h | **FAIL** |
| Taxi (all weather/passenger variants) | gymnasium | 13 | 1,000,000 | 3,102,602 | expected-in | 7 yr | 400 yr | **FAIL** |
| LunarLander / LunarLanderContinuous (statistical tier) | gymnasium | 6 | 9,000,000 | 12,226,428 | untested | ~10^4 yr | ~10^5 yr | **FAIL** |
| published-table supermartingale corpus | supermartingales | 28569 | 2,000 | 50,000 | expected-in | 97 ms | 84 min | **PARTIAL (median passes, max does not)** |
| semiconductor fluid ladder (minifab) | semicon | 17 | 14,500 | 18,451 | untested | 65 s | 2.5 min | **PASS** |
| semiconductor fluid ladder (mimac) | semicon | 12 | 361,625 | 401,455 | untested | 0.18 yr | 0.27 yr | **FAIL** |
| interval-enclosure certificates (primitive applications) | (cross-cutting) | 1 | 603 | 603 | out | 4.4 ms | 4.4 ms | **OUT OF FRAGMENT** |

Fragment rule: "expected in" rows share their rule mix with an accepted
ladder rung; a species' FIRST Lean attempt (smallest instance) settles
fragment membership cheaply before any batch run. "untested" rows should
run that one-instance probe first regardless of size verdict.

Verdicts are computed at the species' median and maximum. A FAIL species
whose *minimum* sits under the ~28 KB threshold can still probe its
small end (the fragment probe doubles as that); the verdict governs
batch attempts, not curiosity.
