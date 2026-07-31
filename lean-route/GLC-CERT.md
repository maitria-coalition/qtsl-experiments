# sealed containers (`.glc`) for the lean-route ladder

The fifteen ladder certificates are stored twice, same content, two
forms:

- **reference listings** (`fixtures/*.rows.txt`) — geolog object rows
  in the reference listing notation, replayed by the pinned TypeScript
  reference lane (`make lean-route-recheck`);
- **sealed containers** (`out/glc/*.glc`) — the same rows as canonical
  binary bodies inside signed geolog commit envelopes (container
  grammar `geologfl` v1), the form the fast-path checker consumes.

The committed `fixtures/*.geolog` twins predate this lane: they were
sealed on an earlier three-commit staging of the theory chain and are
what the Lean-route timing receipts were measured against — they stay
byte-for-byte untouched. The `out/glc/` containers carry the same
certificate rows re-sealed on the current staged BLstack/BLCert
revisions, the chain the fast-path checker's vocabulary pins
(`reprovocab::bl`).

**Content identity is proven at production, not assumed**: the
producer parses each committed listing, seals it, decodes its own
sealed body back, re-renders the listing notation, and requires a
byte-match against the committed `.rows.txt` (comment lines aside) —
the sealed form changes the *container*, never one rational of any
published number. Sealing is deterministic (canonical body transform,
fixed skeleton layout, RFC 8032 signing, public reproduction seed,
timestamp 0): regenerating reproduces content addresses exactly.
`out/glc/GLC-MANIFEST.json` records, per container: sha256, commit
content address, source listing + its sha256, and the per-relation
obligation counts.

## what the fast-path checker verifies

`glc-check` (built from the pinned qtsl checkout,
`crates/qtsl-geolog-provider`, `cargo build --release --bin
glc-check`) dispatches on the container's prerequisite table and, for
these containers, enumerates the **applicative obligation roster**
from the certificate's own rule nodes — nothing is trusted from the
certificate:

1. **envelope integrity** — grammar, ed25519 signature over the
   skeleton, body-hash binding, canonical body decode;
2. **reference discipline** — every prerequisite and dep resolves
   against the pinned roster;
3. **obligation discharge**, per rule-node kind, in exact rational
   arithmetic through the ground provider:
   - *Bernstein cone* (`PBernCone`): the forward-difference
     coefficient table recomputed from (simplex, extension set,
     degree, polynomial, predicate) and swept against the predicate;
   - *Farkas* (`PFarkas`): the nonnegative combination replayed from
     the stored coefficients against the node's own premise (reached
     through the proof's concluded judgment, never a carried formula)
     and met against the conclusion;
   - *syzygy* (`PSyzygy`): the signed combination replayed from
     component posts (`syzygyCombinationValid`) AND the conclusion
     met against the node's own judgment post
     (`syzygyConclusionFormRel`) — the second tie is what kills a
     self-consistent (coefficients, conclusion) co-forgery;
   - *enclosure* (`PEnclosure`): the Maclaurin remainder bound
     recomputed from the concluded judgment's (function tag, degree,
     radius) and compared against the claimed cap;
   - *promise* (`PPromise`): the two wire obligations — contract
     confinement to the declared interface, and supplier/context wire
     separation over the recomputed context.

**Coverage split, stated plainly**: the fast path discharges the
applicative obligations — every rule whose side condition carries
arithmetic or wire-scope content, which on this ladder means all
fifteen rungs carry at least one fast-path-discharged obligation
(2,875 across the ladder). The *structural* rules of the proof
calculus (axiom pre/post equality, conjunction introduction and
elimination selection, disjunction-elimination premise replacement,
cut and equality-elimination plumbing, convexity intro/elim, and the
promise rule's formula plumbing) are checked by the reference lane
and by the certified checker; on the fast path they arrive with
engine-native theory conformance (the chase), which is assembling.
The obligation-COUNT gate lives in the driver: every positive must
report exactly the manifest's per-relation counts (zeros enforced),
because deleting a rule node leaves every surviving obligation true.

## run

```
GLC_CHECK=/path/to/glc-check make lean-route-glc
# or directly:
GLC_CHECK=... node --experimental-strip-types lean-route/glc-recheck.ts
```

Expected: 15/15 ACCEPT with exact per-relation obligation counts; the
14-mutant battery lands each mutant in its declared class:

- re-sealed content mutants **FAIL at their designed loci** — a
  predicate bound made absurd (whole-family kill on its rung: the
  bound constant is shared by every cone sweep and Farkas premise
  there, disclosed in the manifest), a degree lie (single cone
  obligation), a single Farkas coefficient bump (that one combination
  and no other), a promise contract swapped to a formula whose wires
  escape the interface (contract confinement and nothing else), an
  enclosure cap tightened below the exact tail bound;
- the **coherent-forgery crown**: one syzygy node's coefficients all
  doubled AND its conclusion re-forged to match — the combination
  replay HOLDS on the self-consistent pair, and the certificate dies
  only at the conclusion-form tie to the node's own judgment post;
- the **claim-swap CONTROL** (expected ACCEPT): an enclosure judgment
  re-forged coherently (degree raised, cap recomputed as the exact
  tail) is a true statement of a *different* claim — the lane accepts
  it by design, and the falsifier with that reach is the
  production-time byte-identity gate against the committed listing
  (plus the manifest's source sha256); recorded here so the lane's
  reach boundary is a documented fact, not a surprise;
- node-drop mutants **ACCEPT undercounted** (the per-relation count
  gate is the witness), and byte-level mutants **REFUSE** at their
  loci (body hash, signature, truncation, magic, version).

## the baked-closure lane (`out/glc-baked/`)

The same fifteen certificates re-sealed on the **baked theory-pair
closure** (the staged-chain lane above stays committed and checked
forever): explicit claim rows discharged through the exact-rational
ground provider against the vocabulary index the checker derives from
the pinned chain's own bytes (`glc-check --baked-cert`), same
per-relation roster, same exact-count discipline, plus a re-derived
14-mutant battery with the same designed loci (the coherent-forgery
crown and the claim-swap control included). The re-emission's content
deltas are disclosed per container in the manifest's `transform`
field; two species the staged chain carried as constructors are
respelled on the baked closure — the predicate hole rides the
reserved wire spelling (a `TVar` row whose single argument is the nat
literal 4294967295, the Lean model's own projection transported to
row grain), and the spec-hole conjunct is sharpened to the trivially
true formula (the towers prove their conclusions from the region
atoms alone; the sharpened judgment is what these containers record).
Checking this lane therefore requires a checker revision that
classifies the reserved hole spelling — at an older pin every cone
claim refuses loudly; the pinned `QTSL_REV` carries it.

This lane's prerequisite pins the baked closure's **current revision**
and moves with it — the checker derives its vocabulary index from the
pinned chain's own bytes and deliberately grows no multi-revision
reader, so when the theory pair re-mints, these containers are
re-emitted at the new revision (same producer, no emitter changes) and
the checker at the older pin refuses the new prerequisite loudly, as
`unknown_prereq`. Across such a re-emission every content field is held
identical — per-relation obligation counts, source listing hashes,
transform audits, and the mutant battery's designed loci — and only the
container bytes move (the prerequisite content address plus the
declaration renumbering it induces). The staged-chain lane above is the
committed-forever form the versioning law keeps checkable regardless.
