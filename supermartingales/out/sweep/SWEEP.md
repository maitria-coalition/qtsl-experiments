# Monday-integration sweep — BL-certified bounds vs arXiv 2301.06136, Tables 1+2

Master table for the sweep goal (set 2026-07-18): a certified
house bound for every row of the results table of Abate–Edwards–Giacobbe–
Punchihewa–Roy, *Quantitative Verification with Neural Networks* (LMCS 22(2),
2026; arXiv 2301.06136 v7). Their columns are transcribed from the fetched TeX
source (`../../manifest.toml`, 0 cell errors vs mechanical extraction); house
columns come from the four wave-1 legs (`../sec3/`, `../sec2/`, `../sec1/`,
and the persist_2d leg — receipts index below). **All 17 published
rows are swept, unconditionally: every row — including both persist_2d twin
reconstructions (noise class unpinned by the paper) — carries a certified
exact-$\mathbb{Q}$ house bound that beats their best published number.**
Campaign complete 2026-07-18 evening.

> **2026-07-20 re-check**: all bounds re-verified through the current
> checker stack (57,138 fastcheck checks + 5,003 reference checks, zero
> refusals; Mehrotra-fix relevance ruled out by grep witness), with a
> same-box old-vs-new checker wall-clock A/B — see
> `UPDATE-2026-07-20.md` beside this file.

> **Format note (historical vocabulary)**: during the campaign the
> certificates were emitted and checked as `bernfmt` bytes — a binary
> certificate format since retired — with `fastcheck` as the native
> checker tier ("fastcheck-Trusted") beside the reference checker
> ("reference-Checked"). Mentions of both below are records of what ran
> at the time. The certificates ship in this repository as geolog
> listing rows (`*.rows.txt`; per-file pins in `../../MANIFEST.jsonl`),
> re-verified by `make supermartingales` through the pinned qtsl
> reference checker.

## Methods and conventions (the cross-wave reconciliation)

The three wave-1 legs used slightly different conventions; this table
reconciles them explicitly:

- **"their best neural"** $= \min(\mathrm{PA}, \mathrm{PW})$ of their two
  published neural columns, route tagged; **"their s"** = learn + verify time
  (their Table 2, A40 GPU + Z3) *of the route achieving that best bound* —
  the convention wave A already used, applied uniformly here.
- **display**: house bounds (upper bounds) round UP at the 7th decimal;
  truths round to NEAREST at the 7th decimal; sub-$10^{-9}$ values in
  scientific notation, 4 significant digits. (Wave A's fragment ceiled its
  truth column too; two of its truth displays were one ulp high — see the
  discrepancy log.)
