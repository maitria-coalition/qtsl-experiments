# Circuits re-run — measurement receipts (2026-07-22)

Companion to `delta-ledger.md` (same pins). Every number below carries its
machine tag (see `MACHINES.md` at the repository root); all walls here are
instrument walls.

## Silicon

- **gb10** (NVIDIA GB10: Grace, aarch64, CPython 3.12.3): heterogeneous —
  Cortex-X925 @ 3.9 GHz (cores 8,9,16–19-class) and Cortex-A725 @ 2.808 GHz.
  All GB10 numbers below were pinned with `taskset` to core 19 (X925) unless
  marked A725. Background load 1.8–2.9 during runs (other-core; pinned-core
  interference bounded by the 3-run spreads shown).
- **epyc** (GCP n2d-highmem-16 spot, AMD EPYC 7B13, CPython 3.12.3,
  node 22.23.1): batch CPU box, load ~0–1.1, `taskset` core 15, every
  wall-bearing run serialized under `flock /workspace/timing.lock`.

## In-repo spine (`circuits/recheck.ts`)

| lane | qtsl checkout | result | wall (EPYC, /usr/bin/time) |
|---|---|---|---|
| a (committed pin) | c94af1f7 | accept sweep 245/245 + battery ALL PASS, exit 0 | 1.23 s, 93.2 MB peak |
| b (latest tips) | c0099c932 | identical (checkers byte-identical, verified on-box) | 1.24 s, 93.2 MB peak |

`make circuits-recheck` also run to completion on gb10 (exit 0), and
`make tables` regenerated `chapter/generated/nmc-table.tex`:
verify[nmc-table] MATCH (7 rows).

## Full-suite replay (the sweep scripts' own perf_counter protocol)

GB10-X925, 3 runs each (spread shown); published values in parentheses:

| number | published | re-derived |
|---|---|---|
| liveness produce / check | 10.6 / 77.4 ms | 10.5–10.6 / 76.6–76.9 ms |
| liveness worst single | 1.15 ms (vga_1) | 1.13–1.14 ms (PWM_1; near-tie class) |
| liveness chains | 81.7 ms | 81.8 ms |
| safety produce / check | 55.3 / 194.8 ms | 54.1–54.6 / 195.1–196.5 ms |
| safety chains produce / check | 19.2 / 117.4 ms | 18.8–19.0 / 115.6–116.8 ms |
| safety whole sweep | 387.3 ms | 384.5–387.5 ms |
| SL produce / check | 65.3 / 170.6 ms | 65.0–65.8 / 168.4–171.8 ms |
| SL chains produce / check | 16.9 / 149.7 ms | 16.8–17.2 / 149.3–151.2 ms |
| SL whole sweep | 403.5 ms | 400.8–406.6 ms |

epyc (run 3 of 3): liveness 26.1 / 204.2 ms (worst single
2.90 ms PWM_1); safety 115.4 / 476.2 + chains 54.1 / 297.8, whole 945.9 ms;
SL 149.2 / 439.2 + chains 51.7 / 384.8, whole 1028.0 ms; liveness chains
221.0 ms. Full logs retained on the measurement box (runs 1–3 per class).

## Mutant batteries (both boxes, identical verdicts)

- liveness fixture battery: 4 controls + 17 attacks — ALL PASS
  (the suite's `mutants.py`; recheck.ts ports all 17+4)
- liveness chain battery: 4 controls + 8 attacks — ALL GREEN
  (`mutants_chain.py`; uncounted in the published battery sentence)
- safety: 3 controls + 13 attacks (8 fixture-layer M1,M2,M3a,M3b,M4–M7 +
  5 chain-layer C1,C2a,C2b,C3,C4) — ALL PASS. The published "14 (9
  fixture)" was a vendored-rider miscount; `mutants_safety.py` has exactly
  one commit, so the battery never had 9 fixture attacks.
- SL: 9 controls + 14 attacks (8 fixture + 6 chain; M3a/M8a cross-layer
  accept-then-chain-refuse as designed) — ALL PASS, matches published
  14+9 exactly.

## Groundtruth instruments

| instrument | GB10-X925 | EPYC | verdict |
|---|---|---|---|
| groundtruth.py (liveness sims + gap-vs-bound) | 3.06 s | 6.31 s | ALL PASS (blink gap 257 = bound, tight) |
| groundtruth_safety.py (re-transcription + monitors) | 7.21 s | 14.85 s | ALL PASS |
| groundtruth_sl.py (re-transcription + demonic falsifier) | 54.97 s | 104.39 s | ALL PASS (specs not falsified; demon-win sets as published) |

## Bridge receipts (in-process median-of-200, `tools/bridge_median.py`)

| fixture | gb10 X925 | gb10 A725 | epyc |
|---|---|---|---|
| delay_1 (smallest family) | 0.0517 ms | 0.1078 ms | 0.1483 ms |
| vga_1 (largest liveness) | 1.0851 ms | — | 2.6505 ms |

Silicon deltas (X925 → EPYC 2.4–2.9x; X925 → A725 ~2.1x) bound every
cross-box wall delta in this ledger: all class-wall motion vs published is
within these ratios + the ≤2% run spread ⇒ measurement-noise, no compute
regression anywhere.

## Fig6 bank (addendum schema)

`fig6-gb10.tsv` / `fig6-epyc.tsv`: 634 task rows each — columns
`instance_id section lane config outcome t_produce_ms t_check_ms t_total_ms
timeout_s silicon cert_bytes obligation_count state_space_size retrain_count
denominator_bits_max rational_count property_class`. Protocol:
`t_check_ms` = median of 5 in-process `check()` calls; `t_produce_ms` = the
same-box sweep receipts' per-instance producer wall; conjunction/pair tasks
sum their certificates. `denominator_bits_max`/`rational_count` counted by
monkeypatching the checker's own `dec_frac` (exactly what the replay
decodes). `state_space_size` = integer-point count of the manifest's
declared `state_wires` box (unit: states). `fig6-chains-*.tsv`: 634
chain-witness rows each (median-of-5 `ck_specchain.check`).

## Producer bit-identity

The producer suite re-emits all 885 certificates + 634 chains
bit-identical to its committed artifacts; the 245 public fixtures are
byte-identical up to the schema-id rename (5 bytes,
`…circuit-manifest-recurrence`, 245/245 verified).
