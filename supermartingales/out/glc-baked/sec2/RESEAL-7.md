# sec2 shared vintage — re-seal at theory revision event 7

The eight shared-subtree containers in this directory previously sealed
at the revision event 5.1 closure (qc `6862edc8…`); the family-wide pin
advance to revision event 7
(qc `9952506dd2491e880a6107471a1ca92f66a4a33a64a8cc250833d00402e3b0e6`)
re-seals the corpus. Unlike the event-5.1 re-seal (RESEAL-5.1.md), whose
byte deltas were confined to the envelope, this event's declaration
counters moved under the closure, so each container also carries the
induced motion in its body's external-reference cells — which is why
the verdict here rides the semantic instrument rather than a byte
window.

## Producers

- Seven chain benchmarks: the fresh-synthesis re-conduct driver
  (`supermart-synth`, host lane; emission bytes are lane-independent
  per the standing cross-lane byte-identity receipts).
- `repulse100_uniform` (continuous noise; outside the fresh-synthesis
  genus): the listing-replay forest producer (`supermart-share
  --mode shared`) over the committed listing corpus
  (`../../sec2/rows-certs.tar.zst`).
- Cross-producer witness: `repulse100_uniform__discrete` was emitted by
  BOTH paths and the two emissions are byte-identical in full
  (envelope included).

## Per-container verification

- **Semantic identity across vintages** (`crossdiff` — external
  declaration references resolved to declaration NAMES through each
  container's own prerequisite closure, then canonical op-term
  multisets compared): SEMANTICALLY IDENTICAL, 8/8, zero unresolved
  references either side; op counts equal per container.
- **Checker at the pin** (`glc-check --baked-cert --expect-claims`):
  ACCEPT 8/8 at exactly the manifest's recorded claim tallies
  (2188 / 76 / 76 / 76 / 207 / 4534 / 205 / 207).
- File lengths equal old vs new for all eight; `MANIFEST.json` moves
  per-container `sha256` and the `closure` field only —
  `bytes`, `proof_trees`, `distinct_trees`, `claims`, `ops` all
  verified unchanged.
- Each certificate additionally records its superseded byte vintage in
  `prior_sha256s` (append-only): the committed timing receipts are
  frozen records of the runs they measured, and the receipt validator
  accepts a receipt citing a recorded prior vintage of the same
  semantic content while refusing anything outside the family.
- Producer in-loop check (independent engine from the pinned checker):
  zero refusals, zero violations, zero residue on all emissions.

## Tool provenance

- Re-conduct driver (`onesys-driver`) built at the canonical toolkit
  checkouts (mtk main `09dccea1b`, qtsl provider
  `f0ecbf32d03af43921a11ee7e0aee7a35d51822d` — the pinned revision).
- `glc-check` built from the qtsl checkout at provider bytes identical
  to the pin `f0ecbf32…` (`crates/qtsl-geolog-provider`, release).
