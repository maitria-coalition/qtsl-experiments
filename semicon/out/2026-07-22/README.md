# Semicon re-run receipts (2026-07-22)

The re-run re-derives every published number of
`chapter/sections/semicon.tex` at the pinned revisions, banking wall-clock
and bound numbers. The verdict spine is `delta-ledger.md` in this directory.

## Files

- `delta-ledger.md` — the number|published|re-derived|cause-class ledger +
  the CANNOT/NO-LANE receipts (mtk producer, checker-at-tips, provider lanes).
- `fig6.tsv` — per-instance rows per the Fig6 addendum (mandatory columns +
  the optional `denominator_bits_max`/`rational_count` pair + one extra
  column `verify_kernel_median_ms_b200`). 44 rows: 20 certificates × 2 boxes
  + 2 producer probes × 2 boxes. Units: `state_space_size` = cover cells
  (modes); `timeout_s`=0 means no per-instance limit was in force.
  `t_check_ms` is the end-to-end single-job process wall (decode+verify),
  median of 200 reps for minifab rows / 7 reps for the larger rows;
  `verify_kernel_median_ms_b200` is the checker's own `--bench 200` in-process
  verify median. mimac7 `t_produce_ms` is empty because the analytic cover
  producer does not instrument per-row production; whole-run producer walls
  are in `delta-ledger.md`.
- `bench_gb10.json` / `bench_epyc.json` — raw timing rows incl. verdict
  lines (the `extras/*` rows are the rung-2.5 certificates; see ledger).
- `rerun_receipts_fluid_{gb10,epyc}.json`,
  `rerun_receipts_cover_{gb10,epyc}.json` — full producer re-run receipts
  from both boxes (byte-stability + checker verdicts inline).
- `rational_census.json` — per-cert exact-rational census (count + max
  denominator bit-length), the Lean-route regressor pair.

## Machines (see MACHINES.md)

- `gb10` (GB10, aarch64) — the published numbers' box class; bridge lane,
  `nice -n 10`, ambient shared load.
- `epyc` (n2d-highmem-16 spot, EPYC 7B13, europe-west1-b) — the
  batch CPU box; timed runs serialized under the `/workspace/timing.lock`
  flock convention (owner note written first), ambient load ≈1/16 cores at
  run time. Measured cross-silicon delta on this workload: GB10 single-core
  ≈1.7–1.9× faster than the EPYC spot core, uniform across producer walls
  and check kernels (this is the bridge receipt that lets cross-box deltas
  be classed measurement, with evidence).

## Method (reproduction)

1. Checker: `cargo build --release -p fluidcheck-cli` at the committed
   Cargo.lock (qtsl pin `c94af1f7…` — the last rev anywhere that carries the
   crate; see ledger).
2. Battery: `node --experimental-strip-types semicon/recheck.ts` — expect
   `16 positives + 13 mutants — ALL VERDICTS AS EXPECTED` (observed on both
   boxes).
3. Producers (historical instruments, upstream:
   `{minifab_fluid_producer,mimac_cover_producer}.py`,
   run in a scratch mirror of their expected layout): every emitted cert
   byte-identical to the committed ones on both x86_64 and aarch64; the
   96 lots/wk and setupadv-79 probes correctly emit no certificate; the two
   over-ceiling cover probes correctly refuse.
4. Timing: this directory's JSONs; the bench driver is 30 lines of
   subprocess+perf_counter orchestration (python as glue only;
   all computation under test is the Rust checker).
