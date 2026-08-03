# Guarded specifications — sealed certificates over MCC 2026 nets

Guarded-specification satisfaction certificates for six place/transition
nets from the Model Checking Contest 2026 corpus, certified end-to-end
through sealed `.glc` containers and the geolog fast-path checker. This
family is the guard former's first certificate corpus: a specification
whose initial-data hypothesis has been discharged *into* the former, so
that the certified conclusion reads

```
S ⊨ Guard(R, φ)
```

— the system satisfies φ *given* that its initial data lie in the
region R, with the "given" carried as former structure and its
discharge carried as a checked claim, not as prose.

## What each container certifies

One container per net, sealed as a single signed geolog commit against
the baked theory closure (the checker derives its vocabulary index from
the closure chain embedded in its own binary; the container's
prerequisite table pins the closure revision). Content:

- **the system**: the net as a bag-mechanism — dimensions, sparse flow
  and marking rows, primary data only;
- **the specification**: an always-invariant formula
  φ = (y·m ≤ c) ∧ (y·m ≥ c), where y is the net's first P-invariant
  vector and c = y·m₀ its initial charge, attached as the
  specification's single component; and the **guard region**
  R = the degenerate box at the initial marking m₀ (per-wire closed
  intervals [m₀_w, m₀_w]);
- **the warrant**: a region-satisfaction witness tying φ to R, wired to
  the satisfaction claim through the guard-warrant route (root ties are
  stored hash-consed-image equalities — store-and-check, never
  repair-by-derivation);
- **the claim rows** the checker discharges, exactly counted in
  `GLC-MANIFEST.expected.json` per net:

| relation | what it checks |
|---|---|
| `qtsl_bag_wf` | the bag mechanism is well-formed |
| `qtsl_tr_content` / `qtsl_tr_grain` / `qtsl_tr_endo` / `qtsl_massone` | per-transition structure (content, granularity, endomorphism typing, mass-one) |
| `qtsl_init_member` | **the guard's evaluation leaf**: the initial marking lies in R — closed-interval containment per wire, exact rational arithmetic |
| `qtsl_warrant_member` | the warranted formula is among the specification's consumed components |
| `qtsl_spec_wf` | specification well-formedness, including agreement between the stored guard region and every guard-warrant's region (store-and-check counting) |
| `qtsl_cert_claim_wf` | warrant-route exclusivity on the satisfaction claim (at most one route, never both) |

A refusal is never a verdict; any non-holding obligation fails the
file, and `--expect-claims` binds the exact per-relation counts so a
deleted claim row cannot pass silently.

## Why the containers are not committed here

The MCC model corpus carries no explicit license statement; per the
contest's own guidance this repository does not redistribute the model
files **or artifacts derived from their structure — the sealed
containers included** (each container carries the net's flow and
marking rows), pending clarification. See `../README.md` for the same
disposition on the P-invariant family. What is committed is
`GLC-MANIFEST.expected.json`: sha256, byte length, and exact
per-relation claim counts for every container (six positives and the
seven-member specimen battery), so any materialized or regenerated
copy verifies bit-for-bit. Sealing is deterministic: regeneration from
the model files has been observed to reproduce all thirteen containers
byte-identically, so the committed hashes pin the corpus as strongly
as committed bytes would.

## Re-derivation path

1. Fetch the model archives from `https://mcc.lip6.fr/2026/` (politely —
   sequential, with a contact user-agent) and extract the six named
   model directories (`AutonomousCar-PT-01a`, `AutonomousCar-PT-02a`,
   `CircadianClock-PT-000001`, `CircadianClock-PT-100000`,
   `DrinkVendingMachine-PT-02`, `EGFr-PT-02010`, each containing
   `model.pnml`).
2. Regenerate and seal the containers with the producer (PNML parse,
   exact P-invariant proposal for the formula charge, conductor-driven
   emit with in-loop claim discharge, deterministic seal), one green
   container per net plus the specimen battery over the
   CircadianClock-PT-000001 base. The producer rides the modelling
   toolkit's strategy surface; the conducting driver currently lives in
   the maintainers' workshop and returns with the toolkit's producer
   migration — until then, a materialized copy of the containers
   verifies against the committed manifest identically.
3. Check from the bytes:

   ```
   GLC_CHECK=<pinned qtsl>/target/release/glc-check \
   GUARD_GLC_DIR=<materialized dir> \
   node --experimental-strip-types petri/guard/glc-recheck.ts
   ```

   (`make petri-guard-glc` builds the checker from the pinned qtsl
   checkout and runs exactly this; with no materialized containers the
   driver says so and skips honestly. The materialized directory holds
   the six positive `.glc` files with the specimens under
   `specimens/`.)

## The specimen battery (seven members, CircadianClock base)

Typed content mutations, recompiled and re-sealed through the full
producer path — nothing is byte-patched — each caught at its named
gate by the independent downstream checker:

| specimen | mutation | caught by |
|---|---|---|
| Control | none | ACCEPTs; byte-identical to the green base container |
| InitOutside | region box shifted off the initial marking | `qtsl_init_member` — the guard violation, failed by exact interval arithmetic |
| RegionMismatch | stored guard region ≠ warrant's region | `qtsl_spec_wf` — the store-and-check agreement count |
| BothRoutes | a second warrant route on the same claim | `qtsl_cert_claim_wf` — route exclusivity |
| AlienComponent | warranted formula not among the specification's components | `qtsl_warrant_member` |
| MemberDrop | membership claim row deleted outright | the count table (`--expect-claims`) — the bare file ACCEPTs, and the driver asserts **both** verdicts: per-obligation discharge alone cannot see whole-row deletion; the manifest's counts can |

A seventh specimen, RootTieBreak (warrant formula ≠ specification root
component, accepted by the claim walk BY DESIGN), documented the claim
walk's reach boundary through revision event 6.1 and retired at the
event-7 re-emission: the event-6 theory revision rekeyed the warrant
onto an entity whose component attribute IS the citation, so the tie
divergence the specimen exercised became unrepresentable — the boundary
it recorded is now enforced one stratum earlier, by the vocabulary
itself.

## Scope honesty

Two obligations of the guarded former live at the theory stratum, above
this family's claim rows, and are stated rather than smuggled:

- **the root tie** (the warrant's formula is the specification's
  component, as a hash-consed-image equality) is enforced by the
  vocabulary since revision event 6 — the warrant entity's component
  attribute IS the citation, so a divergent tie cannot be stated
  (the retired RootTieBreak specimen, above, machine-checked this
  boundary while it was still expressible); mechanized countermodels
  for the wrong-region and dropped-membership loci ride the upstream
  theory repository's proof battery;
- **the warrant's semantic content** (φ holds throughout R) is a
  theory-stratum discharge. In this corpus it is honest by
  construction: R is the degenerate box at m₀ and φ is the invariant
  charge equality at m₀, so the singleton region satisfies φ exactly.

The P-invariant family in `../` certifies the conservation identities
that make φ an invariant charge; this family certifies the guarded
satisfaction *formulation* — specification, region, warrant, and
route discipline — over the same nets.
