# Certified-Gymnasium experiments — run artefacts

**Status.** The python producers that synthesized this family's
certificates are migrating to TypeScript on the modelling toolkit's
public API and return with that migration. Everything the trust story
rests on ships now: the certificates themselves (`out/*/*.cbor`,
committed byte-for-byte), the receipts, the mutant batteries, and the
re-check drivers (`run_checks.ts` / `run_tt_checks.ts`; `make
gymnasium` from the repository root), which replay every committed
certificate and mutant through the independent Rust checkers.
Producer file names below are kept because they document how the
fixtures were produced — read them as provenance, not as paths in
this tree.

Waves 1–2 (2026-07-20): certified utility lower bounds for the complete
toy-text tier — FrozenLake ×4 (hitting lanes), CliffWalking ×2, Taxi ×4
(incl. rainy / fickle belief-MDP), Blackjack (reward-weighted lanes).
Wave 3 (2026-07-20): the integer lanes (dyadic + rwdyadic) re-shaped
into i64 residual slabs riding the maitria-coalition/kernels `sweep`
kernel (qtsl's `crates/flcheck/src/sweeplane.rs`; parity battery
qtsl's `crates/flcheck/tests/proptest_sweep_parity.rs`; receipts
`out/checker_receipts_sweep.json`; findings in `out/RESULTS.md` wave-3
section — incl. the blackjack big-D envelope decline and its low-k
re-emission, `emit_bj_lowk.py` in the migrated producer layer).

Master tables and findings: `out/RESULTS.md`. All certificates:
`out/*/*.cbor`.

## Components

- `fl_producer.py` (migrated producer layer) — wave 1: FrozenLake
  hitting-table producers (exact + dyadic). Kernel source of truth:
  the staged model builder `frozenlake_model.py`, same layer (pinned
  Gymnasium sha in-file).
- `toytext_kernels.py` (migrated producer layer) — wave 2: exact-Q
  kernel builders for CliffWalking / Taxi / Blackjack, transcribed
  from the pinned sources and cross-checked equal to gym's own `env.P`
  tables (venv install of the pinned tree; cross-check recorded in
  `out/producer_receipts_tt.json`).
- `tt_producer.py` (migrated producer layer) — wave 2: reward-weighted
  deadline tables — scaled-integer slab DP (rwslab), crude-tail
  variants, signed fixed-point DP (rwdyadic); jax.numpy
  cross-instrument; mutant emission.
- `run_checks.ts` / `run_tt_checks.ts` — checker sweeps over the
  committed certificate sets and mutant batteries (per wave); these
  are the drivers `make gymnasium-text` runs today.
- `flcheck` — the independent Rust checker, four lanes (exact / dyadic
  hitting; rwslab / rwdyadic reward-weighted), imported from the qtsl
  repository (`crates/flcheck`); proptest batteries in qtsl's
  `crates/flcheck/tests/` (incl. the ciborium poison-octave
  regression).
