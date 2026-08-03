# sealed containers (`.glc`) for the supermartingale certificates

The certificates of this campaign are stored twice, same content, two
forms:

- **reference listings** (`out/**/*.rows.txt`) — geolog object rows in
  the reference listing notation, read by the reference checker lane
  (`checkers/supermartingales` in the pinned qtsl checkout);
- **sealed containers** (`out/glc/`) — the same rows as canonical
  binary bodies inside signed geolog commit envelopes (container
  grammar `geologfl` v1), the form the fast-path checker consumes.

One `.glc` per stored listing: `out/glc/sec1/` and `out/glc/sec3/`
mirror the listing tree (464 containers); the wave-B corpus rides
`out/glc/sec2-glc.tar.zst` (4,105 containers + their own manifest,
extracted to `build/sec2-glc/` at check time).

**Content identity is proven at production, not assumed**: the
producer parses each committed listing, seals it, decodes its own
sealed body back, re-renders the listing notation, and requires a
byte-match against the committed `.rows.txt` (comment lines aside) —
so the sealed form changes the *container*, never one rational of any
published number. Sealing is deterministic (canonical body transform,
fixed skeleton layout, RFC 8032 signing, public reproduction seed,
timestamp 0): regenerating reproduces content addresses exactly.
`out/glc/GLC-MANIFEST.json` records, per container: sha256, commit
content address, source listing + its sha256, and the obligation
count.

## what the fast-path checker verifies

`glc-check` (built from the pinned qtsl checkout,
`crates/qtsl-geolog-provider`, `cargo build --release --bin
glc-check`) dispatches on the container's prerequisite table. These
containers name the staged BLstack/BLCert theory revisions — the same
revisions the reference lane's `theories/` listings were rendered
from — and the checker's vocabulary pins those revision chids as
constants (`reprovocab::bl`). Per container:

1. **envelope integrity** — grammar, ed25519 signature over the
   skeleton, body-hash binding, canonical body decode;
2. **reference discipline** — every prerequisite and dep resolves
   against the pinned roster;
3. **obligation discharge** — the checker enumerates the Bernstein-cone
   obligations from the certificate's PBernCone nodes itself and
   re-derives each in exact rational arithmetic through the ground
   provider: the forward-difference Bernstein coefficient table is
   recomputed from (simplex, extension set, degree, polynomial,
   predicate); nothing is trusted from the certificate.

**Coverage split, stated plainly**: the fast path discharges the
mathematical obligations (the coefficient sweeps — the content that
makes each bound true) plus full envelope verification. The
*structural* rules of the proof calculus (axiom pre/post equality,
conjunction-elimination selection, disjunction-elimination premise
replacement) are checked by the reference lane and by the certified
checker; on the fast path they arrive with engine-native theory
conformance (the chase), which is assembling. The obligation-COUNT
gate lives in the driver: every positive must report exactly the
manifest's obligation count, because deleting a proof node leaves
every surviving obligation true (mutant `m4` witnesses exactly this).

## run

```
GLC_CHECK=/path/to/glc-check make supermartingales-glc
# or directly:
GLC_CHECK=... node --experimental-strip-types supermartingales/glc-recheck.ts
```

Expected: 464/464 (sec1+sec3) and 4,105/4,105 (sec2) ACCEPT with exact
obligation counts. A fully-green run also writes a per-benchmark
wall-clock receipt (machine, load, checker-binary hash, and per-group
`n`/`wall_s`) — out of tree by default (`build/supermartingales/`), so
verification runs leave the tree clean; with `GLC_PUBLISH_TIMINGS=1` it
refreshes the committed receipt `out/glc/GLC-TIMINGS.json` behind the
chapter's wall-clock comparison table's geolog column (see
`receipts/TIMINGS-PATH-NOTE.md`); the 9-mutant battery lands each mutant in its
declared class — re-sealed content mutants FAIL at the attacked
obligation (`m1` predicate bound made unsatisfiable, `m2` declared
degree lied below the polynomial's, `m3` a polynomial-closure
coefficient bumped toward the predicate's violating side), the
node-drop mutant ACCEPTs undercounted (the count gate catches it), and
byte-level mutants REFUSE at their loci (body hash, signature,
truncation, magic, version).

## relation to the certified-checker replay

The wave-B corpus in `out/sec2/certified-checker/` records a replay of
the same certificate content through the certified checker (sealed on
the same staged theory chain). The certified checker verifies the
full calculus — structural rules included — from sealed bytes; the
fast path trades that completeness for speed (whole-corpus walls of
seconds rather than minutes) and never carries the trust story. Same
mathematics, two consumers, honest costs — which is the point of
storing certificates as sealed geolog content in the first place.

## the baked-closure lane (`out/glc-baked/`)

The same campaign re-sealed a second time, on the **baked theory-pair
closure** rather than the staged chain — the reproduction route the
checker shares with every baked-certificate family: the vocabulary
index is derived from the pinned chain's own bytes (no per-family
pinned constants), and each container carries **explicit claim rows**
that the checker discharges through the exact-rational ground provider
(`glc-check --baked-cert --expect-claims name=N,...`; the per-relation
count gate is load-bearing for the same node-drop reason as above).
The synthetic-chain lane above stays committed and checked forever —
a committed lane never dies; the baked lane adds.

