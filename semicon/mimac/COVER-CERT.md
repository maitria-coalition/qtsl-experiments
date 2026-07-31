# MIMAC rung 2 — aggregated-mode fluid-drift certificates (set7/Fab7)

*Semiconductor-ladder rung 2, 2026-07-20. Builds on: rung 1
(`../minifab/FLUID-CERT.md`, whose rung-2 mechanics paragraph this
stage executes), the MIMAC 7 recovery (`PROVENANCE.md` in this
directory — which corrected the literature's "six datasets" to SEVEN;
set7 is
both the recovered seventh and the smallest, so it is the rung-2
opener), and the testbed format spec (`testbed.tex`, Feigin/Fowler/
Robinson/Leachman 1994). Certificate species + aggregation lemma:
the qtsl repository's `crates/fluidcheck/src/cover.rs` (the module
doc-comment is the format's law). Certificates + mutants + receipts:
`../out/mimac7/`.*

## What is certified, in one paragraph

For the **fluid model of the MIMAC set7 (Fab7) skeleton** — 172
re-entrant route steps over 24 tool groups (38 machines), single
product, per-lot occupancies read directly from the testbed's
Time-per-Lot field — under a named static-priority dispatch policy
(LBFS or FBFS), all residual choices demonic as in rung 1: a linear
Lyapunov function $V(q) = \langle w, q\rangle$ decreases at certified
rate $\ge \varepsilon$ at a.e. time while $q \ne 0$, at release rates
up to $\theta^\* = 13.22\%$ (core) / $11.58\%$ (availability-interval)
of the dataset's nominal release. The structural novelty — rung 2's
actual deliverable — is that the certificate quantifies over sign modes
**by aggregated cover, not enumeration**: the $2^{172} - 1$ mode walk is
replaced by 172 cells whose relaxed polytopes provably over-approximate
every member mode (the aggregation lemma below), checked by the same
independent exact-$\mathbb{Q}$ Farkas replay as rung 1. The certified
load is honestly weak — the aggregation gap at this coarsest cover is
the priced cost, not fine print — and the refinement path to stronger
bounds is part of the format (binary sign-split trees, anytime-refinable
down to the full walk).

## The aggregation lemma (stated exactly)