- **slack conventions** (all three produce sound exact-$\mathbb{Q}$ upper
  bounds): wave A (§3 block) additive $\delta = 10^{-6}$ (bound $=
  p_{\mathrm{true}} + \delta$ exactly); wave B (§2 block) multiplicative
  (bound $\le p_{\mathrm{true}}(1 + 2^{-20})$); wave C (§1 block)
  $\varepsilon\,T$-inflation, $\varepsilon = 10^{-12}$ (bound $=
  p_{\mathrm{true}} + \varepsilon\,T(s_0)$). Rows certified by two waves
  independently carry the tighter bound (always wave B's).
- **"house s"** = synthesis + both checker tiers, workstation CPU, per the leg's
  receipts. Wave B's produce time includes its reference-check pass (not
  broken out in its receipts); wave C recorded no wall times (its rows are
  the same sub-second class as wave A's — but no receipt, so no number).
- **persist_2d conventions** (the fourth leg): state is continuous 2D, so
  the certificates are piecewise-bilinear lattice interpolants (uniform
  twin: pitch $1/5$, $\varepsilon = 1/1024$; gaussian twin: pitch $1/10$,
  $\varepsilon = 1/4096$, drift rows as *widened claims* absorbing
  exact-rational erf/exp series enclosures to $2^{-224}$); the truth
  column is Monte Carlo only (12e6 chains, 3 seeds, exact horizon
  bracketing — no exact truth exists for continuous state); the reference
  checker ran on 250-row random samples while fastcheck checked all
  emitted files (4800 uniform, 19200 gaussian) — the receipts column marks
  the sample. The report's own "1.26×"/"1.08×" figures are bound/MC-truth
  (lattice-pitch conservatism); this table's ratio column stays
  their-best/house-bound as everywhere else.
- **repulse100_2d timing footnote** (reviewer question): the 222.1 s
  house time is dominated by the exact-*truth* VI computation — the
  certification+check+emit slice is 2.10 s (`ms_certify_check_emit` in
  wave B's receipts.json is the witness); exact-$\mathbb{Q}$ denominators
  reach $\sim 10^{46}$ on the 2D product space. A bound-only dyadic-VI
  route (skip the exact truth column) would collapse the row to seconds.
- **receipts column** = reference-checker accepts / claims $\cdot$ fastcheck
  accepts / claims, on emitted bernfmt bytes. Certification tier for every
  certified row: fastcheck-Trusted + reference-Checked (exact $\mathbb{Q}$);
  Lean glue is wave 2, pending. Mutant batteries: wave A 4 mutants/row, wave
  B 7 global, wave C 3/row — all refusals refused (receipts.json per leg).
- **comparability caveats** (binding on every comparison): their
  neural bounds are *averages over successful stochastic runs* (success
  ratios 0.8–1.0 of 10); house numbers are deterministic — 25/25
  certificate runs certified across the four legs (22 distinct rows
  including alternates; 3 rows dual-certified by waves A and B). Their Z3 check quantifies over $\mathbb{R}^n$
  under manual invariants (shown non-inductive at guard exits — findings
  below); house rows check over the invariant box with invariance rows
  discharged per branch. Their timings are A40 GPU + Z3; house timings are
  workstation CPU — report both, claim only the qualitative gap.
- **provenance labels**: `listed` = verbatim in-paper listing; `+fingerprint`
  = reconstruction reproducing their exact Farkas optimum as an LP
  fingerprint; `per-description` = reconstruction validated only by negative
  fingerprint + neural-bound envelope (no public artefact exists to pin
  against — wave C's procurement leg confirmed the CAV'25 successor artefact
  contains zero CONCUR'23 benchmark names).

## Master table

Sections are the paper's own grouping: §1 = no linear supermartingale exists
(their "only neural" showcase), §2 = Farkas beaten by neural, §3 = neural
comparable to Farkas. Indented rows are house-frozen alternate readings of
ambiguous definitions (each a finding, not a redundancy).

| benchmark | their Farkas | their best neural | house certified bound | exact truth | house vs their best | receipts (ref · fc) | their s | house s | provenance |
|---|---|---|---|---|---|---|---|---|---|
| **§1** | | | | | | | | | |
| persist_2d (uniform twin) | — | 0.1026 (PA) | **0.1000992** | — (MC 0.07940(8)) | 1.03× (beats both PA and PW) | 250/250 sample · 4800/4800 | 254.5 | n/r | per-description (twin) |
| ⌙ persist_2d (gaussian twin) | — | 0.1026 (PA) | **0.0975251** (beats both PA and PW) | — (MC 0.09025(8)) | 1.05× | 250/250 sample · 19200/19200 | 254.5 | n/r | per-description (twin, grammar-favoured) |
| faulty_marbles | — | 0.0649 (PW) | **0.0623604** | 0.0623603 | 1.04× | 23/23 · 23/23 | 44.5 | n/r | per-description |
| faulty_unreliable | — | 0.0536 (PW) | **0.0516561** | 0.0516560 | 1.04× | 29/29 · 29/29 | 52.3 | n/r | per-description |
| faulty_regions | — | 0.0411 (PW) | **0.0406530** | 0.0406530 | 1.01× | 29/29 · 29/29 | 49.9 | n/r | per-description |
| **§2** | | | | | | | | | |
| cliff_crossing | 0.4546 † | 0.0553 (PA) | **$8.604\times10^{-10}$** | $8.604\times10^{-10}$ | $6.4\times10^{7}$× (7.8 orders) | 1105/1105 · 1105/1105 | 153.6 | 1.27 | listed |
| repulse100 | 0.0991 † | 0.0268 (PW) | **$9.556\times10^{-20}$** | $9.556\times10^{-20}$ | $2.8\times10^{17}$× (17.4 orders) | 207/207 · 207/207 | 10.2 | 0.14 | listed |
| repulse100_uniform | 0.0991 ‡ | 0.0344 (PA; PW failed) | **$6.157\times10^{-61}$** ‡ | unknown (MC 0/2e6) ‡ | $5.6\times10^{58}$× (58.7 orders) | 205/205 · 205/205 | 35.5 | 4.48 | per-description |
| ⌙ repulse100_uniform__discrete | 0.0991 | 0.0344 (PA) | **$1.471\times10^{-35}$** | $1.470\times10^{-35}$ | $2.3\times10^{33}$× (33.4 orders) | 207/207 · 207/207 | 35.5 | 0.13 | alternate reading |
| repulse100_2d | 0.0991 | 0.0541 (PW) | **$9.556\times10^{-20}$** | $= $ 1D truth exactly | $5.7\times10^{17}$× (17.8 orders) | 2267/2267 · 2267/2267 | 63.5 | 222.1 | +fingerprint |
| faulty_concave | 0.1819 | 0.1356 (PW) | **0.1183888** | 0.1183887 | 1.15× | 38/38 · 38/38 (+22/22 wave A) | 21.3 | 0.05 | listed |
| faulty_varying | 0.1819 | 0.0864 (PA) | **0.0694711** | 0.0694710 | 1.24× | 38/38 · 38/38 (+22/22 wave A) | 26.8 | 0.04 | +fingerprint |
| ⌙ faulty_varying__last_step | 0.1819 | 0.0864 (PA) | **0.0353604** | 0.0353604 | 2.44× | 38/38 · 38/38 (+22/22 wave A) | 26.8 | 0.04 | alternate reading |
| **§3** | | | | | | | | | |
| fixed_loop | 0.0091 | 0.0094 (PW) | **0.0089651** | 0.0089641 | 1.05× | 22/22 · 22/22 | 5.8 | 0.05 | +fingerprint |
| faulty_loop | 0.0181 | 0.0184 (PW) | **0.0178400** | 0.0178390 | 1.03× | 22/22 · 22/22 | 6.4 | 0.03 | listed+adjudicated ($i_0{=}1$) |
| ⌙ faulty_loop__as_listed | 0.0181 ✗ | 0.0184 ✗ | **0.0198024** | 0.0198013 | truth exceeds all 3 published | 22/22 · 22/22 | 6.4 | 0.02 | listed ($i_0{=}0$) |
| faulty_uniform | 0.0181 ✗ | 0.0221 (PW) | **[0.0184458, 0.0185556]** (two-sided) | — (MC 0.018582(96)) | 1.19× | 30/30 · 30/30 | 8.1 | 0.41 | +fingerprint |
| faulty_rare | 0.0019 | 0.0022 (PW) | **0.0017994** | 0.0017984 | 1.22× | 22/22 · 22/22 | 6.7 | 0.04 | +fingerprint |
| faulty_easy1 | 0.0801 | 0.0865 (PW) | **0.0768847** | 0.0768837 | 1.13× | 84/84 · 84/84 | 12.5 | 0.12 | +fingerprint |
| faulty_ndecr | 0.0561 | 0.0630 (PW) | **0.0544619** | 0.0544609 | 1.16× | 60/60 · 60/60 | 10.0 | 0.08 | +fingerprint |
| faulty_walk | 0.0121 | 0.0166 (PW) | **0.0119293** | 0.0119283 | 1.39× | 16/16 · 16/16 | 10.3 | 0.02 | +fingerprint |
| ⌙ faulty_walk__det12 | 0.0121 | 0.0166 (PW) | **0.0119353** | 0.0119342 | 1.39× | 28/28 · 28/28 | 10.3 | 0.04 | alternate reading |

**Bold claims and their witnesses:**

- **House beats their best published bound on all 17 published rows — on
  every frozen reconstruction, both persist_2d twins included** — witness:
  every table row's house-bound column is below its their-best column
  (including the faulty_uniform bracket's upper end 0.0185556 < 0.0221,
  and both persist_2d twins: 0.1000992 and 0.0975251, each < 0.1026 and
  0.1175); 25/25 certificate runs certified (12 wave A + 8 wave B + 3
  wave C + 2 persist_2d, counting alternates and the uniform row's
  two-sided pair as one row each). On the gaussian twin the house bound is
  also tighter *relative to MC truth* than their PA bound (1.081× vs
  1.137×). The margin is architecturally two regimes, not one story: near
  ×1 on counter rows, 7.8–58.7 orders of magnitude on walk rows (the
  tightness inversion, below).
- **Two structural findings from the gaussian certification** (the
  persist_2d leg's report, §gaussian — receipts index below): (1) **pitch $1/10$ was
  forced, not chosen** — at the uniform twin's pitch $1/5$ the lattice
  VI's own discretization already exceeds the envelope ($\hat h(init) =
  0.1023$ vs their 0.1026), so no certificate at that pitch could beat
  their number; (2) **the feared truncation+tail-mass discipline is
  satisfied vacuously** — writing $V \equiv 1$ on the hazard makes
  $\mathbb{E}[V(z+w)] = 1 + \kappa^2\sum_{\mathrm{cells}}\int (V-1)e\,e'$
  exactly, so the Gaussian mass outside the box is pre-paid inside the
  leading 1 and kernel truncation can only *overstate* the drift
  expectation — fail-safe by construction, witnessed by the
  M4_tail_laundering mutants (dropped-tail and renormalized variants both
  refused).
- **The impossibility theorem (persist_2d)** — scope stated exactly: for
  the frozen persist_2d twin dynamics, **no global polynomial $V$ of total
  degree $\le 4$ satisfies the strict-margin drift condition
  $\mathbb{E}[V \circ f] - V \le -\eta$ (any $\eta > 0$) on the active
  annulus** — witnessed by an 18-point exact rational moment-stationary
  measure (Farkas dual), certified by exact phase-1 simplex + exact
  verification, symmetrized over $s \mapsto -s$ to cover all polynomials
  of that degree (the leg's `impossibility.json`; float-LP
  infeasibility empirics extend through degree 14, degrees 6/8 priced in
  the same ledger as the named exact-certification successors). The claim
  class is bounded-degree *global-polynomial* certificates of this drift
  condition on this reconstruction — piecewise routes are untouched (ours
  is one). Consequence: the paper's 2-layer (3,1) ReLU architecture on
  this row is a *necessity* against that class, not a convenience — the
  upgrade of wave C's faulty_regions finding (slow polynomial convergence,
  corroboration) to an exclusion. Precedent note: exact-certificate-plus-
  independent-checker for polynomial-system Lyapunov questions has a
  finite-dimensional same-shape precedent (Devadze–Magron–Streif, JAR
  2025, per the novelty adjudication); the pattern is not claimed as
  novel — the per-instance certified *nonexistence* application here is
  simply this sweep's use of it.
- **† sound-linear garnish (their own listings, not reconstructions)**: on
  both §2 headliners the published Farkas number is below the best *sound*
  linear bound. Re-solving their own linear-template LP with the guard-exit
  nonnegativity holes closed: repulse100 sound-linear optimum $12/103
  \approx 0.1165$ (certified optimal) vs their published 0.0991;
  cliff_crossing $17/37 \approx 0.4595$ vs their published 0.4546. Both
  rows' dynamics are verbatim in-paper listings, so this finding is
  unconditional on reconstruction: their Farkas baseline's manual-invariant
  convention (V unconstrained at guard-exit states) is non-inductive over
  the reachable space — their published cliff certificate surface goes
  negative at a reachable exit state (wave-0 adjudication).
- **✗ faulty_uniform refutation (conditional on the frozen reconstruction)**:
  the house-certified *sub*martingale lower bound 0.0184458 exceeds their
  published Farkas value 0.0181 by a certified $3.46\times10^{-4}$ (and
  exceeds the exact LP optimum $2/111$ it ceils by $4.28\times10^{-4}$).
  Conditionality stated plainly: the row is a reconstruction — continuous
  advance $U[0,1]$, frozen at wave 0, fingerprint-matched to their Farkas
  LP — so the certified statement is "for this chain, 0.0181 is not a sound
  bound"; the mechanism (linear surface negative on the reachable overshoot
  strip $(10,11]$) is the same exit-hole non-inductiveness as †, which is
  witnessed on their verbatim listings. Independent MC witness: pooled
  two-seed $\hat p = 0.01851 \pm 6.7\times10^{-5}$, $7.3\sigma$ above
  $2/111$.
- **✗ faulty_loop as-listed**: the in-paper listing says $i_0 = 0$; the
  exact truth of that chain, 0.0198013, exceeds all three of their
  published numbers for the row (PA 0.0195, PW 0.0184, Farkas 0.0181). The
  benchmarked instance evidently started at $i_0 = 1$ (fingerprint LP
  adjudication, wave 0); the primary row carries that reading, certified
  bound 0.0178400.
- **repulse100_uniform best-known bound** — the certified
  $6.157\times10^{-61}$ carries its two caveats verbatim from the wave-B
  README: (1) the house-sound target $A = [100,101]$ (closure of the
  reachable violation set $\{x > 100\}$); their Farkas column reused the
  integer row's $A = \{x \ge 101\}$ — our bound is for a superset event,
  sound and marginally conservative; (2) **no truth column exists** for this
  row (MC 0/2e6; the certified bound is an upper bound with no known
  matching lower bound). This is also the row their program-aware synthesis
  failed on entirely (indicator post-expectations, non-smooth for their
  optimizer); house exact moment integration has no such obstacle.
- **The representation finding (wave C, now completed by persist_2d)**: on
  faulty_regions a single global polynomial converges slowly (degree
  ladder 6/8/10 → 0.0563/0.0508/0.0498 vs their 0.0411) — corroboration;
  on persist_2d the impossibility theorem above makes the same story an
  exclusion. In both cases piecewise representation is what the row
  rewards (theirs via ReLU layers, ours via cells), and the house
  piecewise route wins both rows outright, exact and certified
  (0.0406530 < 0.0411; 0.1000992 < 0.1026).

## Consistency-pass discrepancy log

Findings from reconciling the three legs (each verified at reconciliation
time against the receipts JSONs):

1. **Cross-wave replication, exact**: faulty_concave, faulty_varying,
   faulty_varying__last_step were certified independently by waves A and B
   under different slack conventions; their exact truth *fractions* are
   equal (`Fraction` equality, not display equality), and both bounds are
   sound; the table carries the tighter (B's). repulse100_2d's truth equals
   repulse100's exactly, as the wave-0 spectator-coordinate argument
   predicted.
2. **Wave A truth-display ceiling**: wave A's fragment applied its
   upper-bound ceiling display to the *truth* column; two rows print one
   ulp high (as_listed: 0.0198014 → correct nearest is 0.0198013; det12:
   0.0119343 → 0.0119342). Exact fractions were always right; this table
   uses nearest-7dp for truths.
3. **Refutation margin reference**: wave A's README quotes the
   faulty_uniform margin as $4.28\times10^{-4}$ — that is the margin over
   the exact LP optimum $2/111$; over their *published* (4-dp-ceiled)
   0.0181 the certified margin is $3.46\times10^{-4}$. Both positive; the
   refutation stands under either reference; this table states both.
4. **MC uncertainty bookkeeping**: the wave-0 status ledger's
   "0.018582(10)" was a typo
   for "(96)" (per-seed $n = 2\times10^6$, $\sigma \approx 9.5\times
   10^{-5}$); wave A's "±6.7e-5" is the pooled two-seed figure ($n =
   4\times10^6$). Consistent; the ledger was corrected at reconciliation.
5. **Stale drift guess superseded**: the wave-0 status ledger carried
   "truth ~1e-39 by
   drift" for repulse100_uniform; the certified upper bound
   $6.157\times10^{-61}$ places the truth at least 21 orders below that
   guess. Corrected at reconciliation.
6. **"Their s" convention**: wave A's fragment reported their time for the
   best-bound route only; wave B's receipts carried all four timing cells.
   This table applies wave A's convention uniformly (values recomputed from
   the manifest for §1/§2 rows).
