# sec2 — re-conduct timing receipts (a40)

Wall-clock receipts for the sec-2 corpus re-conducted through the
pinned toolkit producer on an `a40` machine (see `MACHINES.md` at the
repository root) — the same GPU model the published pipeline reports,
at the published protocol's repeat count. All runs serial, one process
at a time; 10 repeats per benchmark, plus one flagged warmup on the
produce runs.

- `produce-gpu.json` — the full checked-production pipeline per
  repeat: spec parse, exact synthesis (float-proposing /
  exactly-certifying solves with iterative refinement and
  continued-fraction reconstruction), listing generation, forest
  merge, in-loop check, seal, and a byte-compare against the
  committed container (`../MANIFEST.json` hashes). GPU-proposing
  solver lane. Each row carries the `produce_s` / `check_s` /
  `seal_s` walls, the `byte_identical` verdict, and the synthesis
  telemetry.
- `produce-host.json` — the same protocol on the CPU-proposing
  solver lane. The proposer changes; the exact certification and the
  emitted bytes do not (`byte_identical` on every repeat, both
  lanes).
- `geolog-shared.json` — `glc-check` wall over each benchmark's
  shared-subtree container, 10 repeats, accept verdict per repeat.
- `expand.json` — `qtsl2bl --expand` wall per container (the
  extraction inverse documented in `../README.md`), with the
  extracted tree count.
- `PROVENANCE.json` — machine, binary hashes (`onesys-driver` is the
  serial run driver; `glc-check` and `qtsl2bl` are the pinned
  toolkit builds), and protocol.

The sweep wall-clock table's produce and geolog cells for the
re-conducted rows re-derive from `produce-gpu.json` and
`geolog-shared.json` (`tables/gen_tables.ts`); nothing here is
transcribed by hand.
