# Delta ledger — supermartingales section re-run (2026-07-22)

The re-run re-derives all of this section's experiments on latest tips,
banking wall-clock + bound numbers. Section: `chapter/sections/supermartingales.tex`.

## Pins actually used

| repo | rev | role |
|---|---|---|
| qtsl-experiments | as committed here | this tree |
| qtsl (committed pin, lane a) | `c94af1f735254b91298202aa8212a2b9004542b8` | Makefile QTSL_REV replay — regression control |
| qtsl (latest tips, lane b) | `c0099c932aa49ba8295c49a84245ce741d2dc8cb` = main | tips lane (an earlier pin record `fc29210dd` was an erratum — a stale branch tip; corrected mid-run, lane b redone at true main) |
| fastcheck + producer source tree (upstream, private) | `f2a9d9891` (fastcheck dir last commit `cbcfffde7`) | bernfmt corpus lane + producer re-runs |
| fastcheck A/B "old" | `adebdec18` (the 2026-07-20 update's "current") | rev-delta control |
| mtk | `184fa3014389050289a6ec4af055aa517eec0af6` | producer-capability audit only (section below) |
| geolog-alpha | `8c2c14a54da2e205a562ee96743d1571174c006e` | fast-path plumbing audit only |

Checker-code note: `checkers/supermartingales/` (read_rows.ts, bl_check.ts,
check_rows.ts, qpoly.ts, theories/) is **byte-identical** between the
committed pin and qtsl main c0099c9 — only `testdata/` moved. Lane a/b
verdict equality is therefore expected by construction, and observed.

## Delta ledger (`number | published | re-derived | cause-class`)

| # | number | published | re-derived (this leg) | cause-class |
|---|---|---|---|---|
| 1 | 17/17 rows strictly improved | all 17 below best published sound column | **REPRODUCED** two independent ways: (i) TS reference lane re-accepts all 4,569 shipped row-certificates at both qtsl pins (bounds are content-fixed cert bytes; a moved bound could only appear as a refusal — zero refusals); (ii) `gen_tables.ts` regenerated sweep-table.tex + sweep-table-full.tex from receipts with its cell-for-cell verification pass green (exit 0) | — (no delta) |
| 2 | bounded-counter margins 1.01×–1.24× | 1.01×–1.24× | same receipts, same cells (gen_tables verify green) | — |
| 3 | random-walk gaps 7–58 orders of magnitude | 7–58 orders | same receipts, same cells | — |
| 4 | the refutation: Farkas 0.0181 (faulty_uniform) < true p ∈ [0.0184458, 0.0185556] | interval endpoints 150486/8158267-side lower witness `429788929984053075589523987651/23300061339378910715200000000000` ≈ 0.01844583, upper `150065/8087357` ≈ 0.01855551 | interval re-verified: the 30 faulty_uniform bracket certs re-accept in both lanes; endpoint decimals recomputed exactly; 0.0181 < lower endpoint confirmed as exact-ℚ comparison | — |
| 5 | 13/13 published Farkas values reproduced as exact-ℚ LP optima | fingerprint scorecard | **wave-0 producer re-run, 2026-07-22, semantically bit-identical**: every Farkas exact optimum, exact certificate vector, exact p_true rational, ceil-4 match flag, and envelope verdict equal to the golden receipts (`wave0-rerun/` here; timings + newly-added MC cross-checks are the only diffs). MC cross-checks (--mc-all, seeded, jax-CPU): consistent (e.g. faulty_concave z −1.03; cliff 0 hits / 10⁶ at p_true 8.6e-10) | — (producer re-derivation, deterministic) |
| 6 | corpus replay: 28,569 files, >57,000 checks, zero refusals, no bound moved | claim carried from 2026-07-20 update (57,138 checks) | **RE-ESTABLISHED at latest tips**: corpus enumeration reproduced exactly (23+3+29+3+29+5+4800+19200+372+4105 = 28,569); full replay on epyc, fastcheck at tips (f2a9d989-era tree) A/B vs adebdec18 — **116,670 checks this run, zero refusals, zero errors** (per-rep accepts==n asserted; `epyc-replay/replay_results.json`). Certs are content-fixed bytes, so no bound moved. Rev delta adebdec18→HEAD (15 fastcheck/bernfmt commits): flat everywhere (uniform twin 151.6→150.0 s, gaussian 7.06→7.01 s, cliff 0.079→0.078 s — within jitter) | measurement-noise (rev delta nil) |
| 7 | per-row check envelope 1.4 ms–4 s one CPU core | smallest-cert floor | gb10 bridge (same silicon class as published): median-of-200 single-file walls 1.68 ms (min 0.91 ms; in-process parse+check 61 µs — process spawn dominates); envelope shape unchanged | measurement-noise |
| 8 | the 136 s outlier (uniform persist_2d twin, large-denominator ℚ) | 136 s (07-20 re-check: 133.6/135.9 s min-wall gb10) | gb10 @ tips: **130.66 s** min-of-2 (`outlier-gb10.json`; independent same-day confirmation 132.16 s min-of-3 by the re-emission run) — ranges overlap the 07-20 pair | measurement-noise (mild favorable drift) |
| 9 | "a cheaper re-emission identified in the repository" | identified only (dyadic `scale_bits` route; identified 07-20, open since) | **EXECUTED in this re-run** (upstream re-emission run): `--dyadic-bits 32` re-emission of all 4800 certs, bound **bit-identical** `58920851522213/588624739134464`, 4800/4800 accept ×3 + 3/3 mutants refused + 250/250 reference checks; batch wall **132.16 s → 0.68 s (194×)**, per-file median 28.33 ms → 0.114 ms (248×), cert bytes 40.5 MB → 4.96 MB, min_tau 3,144 digits → 22; emission end-to-end 65.3 s (gb10). The ~100× prediction was conservative 2×; the dieted set now checks FASTER per file than the gaussian twin (0.114 vs ~0.25 ms). **The published outlier caveat can be retired** once that re-emission lands: the one row of 17 whose check was same-order as their pipeline drops to sub-second | producer-improvement |
| 10 | contrast row: their 5.8–254.5 s (A40 GPU + Z3, learn-and-verify) vs our re-check; **no ratio printed** | qualitative contrast, jobs differ | **still no fair ratio printable at these tips** — see the producer-capability audit below: mtk at `184fa3014` has no producer lane for any of the 17 row families, so no invention+check wall exists to match their learn-and-verify job. The published caveat STANDS with its reason re-verified | — (fairness discipline) |

## Producer-capability audit: CANNOT-YET-RE-RUN-THROUGH-MTK, per row family

mtk at `184fa3014` (audited: crates/mtk/src/*, gymdp/, cegis.rs, witness-types,
mtk-eventloop): producer tiers + kernels exist (bern/cheb/lp1st/ipm/eig,
gymdp exact-DP with CBOR cert emission byte-identical to the *Gymnasium*
python corpus, cegis e2d exact-cert.v0 loop), but none targets this
section's certificate species (BL rows-listing / retired-bernfmt
supermartingale certs). Per family, the missing plumb:

1. **exact-VI / chain rows** (fixed_loop, faulty_loop(+as_listed),
   faulty_rare, faulty_easy1, faulty_ndecr, faulty_walk(+det12),
   cliff_crossing, repulse100 family, faulty_concave, faulty_varying(+last_step)):
   no exact absorbing-chain elimination / VI-export strategy in mtk, and no
   BL rows-listing emitter (gymdp's emitter targets the Gymnasium cert
   species). Producers remain upstream python (`chain.py`,
   `vi_export.py`, and kin).
2. **ReLU-hypernet envelope rows** (faulty_marbles, faulty_unreliable,
   faulty_regions): mtk cegis.rs drives the e2d certificate family, not the
   BL hypernet species these rows use; no strategy driver + no emitter.
3. **persist_2d twins**: no 2D bilinear-lattice producer, no erf-enclosure
   emission (upstream `persist2d/*.py`).
4. **faulty_uniform two-sided bracket**: exact-moment bracket construction
   exists only upstream.

One real producer wall DOES exist in this re-run: the dyadic re-emission of
the uniform twin (upstream python producer, per-instance walls banked in the
fig6 lane by the re-emission) — a producer timing, but not an mtk one.

## 136 s outlier through the provider fast path: CANNOT-YET, precisely

Brief question: route the outlier row + heaviest decile through
qtsl-geolog-provider (CPU) / qtsl-geolog-provider-gpu at latest tips.
Audit result (qtsl main c0099c9 + geolog-alpha 8c2c14a):

- the provider's claimed roster (43 relations: blapplic Farkas/Syzygy/PSD
  applicability, atomic-compare, masks, system-spec, ProbLog, conclusion
  forms, lattices, enclosure) **does not include a BernCone
  grid-nonnegativity relation** — and the uniform twin's entire 136 s is
  BernCone arithmetic on ~10k-bit rationals. The heavy obligation species is
  outside the provider's vocabulary at these tips.
- no rows.txt→engine ingestion verb exists: the `geolog` CLI consumes
  `.geolog` theory text + instance blocks (no ground-provider wiring at CLI
  level); no binary links geolog-chase + qtsl-geolog-provider into a
  per-certificate verdict surface (provider consumed only via seam tests +
  extracted Phase-V plane benches).
- therefore GPU lane for THIS row: nothing to run — no GPU box was needed
  (the empty lane is a verified finding, not a skipped job).

Missing-plumb list:
(a) BernCone side-condition relation (+ evaluator) in qtsl-geolog-provider,
(b) a listing→commit ingestion + chase-check driver binary,
(c) provider-gpu lane for (a) (the ~10k-bit rational BernCone batch is
exactly the "large certificates" shape the upstream RNS machinery was built for).

What IS the fast path today: the dyadic re-emission (ledger row 9) — it
attacks the denominators themselves rather than the checker.

## Cross-silicon bridge (published numbers were GB10 single-core)

| instrument | gb10 (aarch64) | epyc n2d (x86) | ratio |
|---|---|---|---|
| smallest cert, median-of-200 process wall | 1.68 ms (min 0.91; in-process 61 µs) | 2.26 ms (min 2.04; in-process 75 µs) | GB10 ≈ 1.2–1.3× faster |
| uniform twin, full 4800, min batch wall @ tips | 130.66 s | 149.99 s | GB10 1.15× faster |
| gaussian twin, full 19200, min batch wall @ tips | 4.04 s (07-20, prev binary) | 7.01 s | GB10 ~1.7× faster (parse-heavy row) |
| TS reference lane, all 28 sets, sum of set walls | 6.88 s | 12.65 s (verdict profile identical: 4,571 accepts incl. 2 recorded-only, 23/23 mutant refusals, 0 errors) | — |

So epyc-vs-published wall deltas of this magnitude are silicon, not
regression — classed measurement-noise with this table as the evidence.
(Load notes: epyc runs under flock timing-lock, residual setup load
~1.3–2.3 recorded in receipts; gb10 load 4.7–7.5, nice −10, min-of-reps.)

## Fig6 banking

Per-instance rows per the addendum columns: `epyc-replay/perfile-tips.tsv.zst`
(raw fastcheck per-file lines, rep 1 of tips) + `fig6-supermartingales.tsv.zst`
(addendum-shaped, 28,569 rows; `instance_id` is row-prefixed —
`<row>/<file>` — because the wave-A duplicate-certification sets share
basenames with their sec-2 twins). Producer-side walls
exist only for the re-emit leg (uniform twin, config `dyadic-scale_bits`);
all other rows are checker-lane rows with `t_produce_ms` null (producers not
re-runnable through mtk at these tips — see the audit above). `cert_bytes` +
`obligation_count` (= fastcheck node count; unit: proof nodes) banked per
instance. Denominator-bit-length per instance was NOT in the addendum's
final column list and is not banked (the uniform twin's ~10k-bit receipts
note stands in; flagged for the scaling-law analysis if wanted).