A **cell** is a disjoint pair $(P, Z)$ of buffer sets ("$P$ positive,
$Z$ empty, the rest unknown"); its relaxed polytope $Q_{P,Z}$ keeps all
canonical inequality rows and only the boundary equalities for $k \in Z$
and the priority equalities for members of $P$.

**Lemma.** If (i) *coverage*: every nonzero sign mode $\sigma$ has some
cell with $P_i \subseteq \sigma$, $\sigma \cap Z_i = \emptyset$, and
(ii) *drift*: each cell carries a Farkas dual with $y_i \ge 0$,
$A^\top y_i + E_i^\top z_i = c(w)$, $b^\top y_i + f_i^\top z_i \le
-\varepsilon - \langle w, \lambda\rangle$, then every admissible fluid
trajectory satisfies $\dot V \le -\varepsilon$ at a.e. $t$ with
$q(t) \ne 0$ — exactly the rung-1 claim.

*Proof sketch* (full proof in the checker doc-comment): mode $\sigma$'s
polytope asserts boundary equalities for **every** empty buffer
$\supseteq Z_i$ and priority equalities for **every** positive buffer
$\supseteq P_i$, so $Q_\sigma \subseteq Q_{P_i,Z_i}$ — dropping
constraints is the sole aggregation move, which makes soundness monotone
by construction; weak duality and the telescoping identity finish, and
empty cells are covered vacuously exactly as in rung 1. The M7 boundary
lemma (a.e. $\dot q_k = 0$ on $\{q_k = 0\}$) is inherited per-buffer,
unchanged: the cell's $Z$-equalities are precisely those the lemma
licenses. $\square$

Coverage must be *checkable, not asserted* (general subcube-cover
completeness is coNP-hard), so the format admits two structural cover
kinds: **station-top** ($K$ cells "top positive buffer of station $s$
is at priority position $p$"; coverage is a two-line checker-side
theorem) and **tree** (binary sign-split trees; leaves partition the
mode cube by construction; fully refined = the v0 walk). Semantic
cross-check: at $K = 3$ the test suite brute-force-enumerates every
vertex of every v0 mode polytope and confirms the cover-certified
$(w, \varepsilon)$ modewise, with no Farkas machinery in the loop.

## Model ledger (decisions priced, rung-1 style)

- **M1′ — occupancy = Time-per-Lot.** Set7's route rows populate *only*
  the Time-per-Lot field (verified against the raw fixed-width bytes:
  load/unload/per-wafer/travel columns blank, batch/setup/rework/scrap
  columns absent — the lines end at the Time-per-Lot column); operators
  and rework are absent from the dataset itself (`.os`/`.rw` empty). So
  the folding decisions that rung 1 had to price (M1–M3, M5) are
  **vacuous for Fab7** — the model gap here is genuinely smaller than
  Minifab's. Multi-sequence/batch semantics return at set1–set6, which
  populate those columns; the parser generalization is a named
  successor.
- **M2′ — deterministic arrivals at exact rate.** Release =
  $\theta \cdot \lambda_{\text{nom}}$, $\lambda_{\text{nom}} =
  59/150000$ lots/min ($= 0.5664$ lots/day $= 13.5936$ wafers/day at 24
  wafers/lot, the `.vr` line exactly; the `.cf`'s "approx. 408 wafer
  starts/month" and "0.0236 lots/hr" both check against it).
- **M3′ — failures, two sound variants.** `core`: no downtime (matches
  the capacity arithmetic). `avail`: service-rate **intervals**
  $\mu_k \in [A_t/m_k,\ 1/m_k]$ with $A_t = \mathrm{MTBF}/(\mathrm{MTBF}
  + \mathrm{MTTR})$ per tool group (set7 models one FAILURE clause per
  group, time-based, first moments only — the dataset's own scope), the
  adversary resolving the rate inside the interval at every instant.
  This is the "capacity intervals from availability moments" row that
  rung 1's mechanics paragraph named: sound at the declared-model level;
  the declared-prior gap is that real downtime is excursions to $0$
  with compensating uptime, whose fluid-averaged reading this interval
  model idealizes — the genuine Foster–Lyapunov row on the semi-Markov
  chain remains the research successor (G4 vocabulary).
- **M4′ — dispatch.** LBFS (later-step-first) and FBFS as rung 1;
  set7's own sample run is FIFO, which static-priority certificates do
  not directly cover (rung-1 M6 discussion applies verbatim). See
  finding 3 for what the coarse cover can and cannot say about policy.

## The analytic producer (no LP at this scale, and why that is honest)

With **remaining-work weights** — $u_k = m_k / c_{s(k)}$ (worst-case
station-minutes per lot at step $k$), $w_k = \sum_{l \ge k} u_l$ along
the route, so $V(q)$ = total remaining station-minutes of work in the
system — every station-top cell admits a closed-form exact dual:
$y[\text{rate-lo}_j] = 1/(c_s \mu^{lo}_j)$ on the cell's priority-row
members, $y[\text{nonneg-}T_j] = u_j$ (plus a $\zeta$-chain term when
$Z$-chains feed $j$) elsewhere, $z_{\text{prio}} = -1/c_s$, giving D3
value exactly $-1 - \sum_{k \in Z} \zeta_k \lambda_k$. The certificate
condition is therefore $\varepsilon \le 1 - \langle w, \lambda \rangle
= 1 - \theta \sum_s \rho_s$: the station-top ceiling is
$\theta^\* = 1/\sum_s \rho_s$, **exactly** $1000000/7563387 = 0.132216$
(core; the decimal-minute process times make $\sum_s \rho_s = 7.563387$
a terminating decimal) and $0.115776$ (avail).

This ceiling is the **cover's**, not the weight choice's: any
certifying $w$ must decrease along the route (else some cell's
adversary pumps a positive-coefficient throughput term unboundedly),
and then the cell "$b$ top-positive, $Z = \emptyset$" has exact LP
value $-u_b \mu^{lo}_b c_{s(b)}$, so the best any $w$ achieves on the
station-top cover is $\min_k u_k \mu^{lo}_k c_{s(k)} \ge \varepsilon +
\langle w, \lambda \rangle$ — maximized precisely by equalizing
$u_k \mu^{lo}_k c_{s(k)}$, which is the remaining-work choice. Beating
$\theta^\*$ requires a **finer cover**, not a better $w$: that is the
tree-refinement axis, and the observable mechanism is the
$\zeta$-slack (a spine-tree cell in the test suite shows D3 value
$-3/2$ where the station-top cell shows $-1$ — the $Z$-chains force
flow-through that buys certified rate).

## Results (all ACCEPT by `fluidcheck`; receipts JSON beside the certs)

| certificate | policy | variant | $\theta$ (of nominal) | $\varepsilon$ (station-min/min) |
|---|---|---|---|---|
| `lbfs_core_th100` | LBFS | core | 1/10 | 0.2436613 |
| `lbfs_core_th125` | LBFS | core | 1/8 | 0.05457662 |
| `lbfs_core_th132` | LBFS | core | 33/250 = 0.132 | 0.00163291 |
| — probe | LBFS | core | 27/200 = 0.135 > $\theta^\*$ | none (correctly) |
| `fbfs_core_th125` | FBFS | core | 1/8 | 0.05457662 |
| `lbfs_avail_th100` | LBFS | avail | 1/10 | 0.13626331 |
| `lbfs_avail_th115` | LBFS | avail | 23/200 = 0.115 | 0.00670281 |
| — probe | LBFS | avail | 3/25 = 0.12 > $\theta^\*$ | none (correctly) |

$\varepsilon$ is emitted floored to $10^{-8}$ granularity (sound: any
$\varepsilon$ below the cell-dual value certifies; the avail-variant
exact maxima have $\sim$100-digit terms from compounded availability
ratios — receipts carry both). Worked reading of the flagship: at
13.2% of nominal release, a backlog of 10 lots at step 1 carries $V
\approx 192{,}298$ station-minutes at drain rate $\ge 0.00163$/min —
the near-ceiling row is deliberately tight; the 1/8 row's
$V(q_0)/\varepsilon$ is the practically-readable one.

Findings worth carrying:

1. **The gap decomposition, honestly.** Bottleneck (PHGCA, the
   3-machine stepper group; $\rho = 0.7382$ core, $0.9014$
   avail-adjusted at nominal) puts model capacity at $\theta_{\text{cap}}
   = 1.355$ (core) / $1.109$ (avail) of nominal. Certified:
   $\theta^\* = 0.1322$ / $0.1158$. Bound-strength ratio
   $\rho^\*_{\text{cert}} / \rho^\*_{\text{cap}} \approx 0.098$ / 0.104
   — an order of magnitude of headroom conceded to the aggregation, vs
   rung 1's 0.99 at full enumeration. That is the real price of $K
   \sim 170$, and closing it (not re-deriving it) is the successor
   work. Consistency triangle: certified-stable $\le 0.116 \cdot$
   nominal $<$ testbed's own DES stably running at nominal (FIFO,
   726.64 hr avg cycle time, `.cf` sample run) $<$ capacity $1.109
   \cdot$ nominal.
2. **The station-top ceiling is $\sum_s \rho_s < 1$** — i.e. the
   coarsest sound aggregation certifies only loads where the *sum* of
   station utilizations is below one (every station must be able to
   drain the whole system's weighted arrivals alone, because a cell
   pins only one station's behaviour and the adversary idles the rest).
   For a 24-station fab this is brutal and *structural*, proved
   optimal-for-this-cover above. Corollary worth knowing: the
   station-top certificate is **policy-uniform** — identical
   $\varepsilon$ for LBFS and FBFS (the cell duals use only the
   top-positive row, which every static priority provides), so
   rung 1's policy-discriminating findings (FBFS edging LBFS near
   capacity) are invisible at this granularity and live strictly up
   the refinement curve.
3. **Checker economics at rung-2 scale** (the demand profile for a
   batched checker lane): 172 cells, 59,340 obligations, 17–21 ms (core) / 26–30 ms
   (avail) single-core BigRational on a GB10 — $\sim 0.3\,\mu$s per
   obligation, $\sim 100$–$170\,\mu$s per cell. Producer end-to-end
   (6 rows + self-replay + byte-stability double-emit + 6 mutants):
   4.8 s in python, measured. Extrapolating to SMT2020 ($K \sim 10^3$–$10^4$
   buffers) at tree-refined cover sizes ($10^3$–$10^5$ cells): checker
   minutes-to-hours single-core — the batched/GPU Farkas lane stops
   being optional exactly where the campaign's scoping notes predicted.
4. **No LP was needed at rung 2** — a genuine surprise against the
   rung-1 mechanics paragraph (and this rung's own working plan), which
   budgeted large-memory simplex time. The analytic dual family
   covers every single-positive cell in closed form; LP enters with
   $|P| \ge 2$ cells (tree refinement), where the joint problem stays
   an LP by the observation in successor item 1.

## Named successor work

1. **The refinement LP producer** (the real rung-2.5 prize —
   *executed: see `JOINT-LP-CERT.md`, this directory; certified
   $\theta = 1964/9903 \approx 0.198$, $1.50\times$ this rung's
   ceiling, on a 244-cell tree*): for a
   *fixed* cover, the condition "$\min_x$ over each cell $\ge
   \varepsilon + \langle w, \lambda\rangle$" is concave in $w$, so by
   LP duality the joint search over $(w, \varepsilon, \text{all cell
   duals})$ is **one sparse LP** — solve float (HiGHS-class), then
   exact per-cell repair via basis re-solve in $\mathbb{Q}$. Together
   with tree refinement guided by which cells' duals are tight, this
   walks the curve from $\theta^\* = 0.13$ toward rung-1-style
   near-capacity certificates, cell count as the budget knob.
2. **set1–set6 parser generalization**: the other six datasets populate
   batch/setup/rework/operator fields (set1: 2 products, 83 tool
   groups, ~450 route steps; multi-product = disjoint route chains,
   which `fluidcheck`'s network schema already carries). Each dataset
   needs its own M-ledger pass; batching-as-rate and setup-adversarial
   intervals port from rung 1 unchanged.
3. **Foster–Lyapunov moment rows** on the discrete/semi-Markov chain
   (downtime beyond availability-intervals; cycle-time moments via
   Little) — the honest successor M3′ names.
4. **U4 e-process lane** (rung 1 successor 5, unchanged): regenerative
   simulation bounds against a DES oracle; the LunarLander e-process
   checker lane (`gymnasium/`) is this repository's precedent, and at rungs 3–4 it
   is the day-one blanket.

## Rung-3 (SMT2020) mechanics, one paragraph

SMT2020 (Kopp–Hassoun–Kalir–Mönch 2020; datasets at the same
p2schedgen page; PySCFabSim as maintained DES oracle) scales to
$\sim$10³ tools and 4 scenarios (HVLM/LVHM ± engineering lots), with
routes an order of magnitude past set7 ($K \sim 10^3$–$10^4$ total
buffers across products). The cover species carries over unchanged —
nothing in `fluid-drift-cover-v1` is set7-specific, and `MAX_K_COVER`
= 4096 admits the low end of that range; raising it is one line — so rung 3 is
gated on exactly three things: (a) the dataset parser (SMT2020 ships
as Excel workbooks with per-product route sheets — a different
extraction, same TOML target), (b) the successor-item-1 LP producer,
because at $\sum_s \rho_s \sim$ dozens the station-top ceiling
($\theta^\* \sim 1$–3% of nominal) stops being worth certifying — tree
refinement plus jointly-optimized $w$ is where nontrivial loads live,
and (c) checker throughput (finding 3's extrapolation): batched Farkas
replay — thousands of structurally-identical column checks per cell
over cells sharing one inequality-row set — is embarrassingly parallel
and is the concrete workload for that batched lane. The U4 statistical lane rides
PySCFabSim from day one, independent of all three gates.
