# Atari re-run receipts (2026-07-22) — units and provenance

- `gb10/` — GB10 Grace (aarch64), the published-protocol machine; runs
  nice'd, box in normal shared service (published condition was "2–10
  sibling checkers"; these runs were 1 timed check at a time + background
  load). `laneA-*` = this repo's committed claims + the export-pinned AIG
  generation; `laneB-*` = the upstream tips claims + tips AIG generation.
- `epyc/` — GCP n2d-highmem-16 (EPYC 7B13, x86_64), batch CPU box,
  flock-serialized timed runs, otherwise idle. Same lane naming.
  `bench-*.txt` = replaycheck bench-verb calibration.
- `fidelity/` — posture-2 differential re-run (gb10; atrcdiff JSONs +
  summary). Not part of the trust story; ledger rows only.
- `gb10/mutants_receipt-rerun-2026-07-22.json` + `mutants-ceremony.log` —
  the 2026-07-22 latest-tips mutant battery (11 mutants + 2 controls, jobs=4,
  nice'd). The COMMITTED `../../certs/mutants/mutants_receipt.json` remains
  the published artifact; this is the re-run beside it.
- `fig6.tsv` — per (game × lane × machine × claim-generation) rows per the
  fig6 addendum. Unit notes: `obligation_count` = frames compared
  (31,465 = Σ (1000+k) over k∈{0..30}); `t_produce_ms` null (claim-bundle
  producer is the migrating python layer, deliberately not re-run; trace
  witnesses are pinned inputs); `state_space_size` = 2^latches;
  `config` = replaycheck bit-parallel 31-lane u64.
- ROM-derived inputs (AIGs, ROM hex, trace bytes) are referenced by sha256
  ONLY (upstream provenance retained privately; license law upheld —
  nothing ROM-derived is committed here).
