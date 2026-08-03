# The three comparison programs: sources, encodings, ground truth

The comparison set is the three-program table the Diabolo paper (POPL
2025, §evaluation, "comparison with GuBPI") uses to compare its
residual-mass semantics against GuBPI, both tools "configured to
produce the same bounds." Sources are vendored verbatim from the
Diabolo repository (MIT), which carries both tools' versions of each
program (byte-identical to the Zenodo artifact, 10.5281/zenodo.14169507
— all six files diffed). The comparison configurations are the Zenodo
artifact's actual commands — unrolling **100 / 14 / 6** at default
limit, i.e. Diabolo dialed down to match GuBPI's bound quality ("same
bounds" is a quality match, with the repository's `comparison.sh`
carrying different, tighter settings than the published table used);
the GuBPI configurations are embedded in the `.spcf` twins' headers.
One upstream caveat, recorded by the artifact's own README: GuBPI does
not successfully normalize `die_paradox.spcf` (its normalized bounds
are vacuous), so the die-paradox tightness comparison follows the
artifact's practice of comparing unnormalized lower bounds for GuBPI.

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

## beauquier-etal-3, query Q1 (`beauquier-3`)

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
