# Baked-vocabulary fluid containers: vendoring + driver flip — findings

Working notes for vendoring the first baked-vocabulary `.glc` fluid
certificates and flipping the semicon sealed-container lane to the
baked checker invocation. Appended as work proceeds.

## Plan

1. Vendor the ten baked-vocabulary fluid `.glc` containers + nine
   mutants + manifest, byte-identical (sha256 verified against the
   manifest's own entries — artifact-pair, not trust).
2. Flip the semicon sealed-container driver to the baked-cert lane
   (`glc-check --baked-cert --expect-claims ...`), keeping the
   legacy synthetic-vocabulary containers checkable (versioning law:
   old lanes never die).
3. Advance QTSL_REV to the revision the artifacts pin (read from the
   pins file at that revision matching the manifest's
   vocabulary_rev — verified, not guessed).
4. Regression: sibling glc lanes still green under the advanced pin.

## Log

### Vendoring (step 1) — done

19 files (10 positives, 9 mutants) + manifest copied into
`semicon/out/minifab/glc-baked/`. sha256 verified two ways per file:
against the manifest's own `glc_sha256` entries AND against the source
bytes — 19/19 OK both ways, manifest byte-identical (`cmp`). Flagship
`lbfs_core_84wk.glc` = `b53dbf6e…c590` as expected.

### Pin verification (step 3) — read, not guessed

Manifest `vocabulary_rev` =
`55fba2d2d0b3f06bcf32f78156a182c40abf08b21cafd39ac6d8e895808fc964`.
At qtsl revision `92ebbe8bf703d263439c460e7a3e564067ae0dbd` (reachable
on the qtsl-dev remote's main — verified with branch --contains, so
reproducers can fetch it), `theories/pins/baked_v1_pins.txt` states
`closure qc revision 55fba2d2…` — exact match. Old pin
`aafb00c4…` is an ancestor of the new one (fast-forward advance,
verified with merge-base --is-ancestor).

### Driver flip (step 2) — done

`semicon/glc-recheck.ts` rewritten as a manifest-discovering driver
(walks every GLC-MANIFEST.json under semicon/out/, dispatches on the
manifest's own shape: positives carrying `expect_claims` → baked-cert
lane, modes-only → legacy lane; mutants inherit their base positive's
expectation). New make target `semicon-glc` mirrors the four sibling
sealed-container targets. Run receipt (exit 0):

- baked lane: 10/10 ACCEPT, all four relations 63/63 on every cert
  (2520/2520 claims), 9/9 mutants in their manifest classes
  (m1 constExactEvalTrue 41/63; m2/m3 farkas 62/63; m4 all four 62/63
  under the count gate; m5–m9 REFUSE at the envelope loci).
- legacy lane: 10/10 ACCEPT + 9/9 mutants, still green under the
  ADVANCED pin — the versioning law verified by execution, not
  assertion.

### Sibling regression under the advanced pin (step 4) — all green

Same built checker (92ebbe8): supermartingales-glc exit 0 (4105/4105
positives, 9/9 mutants), lean-route-glc exit 0 (15/15 + 14/14),
gymnasium-glc exit 0 (7 families, 59 positives + 43 mutants), petri-glc
exit 0 (honest skip — corpus not redistributed, as designed).

### Surprises

1. **The semicon sealed-container driver was never wired into the
   Makefile.** All four sibling lanes (supermartingales, lean-route,
   gymnasium, petri) have `-glc` targets; semicon's driver existed
   since the lane landed but no target invoked it — the flip leg is
   also the wiring leg. Nothing was broken; it was unreachable from
   `make`.
2. **supermartingales-glc rewrites a tracked receipts file in place**
   (`supermartingales/out/glc/GLC-TIMINGS.json`: date, loadavg, wall
   ms) — any regression run dirties the tree with timing noise. This
   run restored it to the committed state rather than committing
   load-14 timings. Possibly worth an out-of-tree default for
   non-banking runs; not this leg's call.
3. The workspace Cargo.lock pins qtsl at `c94af1f` for the
   fluidcheck reference lane's crate deps — a separate, deliberate pin
   (the recheck delta-ledger names it "lane a"); QTSL_REV governs only
   the pinned checkout the glc lanes build from. No lock-step edit
   needed there.

### Chapter lock-step check

`chapter/sections/semicon.tex` cites no container-lane facts (no .glc,
no vocabulary, no sealed-container numbers); its certificate numbers
(819 obligations, 1–3 ms, mode counts, capacity fractions) are
invariant under the re-pin (content-identical rows — the re-emission
changed vocabulary pinning, not mathematics). **No chapter edit
required by this change.** `semicon/minifab/FLUID-CERT.md`'s
sealed-containers section updated in-repo to document both lanes.

### Felt-before-clock

Felt (written before reading any clock at report time): **~50 min**.
Wall: first commit 03:47:57 UTC → clock-read 03:55:41 UTC ≈ **0.13 h**
(boot/reading before the first commit adds a few minutes; ~0.2 h
end-to-end). Directed ratio: **felt-over-wall ≈ 4–6×** — the same
signature the chain's earlier legs logged: many short verification
cycles read from inside as much longer.
