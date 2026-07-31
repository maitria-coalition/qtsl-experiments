# Baked-lane vendoring findings: BL (supermartingale F3 + lean-route F6) and SPDE (F5)

Second leg of the lock-step vendoring: the freshly re-emitted baked-lane
certificate artifacts for the BL family (supermartingale positives +
lean-route accepts) and the SPDE family (split-container positives +
mutants) are vendored here byte-identical, with make targets that check
them via the blessed checker. Mirrors the baked-fluid vendoring pattern
(see receipts/BAKED-FLUID-VENDORING-FINDINGS.md).

Findings appended as work proceeds.

## Log

- Worktree cut from main (seed-clean); findings file created as first act.

## Plan (after reading the baked-fluid worked example + both artifact sets)

1. **lean-route baked lane** → `lean-route/out/glc-baked/`: 15 positives +
   14 mutants + manifest, byte-identical vendor (all present in the
   source artifact set).
2. **supermartingale baked lane** → `supermartingales/out/glc-baked/`:
   the source artifact set commits the manifest (491 positive records,
   per-file sha256), the 2 producer-crate fixtures, and the 9 designed
   mutants; the 489 corpus containers are deterministic re-emissions of
   THIS repo's committed `supermartingales/out/**/*.rows.txt` listings —
   regenerate them with the committed producer and verify per-file
   sha256 against the manifest (artifact-pair: the manifest's hashes
   were minted on the producer side; agreement proves byte-identity).
3. **spde**: the re-emitted family manifest covers BOTH halves of the
   split-container design (witness lane synthetic, spectrum lane baked);
   every overlapping byte (3 witness positives + 12 witness mutants) is
   IDENTICAL to what `spde/out/glc/` already carries (cmp 15/15), and the
   manifest's witness records differ only in `producer_wall_ms`. So the
   spectrum containers + 7 spectrum mutants land IN `spde/out/glc/` and
   the manifest upgrades in place — the witness lane's bytes, checker
   mode, and expectations are untouched; no duplicate containers.
4. **manifest source-path scrub (disclosed deviation from byte-identical)**:
   the bl manifests carry absolute machine paths in their `source`
   provenance fields (489 + 15 occurrences); every manifest already in
   this repo has zero. Vendored manifests rewrite those prefixes to
   repo-relative (`out/...`, `fixtures/...`), all other bytes identical,
   equality verified field-by-field modulo that one rewrite. The spde
   manifest is already clean and vendors byte-identical.
