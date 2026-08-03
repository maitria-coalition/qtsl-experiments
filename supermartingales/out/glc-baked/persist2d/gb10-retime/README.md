# persist2d — re-conduct timing receipts (gb10)

Wall-clock receipts for the `persist_2d` re-conduct lane
(`../MANIFEST.json`: the uniform twin as one whole-twin shared-subtree
container, the gaussian twin as four sorted-quarter containers) on the
`gb10` machine (see `MACHINES.md` at the repository root), against the
revision-event-7 vintage the repository pin serves. All runs serial,
one timed process at a time, at scheduler niceness 19 set absolutely
(`os.setpriority`) with the effective value read back from the process
(19), on a shared co-tenant machine; host 1-minute load averages
sampled before and after every timed row and recorded in the produce
rows and in `PROVENANCE.json`.

- `produce-host.json` — the re-conduct pipeline per repeat and per
  container: the committed reference listings through the shared-subtree
  producer (forest merge with structural hash-consing, the in-loop
  check, deterministic sealing), CPU lane, pinned to seven Cortex-X925
  cores, 1 flagged warmup + 10 measured repeats per container, and a
  byte-compare against the committed container sha on EVERY repeat
  (`byte_identical` true throughout). There is no `produce-gpu.json`:
  this lane has no proposer at all — the twins' certificates are a
  synthesis genus the toolkit producer does not yet cover, so the row's
  produce cells time the re-conduct of the committed corpus (the sweep
  table's mark a), not fresh synthesis. Each row carries the driver's
  whole conduct wall (`wall_s`) plus its disjoint `produce_s` /
  `check_s` / `seal_s` subphases; the version-1 receipt feeds the table
  by `wall_s`.
- `geolog-shared.json` — `glc-check --baked-cert --expect-claims` wall
  over each container, single-container invocations (each pays the
  checker's whole per-process vocabulary derivation), one Cortex-X925
  core, 10 repeats, accept verdict gated per repeat with the exact
  per-relation claim tally (4,800 per container).
- `PROVENANCE.json` — machine, binary hashes (driver and the checker
  built from the pinned qtsl checkout at the repository's `QTSL_REV`),
  pinning masks, niceness read-back, co-tenancy statement, protocol,
  and host load snapshots around each timed block.

The sweep wall-clock table's cells re-derive from these receipts via
`tables/gen_tables.ts`, which folds the per-container medians into
per-row serial totals (the gaussian row is the sum of its four
quarter-containers' medians; the uniform row is its one container's)
and then selects the best machine/lane per row across every
`<machine>-retime/` sibling. Nothing here is transcribed by hand.
