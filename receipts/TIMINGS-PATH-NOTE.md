# Timing receipts: out-of-tree by default, committed only when publishing

## The problem

`make supermartingales-glc` rewrote the committed receipt
`supermartingales/out/glc/GLC-TIMINGS.json` on every fully-green run.
The receipt embeds run-local noise by design (timestamp, load average,
wall-clock walls), so any casual verification or regression run dirtied
the working tree with timing churn that nobody should commit — a
verification run on a loaded machine would otherwise overwrite the
published numbers with worse ones.

A survey of the other sealed-container lanes (`semicon-glc`,
`lean-route-glc`, `gymnasium-glc`, `petri-glc`) found none of them
write any file at all from their recheck drivers; the defect was
singular to the supermartingales lane.

## The fix

`supermartingales/glc-recheck.ts` now routes the timing receipt by an
explicit environment variable:

- **default** (unset): the receipt is written under
  `build/supermartingales/` — inside the gitignored `build/` scratch
  area, same file-name rules as below — so verification runs leave
  the tree clean.
- **`GLC_PUBLISH_TIMINGS=1`**: the receipt is written to a committed
  path under `supermartingales/out/glc/`. Use this when a run's
  numbers are the ones the published tables should carry.

The published destination further depends on the run's machine
identity (`GLC_MACHINE_ID`, which must name a row of the repository's
machine registry, `MACHINES.md` — the driver refuses an unregistered
id at startup, and never guesses an id from the host):

- **`GLC_MACHINE_ID=<id>`** set: the receipt carries `machine.id` and
  publishes to the per-machine path
  `supermartingales/out/glc/GLC-TIMINGS.<id>.json`. Once any
  per-machine receipt is committed, the tables build selects the best
  (minimum) wall per row across the per-machine receipts and tags the
  cell with the winner — and stops reading the bare legacy file.
- unset: `machine.id` records `"unknown"` and the publish path stays
  the legacy bare `GLC-TIMINGS.json` (which can therefore never shadow
  a per-machine receipt). The tables build consumes the bare file,
  untagged, only while no per-machine receipt exists.

The committed files remain the artifacts the tables build reads
(`tables/gen_tables.ts`), and the reproduction path still regenerates
them, e.g. `GLC_PUBLISH_TIMINGS=1 GLC_MACHINE_ID=gb10 make
supermartingales-glc`. The receipt's format is unchanged apart from
the added `machine.id` field; the scratch-vs-published split is
unchanged.

## Verification

Both modes were exercised through the real make target (checker built
from the pinned checkout; 464/464 + 4,105/4,105 positives ACCEPT,
9/9 mutants in class, both runs):

- `make supermartingales-glc` — the working tree stayed clean
  (`git status` empty); the receipt landed at
  `build/supermartingales/GLC-TIMINGS.json`.
- `GLC_PUBLISH_TIMINGS=1 make supermartingales-glc` — the committed
  receipt was refreshed in place; the diff touched only the run-local
  fields (`date`, `loadavg_at_end`, `wall_s`, checker-binary hash) with
  the structure identical.
