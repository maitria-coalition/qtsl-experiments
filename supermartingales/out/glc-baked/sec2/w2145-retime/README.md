# sec2 — re-conduct timing receipts (w2145)

Wall-clock receipts for the sec-2 corpus re-conducted through the
pinned toolkit producer on the `w2145` machine (see `MACHINES.md` at
the repository root) — a CPU-only host, so only the CPU-proposing
solver lane appears (there is no `produce-gpu.json` sibling here). All
runs serial, one process at a time, inside verified zero-co-tenant
windows on this shared build host (window log in `PROVENANCE.json`);
10 repeats per benchmark, plus one flagged warmup on the produce runs.

- `produce-host.json` — the full checked-production pipeline per
  repeat on the CPU-proposing solver lane: spec parse, exact synthesis
  (float-proposing / exactly-certifying solves with iterative
  refinement and continued-fraction reconstruction), listing
  generation, forest merge, in-loop check, seal, and a byte-compare
  against the committed container (`../MANIFEST.json` hashes). Each
  row carries the `produce_s` / `check_s` / `seal_s` walls, the
  `byte_identical` verdict, and the synthesis telemetry.
- `geolog-shared.json` — `glc-check` wall over each benchmark's
  shared-subtree container, 10 repeats, accept verdict per repeat.
- `PROVENANCE.json` — machine identity, binary hashes, protocol, and
  the per-cell co-tenancy window log (load averages and job-arrival
  checks before/after every timed block).

The sweep wall-clock table's produce and geolog cells select the
fastest lane's median across every `<machine>-retime/` sibling with a
`MACHINES.md` row (`tables/gen_tables.ts`); this directory enters that
selection with the machine tag `w2145/host`. Nothing here is
transcribed by hand.
