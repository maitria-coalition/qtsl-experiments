# sealed containers (`.glc`) for the Gymnasium hitting certificates

The hitting-lane certificates of the Gymnasium chapter (exact and
dyadic deadline tables: the FrozenLake families and the CliffWalking
hitting companion) are stored in up to three forms, same claims:

- **cbor certificates** (`out/<family>/*.cbor`) — the incumbent
  checker's format, verified by `make gymnasium-text`;
- **sealed containers** (`out/<family>/glc/*.glc`) — the same
  certificate content as one geolog commit's rows (container grammar
  `geologfl` v1), the form the fast-path checker consumes;
- **baked-closure containers** (`out/<family>/glc-baked/*.glc`) — the
  stationarily-composable subset of the same claims, re-emitted
  against the baked theory closure as pure Markov chains (see "the
  baked lane" below for which certificates qualify and why).

Unlike the cbor form, a container carries the WHOLE claim as typed
rows on the staged theory chain: the system (kernel and policy
mechanisms composed in a closed loop), the specification (the
reach-by-deadline probability bound), the satisfaction judgment, and
the proof rows (the backward-induction table under a deadline-table
rule node). No sidecar header remains: every field is a row,
determined by rows, or provenance outside row-checkable reach.

**Content identity is proven at production, not assumed.** The
producer (`gymdp glc`, in the mtk repository) REGENERATES every
certificate from the pinned kernel builders and dynamic-programming
lanes — never converting bytes — and gates each one twice before
sealing: the regenerated cbor must byte-match the committed corpus
here, and the WF/B/S/C obligations must hold in exact rational
arithmetic over exactly the row content (absorbing self-loops
re-synthesized from the absorbing marks — determined, never stored).
Sealing is deterministic (canonical body transform, fixed skeleton
layout, RFC 8032 signing, public reproduction seed, timestamp 0);
every seal is double-emitted and asserted byte-identical, and the
row-content multiset is asserted to survive the canonical transform.
Per family, `out/<family>/glc/GLC-MANIFEST.json` records each
container's sha256, its exact obligation count, the source cbor path,
and the claimed bound verbatim.

## what the fast-path checker verifies

`glc-check` (qtsl repository, `crates/qtsl-geolog-provider`), gym
container mode: container and envelope integrity (grammar, signature,
body-hash binding, canonical-body fixed point), reference discipline
against the three pinned staged revisions, then obligation
ENUMERATION — every deadline-table rule node found in the rows is
discharged through the exact-rational deadline-table walk, which
recomputes from the node's closure alone: kernel groups summing to
one with probabilities in (0, 1], the initial distribution, the
policy's choice at every needed (state, step), the goal set evaluated
pointwise from the specification's own formula, the boundary and
one-step inequalities of the table, and the claimed bound against the
initial mixture. Dyadic-lane tables embed exactly in the rationals
(cell = mantissa / 2^k, canonically reduced): the incumbent checker's
integer inequalities and the walk's rational ones are the same
inequalities scaled by a positive constant, and the equivalence is
exercised per certificate rather than assumed — both checkers run on
every one of these claims (`make gymnasium-text` for the incumbent,
`make gymnasium-glc` for the containers). A container with no
deadline-table node REFUSES: it certifies nothing.

Every certificate of the cbor corpus has a container. The claim
former carries BOTH orientations of the probability wall species:
lower bounds spell as the GE face (`SProbGE`, the sub-solution
reading above), and a zero claimed bound — `frozenlake-4x4/exact_T5`,
whose goal is unreachable within its deadline — spells as the LE face,
`SProbLE(0, PDeadline(5, goal))`: the upper probability of hitting
within the deadline is 0, the possibilistic prohibition, with the
threshold inside the LE face's canonical interval [0, 1) — no
boundary special-case. The checker runs the SAME exact table in the
super-solution direction (boundary and step inequalities flipped, the
on-goal floor V_t ≥ 1 made explicit, and the initial mixture bounded
ABOVE by the claim). One exact table satisfies both faces with
equality, which is why a single backward induction discharges either
orientation. (An earlier revision of this corpus excluded exact_T5 as
unspellable under the then-staged (0, 1] threshold demand; the
spec-atoms re-founding landed the LE face and the exclusion retired
with the 29/29 reseal.)

## the baked lane (`out/<family>/glc-baked/`)

The staged containers above cite three staged theory revisions and
carry the full decision frame (kernel, action layer, policy rows).
The baked theory closure factors that frame differently: its chain
species is a pure discrete-time Markov chain — transition rows
`dprob {src, dst} → Q`, initial rows `dinit {chain, state} → Q`,
absorbing states as explicit self-loop rows at probability one, and
NO action or policy layer (the decision former, action-keyed with the
policy a separate mechanism, is a sibling species of the closure, not
yet part of it). A hitting-lane certificate can ride the chain
species faithfully exactly when the underlying decision problem
admits a **stationary composition**: one action per state that is
admissible at every step of the deadline (equivalently, the selection
lies in the argmax at every step, so the composed chain's exact value
equals the optimal value and the corpus table + claimed bound carry
over **byte-verbatim** — the baked containers certify the same
numbers, not recomputed ones; on the LE face every action is
admissible, since the table there bounds from above).

The corpus splits 9 / 29 on that criterion, and the split is semantic
content, not an artifact:

- **9 compose** (these directories carry containers): all 6
  `frozenlake-4x4-det` certificates and `cliffwalking-det`'s
  `hitting_exact_T13` (deterministic dynamics: the optimal policy is
  stationary), plus `frozenlake-4x4`'s `exact_T5` (LE face) and
  `exact_T10`.
- **20 are genuinely time-varying** (no containers; the exclusion is
  data, recorded per certificate): slippery FrozenLake at deadlines
  T ≥ 25 (`frozenlake-4x4` T25/T50/T100 exact + all four dyadic,
  `frozenlake-8x8` all 8, `frozenlake-4x4-sr34` all 5) has states
  where NO single action is admissible at every step — near the
  deadline the optimal policy gambles, far from it the policy
  detours, and the two choices differ at the same state. Each family
  manifest records every excluded certificate with the obstructing
  states (`exclusions[]`: stem, source cbor, the states at which no
  uniformly admissible action exists). `frozenlake-4x4-sr34` and
  `frozenlake-8x8` carry exclusion-only manifests — zero containers,
  present so the exclusion record lives where a reproducer looks.
  These 20 claims remain fully checkable on the staged lane above
  (lanes version; they never die); their baked re-emission awaits the
  decision-former sibling species.

`GLC-MANIFEST.json` in each `glc-baked/` directory records the
containers' sha256 lines, per-certificate claim expectations
(`--expect-claims qtsl_deadline_table=1`), the composed selection's
provenance for each positive, and the exclusions. One reading note:
in `frozenlake-4x4-det`, `exact_T100` and all four `dyadic_k*_T100`
containers are **byte-identical** (one sha256, five names) — a
deterministic chain's dyadic tables reduce to the same 0/1 rationals
at every mantissa width, and nothing else in the rows distinguishes
the five certificates; the stems live in filenames and manifest
records, not in the sealed content. That is content addressing
reporting the truth, not a copy error.

The baked checker invocation (`glc-check --baked-cert`) verifies
container + envelope integrity, then discharges the claim row against
the DERIVED index of the baked closure: the deadline-table walk
recomputes well-formedness, boundary, step, and claim obligations in
exact rational arithmetic from the node's closure alone, over the
chain's dprob/dinit rows — the same mathematics as the staged walk,
consumed through the chain frame. Zero-claim containers REFUSE.

The baked lane carries its own thirteen-class mutant battery
(`out/frozenlake-4x4/glc-baked/mutants/`, verdicts pinned in the
manifest): six semantic loci on the `exact_T10` base FAIL at their
designed obligations (raised table head, inflated claim, bumped
outcome probability, cut initial mass, emptied goal, raised
boundary cell); a claim-row pop REFUSES (a certificate that claims
nothing certifies nothing); four envelope loci REFUSE at their
integrity gates (body byte, signature byte, truncation, magic) plus a
container-version bump REFUSES as unsupported; and the LE-face
falsifier on the `exact_T5` base (V_0 at the initial state raised)
FAILs at the claim obligation.

## the staged-lane mutant battery

Thirteen classes on the `frozenlake-4x4` bases
(`out/frozenlake-4x4/glc/mutants/`), verdicts pinned in the manifest:
six semantic loci re-sealed from perturbed rows on the `exact_T10` GE
base (a raised table head, an inflated claim, a perturbed kernel
probability, an emptied goal region, a vacated policy slot, a raised
terminal cell — each FAILs at its designed obligation); five envelope
loci as byte edits on the sealed base (body byte, signature byte,
truncation, magic, version — each REFUSES at its integrity gate); one
coherent-forgery CONTROL (the kernel's slip rate moved with table and
claim recomputed — a true statement about a different system,
ACCEPTED in-lane by design); and one LE-face semantic locus on the
`exact_T5` base (V_0 at the initial state raised from 0 to 1 — the
super-solution obligations all SURVIVE the raise, so the initial
mixture exceeding the claimed upper bound 0 is the single designed
falsifier; FAILs at the claim obligation).
The control documents the checker's reach honestly: the walk verifies
internal system/specification/proof consistency; binding the kernel
to the named environment is provenance, and the falsifier with that
reach is regeneration byte-identity against this corpus (the
manifests' sha256 lines plus the producer's cbor gate).

## re-checking

```
make gymnasium-glc
```

builds `glc-check` from the pinned qtsl checkout and runs the
manifest-driven driver (`gymnasium/glc-recheck.ts`): it discovers
every `GLC-MANIFEST.json` under `gymnasium/out/`, dispatches on each
manifest's `mode_flag`, verifies each container's sha256, and demands
the manifest's exact verdict and obligation count per positive and
per mutant class. Sibling Gymnasium container families (the
reward-weighted and episode-process lanes) drop in as manifests with
zero driver edits — the baked hitting lane above landed exactly that
way.
