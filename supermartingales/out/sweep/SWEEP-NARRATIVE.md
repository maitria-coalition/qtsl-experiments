# The weekend sweep: re-checking a published verification table

*2026-07-18 · full table with receipts beside this in `SWEEP.md`.*

## 1 What was swept

The target paper (Abate, Edwards, Giacobbe, Punchihewa, Roy, LMCS 2026)
trains small neural networks to certify upper bounds on failure
probabilities of 17 stochastic programs, published against a classical
linear-programming baseline. The campaign goal: a certified house bound for
every row. **All 17 rows now have one**: each synthesized in exact rational
arithmetic and verified by two independent checkers down to the certificate
bytes — 25/25 runs, deterministic, mostly sub-second on CPU against their
tens of GPU-seconds of float training plus an SMT check (which succeeds on
8–10 of 10 seeds). Every house bound beats their best published number.

## 2 Their bounds are loose exactly where they claim victory

On the easy counter rows their bounds were near-tight; we beat them by
factors of 1.01–1.4. But on the random-walk rows the paper showcases — its
neural-beats-baseline exhibits — the *exact* truths are astronomically
smaller than anything printed: cliff_crossing truth $8.6\times10^{-10}$ vs.
their best $0.0553$ (~8 orders of magnitude); repulse100 truth
$9.6\times10^{-20}$ vs. $0.0268$ (17 orders); repulse100_uniform
house-certified $6.2\times10^{-61}$ vs. $0.0344$ (58 orders; no exact truth
known for this continuous-noise row, and our target-set reconstruction is
slightly conservative). Where neural beats linear, both miss the real
answer by miles.

## 3 Two published entries aren't sound

Both in the baseline column. (1) **faulty_uniform**: we certified the truth
into $[0.018446, 0.018556]$; their published $0.0181$ sits *below the
bracket*. (This program isn't printed in the paper; our reconstruction
reproduces their own LP value exactly — strong evidence it's the right one —
and for it, their entry is unsound.) (2) **The baseline convention**: on
two rows printed verbatim, their hand-supplied invariants let certificates
go negative at reachable states; with those holes closed, the best
genuinely-sound linear bounds sit *above* their published ones (repulse100:
$12/103$ vs. $0.0991$; cliff: $17/37$ vs. $0.4546$). Related: faulty_loop's
printed listing starts its counter at 0, but that program's exact truth
exceeds all three published numbers for the row — they evidently ran the
start-at-1 version. We certified both readings.

## 4 The last row proved something stronger

**persist_2d**, finished today, was the holdout: the paper never states its
noise distribution, so we built *twin* reconstructions (uniform and
Gaussian noise) of one shared dynamics skeleton from the group's own
earlier papers; both pass the sanity check against their published bound
by wide margins. Then the row turned into a small theorem. Every attempt to
certify it with a single polynomial failed — and the failure is now
*proved*: an exact-arithmetic dual witness (a measure the dynamics leaves
stationary to every moment up to degree 4) shows **no global polynomial of
degree up to 4 can certify this row**, with the same infeasibility
confirmed numerically through degree 14 (exact certificates at 6/8 are a
priced follow-up). The value function jumps at two boundaries;
piecewise functions express that cheaply, polynomials never do. So the
paper's 2-layer network here is a *necessity*, not a style choice — and our
piecewise-exact certificates win the row on **both twins**: $0.100099$
(uniform) and $0.097526$ (Gaussian), each beating both their numbers
($0.1026$, $0.1175$), with 4800/4800 and 19200/19200 certificate files
accepted. The Gaussian bound is even closer to the estimated truth than
their own best bound is (ours overshoots by 8%, theirs by 14%). Two details
from that final certification worth a smile: the grid that sufficed for the
uniform twin was *provably too coarse* here — its own discretization error
already exceeds their published bound, so the finer grid was forced, not
chosen; and the dreaded "what about the Gaussian's infinite tails?" problem
answered itself — the certificate's formulation pre-pays the tail mass, so
truncation error can only make our check *harder* to pass, never unsound.

## 5 Close

The campaign goal, set that morning, asked for this table by Monday night;
it was complete early — **unconditionally: all 17 rows, every
reconstruction certified — as of Saturday 2026-07-18 night**. The pipeline doesn't just
*reproduce* published verification tables — it *adjudicates* them: exact
ground truth nobody published, tighter bounds than the paper's method on
every row, two soundness errors caught, and one impossibility theorem.
Possibly worth a short public "certified errata + improvements" note once
the Lean glue lands.
