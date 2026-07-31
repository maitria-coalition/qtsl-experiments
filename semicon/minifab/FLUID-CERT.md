# Mini-Fab rung 1 — the first certified artefact (U5 fluid-drift stability)

*Semiconductor-ladder rung 1, 2026-07-20. Builds on: `KEMPF-SPEC.md`
(parameterization, primary-source witnessed). Certificate species + claim
semantics + canonical row order: the qtsl repository's
`crates/fluidcheck` (the checker's doc-comment is the format's law).
Certificates + mutants + receipts: `../out/minifab/`.*

## What is certified, in one paragraph

For the **fluid model of the Mini-Fab deterministic-core skeleton**
(three stations, five machines, six re-entrant steps; occupancies from
the Kempf tables) under a **named static-priority dispatch policy**
(LBFS or FBFS), with all residual choices resolved **adversarially**
(allocation slack, idling below priority, rate realization inside
declared intervals): a linear Lyapunov function $V(q) = \langle w, q
\rangle$, $w \in \mathbb{Q}_{>0}^6$, decreases at certified rate
$\ge \varepsilon$ at a.e. time while $q \ne 0$ — so any work-in-progress
profile $q_0$ drains within $V(q_0)/\varepsilon$ minutes and the line
stays drained. The certificate is checked by an independent Rust checker
(`fluidcheck`) that **rebuilds every polytope itself** from the declared
network data and replays one exact-$\mathbb{Q}$ Farkas dual per sign
mode; soundness rests on weak LP duality and one measure-theory lemma
(M7 below), nothing else. Claim class: **U5** (long-run stability /
drain-rate), almost-sure at the declared-model level; the model gap to
the discrete line is a declared ledger (M2/M3/M4), not fine print.

## The model, with decisions priced (the rendering's spine)

Constants are `KEMPF-SPEC.md`'s, machine-occupancy convention: per-lot
occupied minutes $m_k$ = load + process + unload (per-batch at
diffusion, decision point 1 of the spec digest):

| step $k$ | station | $m_k$ (min) | fluid rate $\mu_k$ per machine |
|---|---|---|---|
| 1 | diffusion (2) | $285/3 = 95$ | $1/95$ |
| 2 | implantation (2) | $60$ | $1/60$ |
| 3 | lithography (1) | $75$ | $1/75$ |
| 4 | implantation (2) | $80$ | $1/80$ |
| 5 | diffusion (2) | $315/3 = 105$ | $1/105$ |
| 6 | lithography (1) | $30$ | $1/30$ |

Utilization at the Kempf release (84 lots/week, $\lambda = 1/120$
lots/min): lithography $105/120 = 87.5\%$ (bottleneck), diffusion
$200/240 = 83.3\%$, implantation $140/240 = 58.3\%$ — the reference
DES's measured occupancies match all three (sim README). Fluid capacity:
$\lambda^* = 1/105$ (96 lots/week), litho-pinned.

- **M1 — occupancy folding.** Operator handling is folded into machine
  occupancy (the machine is held during load/unload), per the thesis's
  own Appendix-H arithmetic; *operators as a separate contended resource*
  are excluded — the operator-contention layer, and the DES README
  quantifies what the excluded
  layers own (core ceiling 87/wk vs full-model 57.5/wk). Alternative
  priced: modelling operators as a second resource class needs G1
  vocabulary and doubles the allocation dimension; correct successor
  once G1 staging lands, not a competitor to this artefact.
- **M2 — single class per step.** Products share all process times
  (Kempf tables are per-step); test-lot dedication and batch-mix rules
  are granularity-level (F2/F3 of the campaign's model-gap analysis,
  whose finding labels this document cites) and vanish in fluid
  scaling. The mix enters only through total release. Alternative:
  per-product buffers (18 classes) — the format supports it (per-buffer
  routes/arrivals), buys nothing at fluid scale here, costs $2^{18}$
  modes under the v0 mode enumeration (see M8/rung-2).
