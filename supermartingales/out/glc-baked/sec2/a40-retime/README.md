# sec2 — re-conduct timing receipts (a40)

Wall-clock receipts for the sec-2 corpus re-conducted through the
pinned toolkit producer on an `a40` machine (see `MACHINES.md` at the
repository root) — the same GPU model the published pipeline reports,
at the published protocol's repeat count, against the shared vintage
sealed at revision event 5.1 (the repository pin's revision event; the
re-seal record is `../RESEAL-5.1.md`). All runs serial, one process at
a time; 10 repeats per benchmark, plus one flagged warmup on the
produce runs; the expand pass runs once per container.

- `produce-gpu.json` — the full checked-production pipeline per
  repeat: spec parse, exact synthesis (float-proposing /
  exactly-certifying solves with iterative refinement and
  continued-fraction reconstruction), listing generation, forest
  merge, in-loop check, seal, and a byte-compare against the
  committed container (`../MANIFEST.json` hashes). GPU-proposing
  solver lane. Each row carries elapsed `wall_s`, disjoint
  `produce_s` / `check_s` / `seal_s` subphase walls, the
  `byte_identical` verdict, and synthesis telemetry. These unversioned
  legacy receipts feed the table as the sum of the three disjoint
  subphases; the elapsed wall remains committed as protocol evidence.
- `produce-host.json` — the same protocol on the CPU-proposing
  solver lane. The proposer changes; the exact certification and the
  emitted bytes do not (`byte_identical` on every repeat, both
  lanes).
- `geolog-shared.json` — `glc-check` wall over each benchmark's
  shared-subtree container, 10 repeats, accept verdict per repeat
  (each accept additionally required the discharged-claim tally to
  equal the manifest's).
- `expand.json` — `qtsl2bl --expand` wall per container (the
  extraction inverse documented in `../README.md`), with the
  extracted tree count; one pass per container.
- `PROVENANCE.json` — machine, binary hashes (`onesys-driver` is the
  serial run driver; `glc-check` and `qtsl2bl` are the pinned
  toolkit builds), protocol, and host load snapshots around each
  timed block.

The sweep wall-clock table's produce and geolog cells for the
re-conducted rows re-derive from `produce-gpu.json`,
`produce-host.json`, and `geolog-shared.json`
(`tables/gen_tables.ts`): each produce cell shows the faster lane's
median, with the winning lane named in the cell's machine tag (`a40`
the GPU-proposing lane, `a40/host` the CPU-proposing one). A sibling
`<machine>-retime/` directory with the same receipt shapes enters the
same selection automatically, provided the machine has a row in
`MACHINES.md`. Nothing here is transcribed by hand.
