# sec2 — re-conduct timing receipts (gb10)

Wall-clock receipts for the sec-2 corpus re-conducted through the
pinned toolkit producer on the `gb10` machine (see `MACHINES.md` at
the repository root) — the four-estate re-timing campaign's
Grace–Blackwell estate, pass 2, against the shared vintage sealed at
revision event 5.1 (the repository pin's revision event; the re-seal
record is `../RESEAL-5.1.md`). Pass 1 halted at the vintage probe
(its verdict lives on that pass's branch history); this directory
carries only pass-2 receipts. All runs serial, one timed process at a
time, at scheduler niceness 19 on a shared co-tenant machine; 10 repeats
per benchmark, plus one flagged warmup on the produce runs; the
expand pass runs once per container (as does the a40 set's — one
pass, not ten).

- `produce-gpu.json` — the full checked-production pipeline per
  repeat: spec parse, exact synthesis (float-proposing /
  exactly-certifying solves with iterative refinement and
  continued-fraction reconstruction), listing generation, forest
  merge, in-loop check, seal, and a byte-compare against the
  committed container (`../MANIFEST.json` hashes). GPU-proposing
  solver lane, `byte_identical` on every repeat. Each row carries an
  elapsed `wall_s` plus disjoint `produce_s` / `check_s` / `seal_s`
  subphases. These unversioned legacy receipts feed the table as the
  sum of the three disjoint subphases; the elapsed wall remains
  committed as protocol evidence.
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
  extracted tree count gated against the manifest's
  `distinct_trees`; one pass per container.
- `PROVENANCE.json` — machine, binary hashes, pinning masks (produce
  lanes on a fixed seven-core Cortex-X925 set; single-process cells
  on one X925 core), co-tenancy statement, protocol, and host load
  snapshots around each timed block — including the sibling receipt
  cells this estate ran outside this directory (staged-corpus GLC
  timings, certified-checker census, gym, lean-route glc, circuits).

The sweep wall-clock table's produce and geolog cells re-derive from
these receipts via `tables/gen_tables.ts`, which selects the best
median per benchmark and column across every `<machine>-retime/`
sibling automatically (winning lane named in the cell's machine tag,
`gb10` the GPU-proposing lane, `gb10/host` the CPU-proposing one).
Nothing here is transcribed by hand.