Layout: `out/glc-baked/sec1/` and `out/glc-baked/sec3/` mirror the
listing tree (489 containers, deterministic re-emissions of the same
committed `.rows.txt` listings — regenerating them reproduces every
sha256 in `out/glc-baked/GLC-MANIFEST.json`); `fixtures/` carries the
producer crate's two fixture certificates (the designed mutant
battery's bases); `mutants/` the 9-mutant battery on the sealed bytes.
The manifest's `transform` field discloses, per container, every
content delta of the re-emission (producer bookkeeping rows dropped,
shape/dtype constructors folded to literals, the two hole species
respelled, claim rows minted).

Two reader notes, so the manifest's shape surprises nobody:

- **the `sec3/mutants/` records are the corpus's committed falsifier
  listings**, sealed as containers like everything else the listing
  tree contains. 23 of the 25 FAIL claim discharge, as their listings
  were designed to. The two `M3_byte_flip` listings **ACCEPT**: that
  mutant class was a *container-byte* tamper, and a listing-level
  re-seal legitimately erases it — the flip lives in content the
  coefficient sweep proves true. The sealed-byte tamper class is
  covered on this lane by the designed battery's REFUSE mutants
  (body hash, signature, truncation, magic, version). The driver
  encodes exactly this classification.
- **the two `fixtures/` sources** are the producer crate's own listing
  fixtures (recorded with their sha256s), not part of the published
  tables; they ride the manifest because the designed mutants are
  derived from one of them.

## the persist_2d re-conduct lane (`out/glc-baked/persist2d/`)

The one continuous-state row's certificates — both twins of a noise
model the paper leaves unpinned (see `specs/persist_2d_uniform.toml` +
`specs/persist_2d_gaussian.toml` for the reconstruction provenance;
neither twin claims the published row's noise class) — re-conducted
onto the baked qc closure as **shared-subtree (proof hypernet) forest
containers**, the sec2 lane's container form:

- `persist_2d__uniform.glc` — the whole uniform twin, one container:
  4,800 drift-triangle proof trees, hash-consed, 510,674 ops. The
  committed listings transcribe the dyadic re-emission of the
  certified uniform certificates (one shared power-of-two denominator;
  bound bit-identical to the exact published fraction
  $58920851522213/588624739134464$).
- `persist_2d__gaussian_p{0..3}.glc` — the gaussian twin as four
  sorted-quarter containers of 4,800 trees each (~518k ops each): the
  whole twin's merged forest (~1.94M ops) exceeds the container
  envelope's per-commit body cap ($2^{20}$ ops), so the twin ships
  split, the spde split-family precedent. Quarters partition the
  sorted listing order; the manifest's `split` note pins the ranges.
  Certified exact bound $6701869649/68719476736$ (0.0975251 rounded
  up), below both published numbers.

Sources: `out/persist2d/rows-persist2d.tar.zst` — the committed
reference listings, five group directories, sha256 pinned in the
manifest. Produce receipts (conductor lock, closure revision,
component pins, in-loop check tallies) sit in `receipts/` beside the
containers; container bytes are deterministic (independent double-runs
byte-identical, all five). The expansion inverse (`qtsl2bl --expand`)
re-derives per-tree containers that match the unshared per-listing
emission row-multiset-exactly (renaming-invariant digests; verified
for the uniform twin and a gaussian quarter at production).

`mutants/` carries the designed battery on the uniform base — the
five semantic classes (recompiled at the source-graph level: absurd
predicate bound, degree lie, coefficient bump, and the two
count-gate witnesses, which ACCEPT bare with 4,799/4,799 and are
caught exactly by the manifest's claim expectation) plus the four
envelope integrity surgeries, and one cross-container envelope
witness on a gaussian quarter. Content classes ride the
representative uniform base per the committed-battery precedent
above. The driver checks every mutant to its expected class.

Timing cells for this row (the sweep-times table) remain holes until
the standard retime protocol's receipts exist; the conductor walls in
`receipts/` are production bookkeeping, not protocol timings.
