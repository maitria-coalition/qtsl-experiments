# Erratum to `FloorContiguity.lean` (2026-10-19)

`FloorContiguity.lean` is vendored **verbatim** from the original MAITRIA
archive. Its sha256 (`6e91d9cc…ad925a`, in `../../ORIGINALS.sha256`) is the
archive original's, and the Lean mutants are pinned to its line numbers, so
the file is not edited. This note supersedes one passage of its docstring.

## The superseded sentence (docstring, lines 20–22)

> The model is DELIBERATELY WEAKER than the net (no release meter, no
> start/finish meters, no station-1/2 capacities, no w-guards), so every run
> of the carved net projects onto an `MFSchedule` and the floor transfers a
> fortiori.

**This is false.** The model is weaker in the ways listed, but it is also
*stronger* than the net in two ways. So some lawful net runs have no
`MFSchedule` image, and the theorem does not transfer to the net as stated:

1. **Same-round fresh-token tick.** `hwin` puts a lot's ticks in the open
   window `Ioo start finish`. The net lets a lot take `tick_k` in its own
   start round when another lot's token made `tick_k` due. Regression test:
   `glc-replay/tests/mutants.rs::same_round_fresh_token_tick_is_lawful`.
2. **Same-round machine handover.** `hcap11`, `hcap15` and `hcap55` require
   strict separation (`finish < start`) of services on one diffusion
   machine. The net lets a start fire in the round in which a finish frees a
   machine. When two diffusion starts (start1 and start5, which are
   different schemas) both fire in that round, one of them has to take the
   machine that was just freed. In that case the three closed service
   intervals pairwise intersect, and no Bool machine labelling exists.
   Regression test: `same_round_machine_handover_is_lawful` (found
   independently by the astra and opus reviews).

## What still holds

* The theorem itself, `floor_contiguity : 4 ≤ N → (∀ l, finish l 4 ≤ B) →
  255N + 375 ≤ B`, is kernel-checked about `MFSchedule`. The axioms are
  {propext, Classical.choice, Quot.sound}, there are 0 sorries, and the four
  mutants refuse.
* Model R (`../../README.md` §R) admits both moves. For N ≤ 7 the optimum
  is certified on R from both sides.
* For N ≥ 8 the floor is **not proved for the net**. Closing it needs either
  a normalisation lemma that removes both moves from some optimal run, or a
  restated `MFSchedule` (an `Ico` window with the due-by-another-lot side
  condition, *and* a non-strict same-machine handover with the
  two-starts-in-one-round case), followed by a re-proof. Changing `Ioo` to
  `Ico` alone is not enough.
