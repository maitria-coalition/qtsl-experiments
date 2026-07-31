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

- **default** (unset): the receipt is written to
  `build/supermartingales/GLC-TIMINGS.json` — inside the gitignored
  `build/` scratch area — so verification runs leave the tree clean.
- **`GLC_PUBLISH_TIMINGS=1`**: the receipt is written to the committed
  path `supermartingales/out/glc/GLC-TIMINGS.json`, exactly as before.
  Use this when a run's numbers are the ones the published tables
  should carry.

The committed file remains the artifact the tables build reads
(`tables/gen_tables.ts`), and the reproduction path still regenerates
it: `GLC_PUBLISH_TIMINGS=1 make supermartingales-glc`. The receipt's
format and content are unchanged in both modes; only the destination
moves.

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