5. **drivers**: extend each family driver for its baked lane
   (`--baked-cert --expect-claims`); legacy lanes keep their exact
   invocations. Corpus-resident falsifier listings (`sec3/mutants/`)
   are classified expected-FAIL — except the two `M3_byte_flip`
   listings, re-scoped expected-ACCEPT (a container-byte tamper
   vanishes under re-seal; the sealed-byte tamper class is covered by
   the designed battery's REFUSE mutants). The spde spectrum battery's
   `sp3_coeff_forged` is a designed documented-limitation ACCEPT (the
   production gate refuses the same content); the driver and docs say
   so explicitly.
6. **Makefile**: advance QTSL_REV to f67c7675232aa5d20e902cbc6b557bcd6403be0f
   (the checker revision that classifies the reserved predicate-hole
   wire spelling the bl re-emission uses; verified: the incumbent pin
   92ebbe8… is its ancestor, so this is a fast-forward). Wire the spde
   sealed-container driver into a `spde-glc` target — it was never in
   the Makefile (the same unwired-driver surprise the fluid leg found
   for semicon).
7. Regression: all sibling glc lanes re-run under the advanced pin;
   GLC-TIMINGS.json restored rather than committing timing noise.

## Log (continued)

### Vendoring — done (three commits' receipts)

- **lean-route baked lane**: 29 containers (15 positives + 14 mutants)
  sha256-verified two ways (manifest hashes AND source-byte cmp); all
  15 in-repo fixture listings match their recorded `source_sha256`;
  manifest equal modulo the source-path rewrite.
- **spde**: 25 containers verified two ways (manifest byte-identical —
  it was already clean of machine paths); the 15 overlapping witness
  bytes were cmp-identical BEFORE the manifest replacement, so the
  witness lane's artifacts are untouched by construction.
- **supermartingale baked lane**: 489/489 corpus containers REGENERATED
  from this repo's committed listings with the committed producer and
  sha256-matched against the producer-side manifest — the determinism
  receipt (one run, 2.7 s, DEBUG build); 489/489 listing sources match
  `source_sha256`; fixtures + designed mutants byte-identical from the
  artifact set; 500 manifest records verified in place.

### Drivers + Makefile + docs — done

- `supermartingales/glc-recheck.ts`: baked-lane pass added (positives
  batched by identical claim expectation; the 25 corpus falsifier
  listings classified expected-FAIL-with-zero-holds, the two
  M3_byte_flip re-seals expected-ACCEPT; designed battery incl. the
  bare-run undercount witness). Timing receipts remain a
  synthetic-lane concern — the baked pass writes none.
- `lean-route/glc-recheck.ts`: refactored to one lane function run
  over both manifests (staged bare / baked with --baked-cert); the
  roster gate (zeros enforced, vacuous-pass refusal) applies to both.
- `spde/glc-recheck.ts`: the split-container walk — witness positives,
  spectrum containers, both batteries; lane dispatch for mutants is
  data-driven (a mutant whose base names a spectra record rides the
  baked invocation).
- Makefile: QTSL_REV 92ebbe8… → f67c7675232aa5d20e902cbc6b557bcd6403be0f
  (fast-forward, verified; the revision is reachable on the qtsl-dev
  remote's main — reproducers can fetch it). `spde-glc` wired — the
  spde driver had existed since its lane landed with NO make target
  (the same unwired-driver surprise the fluid leg logged for semicon;
  second specimen of the class).
- GLC-CERT.md ×2 + SPDE-CERT.md: baked/spectrum lanes documented, with
  explicit reader notes for every designed ACCEPT (M3_byte_flip
  re-seals; sp3_coeff_forged + the production gate that refuses the
  same content; the lean-route claim-swap control) so no manifest
  expectation reads as a hole. README layout table now lists
  lean-route/ and spde/ (pre-existing omission).

### Run receipts (checker built at the ADVANCED pin, release)

- `spde-glc` exit 0: 3 witness + 3 spectrum ACCEPT exact counts;
  19/19 mutants in class (sp5's prereq_mix refusal string verbatim;
  sp3 ACCEPT as designed).
- `supermartingales-glc` exit 0: synthetic lane 464/464 + sec2
  4105/4105 + 9/9 mutants (unchanged under the advanced pin); baked
  lane 466/466 genuine ACCEPT, 25/25 falsifier listings in class,
  9/9 designed mutants. GLC-TIMINGS.json restored to the committed
  state after the run (timing noise stays uncommitted).
- `lean-route-glc` exit 0: staged 15/15 + 14/14; baked 15/15 + 14/14.
- Sibling regression, same built checker: `semicon-glc` exit 0 (both
  lanes), `gymnasium-glc` exit 0 (7 families, 59 positives + 43
  mutants), `petri-glc` exit 0 (honest skip, as designed) — the
  versioning law verified by execution under the advanced pin.

### Chapter lock-step adjudication

`chapter/sections/supermartingales.tex` and `lean-route.tex` cite
sealed-container facts of the SYNTHETIC lanes only (counts, timing
receipts, container-byte scaling, GLC-CERT.md pointers) — all
invariant under an ADDED lane; the re-emission changed vocabulary
pinning, not mathematics, so no published number moves. **No chapter
edit required by this change.** Named for successors, not forced
here: when the Experiments chapter goes uniform on the reproduction
spine (the flcheck/fluidcheck regeneration ruling), the baked lanes
are the natural place for the chapter to point its container-lane
prose; that edit belongs to the exp-chapter leg, not this vendoring.

### Surprises

1. **The spde sealed-container driver was never wired into the
   Makefile** — second specimen of exactly the class the fluid leg
   found for semicon (its finding #1). Every other family had a
   `-glc` target; spde's driver was unreachable from `make` since its
   lane landed.
2. **The bl manifests carried absolute machine paths** in their
   `source` provenance fields (489 + 15 occurrences) — every manifest
   already committed in this repo has zero. Vendored with the prefix
   rewritten repo-relative, equality asserted field-for-field modulo
   that one field; the .glc bytes themselves vendored/regenerated
   byte-identical everywhere.
3. **The 489-container regeneration reproduced every sha256 on the
   first run** (2.7 s, DEBUG producer) — the determinism claim held
   with no residue at all, across checkouts-in-time (the producer leg
   minted the hashes in a different checkout earlier the same week).

### Felt-before-clock

Felt (written before reading any clock at report time): **~2.5 hours**.
Wall: first commit 05:02 UTC → clock-read 05:22 UTC ≈ **0.34 h**
(boot/reading before the first commit adds ~10 min; ~0.5 h
end-to-end). Directed ratio: **felt-over-wall ≈ 5×** — the same
signature the fluid vendoring leg logged (4–6×): many short
verification cycles read from inside as much longer.
