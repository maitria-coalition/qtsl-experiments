# Baked-lane vendoring findings: gymnasium family

Working log for vendoring the gymnasium family's baked-lane certificate
containers into this repo. Sibling documents: `BAKED-FLUID-VENDORING-FINDINGS.md`
(semicon; manifest-discovering driver pattern) and
`BAKED-BLSPDE-VENDORING-FINDINGS.md` (bl + spde; multi-family, expected-FAIL
classification).

Status: COMPLETE — all lanes green under the advanced pin; one
landing-order constraint named at the end.

## Scope

- Vendor the 9 baked gymnasium deadline certificates + their mutants +
  manifests, two-way byte-verified (manifest shas AND upstream source bytes).
- Wire the gymnasium make lane's baked-lane check via
  `glc-check --baked-cert --expect-claims` (per-cert `qtsl_deadline_table=1`).
  First: determine whether the gymnasium family has a wired `-glc` make
  target at all (two of five families did not, pre-vendoring).
- Document the 9/29 split honestly for a cold reproducer: 20 of the family's
  29 deadline certificates are genuinely time-varying and are not
  representable in the baked (stationary) lane; each exclusion is recorded
  in the vendored manifests with its obstructing states.
- Verify QTSL_REV reachability for the containers' pins; advance if the
  containers require it, re-verifying all sibling lanes under the advanced pin.

## Findings

### The gymnasium lane was already wired — and already drop-in

Unlike semicon and spde (whose sealed-container drivers existed
unwired — the two prior specimens of that class), gymnasium has had a
`gymnasium-glc` make target since its lane landed, and its driver is
manifest-discovering: every `GLC-MANIFEST.json` under `gymnasium/out/`
is one family, dispatched on the manifest's own `mode_flag`. The baked
manifests were emitted in exactly the driver's schema (`--baked-cert`
mode flag, positives with `expect_claims` + `glc_sha256`, mutants with
`base`/`expected`/`locus`), so the vendoring required **zero driver
edits and zero Makefile target edits** — the drop-in design working as
documented on its first cross-lane specimen.

### Vendoring — two-way byte verification

5 family directories → `gymnasium/out/<family>/glc-baked/`: 27 files
(9 positives + 13 mutants + 5 manifests). Verified two ways per
container: sha256 against the manifest's own `glc_sha256` entries
(22/22 OK), and `cmp` of every vendored file (manifests included)
against the source bytes (27/27 identical). Manifests carry
repo-relative source paths (zero machine-path scrubbing needed — the
producer emitted them clean, the earlier BL-manifest lesson already
absorbed upstream). Hygiene grep over the whole branch diff: clean.

### The 9/29 split, documented for the cold reproducer

`gymnasium/GLC-CERT.md` gains "the baked lane": the stationary-
composition criterion (one action per state admissible at every step
⟺ the chain species carries the corpus table + claim byte-verbatim),
the 9 that compose, the 20 genuinely time-varying exclusions (each
recorded in its family manifest with obstructing states;
`frozenlake-4x4-sr34` and `frozenlake-8x8` are exclusion-only
manifests — zero containers, present so the record lives where a
reproducer looks), the staged lane keeping all 29 checkable, and the
five-names-one-sha content-address collapse in `frozenlake-4x4-det`
(a deterministic chain's dyadic tables reduce to the same 0/1
rationals at every mantissa width — documented so identical sha256
lines read as truth, not copy error). The staged battery section
retitled "the staged-lane mutant battery" to disambiguate from the
baked battery now described beside it.

### Pin verification — read from the graph, not guessed

The gym manifests carry no vocabulary-revision field; the containers
pin the baked closure (revision f6bf1345…) directly, and the pins
file states `closure dc revision f6bf1345…` at BOTH the incumbent
pin f67c7675 and the new one — the closure was never the gap. What
the incumbent pin lacks is the provider's chain-frame deadline walk
(dprob/dinit, no action layer), which landed on the qtsl main line in
merge `36e4f3b3df2ffe2d4b6bc2e5b8e1fceacf368e75`. Graph facts,
verified: the chain-frame branch was cut from 92ebbe8 (so f67c7675 is
NOT its ancestor — the two are siblings), and the merge 36e4f3b
contains BOTH f67c7675 (the reserved predicate-hole classification
the BL lanes need) and the chain frame. QTSL_REV therefore advances
f67c7675 → 36e4f3b, the earliest rev satisfying every lane at once.

**Landing-order constraint (the one open item):** as of this leg's
verification, 36e4f3b is on the qtsl repository's local main but NOT
yet on the qtsl-dev remote (origin/main = 36b2f93, re-fetched and
re-checked at write time). This branch must land AFTER that push, or
`make` reproducers (and CI) will fail at the pin fetch. All
verification here ran with `QTSL_REPO` pointed at the local qtsl
checkout — the driver's own documented pre-landing pattern (the
binary, not the pin, is the contract for pre-landing verification).

### Run receipts (checker built at the advanced pin, release)

- `gymnasium-glc` exit 0: **12 families, 68 positives + 56 mutants,
  all verdicts as expected** (was 7/59/43 — the delta is exactly the
  5 baked manifests, 9 positives, 13 mutants). Baked ACCEPT lines
  match the producer-side transcript ops-for-ops (hitting_exact_T13
  ops=1586; exact_T5 ops=327; exact_T10 ops=487; the det family
  ops=3346). Mutants: b1–b6 + b13 FAIL at 0/1 under the count gate;
  b7 REFUSES (zero-claim); b8–b12 REFUSE at their envelope loci.
  Staged families unchanged under the advanced pin.
- Sibling regression, same built checker: `semicon-glc` exit 0 (both
  lanes, 10+9 baked / legacy green), `lean-route-glc` exit 0 (staged
  15/15+14/14, baked 15/15 + 14/14), `spde-glc` exit 0 (witness +
  spectrum + 19/19 mutants), `supermartingales-glc` exit 0 (synthetic
  + sec2 + baked lanes all green), `petri-glc` exit 0 (honest skip,
  as designed) — the versioning law verified by execution under the
  advanced pin.
- Tree clean after all runs: the supermartingales timing receipt now
  writes out-of-tree by default (`build/supermartingales/
  GLC-TIMINGS.json`, opt-in publish flag) — the timing-noise
  friction two sibling legs logged has been repaired upstream; no
  restore step needed.

### Chapter lock-step

`chapter/sections/gymnasium.tex` cites no container-lane facts at all
(zero matches for glc/container/sealed/baked/manifest) — its numbers
are cbor-corpus facts, invariant under an added lane. **No chapter
edit required by this change.** As with the BL/SPDE leg: when the
Experiments chapter goes uniform on the reproduction spine, the baked
lanes are where its container prose should point; that edit belongs
to the chapter leg, not this vendoring.
