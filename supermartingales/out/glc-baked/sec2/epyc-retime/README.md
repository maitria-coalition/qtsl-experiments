# sec2 — re-conduct timing receipts (epyc)

Wall-clock receipts for the sec-2 corpus re-conducted through the
pinned toolkit producer on an `epyc` machine (see `MACHINES.md` at
the repository root) — the four-estate re-timing campaign's dedicated
x86 CPU estate. All runs serial, one process at a time; 10 repeats
per benchmark, plus one flagged warmup on the produce runs. The box
is a dedicated single-tenant instance (no co-tenant load); load
snapshots per cell block in `PROVENANCE.json`.

- `produce-host.json` — the full checked-production pipeline per
  repeat (spec parse, exact synthesis, listing generation, forest
  merge, in-loop check, seal, byte-compare against the committed
  container hashes in `../MANIFEST.json`), CPU-proposing solver
  lane. No GPU on this estate, so there is no `produce-gpu.json`.
- `geolog-shared.json` — `glc-check` wall over each benchmark's
  shared-subtree container, 10 repeats, accept verdict per repeat.
- `PROVENANCE.json` — machine, binary hashes, protocol, and load
  snapshots.

(Receipt files land beside this README as cells complete; this stub
commits first so the campaign's work is never reaper-bait.)