- **M3 — batching as rate.** A 3-lot furnace batch with per-batch
  handling becomes per-lot rate $3/285$ ($3/315$ high) per machine.
  Sub-batch starvation (fewer than 3 lots of a class present) is a
  granularity effect outside the fluid model; the declared-prior ledger
  carries it. Alternative: explicit batch-formation state is G4-adjacent
  vocabulary (the model-gap analysis's count-guard rows) — again a successor,
  and the DES already witnesses the discrete line's stability at the
  flagship load with true batching in force.
- **M4 — deterministic arrivals at exact rate.** Release is the
  thesis's constant-interval signal-generator idiom; $\lambda$ is exact
  $\mathbb{Q}$ (the type-A interval $3360/17$ is why this model's
  bookkeeping keeps insisting on rationals). Implant failures/PM (G4
  stochastic layer) are **excluded** from the core skeleton — the
  honest successor is capacity-interval rows from availability moments,
  named under rung-2 below, not faked here.
- **M5 — setups, two sound variants.** `core`: zero-setup skeleton
  (matches the capacity arithmetic the DES reproduces).
  `setup-adversarial`: every litho job may pay up to the full
  worst-case 12-min setup, resolved adversarially per instant — encoded
  as service-rate **intervals** $\mu_3 \in [1/87, 1/75]$, $\mu_6 \in
  [1/42, 1/30]$ inside the admissibility polytope, so the certificate
  quantifies over *every* setup sequencing with **no service-rate
  monotonicity assumption** (queueing networks are not monotone in
  rates; intervals sidestep the issue entirely). Per-lot worst-case
  litho occupancy $129$ min $\Rightarrow$ adversarial ceiling
  $10080/129 = 78.14$/wk. Alternative priced: run-length-$r$ policies
  amortize setups to $12/r$ per pass and would certify loads between
  78 and 96 — but need policy-state (current litho class) in the mode
  vocabulary; real successor work, one new mode coordinate.
- **M6 — static-priority dispatch as the named policy.** LBFS
  (later-step-first: $5 \succ 1$, $4 \succ 2$, $6 \succ 3$) and FBFS
  (earlier-step-first), the two classical re-entrant disciplines
  (Kumar–Seidman's destabilization results are exactly about such
  priorities, which is what makes "stable at $\rho$" a falsifiable
  claim). The G3 Dispatch design is the declarative home for
  policy-as-object; this artefact indexes its claim by the policy
  exactly as the model-gap analysis's certificate story prescribes. FIFO is *not* directly certified: its fluid content is
  class-level dispatch (F3), and the setup-adversarial rows cover FIFO's
  setup burden soundly for any within-class order at loads $\le 78$/wk.
- **M7 — the boundary equality (the load-bearing semantics choice).**
  Empty buffers carry $\dot q_k = 0$, not merely $\dot q_k \ge 0$: for
  an absolutely continuous $q_k \ge 0$, a.e. point of $\{t : q_k(t) =
  0\}$ has $\dot q_k(t) = 0$ (a level-set Lebesgue-point fact; the
  standard fluid-model boundary property). Measured with the robust LP:
  with inequality-only semantics **no linear $V$ exists at any tested
  load** (the adversary piles flow into empty buffers whose station
  idles); with the equality, linear $V$ certifies to $95/96$ of
  capacity. The equality also makes many sign modes *vacuous*
  (instantaneous): at the flagship config, 41 of 63 mode polytopes are
  empty — e.g. "only $q_5 > 0$" forces full diffusion output through an
  empty line that lithography cannot pass. Vacuous modes need no case
  split: the same Farkas row shape certifies them (dual unbounded ⇒ a
  dual point below any target exists), which is why the checker has no
  emptiness branch at all.
- **M8 — mode-wise demonic quantification.** Everything the policy does
  not pin — allocation slack on empty classes, idling below the top
  positive priority, interval rates — is inside the adversary. Measured
  price of this generosity: **zero at the tested resolution** (certified
  through 95/wk of a 96/wk capacity; probes at 96 and 79 correctly find
  no certificate). $2^K - 1$ mode enumeration is the v0 scaling limit
  (checker refuses $K > 16$) — rung 2's structural work, below.

## The certified objective (the U-taxonomy row, made precise)

U5 asks for long-run throughput/cycle-time claims. This artefact
certifies the **stability half** with a quantitative rate: every fluid
trajectory drains at $\dot V \le -\varepsilon$, so (i) the line is
stable at the certified release (throughput = release), and (ii) any
WIP excursion $q_0$ clears within $V(q_0)/\varepsilon$ minutes. What it
does **not** yet certify: stationary WIP / cycle-time moments for the
*discrete* line (Little's-law conversion needs a Foster–Lyapunov row on
the discrete count chain — the granularity constants; named successor).
The certified-load axis is the campaign's bound-strength axis:
$\rho^*_{\text{cert}} / \rho^*_{\text{capacity}}$ below.

## Results (all ACCEPT by `fluidcheck`, exact $\varepsilon$; receipts JSON beside the certs)

| certificate | policy | variant | release (lots/wk) | $\rho_{\text{litho}}$ | $\varepsilon$ (V-units/min) |
|---|---|---|---|---|---|
| `lbfs_core_84wk` | LBFS | core | 84 (Kempf) | 0.875 | $1/3600$ |
| `lbfs_core_90wk` | LBFS | core | 90 | 0.9375 | $1/7200$ |
| `lbfs_core_94wk` | LBFS | core | 94 | 0.979 | $1/21600$ |
| `lbfs_core_95wk` | LBFS | core | 95 | 0.990 | $1/43200$ |
| — probe | LBFS | core | 96 = capacity | 1.0 | none (correctly) |
| `fbfs_core_84wk` | FBFS | core | 84 | 0.875 | $\approx 2.798 \times 10^{-4}$ |
| `fbfs_core_95wk` | FBFS | core | 95 | 0.990 | $1/38880$ |
| `lbfs_setupadv_72wk` | LBFS | setup-adv | 72 | 0.921* | $11/74760$ |
| `lbfs_setupadv_76wk` | LBFS | setup-adv | 76 | 0.973* | $23/448560$ |
| `lbfs_setupadv_78wk` | LBFS | setup-adv | 78 | 0.998* | $1/299040$ |
| — probe | LBFS | setup-adv | 79 > 78.14 ceiling | >1* | none (correctly) |
| `fbfs_setupadv_78wk` | FBFS | setup-adv | 78 | 0.998* | $1/287280$ |

(*) setup-adversarial $\rho$ against worst-case-inline occupancy
($129$ min/lot at litho). $V$-units: $\sum_k w_k = 1$, so $V$ is a
weighted lot count; at the flagship, $w = (7,7,7,3.5,3.5,2)/30$ —
remaining-lithography-work-flavoured weights the LP found on its own.
Worked reading: a 10-lot step-1 backlog at 84/wk has $V = 7/3$,
certified clear within $8400$ min ($< 6$ days), against the adversary.

Findings worth carrying:

1. **Certified-$\rho^*$ meets capacity at 1-lot/wk resolution.** The
   demonic relaxation (M8) is measurably free here: certificates exist
   at 95/wk (99.0% of fluid capacity) and 78/wk (99.8% of the
   adversarial ceiling), and the immediately-next grid points have no
   certificate *because the load is genuinely at/over capacity*.
   *(Rung 2.5 sharpened this to sub-tenth-lot resolution: the release
   ceiling bisects to $(95.999939, 96)$, rows at 95.5 and 95.9 lots/wk
   — 99.90% of fluid capacity — are now certified as sign-split tree
   covers, and 96 still correctly fails; see
   `../mimac/JOINT-LP-CERT.md`.)*
2. **FBFS edges LBFS on drain rate near capacity** ($1/38880$ vs
   $1/43200$ at 95/wk; $1/287280$ vs $1/299040$ at setup-adv 78) —
   both stable wherever tested, consistent with the Dai–Weiss picture
   for re-entrant lines; the exact-$\mathbb{Q}$ $\varepsilon$ gap is
   the kind of policy-comparison content the U5 rung was chosen for.
3. **The consistency triangle with the DES** (different policy,
   discrete line, true batching/setups, FIFO): certified
   setup-adversarial stability $\le 78$/wk $\;<\;$ DES-measured FIFO
   stability at 84/wk $\;<\;$ certified zero-setup capacity 96/wk with
   DES FIFO ceiling 87/wk between — the certificate's conservatism
   against worst-case setups (24 min/lot) vs the DES's measured FIFO
   burden (9.4 min/lot) is exactly the gap the run-length successor
   policy would close.
4. **Checker economics**: 819 obligations/cert, verify 1.5–3.0 ms
   single-core (BigRational lane, GB10) — $\sim 2$–$4\,\mu$s per
   obligation; producer end-to-end (float propose + 63 exact duals +
   self-replay + emit) 0.2 s/cert with the in-file Bland's-rule exact
   simplex (sympy's simplex cycled on degenerate modes and was
   replaced; solver bugs are completeness-side only — the checker is
   the gate).

## Sealed containers (`.glc`) — the reproduction route

The ten fluid-drift certificates above are additionally committed as
**sealed geolog commit containers** — the canonical binary form the
toolkit's checkers consume. Each container holds one signed commit
whose body carries, per sign mode, the full obligation rows (mode
polytope, combination coefficients, Farkas conclusion, drift-margin
ground fact, syzygy assembly) **plus four side-condition claim rows**
(`farkasCombinationValid`, `constExactEvalTrue`,
`syzygyCombinationValid`, `syzygyConclusionFormRel`). Emission is
deterministic (canonical body transform, fixed timestamp, RFC 8032
signing): re-producing a certificate reproduces its content address,
which each lane's `GLC-MANIFEST.json` pins alongside sha256s and claim
expectations.

Two container lanes are committed, differing only in the vocabulary
their prerequisite tables pin — the mathematical content of the rows
is identical:

- **Baked lane** (`../out/minifab/glc-baked/`) — the reproduction
  route. The prerequisite table pins the baked theory-pair closure
  revision (the qtsl repository's `theories/pins/baked_v1_pins.txt`
  names it, with the minting recipe); the checker derives the
  vocabulary index from the pinned chain's own bytes and discharges
  every claim through the exact-rational ground provider. The claim
  roster is derived from the revision, so the count gate is
  per-relation (`--expect-claims`, from the manifest) rather than a
  bare mode count.
- **Legacy lane** (`../out/minifab/glc/`) — the original containers
  against the synthetic v0 reproduction vocabulary, kept checkable
  forever (a committed lane never dies); count gate `--expect-modes`.

The re-check driver is `semicon/glc-recheck.ts` (make target
`semicon-glc`), instrument `glc-check` (built from the pinned qtsl
checkout's `qtsl-geolog-provider` crate): container + envelope
integrity (signature, body-hash binding, canonical body decode),
reference discipline, then every claim row re-derived by the
exact-rational ground provider — the same plugin seam the geolog
engine drives. The count gate makes whole-mode deletion refusable; the
committed mutant battery (nine mutants per lane) witnesses every
layer: four content mutants re-sealed with valid signatures whose
mathematics the provider evaluates FALSE (ε inflated, dual negated,
identity broken, mode dropped), five byte mutants the envelope
machinery refuses (body byte, signature byte, truncation, magic,
version). The signature is provenance framing, never the verification
boundary — the sealing key is public by design; the mathematics
carries the verdict.

Status of the lanes: the baked sealed-container lane is the
reproduction route for these certificates; the direct `fluidcheck`
replay above remains the species' reference description (its
doc-comment is the format's law for the *banked* `.cbor` bytes, which
stay committed as historical artifacts) and the producer-side
well-formedness gate at emission time. Theory-conformance chase
checking of the constructor rows against staged theory commits, and
the certified-checker timing column for these same bytes, are the
named successors of this lane.

## Rung-2 (MIMAC) mechanics

Data acquisition: the six SEMATECH MIMAC testbed datasets download
today from Mönch's p2schedgen page (per the campaign's scoping notes —
no visible license gate; fetch the landing page + archive,
then a parser from the testbed format into the `fluidcheck` network
schema, which already carries what MIMAC needs structurally: per-buffer
successor routes (multi-product = disjoint route chains over hundreds
of buffers), per-buffer exogenous arrivals, per-station capacities and
rate intervals). What changes structurally, in order of bite: (i) the
$2^K - 1$ **mode enumeration dies** at $K \sim 300$ — the successor
species certifies drift over an *aggregated* mode cover (station-level
busy/empty patterns, or cone covers from the active-priority structure)
with a completeness side-condition replacing the exhaustive mask walk;
this is the certificate-species successor, and its demand profile (batched
Farkas replay over thousands of cells) is precisely the batched-lane
customer those scoping notes predicted; (ii) **setup/batch families** at
scale need M5's run-length policy coordinate rather than pure
adversarial inflation (at MIMAC loads the 12-min-per-job inflation
would concede too much capacity); (iii) **failures/PM** (the datasets'
downtime distributions) enter as station capacity *intervals* from
availability moments for a first sound row, with genuine Foster–Lyapunov
moment rows on the stochastic chain as the research follow-on — the
G4 reset-kernel vocabulary is the declarative home. A U4 statistical
lane (regenerative e-process bounds against a PySCFabSim-class oracle)
remains the day-one blanket for rungs 3–4 per the scoping notes and needs
none of the above.

## Named successor work

1. **Run-length setup policies** (M5 alternative): one policy-state
   coordinate; closes most of the 78→96 setup gap with certificates.
2. **Discrete-line Foster–Lyapunov / Little's-law row**: granularity
   constants for the count chain; turns drain-rate into certified WIP /
   cycle-time moments (the full U5 promise).
3. **LBFS/FBFS dispatch in the reference DES** (one flag in
   `sim/`): same-policy artefact-pair for these certificates —
   the current FIFO run is a consistency triangle, not a pair.
4. **MIMAC opener** per the rung-2 paragraph: dataset fetch + parser +
   the aggregated-mode certificate species design.
5. **U4 e-process lane for the ladder** (the scoping notes' day-one
   statistical blanket) — independent of all of the above.
