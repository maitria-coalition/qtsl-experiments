# MIMAC rung 2.5 — the joint sparse-LP producer

*Semiconductor-ladder rung 2.5, 2026-07-20. Executes COVER-CERT.md
named successor 1 ("the refinement LP producer"). The producer is a
workshop instrument (python; sparse HiGHS float tier + exact-rational
certifying tier) that is **not part of this repository or of mtk** —
its port to mtk is design-stage, with the shopping list below; what
ships here are its **certificates**, which stand on their own: the
checker (`fluidcheck`, unchanged — zero checker deltas at this rung)
remains the adjudicator, and nothing below rests on the producer's
own arithmetic. Certificates: `../out/mimac7/tree_core_*.cbor`,
`../out/minifab/lbfs_core_191_2wk.cbor`,
`../out/minifab/lbfs_core_959_10wk.cbor`. Receipts:
`../out/producer_receipts_joint.json`.*

## What is new, in one paragraph

For a **fixed cover**, the search over $(w, \varepsilon,
\text{all cell duals})$ is one sparse LP: the obligations (D1)
$y_i \ge 0$, (D2) $A^\top y_i + E_i^\top z_i = c(w)$, (D3) $b^\top y_i
+ f_i^\top z_i \le -\varepsilon - \langle w, \lambda\rangle$ are jointly
linear in all unknowns because $c(w)$ and $\langle w, \lambda \rangle$
are linear in $w$ — the concavity observation of COVER-CERT.md cashed
by LP duality: $\min_x c(w)^\top x$ over a cell is concave in $w$, and
passing to duals turns the concave constraint into linear rows. One
normalization pins the scale (the certificate set is a cone). The
producer builds that LP with sparse assembly (dense assembly at set7
scale would need $\sim 6 \times 10^{13}$ bytes), reconstructs exact
duals at $K = 172$, and drives tree refinement off the joint LP's own
binding-cell marginals.

