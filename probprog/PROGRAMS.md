# The comparison programs: sources, encodings, ground truth

The comparison set began as the three-program table the Diabolo paper
(POPL 2025, §evaluation, "comparison with GuBPI") uses to compare its
residual-mass semantics against GuBPI, both tools "configured to
produce the same bounds," and has since grown case by case: the PSI
burglar alarm and the Beauquier–Gradinariu–Johnen token ring from the
GuBPI suite, the coupon-collector, Israeli–Jalfon, and 3-D
asymmetric-random-walk EV cases from the Diabolo suite, and Herman's
3-process self-stabilization ring — the first case present in BOTH
baselines' suites (Diabolo `psi/herman3`, GuBPI `herman-3`). Sources are
vendored verbatim from each baseline's repository (both MIT; the
original trio byte-identical to the Zenodo artifact,
10.5281/zenodo.14169507 — all six files diffed). For the original
trio the comparison configurations are the Zenodo artifact's actual
commands — unrolling **100 / 14 / 6** at default limit, i.e. Diabolo
dialed down to match GuBPI's bound quality ("same bounds" is a quality
match, with the repository's `comparison.sh` carrying different,
tighter settings than the published table used); the GuBPI
configurations are embedded in the `.spcf` twins' headers. Later cases
run each baseline at its own artifact configuration (the `flags(...)`
headers in the vendored `.sgcl` files, the depth headers in the `.spcf`
files); per-case configurations are recorded beside the numbers in
`out/comparison.json`. One upstream caveat, recorded by the GuBPI
artifact's own README: GuBPI does not successfully normalize
`die_paradox.spcf` (its normalized bounds are vacuous), so the
die-paradox tightness comparison follows the artifact's practice of
comparing unnormalized lower bounds for GuBPI.

## 1. Geometric counter (`geo`)

Source (`geo.sgcl`): `Y := 0; while flip(1/2) { Y += 1 }; return Y`.

Chain encoding: one alive state per layer (Y equals the step index);
per transition: continue w.p. 1/2 (Y increments), absorb into class
Y w.p. 1/2. No rejection.

Ground truth: $P[Y{=}n] = 2^{-(n+1)}$, $\mathbb{E}Y = 1$.

## 2. Asymmetric random walk (`asym_rw`)

Source (`asym_rw.sgcl`, adapted by the Diabolo authors from
Klinkenberg et al., LOPSTR 2020): `X := 1; C := 0; while X != 0
{ C += 1; if flip(0.25) { X += 1 } else { X -= 1 } }; return C`.

Chain encoding: alive states are positions $x \ge 1$; the query
variable C equals the absorption time, so absorption at transition
$t{\to}t{+}1$ from $x{=}1$ downward lands in class $t{+}1$. No
rejection. Absorption only at odd times.

Ground truth: $\mathbb{E}C = 1/(1-2r) = 2$ at $r = 1/4$; per-class
masses are ballot-walk rationals (the producer's DP recovers
$P[C{=}1] = 3/4$, $P[C{=}3] = 9/64$, … exactly); tail
$\Theta(n^{-3/2}(\sqrt{3}/2)^n)$.

## 3. Mossel's die paradox (`die_paradox`)

Source (`die_paradox.sgcl`): `die := 0; throws := 0; while die != 6
{ die ~ UniformDisc(1, 7); observe die in [2, 4, 6]; throws += 1 };
return throws`.

The observation admits 2, 4, and 6; an odd throw rejects the run
(hard conditioning). The count includes the final 6.

Chain encoding: one alive state; per transition: absorb-accept into
class throws w.p. 1/6 (the 6), continue w.p. 2/6 (a 2 or 4), reject
w.p. 3/6 (an odd face). Rejected mass is excluded from the posterior
normalizer by the reject labeling — hard conditioning as a class, not
a score, so every stored row remains a probability row.

