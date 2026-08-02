# sec2 — shared-container check timing receipts (axion)

Wall-clock receipts for the geolog fast-path checker over the sec-2
shared-subtree containers on an `axion` machine (see `MACHINES.md` at
the repository root) — the re-sealed (revision event 5.1) vintage, at
the pinned checker build. All runs serial, one process at a time,
pinned to a single core; 10 repeats per benchmark, interleaved by
repeat index.

- `geolog-shared.json` — `glc-check` wall over each benchmark's
  shared-subtree container, 10 repeats, accept verdict per repeat.
  Every run's claim tally was asserted against `../MANIFEST.json`
  before its row was recorded, and every container's sha256 was
  verified against the manifest before the block ran.
- `PROVENANCE.json` — machine, checker hash, and protocol.

This directory carries check receipts only: the machine has no GPU
and no producer lane was timed here, so there are no `produce-*.json`
receipts (the table generator's best-of selection reads whatever
receipt shapes are present).
