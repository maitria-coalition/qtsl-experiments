# Mini-Fab dispatch synthesis — the sealed certificate family

One command re-verifies the whole family and prints a verdict table:

```
make minifab-dispatch
```

This is the reproduction spine for the discrete Mini-Fab exhibit (the
QTSL book's semiconductor worked example): the same five-machine,
six-step re-entrant line, modelled as a plated hypernet in exact
rational arithmetic, carried through four certificate containers that
tell one story —

| container | what it certifies |
|---|---|
| `minifab.glc` | the **closed baseline**: twelve certificates over the closed net — the work-potential ranking, lot count, six machine conservation bounds, congestion, a ceiling, and the arrival-potential + step-budget pair that yields the worst-case *all lots arrived by* $638N-12$ counting-step deadline, over **every** scheduler interleaving. |
| `minifab-gated.glc` | the **open system**: the plant re-stated as a comb with one policy-shaped hole (the stage-6 start gate). The hole count is itself a checked claim, and the gated deadline dictionary certifies the horizon $T^{*} = \mathrm{fold}_D(m_0) = 626N$ that a policy filling the hole can be held to. |
| `minifab-supervised.glc` | the **synthesized supervisor**, closed composite: the latest-safe-unlock winner found by joint policy+certificate search, with certified completion and *all lots arrived by* $626N$ (52584 at $N=84$, against the 53580 no-policy baseline). |
| `minifab-simplex.glc` | the **Simplex composite**: a certified switching envelope filling the *same* hole, so an uncertified advanced policy can drive dispatch while the certificate bounds the downside — the certificate never reads the policy. |
| `carved/minifab.glc`, `carved/minifab-gated.glc`, `carved/minifab-supervised.glc` | the **carved-clock reseals** of the baseline, open, and supervised containers: every logistical transition declares dwell exactly 1, so timing is owned by the clock semantics and the same checked rows yield native *wall-clock* bounds (rounds-to-all-arrivals at most $638N-12$ open, $626N$ gated) rather than a counting-step proxy. |

The first four containers state their deadlines in counting steps
(each transition firing is one step); the carved reseals state the
same content against the semantic wall clock. Companion containers of
the same family (the throughput-optimality certificate among them)
drop in as further manifest entries with no driver edits.

## What runs, and what each leg witnesses

The two legs partition honestly:

1. **Sealed containers** (`out/GLC-MANIFEST.json`, via `glc-check`
   built from the pinned qtsl checkout — checkers are imported, never
   vendored): everything the bytes witness. Envelope integrity first,
   then every claim row is re-derived through the exact-rational
   ground provider, gated by the manifest's per-relation claim counts
   (the count gate is load-bearing: deleting a whole relation leaves
   every surviving claim true, and only the count refuses it). Every
   artifact is sha256-verified against the manifest before checking,
   and the committed integrity mutants (flipped body byte, flipped
   tail byte, truncation, magic, version) must REFUSE.

2. **Producer batteries** (the pinned mtk checkout's `semicon` test
   suite): everything sealed bytes cannot witness — the campaigns
   re-derived from the model in exact rational arithmetic, the
   exact-tier sequent mutants and encoder-mirror mutants killed at
   their committed loci, and the strategy-tier replays (the synthesis
   grid including the refused never-opening gate, the winner replay
   attaining $T^{*}$ pointwise, and the envelope's bench/recovery
   runs). The manifest pins a minimum test count so an over-narrow
   filter cannot pass vacuously. The suite is self-contained library
   tests — no external stores, no network.

Semantic glue that is argued rather than row-checked (the enabledness
side of completion, and the residue items named below) is graded
explicitly in the worked-example chapter; this kit re-verifies exactly
what is checkable and re-runs exactly the batteries that guard the
rest.

## Honest edges, named

- The gated and supervised containers carry two deliberate encoding
  variants of the same plant (a native-read gate versus a one-shot
  armed key); each is presented with its own tier and neither borrows
  the other's numbers.
- The lower bracket of the winner's bound (that $626N$ is *attained*)
  rides producer-tier exact-drop identities and replay; a
  checker-side row species for it is a named absence, not an implied
  presence.
- The open system's observation read arc is a named residue against
  the arc-separation census, and the filler-interface tie on a wired
  comb is enforced at the producer's encoder mirror; the
  corresponding checker-side evaluator is a named owner.

## Pins

The family's containers pin a newer theory-pair revision than the
sibling baked lanes, so the target builds its checker from its own
qtsl pin (`MINIFAB_QTSL_REV`, second checkout under `build/`); a
checker built at an older pin refuses these containers loudly
(`unknown_prereq`), never vacuously. When the older corpora re-emit
at this revision the pins converge and the second checkout retires.
The battery leg pins mtk the same way (`MTK_REV`); both pins advance
with the family.
