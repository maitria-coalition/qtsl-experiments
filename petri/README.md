# Petri P-invariants — sealed certificates over MCC 2026 nets

Exact P-invariant witnesses for place/transition nets from the Model
Checking Contest 2026 corpus, certified end-to-end through sealed
`.glc` containers and the geolog fast-path checker. (A second
certificate family over the same corpus — guarded-specification
satisfaction, `S ⊨ Guard(R, φ)` — lives in `guard/`, under the same
redistribution disposition.)

## What is certified

For a P/T net with flow matrices `pre`, `post` and incidence
`C = post − pre`, and each certified integer vector `y` with
`yᵀ·C = 0`:

- **per transition `t`**: the conservation identity
  `Σ_p y_p·(post[p,t] − pre[p,t]) = 0`;
- **per invariant**: the initial charge `Σ_p y_p·m0_p = b`.

A container stores **primary data only** — the net's dimensions,
sparse flow and marking rows, and the invariant vectors with their
stated charges. No term trees and no claim rows are stored: the
checker constructs every obligation itself from the net rows
(`K·(T+1)` exact ground facts, enumerated from the dimensions) and
discharges each as `constExactEvalTrue` through the exact-rational
ground provider, so a stored-tree forgery is unrepresentable rather
than discouraged. Containers with no invariants, and invariant
vectors that are identically zero, are refused structurally — `y = 0`
satisfies every identity, and this family's checker treats that
vacuity as malformation, never as a verdict.

Scope honesty: these are the linear-algebra leaves, exactly. The
semantic bridge "per-transition identities ⟹ `y·m` invariant over all
*reachable* markings" is a separate proof obligation that rides its
own discharge machinery, and "no invariant exists" (the trivial-
nullspace case — one net in the shipped set) is a universal statement
outside this certificate species' reach; it is recorded as a manifest
fact, not a sealed claim.

## Why the containers are not committed here

The MCC model corpus carries no explicit license statement; per the
contest's own guidance (attribution, and checking with the organizers
before redistribution) this repository does not redistribute the
model files **or artifacts derived from their structure — the sealed
containers included**, pending clarification. What is committed is
`GLC-MANIFEST.expected.json`: sha256 and expected obligation counts
for every container (positives, the twelve-member mutant battery, and
the vacuity record), so any materialized or regenerated copy verifies
bit-for-bit. Sealing is deterministic (public reproduction seed,
zero timestamps, canonical body transform) and the invariant basis is
a deterministic exact elimination, so regeneration from the model
files reproduces the containers down to their content addresses.

## Re-derivation path

1. Fetch the five model archives from `https://mcc.lip6.fr/2026/`
   (politely — sequential, with a contact user-agent) and verify
   their sha256s; extract the five named model files and verify each
   against the `fixture_sha256` fields in the expected manifest.
2. Regenerate and seal the containers with the producer (PNML parse,
   exact left-nullspace proposal, refuse-to-seal gate through two
   independent incumbent checkers, deterministic seal). The producer
   currently lives in the maintainers' workshop; its migration into
   the public toolkit is planned alongside the license
   clarification — until then, a materialized copy of the containers
   verifies against the committed manifest identically.
3. Check from the bytes:

   ```
   GLC_CHECK=<pinned qtsl>/target/release/glc-check \
   PETRI_GLC_DIR=<materialized dir> \
   node --experimental-strip-types petri/glc-recheck.ts
   ```

   (`make petri-glc` builds the checker from the pinned qtsl checkout
   and runs exactly this; with no materialized containers the driver
   says so and skips honestly.)

## The mutant battery (twelve members, from the SwimmingPool base)

| class | members | caught by |
|---|---|---|
| forged content (FAIL) | coefficient bump, flow-entry bump, charge lie | the constructed identities, recomputed from primary rows |
| coherent shrinkage (ACCEPT_UNDERCOUNT) | invariant dropped, transition dropped — internally consistent | the manifest's expected obligation counts (the count gate) |
| vacuity (REFUSE) | zero invariant vector | the checker's structural zero-vector refusal |
| coherent forgery (ACCEPT_FORGED, control) | a phantom place appended with its own "conservation" — every in-lane check holds on the forged net | byte identity against regeneration (the manifest sha256) |
| envelope (REFUSE) | body byte, signature byte, truncation, magic, version | container + envelope integrity, before any mathematics runs |

The forgery control documents this lane's reach boundary honestly: a
self-consistent container is a true statement about *some* net; what
binds it to *the* net is deterministic regeneration from the model
file, pinned by the committed hashes.
