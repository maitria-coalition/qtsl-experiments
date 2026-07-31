# recheck-2026-07-22 — re-run receipts for the supermartingales section

Re-run of every published number in `chapter/sections/supermartingales.tex`
on latest tips (2026-07-22). Read `DELTA-LEDGER.md`
first — it is the report spine; everything else here is its evidence.

| file | what |
|---|---|
| `DELTA-LEDGER.md` | number-by-number delta ledger + producer/provider audits |
| `lane-a-pinned-c94af1f7-gb10.json` | TS reference lane @ committed Makefile pin (regression control) |
| `lane-b-tips-c0099c9-gb10.json` | TS reference lane @ qtsl main c0099c9 (tips lane) |
| `bridge-gb10-smallest.json` / `epyc-replay/bridge-epyc-smallest.json` | smallest-cert median-of-200 bridge pair (silicon delta) |
| `outlier-gb10.json` | uniform-twin full-set walls, gb10 @ tips |
| `epyc-replay/replay_results.json` | full 28,569-file corpus replay, epyc, tips A/B adebdec18 |
| `epyc-replay/perfile-tips.tsv.zst` | raw per-file fastcheck lines (28,569) |
| `epyc-replay/ts-lane-tips-c0099c9-epyc.json` | TS reference lane on epyc (cross-section CPU-lane box) |
| `fig6-supermartingales.tsv.zst` | addendum-shaped per-instance rows (obligation_count unit: fastcheck proof nodes) |
| `wave0-rerun/` | wave-0 producer re-derivation (results.json semantically bit-identical to golden; MC cross-checks added) |

Not here (retained upstream): the dyadic re-emission of the uniform twin
(194×/248× check-time reduction, bound bit-identical).