- qtsl's `crates/flcheck/src/forall.rs` — the OPTIMALITY mode (`flcheck <cert>
  --forall`, exact lane only): the $\forall a$ supersolution +
  attainment + claim-equality obligations, checker-side — ACCEPT means
  the claim IS the exact optimum of the truncated env, closing the
  "tight by construction" producer-trust seam (committed receipts are
  the fixture: qtsl's `crates/flcheck/tests/forall_fixture.rs`, incl.
  the base-ACCEPT/forall-REFUSE mutant pair exhibiting the gap;
  properties in `crates/flcheck/tests/forall_props.rs`).
- `out/RESULTS.md` — master tables, frontier curves, findings, receipts
  index. `out/mutants/`, `out/tt_mutants/` — 6 + 9 mutants (all refuse
  at designed loci).

## i.i.d. tier (e-process rows)

Wave 3 (2026-07-20): the U4 i.i.d. tier — LunarLander-v3 e-process
rows (a certified Box2D row; claim class confidence-qualified,
displayed separately from the almost-sure rows).

- `ll_sampler.py` (migrated producer layer) — phase 1: the
  10^6-episode rollout ledger (pinned env + in-file heuristic policy;
  seeds + provenance in-file).
- `ll_producer.py` (migrated producer layer) — phase 2: the
  exact-rational range lemma (constants derived in-script,
  outward-rounded), boundary search with `flcheck` as the exact
  disposer, certificate + mutant emission.
- qtsl's `crates/flcheck/src/eprocess.rs` — the `eprocess` lane: betting-mixture
  wealth in Q32.96 fixed point, all rounding directed down; Ville +
  randomized-last-look thresholds; range-witness refusal. Proptests:
  qtsl's `crates/flcheck/tests/eprocess_props.rs` (fp <= exact;
  monotone in m; range-witness; accept-implies-exact).
- `out/ll/` — committed: the ledger, the flagship alpha=1/100 min-cap
  cert (self-contained, `flcheck` it directly), the 20k fixture, both
  receipts JSONs. The other three certs + the 8-mutant battery are
  deterministic re-derivations from the committed ledger; the producer
  that rebuilds them is part of the migrated layer.
- LunarLanderContinuous-v3: same pipeline (the sampler's
  `--continuous` mode, producer prefix `llc`); checker unchanged
  (zero bytes). `out/llc/` follows the same committed/regenerable
  split. Certified $\mathbb{E}[R] \ge 281.185$ at 99% (row +
  gap-regime finding in `out/RESULTS.md`).

## Semiconductor ladder (fluid-network stability)

Models, spec digests, and receipts live in `../semicon/`; the family
target is `make semicon`.

- `fluidcheck` — independent Rust checker for fluid-drift stability
  certificates of re-entrant fluid networks, imported from the qtsl
  repository (`crates/fluidcheck`): mode-wise Farkas replay in
  exact $\mathbb{Q}$ (`fluid-drift-v1`), plus the aggregated-mode cover
  species (`fluid-drift-cover-v1`).
- `minifab_fluid_producer.py` (migrated producer layer) — rung 1: the
  Kempf 1994 Mini-Fab (model ledger + tables:
  `../semicon/minifab/FLUID-CERT.md`; spec digest with primary-source
  witnesses: `../semicon/minifab/KEMPF-SPEC.md`; reference DES:
  `../semicon/minifab/sim/`). Certificates + mutants + receipts:
  `../semicon/out/minifab/`.
- `parse_mimac7.py` + `mimac_cover_producer.py` (migrated producer
  layer) — rung 2: the SEMATECH/MIMAC set7 ("Fab7") wafer-fab dataset,
  172 steps / 24 stations (dataset + provenance: `../semicon/mimac/`;
  model ledger + tables: `../semicon/mimac/COVER-CERT.md`).
  Certificates + mutants + receipts: `../semicon/out/mimac7/`.

## Not included here

The wave-3 "producer time" section of `out/RESULTS.md` records a run of
an optimized Rust/CUDA producer (`gymdp`) that lives in a separate
toolkit repository; its outputs were byte-identical to the python
corpus (53/53). The python producers that were the
kernel-transcription source of truth are migrating to TypeScript on
the modelling toolkit's public API and return with that migration; the
certificate tables they reproduced are committed here in full.

## Reproduce

From the repository root:

- toy-text + reward-weighted lanes: `make gymnasium-text` — drives
  `run_checks.ts` and `run_tt_checks.ts` over every committed
  certificate set and mutant battery through the imported `flcheck`.
- e-process rows: `make gymnasium-box2d`, or replay the flagship
  certificate directly:
  `target/release/flcheck gymnasium/out/ll/ll_u4_min_a100.cbor`.
- semiconductor ladder: `make semicon` (`semicon/recheck.ts` over the
  committed Mini-Fab and MIMAC certificate sets).
- Atari exact-replay rows: `make gymnasium-atari` (see
  `atari/README.md` and the ROM note in the top-level README).

Re-synthesis of the certificates themselves returns with the producer
migration.
