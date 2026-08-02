# sec2 shared vintage — re-seal at theory revision event 5.1

The eight shared-subtree containers in this directory previously
sealed at the closure revision that preceded revision event 5.1
(qc `22fc4290…`); the checker built at the repository's `QTSL_REV`
pin refused them with `unknown_prereq` on the superseded closure id.
This re-emission re-seals the corpus at revision event 5.1
(qc `6862edc88ff1d788550d196078a28cb2fe971f10d478a5e32800f2892a7292b9`)
— a pure re-key: every container body is byte-identical to its
predecessor; differences are confined to the envelope's closure-id
field and the seal/commit-id ranges. This is the same motion the
per-listing corpora made when the pin converged onto revision
event 5.1.

## Producers

- Seven chain benchmarks: the fresh-synthesis re-conduct driver
  (`supermart-synth`, host lane; emission bytes are lane-independent,
  established by the prior A40/host cross-lane byte-identity
  receipts).
- `repulse100_uniform` (continuous noise; outside the fresh-synthesis
  genus): the listing-replay forest producer (`supermart-share
  --mode shared`) over the committed listing corpus
  (`../../sec2/rows-certs.tar.zst`).
- Cross-producer witness: `repulse100_uniform__discrete` was emitted
  by BOTH paths and the two emissions are byte-identical in full
  (envelope included).

## Per-container verification

Byte-diff ranges are 0-based offsets, old vs new, from `cmp -l`;
every differing byte falls in the closure-chid field `[17–48]` or the
seal/commit-id region `[59–270]`; bodies (all bytes past the
envelope) are untouched, and file lengths are equal.

| container | bytes (=) | diff ranges (0-based) | checker at pin | claims |
|---|---|---|---|---|
| cliff_crossing.glc | 136033 | 17–48, 59–154, 215–246, 270 | ACCEPT | 2188/2188 |
| faulty_concave.glc | 5042 | 17–48, 59–154, 215–246, 270 | ACCEPT | 76/76 |
| faulty_varying.glc | 5066 | 17–48, 59–92, 94–154, 215–218, 220–246, 270 | ACCEPT | 76/76 |
| faulty_varying__last_step.glc | 5077 | 17–48, 59–154, 215–246, 270 | ACCEPT | 76/76 |
| repulse100.glc | 9773 | 17–48, 59–154, 215–246, 268 | ACCEPT | 207/207 |
| repulse100_2d.glc | 240974 | 17–48, 59–107, 109–116, 119–154, 215–246, 270 | ACCEPT | 4534/4534 |
| repulse100_uniform.glc | 90121 | 17–48, 59–154, 215–246, 268 | ACCEPT | 205/205 |
| repulse100_uniform__discrete.glc | 12089 | 17–48, 59–154, 215–246, 268 | ACCEPT | 207/207 |

Additional gates:

- `qtsl2bl --expand` smoke on the re-sealed `faulty_concave.glc`:
  3906 ops → 38 trees (700 shared elements), matching the prior
  receipt.
- Producer in-loop check (independent engine from the pinned
  checker): zero refusals, zero violations, zero residue on all
  emissions.
- `MANIFEST.json` per-container `sha256` and the `closure` field
  updated; `bytes`, `proof_trees`, `distinct_trees`, `ops` all
  verified unchanged against the emissions.

## Claim-tally correction for the repulse family (flagged loudly)

The previous manifest recorded `claims` = 414/410/414 for
`repulse100` / `repulse100_uniform` / `repulse100_uniform__discrete`
— twice the container's cone-anchor count, for exactly the three
benchmarks whose proof trees carry ONE Bernstein-cone anchor per tree
(every other benchmark carries two anchors per tree, and their
recorded tallies equal their anchor counts: 2188 = 2×1094,
76 = 2×38, 4534 = 2×2267). At the current pin the discharged-claim
tally is uniformly one claim per cone anchor across all eight
benchmarks — measured independently by the pinned checker and by the
producer's in-loop engine, both with zero refusals — so the manifest
now records 207/205/207 for these rows. The verified objects (the
cone anchors) are unchanged in count and content: the bodies carrying
them are byte-identical to the previously committed corpus. The older
double tallies for this family are consistent with the earlier
checker generation counting two claim rows per cone in the
single-anchor configuration; the historical receipts that quote them
(`a40-retime/`, the prior emission receipts) are frozen records of
their own runs and are deliberately left untouched.

## Tool provenance

- Re-conduct driver (`onesys-driver`) sha256
  `5cea1adce4f3ecbaa2bfe1cd658415ae0307d26dfdf184dc559df2ac446d24e7`,
  built at the canonical toolkit checkouts (mtk
  `18730de566cf26871393e537c6467b5153fb7918`).
- `glc-check` sha256
  `7b06ced7d446f6b793ea94b34096d7cd1570a5bd0ec49b9611aa1cf9ed065f9a`,
  built from a fresh pinned checkout at `QTSL_REV`
  `43cd608130c2c805a5fbd5e416303daf025eb7dc`
  (`crates/qtsl-geolog-provider`, release) — byte-identical to the
  binary the halted re-timing probe recorded, reproduced
  independently.
- `qtsl2bl` sha256
  `531d7f15d3fe858c0a04b896cf17380432759d82b6196c58b092e8109c260300`
  (mtk `18730de…`, release).