**Lineage correction:** rung 1's `propose_w` was *already* the joint
LP over $(w, \varepsilon, \text{all 63 mode duals})$ — only the
release *grid* was hand-picked (FLUID-CERT.md's "weights the LP found
on its own" said as much). So the Mini-Fab question at this rung
sharpens to: where is the release *ceiling*? (Answered below: at
capacity, to $6 \times 10^{-5}$ lots/wk.)

## Normalization (the cone, pinned)

Certificates are scale-invariant ($w, \varepsilon \mapsto t w,
t \varepsilon$), so "maximize $\varepsilon$" needs a normalizer. Two
are implemented: `sumw` — $\sum_k w_k = 1$ (rung-1 Mini-Fab units,
kept for receipts comparability) — and `wlam` —
$\langle w, \lambda \rangle = 1$, under which $\varepsilon$ reads as
the drain margin as a fraction of weighted inflow. At set7 scale
`wlam` is the numerically sane choice: in `sumw` units the entire
$\varepsilon$ scale sits at $\sim 3 \times 10^{-8}$, inside LP-solver
tolerance territory; in `wlam` units it is $O(10^{-1})$. The ceiling
is normalization-invariant.

## Validation pairs (each against an independent claim)

1. **$K = 3$ example** (the checker test-suite network): joint LP on
   station-top gives $\varepsilon = 1/24$ in `sumw` units — the
   analytic remaining-work optimum $(1/4)/6$ exactly; the full tree
   gives $1/8$, a $3\times$ gain, both ACCEPT.
2. **Mini-Fab ladder**: joint eps at 84/90/94/95 lots/wk reproduces
   the rung-1 receipts ($1/3600$, $1/7200$, $1/21600$, $1/43200$) to
   float precision.
3. **set7 station-top, both variants**: COVER-CERT.md *proved* the
   remaining-work weights optimal for this cover, so the joint LP
   must match and never beat the analytic ceiling $\theta^\*$.
   Measured: eps sign flips symmetrically at the ceiling
   ($+1.001{\times}10^{-3}$ at $0.999\,\theta^\*$,
   $-9.990{\times}10^{-4}$ at $1.001\,\theta^\*$); at $\theta = 1/8$,
   joint and analytic eps agree to nine digits in `wlam` units. A
   producer that "beat" this cover would be refuted by the proof; the
   pair ran green in both variants.

## Mini-Fab: the 95→96 gap was the grid, not the weights

Bisecting the release ceiling under the full 63-cell cover (14 steps
from the bracket $[95, 96]$):

- **ceiling bracket $(95.999939,\ 96.000000)$ lots/wk** —
  certificates exist arbitrarily close to the fluid capacity 96
  lots/wk ($\rho_{\text{litho}} \to 1$), and none exists at 96 (the
  rung-1 probe, re-confirmed). The 1-lot/wk table gap of
  FLUID-CERT.md was release-grid granularity; nothing was left on the
  table by the weights.
- New certified rows past the old grid: **95.5 lots/wk**
  ($\varepsilon = 1157/10^8$) and **95.9 lots/wk**
  ($\varepsilon = 231/10^8$, i.e. **99.90% of fluid capacity**), both
  ACCEPT, emitted as fully-refined sign-split **tree covers** — the
  first tree-species certificates on the Mini-Fab (63 leaves + empty;
  the checker's tree walk, not the v0 mask walk, adjudicated them).

## set7: spine trees, grafting, and the measured price of aggregation

All numbers LBFS, `core` variant, `wlam` units, $\theta$ = fraction
of nominal release. Station-top analytic ceiling $\theta^\* =
0.132216$ (exact $1000000/7563387$; COVER-CERT.md).

**The spine-tree observation.** A *spine tree* — station-top
rewritten as a deterministic tree: stations in a fixed order
(descending utilization), each station's priority list chained
top-first — has exactly the station-top cell count (172 = $K$), but
its later cells know every earlier station is *entirely empty*, and
the joint LP converts that knowledge into certified load: measured
ceiling brackets $(0.216,\ 0.227)$ across two independent bisection
replications — **1.6–1.7× the station-top ceiling at identical cell
count**. (Cover choice, not weight choice: the station-top cover
itself is proven weight-optimal at $0.132$.)

**Certified rows** (exact duals, ACCEPT by `fluidcheck`, re-verified
from the bytes after cross-machine production):

| certificate | cells | $\theta$ | vs station-top ceiling | $\varepsilon$ (`wlam`) | checker |
|---|---|---|---|---|---|
| `tree_core_th150` | 244 | $1964/9903 \approx 0.1983$ | **1.50×** | $5750631/5{\times}10^7$ | ACCEPT, 84,180 obligations |
| `tree_core_thc90` | 244 | $1865/9509 \approx 0.1961$ | 1.48× | $794413/6250000$ | ACCEPT, 84,180 obligations |

**The wall, named honestly.** Refinement genuinely walks the curve,
but the walk is sublinear: measured float-LP ceiling brackets reach
$\sim 0.233$–$0.238$ at 847 cells (station-graft refinement, labeled
findings in the receipts, not certificates), i.e. cells ×5 buys
+6–8% ceiling per graft round. The structural reason: the ceiling is
a min over *all* leaves, so the tree species pays product-cover
prices ($\prod_s (|s|+1)$) for what is conceptually per-station
information. Reaching $\theta \approx 1$ through explicit tree covers
is hopeless at fab scale — a *cover-species* limit, not an LP limit
(see the rung-3 delta). The producer run was also stopped early by
schedule (its graft-tree checkpoint is resumable), so the certified
$0.1983$ is a floor twice over: more headroom is measured but not yet
certified, and the refinement itself was not run to diminishing
returns.

**Checker-economics rider**: the tree rows verify in $\sim$0.2–1.4 s
vs the rung-2 station-top rows' 17–21 ms at comparable obligation
counts — LP-found $w$ (denominators up to $10^7$) makes every dual
product a fat rational, where remaining-work $w$ stayed decimal. The
producer's $w$-rationalization cap is therefore a *checker-cost
knob*, and batched-replay budgets must price coefficient growth, not
just obligation count.

## Exactification at $K = 172$ (pipeline, one paragraph)

Float joint LP $\to$ rationalize $w$ (Stern–Brocot
`limit_denominator($10^7$)`, clipped positive) $\to$ per cell: float
LP at fixed exact $w$, dual support from the solver marginals, one
sparse exact-rational Gauss solve of the support system — any exact
solution with $y \ge 0$ is dual-feasible, hence sound by weak duality
regardless of float error; optimality is not required, only value
$\le$ target, which the final $\varepsilon$ (floored to $10^{-8}$
granularity) is chosen to honor. Vacuous cells ride a value-pinned
dual-feasibility solve; an exact Bland's-rule simplex is the
last-resort lane (on the emitted 244-cell rows the fast path
sufficed). Self-replay of D1/D2/D3 in exact $\mathbb{Q}$ precedes
every emit; the Rust checker then re-verifies from the bytes.

## Mutants (flagship = `tree_core_th150`, the 1.50× row)

Six mutants, six refusals, all at the designed loci (verdict lines in
`../out/producer_receipts_joint.json`; files at
`../out/mimac7/mutants/tree_*.cbor`):

| mutant | designed locus | verdict detail |
|---|---|---|
| `tree_eps_inflated` | drift-bound | dual value exceeds $-\varepsilon - \langle w,\lambda\rangle$ |
| `tree_neg_y` | dual-sign | cell 0: $y[343] < 0$ |
| `tree_tamper_w` | identity | cell 0: column 2 combination ≠ objective |
| `tree_drop_dual` | wf | cover has 244 cells, certificate carries 243 duals |
| `tree_child_reused` | wf-cover | tree node 1 reached twice |
| `tree_unsorted_y` | shape | y indices not strictly ascending |

## What the mtk port needs (design-stage; nothing below ships yet)

Every tactic this producer leans on, as the port's shopping list:
(1) a sparse LP solver exposing **duals** (HiGHS-class bindings with
row marginals on eq and ub rows, tolerance knobs to $10^{-9}$;
primal-only bindings are insufficient); (2) sparse COO→CSR assembly
with variable layout $(w, \varepsilon, y_1, z_1, \ldots)$; (3) an
exact rational kit — sparse Gauss–Jordan over $\mathbb{Q}$,
float→$\mathbb{Q}$ rationalization by Stern–Brocot mediant walk, the
support-system builder; (4) an exact simplex fallback (Bland's rule)
for support-resolve and vacuous-cell rays; (5) cell-parallelism
(cells are embarrassingly independent); (6) CBOR emission by linking
the checker crate's encoder (one canonical encoder,
cross-implementation drift impossible); (7) network/TOML loaders;
(8) orchestration — spine-tree builder, DFS-lo-first cell walk
mirroring the checker's `cover_cells` exactly, leaf
splitting/grafting, bisection-with-monotone-eps, checkpoint/resume,
normalization choice; (9) process hygiene inherited for free — the
checker stays the adjudicator, self-replay before emit, receipts as
the artifact of record.

## Rung-3 delta (what this rung changes about the SMT2020 gates)

COVER-CERT.md's gate (b) — the LP producer — is now built and
measured, which sharpens the gate rather than merely closing it:
(1) the monolithic joint LP does not scale to SMT2020 ($\sim 8$ s per
solve at 172 cells, $\sim 2$ min at 1300; at $K \sim 10^3$–$10^4$ the
model is $10^7$–$10^8$ variables) — but the block-angular structure
(cells couple only through $(w, \varepsilon)$) is exactly
Benders/Kelley-shaped, with embarrassingly-parallel cut oracles — the
same batched-Farkas demand profile as checking, now on the producer
side too; (2) the tree species is the wrong currency for per-station
information — a `station-product` cover kind (cells = the product
lattice, represented implicitly, walked lazily by the checker with
duals indexed by tuple) keeps coverage checkable-not-asserted while
collapsing the representation, and the joint LP ports to it unchanged;
(3) the dataset parser, checker throughput, and the U4 statistical
lane are unchanged gates.

## Named successor work

1. **`station-product` cover species** in `fluidcheck` + the joint
   producer's Benders decomposition against it — the pair that makes
   rung-3 loads certifiable at all.
2. **Batched Farkas replay** (checker-side; unchanged from
   COVER-CERT.md finding 3, now with a producer-side twin).
3. **set1–set6 per-dataset modelling-ledger passes** (parser
   generalization; the blocker is modelling, not machinery).
4. **Avail-variant tree run** (the machinery is variant-agnostic;
   this rung ran core end-to-end and validated avail at station-top
   only).
5. **U4 e-process lane** (unchanged).
