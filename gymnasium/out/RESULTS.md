# Certified-Gymnasium — results (waves 1–3)

*Run record, 2026-07-20. Objective: certified utility lower bounds for
Gymnasium environments, with "both the producer strategies and the
sound-checker wall-clock time pushed to the limits."*

## The claim class

Every certificate here asserts, for its shipped (time-varying) policy $\pi$:

$$P_\pi\big[\exists\, t \le T : X_t \in \mathrm{Goal} \,\big|\, X_0 = \mathrm{init}\big] \;\ge\; p$$

with $T$ the env's own `max_episode_steps` truncation — i.e. a **certified
lower bound on the success probability of a concrete shipped policy**, which
for FrozenLake's $+1$-at-goal reward equals a certified expected-return lower
bound (the scoping's U3 semantics). The certificate is a deadline table
$V : \{0..T\} \times S \to \mathbb{Q}_{\ge 0}$; obligations (WF/B/S/C) and the
ten-line backward-induction soundness argument are stated in
the qtsl repository's `crates/flcheck/src/lib.rs`. No optional stopping, no absorption side condition —
the deadline is the env's own semantics.

**Provenance chain**: kernels are imported from the staged single source of
truth (the workshop's staged model builder (now `producers/frozenlake_model.py`),
`build_frozenlake` — transcribed from `frozen_lake.py` at Gymnasium sha
`4347ce947`, generator-asserted row sums). The
float64-vs-$\mathbb{R}$ ledger row from that staging (gym's
`success_rate=1.0/3.0` and `fail_rate` are *different* float64 values; the
transcribed exact $1/3,1/3,1/3$ kernel is the honest model) applies here.

**Trust boundary**: the checker (`flcheck`, Rust) re-verifies everything
downstream of the certificate bytes in exact arithmetic; that the kernel
bytes ARE gym's kernel is producer-side provenance, the same seam as
every certified row of this family. Checker correctness evidence: 3-property
proptest battery (independent in-test DP implementations, including the
round-down-soundness property dyadic $\le$ exact), the 6-mutant battery
below, and a decorrelated jax.numpy float64 VI cross-instrument
($|\Delta| \le 3.5 \times 10^{-15}$ on all four rows). Lean glue is a named
next step: an existing machine-checked `sound_QDeadline` lemma covers the
*geometric-drift* deadline form ($\beta$-decay, $1 \le V$ off-target) and
does not consume this table shape; the finite-horizon
submartingale-table lemma is small (finite-state backward induction) and
not yet written.

## Conventions

- **Lower bounds round DOWN** at the displayed decimal (the mirror of
  the companion tables' upper-bounds-round-up convention).
- **check wall-clock** = median of 200 verification passes over the decoded
  in-memory certificate, single core, GB10 (aarch64), release build,
  `Instant`-timed; decode time reported separately in
  `checker_receipts.json`. ns/row = median / obligation count.
- **registry anchors**: `reward_threshold` and the known-optimum comments
  are from the pinned sha's `envs/__init__.py` registration block (witnessed
  in the pre-run survey). They are the only published comparison points that
  exist — to our knowledge there is no prior *certified* bound for any
  Gymnasium env at this claim class.

## Master table (all rows check on both lanes; 28/28 certificates ACCEPT)

| env (kwargs) | $T$ | registry threshold | registry "optimum" | house certified bound (exact lane) | check ms (exact) | dyadic $k{=}48$ bound | check µs (dyadic) | ns/row |
|---|---|---|---|---|---|---|---|---|
| FrozenLake-v1 (4x4 slippery) | 100 | 0.70 | 0.74 | **0.7441902** $= \frac{127846315164763240597097944751513986598202943912}{3^{99}}$ (exact) | 9.7 | 0.744190287829 | **24.6** | 16 |
| FrozenLake8x8-v1 (slippery) | 200 | 0.85 | 0.91 | **0.9132201** | 192.8 | 0.913220150201 | **139.1** | 11 |
| FrozenLake-v1 (success_rate=3/4) | 100 | — | — | **0.8731132** | 11.2 | 0.873113279953 | **24.9** | 16 |
| FrozenLake-v1 (is_slippery=False) | 100 | — | — | **1** (exactly) | 0.17 | 1 (gap 0) | **11.6** | 7 |

Reading the dominance columns (the Pareto-improvement compass): every row's
certified bound strictly exceeds its registry `reward_threshold` where one
exists (0.7441902 > 0.70; 0.9132201 > 0.85), the soundness level is
exact-$\mathbb{Q}$ (strictly above any empirical leaderboard number), and the
check cost tops out at 193 ms even on the exact lane. The slippery rows'
bounds are the **exact optima of the truncated envs** (tight by
construction — the producer is finite-horizon optimal DP), so no future
producer can beat these numbers at this claim class; they can only be
matched more cheaply.

**Registry-comment refinement (finding):** the registration comments'
"optimal average rewards" 0.74 and 0.91 are truncations of the exact values

- 4x4: $\;0.744190287829267\ldots$ (exact fraction, denominator $3^{99}$, in `frozenlake-4x4/exact_T100.cbor`)
- 8x8: $\;0.913220150201623\ldots$ (denominator $3^{200}$)

computed here, to our knowledge for the first time, in exact arithmetic —
with a machine-checkable certificate.

## The Pareto frontier, measured (bound vs check wall-clock)

Two honest refinement knobs trace the curve — deadline truncation $T'$
(a valid lower bound for the full env, since hit-by-$T'$ implies
hit-by-$T$) and dyadic precision $k$ (round-down integer DP, bound loss
$\le T \cdot 2^{-k}$, obligations become pure integer comparisons):

**FrozenLake-4x4 (T = 100):**

| certificate | bound | check (median) | obligations |
|---|---|---|---|
| exact $T'{=}5$ | 0 (honest: goal unreachable in 5 steps) | 53.8 µs | 156 |
| exact $T'{=}10$ | 0.0414062 | 299 µs | 231 |
| exact $T'{=}25$ | 0.2777996 | 814 µs | 456 |
| exact $T'{=}50$ | 0.5459086 | 2.82 ms | 831 |
| exact $T'{=}100$ | 0.7441902 | 9.65 ms | 1581 |
| dyadic $k{=}16$ | 0.7439727 (gap $2.2{\times}10^{-4}$) | 24.6 µs | 1581 |
| dyadic $k{=}32$ | 0.7441902 (gap $3.5{\times}10^{-9}$) | 24.6 µs | 1581 |
| dyadic $k{=}48$ | 0.7441902878 (gap $5.1{\times}10^{-14}$) | 24.7 µs | 1581 |

**FrozenLake-8x8 (T = 200):**

| certificate | bound | check (median) | obligations |
|---|---|---|---|
| exact $T'{=}25$ | 0.0128870 | 3.58 ms | 1896 |
| exact $T'{=}50$ | 0.2283512 | 13.7 ms | 3471 |
| exact $T'{=}100$ | 0.6407192 | 47.6 ms | 6621 |
| exact $T'{=}200$ | 0.9132201 | 192.8 ms | 12921 |
| dyadic $k{=}16$ | 0.9126892 (gap $5.3{\times}10^{-4}$) | 267 µs | 12921 |
| dyadic $k{=}48$ | 0.9132201502 (gap $1.2{\times}10^{-13}$) | 139 µs | 12921 |

**The headline measurement:** at equal obligation count, the dyadic lane
checks **~700–1400× faster** than the exact lane (11–21 ns/row vs
7–15 µs/row) at a bound cost of $10^{-13}$-order — deep below any
display digit. The exact lane's cost is entirely bignum arithmetic on
$3^{T}$-scale denominators; the dyadic lane is u64/u128 integer compares.

**Design consequence (feeds the START-gate kernel lane):** the dyadic
deadline table is a *shared-denominator integer slab* — literally the
founding shape of the maitria-coalition/kernels `sweep` kernel
(first_violation + minmax over shared-denominator i64 slabs), and
consistent with an independent finding that device-side exact-$\mathbb{Q}$
verification wants the common-denominator integer-AXPY shape (measured
11× over deferred-norm). Certificates should be *engineered for their
checker*: emit integer slabs, keep rationals for the claim line. At
FrozenLake scale the whole corpus checks in µs on one core — the GPU null
at today's sizes (launch floor dominates) applies with
three orders of headroom; the batched lane earns keep only past
$\sim 10^6$-row corpora (the cartpole 5-var route's regime, not this one).

## Receipts

- `producer_receipts.json` — bounds (exact fractions + floats), dyadic
  gaps, produce times, cert sizes, jnp cross-check deltas
  ($\le 3.5{\times}10^{-15}$; 4x4-det exactly 0).
- `checker_receipts.json` — 28/28 ACCEPT verdicts with per-cert decode/verify
  timings and bench distributions (median of 200).
- Mutant battery, 6/6 refused at their designed loci: inflated claim
  (`claim`), raised interior entry (`step` at its $(t,s)$), perturbed kernel
  prob (`wf` sum), boundary violation (`boundary`), out-of-range policy
  action (`policy`), goal-list drop (`boundary` at the true goal — the
  claim-weakening mutant is caught by the indicator row, not by convention).
- Proptest battery (`../checker/tests/`): random-MDP exact-DP certs accept;
  any strict single-entry inflation refuses; dyadic certs accept and never
  exceed the exact bound. 3/3 green, 256 cases each.
- Process cargo: the producer's first run had a classic
  value-never-reassigned bug (every "iteration" recomputed the 1-step
  value); the **jnp cross-instrument caught it before any emission**
  (0.744 vs 0.0 — the artefact-pair discipline, 1.D.(7), doing exactly its
  job). The registry comment "0.74" then confirmed the fixed instrument.

---

# Wave 2 — toy-text tier completion (reward-weighted deadline tables, 7 rows)

*Wave 2 run record, 2026-07-20.  Machinery: `producer/toytext_kernels.py` +
`producer/tt_producer.py` + the two new `flcheck` lanes (`rwslab`,
`rwdyadic`).  With this wave every registered toy-text id at the pinned
sha carries at least one certified row — the tier is complete.*

## The claim class

$$E_\pi\Big[\textstyle\sum_{t<T} r(X_t, A_t, X_{t+1}) \,\Big|\, X_0 \sim \mathrm{init}\Big] \;\ge\; B$$

for the shipped time-varying policy $\pi$, with terminal states absorbing
at reward 0 (gym's episode-over semantics) and init a distribution (Taxi
draws uniformly from 300 states; Blackjack from the exact deal
distribution).  Obligations and the backward-induction soundness argument:
`../checker/src/lib.rs` (RwSlabCert doc).  The hitting lanes of wave 1 are
the special case $r{=}{+}1$ on goal-arrival edges.  Kernel rows are
*outcome* rows $(d, r, n)$ — the same destination may legitimately appear
with two rewards (witnessed necessity: CliffWalkingSlippery state 36
action UP reaches state 36 both by wall-clip at $-1$ and cliff-bounce at
$-100$).

Deadline semantics, labelled honestly per row: Taxi-v4's $T{=}200$ is the
env's own TimeLimit; CliffWalking-v1 / CliffWalkingSlippery-v1 /
Blackjack-v1 register **no truncation**, so $T$ is the certificate's own
declared deadline.  For Blackjack $T{=}22$ exceeds the longest possible
hand, so the bound IS the untruncated expected value; for
cliffwalking-det the companion hitting certificate
(`hitting_exact_T13.cbor`, $P[\mathrm{goal\ by\ }13] \ge 1$) composes
with the reward row to make $-13$ the untruncated return as well.

**Provenance upgrade over wave 1**: the exact-$\mathbb{Q}$ kernels are
not merely transcribed from the pinned sources — they are **asserted
equal to gym's own `env.P` tables** (and `initial_state_distrib`),
outcome-set-identical with probability differences $0$ exactly, except
$5.6 \times 10^{-17}$ on rainy-taxi rows (the declared float64 ledger:
gym's `0.8` / `(1-0.8)/2` literals vs exact $4/5$, $1/10$).  The
comparison runs against Gymnasium 1.3.0 installed from the pinned sha
into a venv (receipts: `producer_receipts_tt.json` → `gym_p_crosscheck`).

## Master table (25/25 certificates ACCEPT; bounds round toward $-\infty$)

| env (kwargs) | $T$ | registry anchor | house certified bound | δ to true optimum | check ms (slab) | check ms (dyadic $k{=}48$) | ns/row (dyadic) |
|---|---|---|---|---|---|---|---|
| CliffWalking-v1 | 100* | none registered; $-13$ derivable (S&B Ex. 6.6) | $\mathbf{-13}$ (exactly) | 0 | 0.47 | 0.05 | 10 |
| CliffWalkingSlippery-v1 | 100* | none; no published value found | $\mathbf{-63.0133733}$ (exact $\mathbb{Q}$, den $\sim 1.7{\times}10^{47}$) | 0 | 0.64 | 0.06 | 12 |
| Taxi-v4 | 200 | thr 8, comment "optimum = 8.46" — **both refuted, see finding** | $\mathbf{7.93} = \frac{793}{100}$ (exactly) | 0 | 5.94 | 1.57 | 15 |
| Taxi-v4 (is_rainy) | 200 | — | $\mathbf{3.9545745165}$ | $7.5{\times}10^{-14}$ (slab T′=100) / $3.1{\times}10^{-14}$ (dyadic) | 13.85 | 1.80 | 17 |
| Taxi-v4 (fickle_passenger) | 200 | — | $\mathbf{8.38} = \frac{419}{50}$ (exactly; slab tail gap 0) | 0 | 13.20 | 3.50 | 17 |
| Taxi-v4 (rainy + fickle) | 200 | — | $\mathbf{4.5756463469}$ | $6.9{\times}10^{-14}$ / $3.0{\times}10^{-14}$ | 32.14 | 4.22 | 20 |
| Blackjack-v1 (sab) | 22* | none; **no published EV for this ruleset exists** (lit search, below) | $\mathbf{-0.0431131055} = -\frac{3954908734551683826282634489}{13^{26}}$ (exactly) | 0 | 1.52 | 0.14 | 20 |

\* = certificate-declared deadline (no registry truncation); see
deadline-semantics paragraph.

Pareto reading, per the dominance compass (dominate the published
comparison point on every axis, not one): for every row the
comparison point is either *nothing* (no prior certified bound at this
claim class exists for any of these envs, to our knowledge) or a registry
number our exact computation strictly refines or refutes — and the rows
that do not sit at $\delta = 0$ sit at $10^{-14}$, below any display
digit.  Soundness level is exact-$\mathbb{Q}$ throughout; check cost tops
out at 32 ms (slab) / 4.2 ms (dyadic) on the largest chain (1000 states
× 200 steps).  The dyadic reward lane holds the wave-1 speed class
(10–20 ns/row vs wave 1's 11–21) despite signed rewards and init
mixtures.

## Findings

**1. Taxi-v4's registry threshold is unsatisfiable, and its "optimum"
comment is a map-correction fossil.**  The pinned env's true optimal
expected return (undiscounted, $T{=}200$ ample: dry episodes end in
$\le 20$ steps) is $\frac{793}{100} = 7.93$, *below* the registered
`reward_threshold=8` — no policy can meet the threshold.  The comment
"optimum = 8.46" is exactly the OLD map's optimum: on the pre-v3 map
(gym ≤ v0.10's row 1 `"| : : : : |"`, no wall segment below R's wall)
the same computation gives $\frac{2539}{300} = 8.4633\ldots$ —
reproduced here by swapping only the map.  The v3 "Map Correction"
added the wall; the registration numbers were never updated.  (Upstream
contribution candidate; per the house upstream norm this stays banked
in-tree pending an AI-PR-stance check.)

**2. First exact value of Sutton-&-Barto blackjack.**  A commissioned
literature search (S&B book full text; Baldwin et al. 1956; Wizard of
Odds/Vegas; arXiv 2603.18642, 2602.17375) found **no published EV for
this exact ruleset** (infinite deck, dealer stands all 17s, hit/stand
only, even-money natural with sab autowin).  Nearest analogue: $-2.42\%$
house edge for the same game *with 3:2 naturals* (forum-grade,
combinatorially verified).  Our exact optimum
$-3954908734551683826282634489/13^{26} \approx -4.311\%$ is consistent:
the gap ($\approx 1.89$ points) is the order of the forgone 3:2 bonus
($P[\text{natural}] \cdot P[\text{dealer not natural}] \cdot \tfrac12
\approx 2.2$ points).

**3. The sab observation is not reward-Markov; the certificate space
fixes it.**  Under sab rules a 2-card natural 21 beats a dealer
non-natural 21 while a 3-card 21 pushes — but gym's observation
`(21, show, usable)` cannot distinguish them.  The certificate's state
space carries the natural flag (10 extra states), and the producer
asserts the shipped policy is *flag-invariant*, so it remains
implementable on the real observation.

**4. Hidden state, done honestly: the fickle rows certify the
belief-MDP.**  The fickle coin is invisible until it fires, so a policy
on (state × armed) would be clairvoyant.  The belief about the coin is
two-valued along every history (maybe-armed at prior $3/10$ /
known-disarmed — the first with-passenger move reveals it either way),
so the belief MDP is again a finite chain (1000 states) and its optimal
policy is exactly implementable from observation history.  Measured
value of clairvoyance (float DP, not certified): rainy+fickle
clairvoyant 4.581050 vs implementable 4.5756463 — information worth
$\sim 5.4 \times 10^{-3}$; under dry movement the gap is 0 (the reveal
coincides with the last decision it could inform).  Bonus structure
finding: fickleness *raises* the optimum (dry $7.93 \to 8.38$, rainy
$3.9546 \to 4.5756$) — a mid-trip uniform re-draw of the destination
is on-average favourable, and the certified policies exploit the
post-reveal re-route.

**5. The crude-tail knob (new frontier axis for reward tables), with a
catch to its name.**  Tail-padded slabs pin rows $t \ge T'$ at
$r_{\min}(T-t)$ for non-terminal states and $0$ for terminal ones, so
the bound loses only $\sim P[\text{alive at } T'] \cdot |r_{\min}|(T-T')$
while the certificate shrinks: taxi-rainy at $T'{=}100$ gives up
$7.5{\times}10^{-14}$ for half the bytes; $T'{=}50$ gives up
$1.9{\times}10^{-3}$; $T'{=}25$ gives up 87 (rain keeps
$P[\text{alive at }25]$ macroscopic) — an honest curve.  The first
emission pinned *terminal* tail rows at $r_{\min}(T-t)$ too — sound but
uselessly slack (gap $\approx 10^3$, charged with probability 1); the
`gap_to_optimum` receipt column caught it before anything shipped.

**6. Instrument finding: ciborium's `Value` cannot represent CBOR
bignums with 16-byte payloads beyond i128.**  Magnitudes in
$(2^{127}, 2^{128})$ decode as "integer too large" (bignum tags with
$\le 16$-byte payloads are converted to native integers; larger ones
stay tags) — and slab tables land in that octave routinely (38–1000
cells per affected certificate).  Certificate bignums therefore ride
sign-prefixed byte strings (`sbytes`), matching the hitting lanes'
magnitude-bytes rationals; regression pinned in
`tests/proptest_rwcheck.rs::poison_octave_roundtrip`.

## Frontier curves (bound vs check cost vs certificate bytes)

**CliffWalkingSlippery-v1, exact ladder** (each row the exact optimum of
its own $T'$-step claim): $T'{=}10$: $-10$ / 80 µs / 7 KB; $T'{=}25$:
$-24.9905962$ / 103 µs / 12 KB; $T'{=}50$: $-47.1022303$ / 490 µs / 26 KB;
$T'{=}100$: $-63.0133733$ / 644 µs / 72 KB.  Dyadic $k$-ladder at
$T{=}100$: $k{=}16$ gap $3.1{\times}10^{-4}$, $k{=}32$ gap
$4.9{\times}10^{-9}$, $k{=}48$ gap $7.4{\times}10^{-14}$ — all
$\sim$60–120 µs.

**Taxi-rainy, tail ladder** (all claims about the full $T{=}200$ env):
see finding 5.  The slab lane pays 57–208 ns/row (bignum) against the
dyadic lane's 15–20 ns/row; at equal $10^{-14}$-order bound quality the
dyadic lane is the checker-axis winner everywhere, and the slab lane's
role is $\delta = 0$ exactness where the optimum is rational with small
denominator (CW-det, taxi-dry, taxi-fickle) or the claim line itself.

## Receipts (wave 2)

- `producer_receipts_tt.json` — exact optima (fractions + floats), tail
  gaps, dyadic gaps, produce times, cert sizes, jnp cross-check deltas
  ($\le 1.1{\times}10^{-13}$ absolute; several exactly 0), gym P-table
  cross-check verdicts.
- `checker_receipts_tt.json` — 25/25 ACCEPT with decode/verify timings
  (median over 200/50/10 passes by size; single core, GB10
  aarch64, release build).
- `tt_mutants/`, 9/9 refused at designed loci: inflated claim, raised
  interior entry, perturbed kernel numerator, positive boundary row,
  out-of-range policy action, **deflated edge reward** (the reward
  lanes' new mutant class — tightness of the DP table makes any strict
  deflation refuse at `step`), perturbed init weights, dyadic interior
  raise, dyadic inflated claim.
- Proptest battery (`../checker/tests/proptest_rwcheck.rs`): random
  reward-MDPs — slab DP certs accept; any single-entry inflation
  refuses; policy-edge reward deflation refuses; dyadic certs accept
  and never exceed the slab bound; plus the poison-octave CBOR
  regression.  4/4 properties × 256 cases green.
- Process notes: the tail-pinning catch (finding 5) and the M9 mutant
  generator briefly producing a 5 GB certificate by multiplying a
  *byte string* by $10^9$ (python repetition semantics) — both caught
  by cross-checking receipts against independent instruments before the
  results were recorded, neither by code inspection alone.

## Named next steps

1. **Toy-text completion** (CliffWalking-v1/Slippery, Taxi-v4 + rainy/fickle,
   Blackjack-v1): pure generator+producer work on `theory Mdp` + this exact
   pipeline; Taxi's action-dependent rewards need the reward-valued variant
   of the deadline table (expected-return DP, same shape, reward-weighted
   rows) — about half a working session by tonight's receipts (4 rows,
   ~2.5 h including the checker build).
2. **The finite-horizon submartingale-table Lean lemma** — closes tonight's
   checker-soundness residue; finite-state backward induction, routed to a remote prover host.
3. **Dyadic slab checker in the kernels repo** — re-express the dyadic lane
   over `sweep`-kernel slabs (NEON/AVX2 receipts exist); admission test
   passes (producer- and consumer-shared, not proof-coupled).
   **DONE — wave 3, below.**
4. **CartPole U1** — the pilot with its three staged gaps (Euler-map
   transport, `Discrete(2)` switching cells, cart-excursion / 5-var route);
   the first row whose check is a genuine batched-checker workload.
5. **Pendulum-v1 U1 cost-bound** — the staging doc's named next research
   rung (transcendental half of the X0 gap).
6. **U4 e-process lane opener** (LunarLander): the breadth blanket for
   solver-semantics envs, via the randomized-last-look e-process device.

---

# Wave 3 (2026-07-20): the integer lanes ride the maitria-kernels `sweep` slabs

Three independent receipts had converged on "certificates should be
engineered for their checkers" and named the shared-denominator integer
slab as the checker-native shape (this file's dyadic lanes, the founding
`sweep` kernel, the E5 L-INT finding). This wave cashes that in for the
whole tabular class: `flcheck` gains `sweeplane` — the `dyadic` (hitting)
and `rwdyadic` (reward-weighted) obligations re-shaped into dense i64
**residual slabs** decided by `maitria-kernels::sweep::first_violation`
(NEON here; AVX2/scalar elsewhere, all verdict-identical by that repo's
conformance battery), with table-range passes riding `sweep::minmax` and
the kernel flattened once per certificate into CSR slabs with
precomputed reward constants $\sum_d n \cdot r \cdot 2^k$.

**The hoisted overflow argument.** The reference lanes prove no-overflow
per operation (checked i128); the slab lane proves it once per
certificate: a checked-i128 **envelope**
$\max_g\big[\sum_d n(|r| 2^k + M) + D M\big] < 2^{63}$ (with $M$ the
table's absolute max, read off by `sweep::minmax`). Inside the envelope,
plain i64 arithmetic cannot wrap; outside it the lane declines and the
dispatcher (`verify_fast`) falls back to the reference lane — the
promotion-ladder / DATERWI shape: deferral costs completeness, never
soundness. Debug builds arm Rust's checked i64 arithmetic, so the whole
proptest battery doubles as a mechanical check of the envelope claim.

## Parity battery (`../checker/tests/proptest_sweep_parity.rs`)

Exact-oracle anchored, the wave-2 pattern: valid certificates from the
independent in-test DPs must ACCEPT on both lanes with equal
`rows_checked` and bound **and must actually engage the slab lane** (no
vacuous parity through fallback); random single-entry inflations and
claim inflations decide equally on both lanes (and refuse on the
reference lane — the DP tables are tight); a deliberately
envelope-breaking certificate ($r = 2^{63}-1$ at $k=48$) declines and
falls back with verdict intact; and **whole-corpus parity**: every
committed certificate and mutant under `out/` (70 CBOR files: 55
certificates + 15 mutants; 30 route through the sweep lane — the 28
engaging integer certificates, blackjack-k48 excepted, plus the two
rwdyadic mutants) decides equally on both lanes. 7/7 properties green
in release and debug.

## Receipts (`checker_receipts_sweep.json`; single core, pinned, GB10 aarch64, NEON lane)

Comparison on `verify_min_ns` over 200/50/10 in-process passes (min =
the uncontended floor; this box hosts many services, and unpinned
medians were ~2x bimodal run-to-run in *both* lanes — the receipts file
carries medians too). Selected rows:

| certificate | rows | ref | sweep | speedup | sweep ns/row |
|---|---|---|---|---|---|
| taxi-fickle rwdyadic k48 T200 | 207,002 | 3454 us | 1063 us | **3.25x** | 5.1 |
| taxi-rainy-fickle rwdyadic k48 T200 | 207,002 | 4083 us | 1516 us | 2.69x | 7.3 |
| taxi-dry rwdyadic k48 T200 | 103,502 | 1540 us | 491 us | 3.14x | 4.7 |
| cliffwalking-det rwdyadic k48 T100 | 5,042 | 49 us | 24 us | 2.04x | 4.7 |
| blackjack rwdyadic k16 T22 | 7,277 | 144 us | 91 us | 1.60x | 12.4 |
| frozenlake-8x8 dyadic (all k) T200 | 12,921 | 137 us | 100 us | 1.38x | 7.7 |
| frozenlake-4x4 dyadic (all k) T100 | 1,581 | 11–13 us | 13–15 us | **0.83–0.89x** | 8.1–9.6 |
| blackjack rwdyadic k48 T22 | 7,277 | 143 us | 158 us | **0.91x (fell back)** | — |

## Findings

1. **The reward lanes are where the slabs pay.** `rwdyadic` reference
   arithmetic is fully checked i128; hoisting the overflow argument to
   the certificate level buys 1.6–3.25x everywhere the lane engages,
   and the big taxi rows now check at **4.7–7.4 ns/row** — the same
   cost class as wave 1's hitting-lane 11–21 ns/row, but on
   certificates with 6 actions, outcome-split rewards, and 200-step
   horizons.
2. **Honest non-domination, named: small hitting certificates.** The
   hitting reference lane was already unchecked u128 with zero setup;
   at 4x4 scale (1,581 rows) the slab lane's per-call CSR + envelope
   build dominates and it runs 0.83–0.89x. Crossover is around the
   8x8/T200 scale (1.38x). The dispatcher stays cost-only and simple
   (verdicts are lane-identical); if the small-cert regression ever
   matters, the fix is a size floor in `verify_fast`, not lane surgery.
3. **The envelope is a certificate-design signal, not just a guard.**
   Blackjack's exact deck denominators reach $D \approx 2^{40.7}$, so at
   $k=48$ the envelope needs $\approx 2^{90}$ — structurally beyond i64;
   the lane declines (cost: the failed attempt, ~9% on that cert) and
   the reference lane carries it. The checker-native re-emission for
   big-$D$ kernels is a **smaller $k$**: the wave adds
   `blackjack/rwdyadic_k16_T22` (bound cost $1.1 \times 10^{-5}$,
   engages, 1.60x) and `k20` (` ~2^{62.3}`, just inside — the edge
   specimen, engages). Producer knob, checker fact: exactly the
   emission-discipline thesis, now with a worked negative
   space.
4. **Same-box benching under shared load needs pinning.** Unpinned,
   both lanes' medians were bimodal by ~2x run-to-run (a busy shared
   box); `taskset` + min-of-N restored run-to-run stability to
   within ~2%. The wave-1/wave-2 receipts' absolute numbers inherit
   this caveat qualitatively (their cross-lane *ratios* were taken
   within single invocations and stand).

Reproduce: `cargo build --release` in `../checker`, then
`python3 ../producer/emit_bj_lowk.py` (the two low-k blackjack rows)
and `python3 ../producer/run_sweep_bench.py`.

## Wave 3 (leg 5): the U4 statistical tier opens — LunarLander-v3, first certified Box2D row

*A different claim class from every row above, displayed per the scoping's
U4 discipline: confidence-qualified, never silently mixed with
exact/almost-sure rows.* The certificate is a betting-martingale ledger:
$n$ i.i.d. rollout returns of a fixed policy, a data-free dyadic
$\lambda$-grid mixture wealth process, and a claimed boundary $m^*$ whose
single check (justified by monotonicity in $m$) the `eprocess` lane of
`flcheck` re-verifies in Q32.96 fixed point, every rounding directed down
(proptest: fp wealth $\le$ exact wealth, fp-ACCEPT $\Rightarrow$
exact-ACCEPT). Soundness spine: Ville for nonnegative supermartingales +
a randomized last look (`ville_ineq_randomized`, machine-checked in Lean, 0 sorries; module not included in this repository).

**Row: LunarLander-v3 @ 4347ce947, default kwargs, policy = the pinned
file's own `heuristic`, $n = 10^6$ episodes** (ledger + seeds:
`out/ll/ll_ledger.cbor`; empirical mean 238.04, sd 105.3, 92.1% landed):

| claim class | $\alpha$ | det. Ville bound | randomized-look bound | trust rows |
|---|---|---|---|---|
| $\mathbb{E}[R] \ge$ (via $Y = \min(R, 340)$ + range lemma $R \ge -190381$ a.s.) | 1/20 | **236.577** | 236.656 | range lemma (b2 clamps + reward arithmetic), PRNG, policy-determinism |
| $\mathbb{E}[R] \ge$ | 1/100 | **236.231** | 236.306 | same |
| $\mathbb{E}[\mathrm{clip}(R,-400,340)] \ge$ (truncated functional) | 1/20 | 237.568 | 237.576 | PRNG, policy-determinism only |
| $\mathbb{E}[\mathrm{clip}(R,-400,340)] \ge$ | 1/100 | 237.533 | 237.541 | same |

Registry threshold ("solved") = 200: cleared by 36 points at 99%
confidence — to our knowledge the first certified statement of any kind at
this claim class for a Box2D env (published numbers are bare empirical
means). Check wall-clock: **203.6 ms median** single-core for the
$n = 10^6$ cert ($1.6\times 10^7$ obligations — **12.7 ns/obligation**,
the dyadic-lane class again); the 20k fixture checks in 1.34 ms. Mutants
8/8 refuse at designed loci (incl. a range-witness plant: a ledger entry
below the lemma floor refuses the whole certificate as a refutation
witness). Full derivations, the discrete/continuous seam, the exact
free-flight transport theorem ($h^2 g/2$ — closed form, no Picard), and
the Box2D friends census: an internal seam note (not included in this
repository).

Anytime-valid corollary (the U4 frontier curve): the same ledger extends —
more episodes can only tighten these bounds, no penalty, no re-planning;
$\mathrm{gap} \approx (\ln\frac{1}{\alpha} + \ln 2)\,G/(\lambda_{\max} n)
+ \sigma\text{-term}$, measured 1.81 (min-cap, $G = 190721$) and 0.51
(clip, $G = 740$) at $n = 10^6$, $\alpha = 1/100$.

## Wave 3 (leg 6): LunarLanderContinuous-v3 — the first friend, same blanket

*Same claim class, pipeline, checker, and range lemma as the leg-5 row
above; the leg's entire delta is one sampler kwarg + one producer prefix
argument. The lemma's variant-independence is witnessed line-by-line in
the seam doc's delta section (short form: $m_\text{pow}, s_\text{pow}
\in \{0\} \cup [0.5, 1.0]$ in the continuous path — line 533 + its
assert, line 579 + its assert — versus $\{0, 1\}$ discrete, so the
$0.33$/step fuel charge already covered it; `world.Step`, scaling,
reward, and termination-replacement have no continuous branch at all).*

**Row: LunarLanderContinuous-v3 @ 4347ce947 (registry id at
`envs/__init__.py:102` — same entry point, `kwargs={"continuous": True}`,
same horizon 1000, same threshold 200), policy = the same pinned file's
`heuristic` (its continuous branch, lines 831–833), $n = 10^6$ episodes**
(ledger + seeds: `out/llc/llc_ledger.cbor`; empirical mean 282.97,
sd 24.04, 99.77% landed):

| claim class | $\alpha$ | det. Ville bound | randomized-look bound | trust rows |
|---|---|---|---|---|
| $\mathbb{E}[R] \ge$ (via $Y = \min(R, 340)$ + range lemma $R \ge -190381$ a.s.) | 1/20 | **281.531** | 281.610 | range lemma (b2 clamps + reward arithmetic), PRNG, policy-determinism |
| $\mathbb{E}[R] \ge$ | 1/100 | **281.185** | 281.261 | same |
| $\mathbb{E}[\mathrm{clip}(R,-400,340)] \ge$ (truncated functional) | 1/20 | 282.876 | 282.878 | PRNG, policy-determinism only |
| $\mathbb{E}[\mathrm{clip}(R,-400,340)] \ge$ | 1/100 | 282.866 | 282.868 | same |

Registry threshold 200: cleared by **81 points at 99% confidence**. The
continuous heuristic is far stronger than its discrete sibling (throttleable
engines land softly: 99.77% vs 92.08% landed, sd 24.0 vs 105.3, mean 282.97
vs 238.04) — and the certificates preserve the entire ordering: the
*certified floor* of the continuous variant (281.19) exceeds the discrete
variant's *empirical mean* (238.04) by 43 points. Check wall-clock: 128.6 ms
median single-core for the $n = 10^6$ flagship (same $1.6 \times 10^7$
obligations — 8.0 ns/obligation; against leg 5's 12.7 ns on identical code
and $n$, the per-obligation figure is evidently data-dependent). Mutants:
8/8 refuse at designed loci (battery regenerated per the
certs-one-ledger schema; `--prefix llc`).

**Gap finding (the regime split, now measured across two ledgers)**: the
min-cap gap barely moved — 1.785 here vs 1.81 for discrete at
$\alpha = 1/100$ — despite the sd falling $105 \to 24$, confirming that the
min-cap row sits in the $G$-dominated regime of the frontier formula
(the $(\ln\frac{1}{\alpha} + \ln 2)\,G/(\lambda_{\max} n) = 1.08$ term
binds; the $\sigma$-term is the residual ~0.7). The clip rows, with
$G = 740$, show the sd through: gap $0.51 \to 0.104$ (ratio 4.9 ≈ the sd
ratio 4.4). Actionable split: for min-cap rows, episodes are nearly spent —
the payoff is a *tighter certified a.s. floor* (every factor-of-$k$ shrink
in $G = k_\text{hi} - k_\text{lo}$ divides the dominant gap term by $k$;
the $\theta$-accumulation term $100 \cdot 1001 \cdot \pi/2 \approx 157{,}240$
is ~83% of $|k_\text{lo}|$ and the $y$-drift term most of the rest, so a
certified angle bound plus a certified altitude corridor would collapse $G$
by two orders); for clip rows, more episodes still pay at the usual
$\sqrt{n}$-ish rate.
---

# Wave 3 — producer time (Rust toolkit `gymdp`)

*Wave 3 run record, 2026-07-20 morning. Scope: on the already-mastered
Gymnasium catalogue, push the *producer* time — how quickly can an
optimized producer find the optimal policy *and* its certificate?
NOTE: the Rust producer (`gymdp`) and its CUDA kernels live in a
separate toolkit repository not included here; this repository
reproduces its tables' python-producer + checker halves, and this
section is retained as the measured record of the toolkit run, with
scope widened to
PTX-in-XLA kernels.*

## What landed

The whole catalogue producer now lives in **mtk** as `gymdp`
(the toolkit's systems-tier `gymdp` producer +
`kernels/gymdp.cu`): kernel builders for all four env families
transcribed from the python single sources of truth, integer-slab exact
DP, dyadic DP, crude-tail lanes, and a cbor2-byte-compatible emitter.

**Correctness gates, all green:**

- **53/53 certificates byte-identical** to the python-produced corpus
  (every table cell, policy entry, kernel row, claim — the strongest
  possible artefact-pair; `gymdp produce --compare` re-proves it on
  every run).
- `flcheck` ACCEPTs all 53 rust-emitted certificates (independent
  checker as ground truth).
- Proptest battery (`mtk/tests/proptest_gymdp.rs`, 5 properties x 256
  cases): slab DP == naive-rational reference DP (values AND argmax
  policies), dyadic round-down soundness ($\le T \cdot 2^{-k}$ exact),
  emission determinism.
- GPU lanes: **bit-identical tables and policies** to the host lanes on
  all 28 dyadic lanes (RTX 5090), and again inside jitted XLA programs
  (4 lanes probed).

## Producer wall-clock, before/after (GB10, single core pinned, min-of-N)

Python column = the original producers' DP spans re-timed pinned
(min of 7); rust = same spans (min of 15). Full per-lane receipts:
`producer_receipts_rust.json` + `producer_bench_py_pinned.json`.

| row (headline lanes) | python | mtk rust 1-core | speedup |
|---|---|---|---|
| frozenlake-4x4 exact $T{=}100$ | 14.1 ms | 0.53 ms | 27x |
| frozenlake-8x8 exact $T{=}200$ | 153.7 ms | 6.6 ms | 23x |
| frozenlake-8x8 dyadic $k{=}48$ | 17.0 ms | 0.23 ms | 73x |
| cliffwalking-slippery optimum | 5.7 ms | 1.10 ms | 5.2x |
| taxi-dry optimum | 76.3 ms | 13.2 ms | 5.8x |
| taxi-rainy-fickle optimum | 453.2 ms | 117.1 ms | 3.9x |
| taxi-rainy-fickle rwdyadic $k{=}48$ | 335.6 ms | 10.1 ms | 33x |
| blackjack optimum | 7.7 ms | 2.2 ms | 3.5x |
| **catalogue total (59 lanes)** | **3242 ms** | **502 ms** | **6.5x** |

Multi-core (rayon over states, 4 cores pinned): the big bignum rows
gain a further ~2-3.7x (taxi-rainy-fickle optimum 117 → 35 ms); small
rows are unchanged by design (work heuristic gates the dispatch).
**Cold start to full catalogue** — process start to all 53 certificates
emitted + byte-compared: **0.69 s** (4 cores, GB10). The python
producers' equivalent end-to-end runs were ~10 s per wave.

One lane honestly lost: blackjack rwslab reduction (1.7 ms python vs
1.8 ms rust) — CPython's C-optimized gcd on a shallow-copied table edges
out the valuation scan at that one shape; immaterial and recorded.

## The optimization ladder (what moved the needle, in order)

1. **Slab-integer DP with factored group sums** — $\sum_d n_d (r_d q_1 +
   w_{t+1}(d)) = (\sum_d n_d r_d)\,q_1 + \sum_d n_d\, w_{t+1}(d)$: one
   big-mul per distinct reward-sum per step (shared across groups)
   instead of one per outcome row; no division anywhere in the DP.
2. **Valuation-based reduction, never bignum gcd** — every denominator
   is $L^m$ with $L$ a known tiny integer, so canonical reduction is a
   per-prime capped valuation: one big division per cell (mod by the
   current $p^{v_{\min}}$ candidate; remainders are small, $p{=}2$ rides
   trailing-zeros free). Replaced a measured 32% gcd profile share.
   Two failed intermediate designs (unit-step and binary-lifting
   valuation scans) regressed the high-valuation rows 2-20x before the
   one-division-per-cell design landed — timings recorded in the
   campaign's working notes (workshop-side).
3. **No deep clones on reuse lanes** — the $T'{=}T$ exact lanes emit
   reduced rows as copy-on-write views over the optimum table (python
   shares list references; the naive port deep-copied ~100 MB on taxi).
4. **rayon over states within a step** (bit-identical order-preserving
   unzip; thresholded so small rows stay serial).

## GPU lane (rented RTX 5090, $0.69/hr, ~1 h)

The dyadic DP lanes as **one kernel launch per certificate** (the whole
backward induction inside the launch, block-wide sync per step —
`mtk/kernels/gymdp.cu`, NVRTC via the toolkit pipeline), plus the same
kernel registered as a **jax.ffi custom call inside jitted XLA**
(`gpu/gymdp_ffi_ext.cu`, an established jax.ffi custom-call pattern).

- Equality: bit-identical to host on all 28 lanes (cudarc path) and in
  XLA (4 lanes probed; `gpu/xla_probe_results.json`).
- Taxi-class rows: **~2.1x** over the box's CPU end-to-end (5.6 vs
  11.8 ms rainy-fickle $k{=}48$); in-jit steady state 5.95 ms with the
  custom-call barrier at noise level, and a fused jnp epilogue (bound
  readout from $m_0$) free — the same study's barrier model confirmed on a
  producer workload.
- Small rows: launch+copy floor (0.24-0.54 ms) loses to CPU (0.02-0.2
  ms) — the "GPU null at today's sizes" finding, now measured
  producer-side. The parallel width of a 1000-state catalogue row
  (~6k group sums/step) cannot feed 170 SMs; the GPU producer lane
  earns keep at CartPole-route state counts, not toy-text.

## Named for successors

- **Bignum slab DP on device** — the exact/optimum lanes are the
  remaining 80% of producer time; the toolkit's qint.cu limb templates
  are the substrate a fixed-slot device slab DP would ride.
  CPU twin of the same idea: a fixed-slot limb arena with hand addmul
  (drop num-bigint allocation churn on the taxi rows).
- **Producer-side GPU at CartPole scale** — re-run this lane's
  measurement when the first continuous row's certificate synthesis
  (Euler-transport cells) gives the state counts where 2.1x becomes
  the multi-SM regime.
- flcheck's box build needs `maitria-kernels` staged (new dep landed
  by wave-3 leg 3); the rented-box recipe strips the toolkit's optional
  fastcheck-diff path deps instead of shipping the transitive closure.