Ground truth: $P[T{=}n \mid \text{acc}] = (2/3)(1/3)^{n-1}$,
$\mathbb{E} = 3/2$, evidence $Z = 1/4$.

## 4. PSI burglar alarm (`burglarAlarm`)

Source: GuBPI repository `benchmarks/Discrete/burglarAlarm/`
(`burglarAlarm.spcf` with its `.psi` original beside it; GuBPI vendors
it from the PSI repository; MIT). Not in Diabolo's benchmark suite —
the Diabolo column for this case is honestly n/a. A finite Bayes net
with hard conditioning:

```
earthquake ~ flip(1/10000);  burglary ~ flip(1/1000);
alarm = earthquake || burglary;
phoneWorking ~ earthquake ? flip(7/10) : flip(99/100);
maryWakes ~ alarm ? (earthquake ? flip(8/10) : flip(6/10)) : flip(2/10);
observe(maryWakes && phoneWorking);  return burglary
```

Chain encoding: layered, one sampled variable per layer (start →
earthquake → +burglary → +phoneWorking), with the final layer folding
the maryWakes sample into the observe test: `mw && pw` absorbs into
class `burglary`, everything else rejects. The program is finite, so
horizon 4 is **full unrolling**: the alive frontier is empty, the
residual is exactly zero, and both posterior brackets are exact
points — the class-uniform width $R/(S+R)$ degenerates to $0$.

Ground truth (PSI's exact inference, recorded in the vendored source's
own comment): $P[\text{burglary} \mid \text{called}] =
2969983/992160802 \approx 0.0029934$. The producer's DP reproduces it
bit-exactly, and an independent 16-assignment enumeration oracle in
the tests re-derives every class and reject mass without the chain
machinery.

## 5. beauquier-etal-3, query Q1 (`beauquier-3`)

Source: the GuBPI benchmark suite's
`benchmarks/ProbEstimation/beauquier-etal-3/beauquier-etal-3-Q1.spcf`
(Sankaranarayanan-18 adaptation of a Beauquier–Gradinariu–Johnen-style
self-stabilizing 3-process token ring; MIT, vendored verbatim). Each
process carries two bits $(d_i, p_i)$, all six sampled iid fair; one
synchronous round reads OLD values throughout (the predecessor of
process 1 is process 3): $d_i$ flips iff $d_i = d_{i-1}$, and $p_i$
resamples uniformly iff additionally $p_i = p_{i-1}$ (the source's
"keep w.p. $1/2$ else flip" coin is a uniform resample). The program
loops while $p_1 + p_2 + p_3 \ge 2$, incrementing `count` on every
call including the exit call, and enters the loop only when the
initial $p$-sum is $\ge 2$; the query is $P[\mathit{count} < 1]$,
i.e. the initial $p$-sum lands at $\le 1$ — exactly $1/2$ by the
binomial count on three iid fair bits.

Chain encoding: **time-homogeneous** (the classes are
time-independent, so the emitter's compact form applies — states
materialized once, not per layer): a start state whose row is the
64-outcome initial sample, class 0 (`count < 1`) absorbing the
$p$-sum $\le 1$ half immediately; 32 alive ring configurations; one
weight per joint coin outcome of a synchronous round; rounds landing
at $p$-sum $\le 1$ absorb into class 1. Score-free — no rejection,
evidence + residual $= 1$ exactly, posterior brackets coincide with
the unnormalized ones. The wire chain is 35 states / 91 stored rows
at ANY horizon; the bracket for class 0 is
$[1/2,\ 1/2 + R_T]$ with $R_T$ the exact alive mass at the horizon
($4.53 \times 10^{-6}$ at $T = 51$, $2.5 \times 10^{-101}$ at
$T = 1000$).

Ground truth: $P[\mathit{count} < 1] = 1/2$ exactly (both baselines'
published intervals for Q1 contain it). Baseline context: this case is
a GuBPI suite member; Diabolo publishes no row for it, so the
comparison table carries no Diabolo column here (absence stated, not
interpolated).