7. **persist_2d leg re-verified at fold-in** (this rev): the exact bound
   fraction $58920851522213/588624739134464$ reproduces the reported float
   0.1000992; fastcheck.json counts 4800/4800 with 0 non-accepts; drift
   receipts 4800 certified / 0 refused, reference sample 250/250; mutants
   refused. One convention divergence reconciled in the methods note: the
   leg's "1.26×" is bound-over-MC-truth, not this table's ratio column.
8. **gaussian twin re-verified at the final flip** (this rev): exact bound
   $6701869649/68719476736$ reproduces float 0.09752504 and beats both
   published numbers; fastcheck 19200/19200, 0 non-accepts; drift 19200
   certified / 0 refused, reference sample 250/250; mutants 4/4 refused
   including both tail-laundering variants. Display-direction catch: the
   leg's completion note "0.0975250" rounds the bound *down* — an upper bound
   displays upward; this table ceils to 0.0975251.

## The twin verdict (campaign complete; remaining legs all optional)

**persist_2d noise class remains honestly unpinned**: the paper's grammar
names Bernoulli and Gaussian (gaussian grammar-favoured); the TACAS'16
predecessor ships both U and N variants of the same controller family; no
public artefact adjudicates. The row therefore ships as **twins over one
shared skeleton** (`../../specs/persist_2d_{uniform,gaussian}.toml`, each
justified line-by-line from the lineage papers), and **both twins are now
certified** — whichever noise class their unpublished instance used, the
house bound for it beats their published numbers. Both twins pass the
envelope check against their PA 0.1026 at ~290 and ~150 standard errors of
margin respectively; the calibration sweep's *disqualified neighbours* are
kept as receipts (the leg's `mc_sweep.json`: $u = 5/2$ gives 0.1145 >
envelope; $\sigma^2 = 9/5$ gives 0.1117; $\sigma^2 = 7/4$ sits on the
envelope within 2 SE) — the calibration freedom was exercised openly, not
laundered. The final residue (gaussian erf-enclosure machinery)
closed in about one working session rather than the full day budgeted
for it: widened claims
absorbing exact-rational series enclosures (the leg's report).
Remaining legs, all optional tightening/deepening: pitch-$1/10$ uniform
rerun (one command, expected bound ~0.086–0.09), exact-simplex
impossibility at degrees 6/8, wave-2 Lean glue.

## Receipts index

Shipped in this repository:

- Wave A (§3 + twins): `../sec3/SWEEP-ROWS.md`, `../sec3/receipts.json`
  (12 rows, 4-mutant battery each; at authoring time the byte-format
  certificates were double-checked on-wire — the listing rows ship under
  `../sec3/certs/`, the mutant battery under `../sec3/mutants/`).
- Wave B (§2): `../sec2/RECEIPTS.md`, `../sec2/receipts.json`,
  `../sec2/sweep_rows.json`, certificate rows `../sec2/rows-certs.tar.zst`
  (4105/4105 claims accepted on both tiers incl. per-cell obligations;
  7-mutant battery; sound-linear column).
- Wave C (§1): `../sec1/SEC1-REPORT.md`, `../sec1/<row>/receipts{,-poly}.json`
  (92 byte-format files at authoring time, 92/92 native-checker accepts;
  dual-route: VI-export primary + decorrelated global-poly LP; 3-mutant
  battery per row; listing rows under `../sec1/<row>/rows{,-poly}/`).
- Wave 0 (specs, truths, adjudication): `../scorecard.md`,
  `../value_functions/*.json`, `../../specs/*.toml` (frozen benchmark
  specs, adjudication notes in their headers).
- Their numbers: `../../manifest.toml` (transcription provenance in header).

Still in the authoring workshop (not yet migrated into this repository;
the numbers they witness are quoted above, and the chapter's table
generator reads the persist_2d bound fraction from this file):

- the persist_2d leg's per-file receipts: `PERSIST2D-REPORT.md`,
  `uniform/{receipts,fastcheck}.json` (4800 files),
  `gaussian/{receipts,fastcheck}.json` (19200 files; widened-claim
  erf-enclosure route), `impossibility*.json` (exact dual witnesses +
  degree ledger), `{mc_final,mc_sweep}.json` — the twin specs themselves
  ship (`../../specs/persist_2d_{uniform,gaussian}.toml`);
- the wave-0 status ledger and the faulty_loop adjudication note.
