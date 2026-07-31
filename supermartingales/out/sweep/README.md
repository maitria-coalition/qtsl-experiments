# out/sweep — the master table and its presentation artefacts

- `SWEEP.md` — the master reconciliation table (all 17 published rows + house
  alternates), methods, discrepancy log, receipts index.
- `SWEEP-NARRATIVE.md` — the narrative summary of the same campaign.
- `UPDATE-2026-07-20.md` — the post-campaign re-check (every bound
  re-verified through the then-current checker stack) with an old-vs-new
  checker wall-clock A/B.
- A typeset side-by-side comparison artefact (both of the paper's tables
  mirrored cell-for-cell — Farkas, program-agnostic and program-aware
  bounds with success ratios, architectures, all five timing columns —
  with the house columns appended) was produced during the campaign and
  stayed workshop-side; `SWEEP.md` carries the same content in table form.

Every cell in `SWEEP.md` was verified against the receipts JSONs (not the
markdown): exact fractions re-rounded independently; wave-B house times
recomputed from `ms_produce + ms_fastcheck`; wave-A totals re-summed from
per-row receipts; MC values re-pooled from the MC receipts; refutation
margins recomputed from the exact `lower_bound` fraction.