Translation extras for this case, beyond the three standing legs: a
one-step direct-transcription oracle (the full 64-config distribution
evolved one round by per-process lets over old values, compared to the
chain rows' action, all 32 alive configs), and the cross-program tooth
— a sequential-update slip variant (guards reading already-updated
neighbor bits) emits an internally-green certificate and is refused by
the Monte-Carlo translation differential at 4-sigma, which is exactly
why the differential is part of the battery.

## 6. Coupon collector, five coupons (`coupon-collector5`)

Source (Diabolo `benchmarks/ours/coupon-collector5.sgcl`, vendored
verbatim): draw uniformly among five coupons until every coupon has
been seen; return the number of draws. The compared query is the
first moment of the return value (the baseline's EV objective).

Chain encoding: **time-homogeneous**; the five coupon flags are
exchangeable, so the chain state is the count $k$ of distinct coupons
collected ($k \in 0..4$ alive; collecting the fifth is the single
terminal class) — from $k$: a new coupon w.p. $(5-k)/5$, a repeat
w.p. $k/5$ (self-loop). The exchangeability lump is a translation
step, discharged by the five-explicit-flags source-semantics
differential and the inclusion–exclusion oracle. The EV certificate
is the survival-form assembly: exact partial sums from the DP plus a
geometric tail from the generator-form template rows at
$\gamma = 4/5$ — exactly the alive spectral radius, so every
transfer inequality holds with equality and the certified tail is
asymptotically the true tail.

Ground truth: $\mathbb{E} = 5 H_5 = 137/12$ exactly; tail rate
exactly $4/5$.

## 7. Israeli–Jalfon self-stabilization, ring of four (`israeli-jalfon4`)

Source (Diabolo `benchmarks/ours/israeli-jalfon4.sgcl`, vendored
verbatim): four token flags sampled iid fair; while more than one
token remains, one loop iteration snapshots the flags and walks the
four if-blocks IN ORDER — a block whose old flag is set clears its
own flag and sets a uniformly-chosen neighbor's (predecessor on
heads, successor on tails); `count` increments once per iteration.
The compared query is the first moment of `count` (the expected
self-stabilization time). The sequential-update semantics is
load-bearing, and the one-step direct-transcription oracle plus the
cross-program synchronous-slip tooth pin exactly it.

Chain encoding: **time-homogeneous**; states are the 16 flag
bit-vectors plus a START state carrying the Bernoulli$(1/2)^4$
prologue as one 16-outcome row; configurations with token count
$\le 1$ classify into the single stabilized terminal class (closed
— no re-entry). With the prologue, the chain absorption time is
$T = \mathit{count} + 1$ on every path, so the EV certificate
carries **anchor-offset 1** and the checker recomputes the
prologue's determinism from the stored rows rather than trusting an
out-of-band subtraction.

Ground truth: $\mathbb{E}[\mathit{count}] = 373/256 = 1.45703125$ by
the exact linear-solve oracle over the 11 alive configurations —
the sequential block order breaks the ring's rotational symmetry
(the 3-token expectations differ by configuration), so class-lumped
analyses over token counts are wrong, and the mechanical oracle
chain is the record. Alive spectral radius exactly $1/2$.

## 8. 3-D asymmetric random walk (`3d-asym-rw`)

Source (Diabolo `benchmarks/ours/3d-asym-rw.sgcl`, vendored
verbatim): $(x,y,z) := (1,1,1)$; while any coordinate is nonzero:
choose an axis uniformly (the source spells it `flip(1/3)` then
`flip(1/2)`), move it $+1$ w.p. $1/4$ else decrement with MONUS at
zero (SGCL variables are naturals — the vendored tool's own
decrement saturates); count every step. The compared query is the
first moment of the count (the absorption time into the origin).

Chain encoding: **time-homogeneous with an infinite alive-reachable
set** — the certificate presents the chain as a pattern (dimension
and up-probability as species data) with stored rows on a finite
window; the mass bracket rides the absorption DP at a finite
horizon, and the EV tail rides the product-profile template rows
(per-coordinate profile ratios ramping from $1$ up to a geometric
rate — a pure per-coordinate geometric weight provably cannot
contract this walk), collapsed to finitely many exact-rational
inequalities by the class-collapse side condition.

Ground truth: no closed form; the float linear-solve oracle gives
$\approx 16.69099$, the $10^6$-path differential agrees, and the
exact bracket at horizon 200 is $[16.690028724,\ 16.702938972]$.

## 9. Herman's self-stabilization, 3 processes (`herman3`)

Source (Diabolo `benchmarks/psi/herman3.sgcl`, vendored verbatim,
originally from the PSI benchmark collection; the SAME program is the
GuBPI suite's Sankaranarayanan-18 member
`benchmarks/ProbEstimation/herman-3/` — the comparison set's first
case that BOTH baselines run): three bits sampled iid fair; while
$p_1 + p_2 + p_3 \ne 1$, one synchronous round — process $i$ compares
its own (still-old) value with the OLD snapshot of its ring
predecessor ($1 \leftarrow 3$, $2 \leftarrow 1$, $3 \leftarrow 2$);
equal $\Rightarrow$ resample fair, different $\Rightarrow$ copy the
predecessor's old value — incrementing `count` per round; return
count. Unlike israeli-jalfon4's sequential blocks, the round is
genuinely synchronous BY THE SOURCE (every later block reads
`oldprocess*` snapshots) — asserted by the one-step
direct-transcription oracle, and the cross-program tooth is exactly
the snapshot-DROPPING slip (blocks reading already-updated neighbor
values), refused by the translation differential.

Chain encoding: **time-homogeneous**; states are the 8 packed bit
configurations plus a START state carrying the Bernoulli$(1/2)^3$
prologue as one 8-outcome row (8 wire states, 24 stored rows). Two
terminal classes, beauquier-style: class 0 = `count < 1` (absorption
on the START transition — GuBPI's Q1 query), class 1 =
`count >= 1`. Score-free; posterior brackets coincide with the
unnormalized ones. With the prologue the chain absorption time is
$T = \mathit{count} + 1$ on every path, so the EV certificate
carries **anchor-offset 1**. The queries compared: the first moment
of `count` against Diabolo's EV row, and $P[\mathit{count} < 1] =
3/8$ against GuBPI's Q1.

Ground truth: $\mathbb{E}[\mathit{count}] = 4/3$ by the exact
linear-solve oracle over the 5 alive configurations (per-state:
$2$ from each two-one configuration, $7/3$ from $000$ and $111$),
matching the vendored source's own header comment;
$P[\mathit{count} > n] = (3/4)(1/2)^n - (1/8)(1/4)^n$ exactly; alive
spectral radius exactly $1/2$. The eigen-tight weights ($1$ on the
two-one configurations, $3/2$ on $000$, $111$, and START) make every
transfer row an equality — but, unlike israeli-jalfon4, the
above-floor states RECUR, so the geometric tail bound is sound
without being achieved and the EV upper endpoint exceeds the truth
by exactly $(1/6)\,4^{-(N-1)}$ at horizon $N$.

## Translation validity

Three legs, all mechanized in `producer/`:

1. **Vendored sources** (above) — the encodings were verified against
   the vendored bytes, not remembered semantics.
2. **Monte-Carlo differential** — each source program's published
   semantics simulated directly (rejection sampling for the hard
   observation), $10^6$ paths, compared to the chain evaluation's
   exact masses at 4-sigma binomial bands (`differential.rs` tests).
3. **Closed-form containment** — every emitted bracket is tested to
   contain the exact ground truth above (`lib.rs` tests).
