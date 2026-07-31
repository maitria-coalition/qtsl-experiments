# Circuits section — delta ledger (re-run on latest tips, 2026-07-22)

Campaign pins (historical provenance: the exact development-tree revisions
used for this re-run, recorded for the ledger's own audit trail — to
reproduce these numbers today, use this repository's pinned make targets,
which fetch and check out everything they need): mtk
`184fa3014389050289a6ec4af055aa517eec0af6`, geolog-alpha
`8c2c14a54da2e205a562ee96743d1571174c006e`, qtsl main
`c0099c932aa49ba8295c49a84245ce741d2dc8cb` (lane b) vs committed replay pin
`c94af1f735254b91298202aa8212a2b9004542b8` (lane a), kernels
`b0143fa3a27ec60bfefe2c992b5be0c8be7b0bd5`, qtsl-experiments as committed
here. Producer layer re-run from the producer suite's source tree
(pre-migration; see provenance note at bottom).

Ledger keys on the latest-tips lane. Cause classes:
{producer-improvement, measurement-noise, REGRESSION} + `prose` for
documented-count/range corrections where every artifact is green and the
published SENTENCE, not the suite, moved.

| # | number | published | re-derived (latest tips) | cause-class |
|---|---|---|---|---|
| 1 | completions, all classes | 634/634 (219 L + 206 S + 209 SL), 885 certs, 634 chains | 885/885 certificates ACCEPT + 634/634 chain witnesses ACCEPT, on BOTH gb10 and epyc, at BOTH qtsl pins | no delta |
| 2 | baseline counts | best single tool 554/634; virtual-best union 617/634; 17 portfolio orphans | `make tables` regen from pinned artefact arrays: verify[nmc-table] MATCH (7 rows) | no delta |
| 3 | whole-suite replay < 1 s single-core | cert checks 78/195/171 ms (L/S/SL) + chain replay 82/118/150 ms, GB10 | GB10 Cortex-X925 core: 76.7 / 195.1–196.5 / 168.4–171.8 ms + 81.8 / 115.6–116.8 / 149.3–151.2 ms (3 runs each) ≈ 0.79 s total — within ~1% of every published wall | measurement-noise |
| 3b | same, batch CPU box | (not published) | epyc (EPYC-7B13): 204.2 / 476.2 / 439.2 + 221.0 / 297.8 / 384.8 ms ≈ 2.02 s — "< 1 s" is GB10-performance-core silicon-specific | measurement-noise (silicon class; see note N1) |
| 4 | certificate sizes "0.4–4 KB" | 0.4–4 KB | per-cert 415–7387 bytes: 865/885 within 0.4–4 KB; the 20 above are all s1/s2_vga safety windows (max s2_vga_10 = 7387 B, listed at 7387 in the VENDORED published receipt table itself) | prose (range understated at writing; fixed in the chapter text) |
| 5 | per-check "tens of µs to a few ms" | tens of µs – few ms | bridge medians-of-200: delay_1 51.7 µs, vga_1 1.085 ms (GB10-X925); worst vendored-table cell s2_vga_10 5.11 ms reproduced-class | no delta |
| 6 | size + check constant along ladders, coeff widths ~log N | claimed | confirmed per family, e.g. delay bytes 415→435 across 16 rungs, check 0.052–0.053 ms; gray3 bytes 1076→1121 across CBITS 8→18; check-ms flat ≤ 2% within every liveness family | no delta |
| 7 | mutant batteries 16+3 / 14+3 / 14+9 | as printed | actual: liveness FIXTURE battery 17 attacks + 4 controls (source suite; in-repo driver had ported 16+3 — blink pair P/M added post-port, ported to recheck.ts this branch → 17+4); safety 13 attacks (8 fixture + 5 chain) + 3 controls — published "14 (9 fixture)" was a miscount born in the vendored rider; SL 14 attacks (8+6, two cross-layer) + 9 controls EXACT. Additionally the liveness CHAIN battery (8 attacks + 4 controls) appears in no published count. ALL 52 attacks refused at the attacked layer, all 20 controls pass, on both boxes | prose (counts; zero refusal-behavior delta) |
| 8 | invention-time table (~75 min campaign; their 5 h/task, ~9000 machine-hours; no ratio printed) | as printed | not machine-re-derivable (session-log-derived historical walls, bracketed-estimate convention intact); paper-side numbers ride the unchanged pin (f2bd71c); no-ratio discipline intact — a matched-jobs comparison remains the mechanism | not-re-derivable (historical) |
| 9 | paper-side findings (VGA aggregate slip; '24 appendix 177-vs-180; delay spec-source disagreement; I2C premise) | prose facts | inputs untouched (same pin sha); incidental re-confirmations: gen_tables' per-family-array path MATCH; chain mutant C3 exercises the delay repair record (REJECT S1 as designed); groundtruth_sl demonic falsifier re-ran PASS (I2C freeze states in demon-win set) | no delta |
| 10 | producer re-derivation | committed fixtures | the producers re-emit ALL 885 fixtures + 634 chains BIT-IDENTICAL (public 245 byte-identical modulo the 5-byte schema-id prefix rename, verified byte-exact after rename on 245/245) | no delta |
| 11 | checker lane a vs lane b | python reference at pin | `checkers/circuits` BYTE-IDENTICAL c94af1f7 ↔ c0099c932 (verified on both boxes); the README's "replacement in progress upstream" has NOT landed — no lane change to ledger | no delta |
| 12 | groundtruth / sim-tightness | sim-tight where measured | all three instruments ALL PASS on both boxes (GB10: 3.1 / 7.2 / 55.0 s; EPYC: 6.3 / 14.9 / 104.4 s); blink worst-case gap 257 = bound 257 (tight) | no delta |

**REGRESSIONS: none.** No refused certificate, no moved completion count, no
moved baseline count, on either silicon, at either pin.

## Census flag — resolved mechanically

`circuits/recheck.ts` replays: **245 fixtures / 219 tasks**
(count cross-verified against SWEEP-RECEIPTS.md by the driver itself) + the
ported fixture mutant battery. The top-level README's "220 certificates over
all 194 benchmark tasks" was STALE (pre-blink; blink '25 added 25 tasks/25
certs) — since fixed. The chapter's "What is not claimed" paragraph
("reproduction spine complete for the pure-liveness class only; safety and
combined receipts vendored; fixtures arrive with the producer layer's
migration") is CORRECT as committed: fixtures_safety (222), fixtures_sl
(418), all 634 chain sidecars, ck_safety/ck_specchain, and the
safety/SL/chain batteries exist in the producer suite's layer only. This
ledger's full-suite rows were re-derived from that layer and are exactly
what the producer-layer migration will bring in-repo.

## Notes

- **N1 (measurement guidance, new finding)**: the GB10 (Grace) is
  heterogeneous — Cortex-X925 3.9 GHz performance cores + Cortex-A725
  2.8 GHz efficiency cores. Pinning an efficiency core reproduces every
  number ~2.15x slower with a shifted worst-instance (166 ms liveness
  check; worst PWM_6 2.40 ms). The published receipts' rider ("GB10,
  single thread") is core-class-silent; all GB10-class receipts should pin
  and name the core class. Bridge triple, delay_1 median-of-200: 51.7 µs
  (X925) / 107.8 µs (A725) / 148.3 µs (EPYC-7B13).
- **N2 (mtk producer lane)**: CANNOT-YET at mtk `184fa3014` — no producer
  surface for any of the 16 NMC families (repo-wide grep: only incidental
  matches). Missing plumbs, per the migration's own scope: (i) TS species
  schemas (`circuit-manifest-recurrence` v0 + safety-window v0), (ii) the
  16 family producers × three property classes, (iii) the
  until-introduction rewrite-chain producer (ltlspec + produce_chains*),
  (iv) the sweep/receipt driver on mtk's public API, (v) mutant emitters +
  groundtruth instruments. An upstream producer-timing lane is the
  named owner; no pilot was possible (nothing invocable).
- **N3 (geolog provider lanes)**: NO-LANE, CPU and GPU — the 50-relation
  qtsl-geolog-provider roster at `c0099c932` (applicability / atomic /
  recompute-kit / mask / system-spec / problog / conclusion-form / lattice
  / enclosure / carillon families) contains no relation receiving a
  lowering of the circuit-manifest, safety-window, or spec-chain species;
  these are standalone-CBOR checker species (C0–C6 / K0–K5 / S0–S5), kin
  to the gymnasium flcheck NO-LANE verdict. No GPU run was needed.
- **N4 (Fig6 bank)**: 634 task rows × 2 boxes (`fig6-gb10.tsv`,
  `fig6-epyc.tsv`) per the addendum schema incl. optional
  `denominator_bits_max` + `rational_count` (counted by the checker's own
  `dec_frac`, i.e. exactly what the replay decodes) + `state_space_size`
  (state-wire box integer-point count); chain-replay walls separately
  (`fig6-chains-*.tsv`, 634 rows × 2). Protocol in RECEIPTS.md.
- **N5 (chain walls total)**: whole-suite chain replay 219+206+209 = 634
  witnesses in ~348 ms (81.8 + 116.4 + 149.9, GB10-X925) / ~904 ms (EPYC).
  The six published class walls sum to 794 ms; the re-derived six sum to
  ~790 ms on GB10-X925 — "under a second, certificates AND chains" holds
  exactly as printed on its own silicon, and runs ~2.02 s on the batch
  EPYC box (row 3b).
