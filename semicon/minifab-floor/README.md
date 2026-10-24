# Mini-Fab optimum on the discrete model: what is rechecked, and how far it reaches

Claim (the qtsl book, worked-example chapter `qtsl-minifab-example.tex`,
§"Optimal throughput, exactly", source lines 1331–1398; restated in this
repository's chapter, `chapter/sections/semicon.tex`):

    T_opt(N) = 255N + 371 + d(N),   d = 0 (N ≤ 2), 2 (N = 3), 4 (N ≥ 4)

T_opt(N) is the round of the **N-th arrival into q6** (the N-th `finish5`)
in the carved-clock Mini-Fab. It is not the round at which all N lots have
shipped. This directory rechecks the claim. It does **not** prove it for
every N. The model is the **discrete lane**: mtk
`crates/mtk-strategies/src/semicon/minifab.rs` @ `8904467`. Its process times
are T = (225, 30, 55, 50, 255, 10), its stations are (diff, impl, litho,
impl, diff, litho), its capacities are (2, 2, 1), and every row has dwell
exactly 1. It is *not* the fluid lane's occupancies m_k.

**Model caveat (2026-10-24).** This model departs from Kempf's 1994
Mini-Fab in a way that matters for any optimality claim. In the net, each
transition fires at most once per round (the meter law below). So in any
round at most one lot can start, receive a unit of service, or finish at a
given process step, even at the two-machine stations. In effect every step
is a single server. Kempf's specification lets the machines of a station
work in parallel. The results in this directory are exact for the model as
stated, and only for it. Whether they carry over to a net that follows
Kempf's specification is open. A re-model with per-machine parallelism is
in progress.

`make minifab-floor-recheck` (or `bash semicon/minifab-floor/recheck.sh`)
re-derives every receipt in `receipts/`. `MAX_N=5` gives a quick run, whose receipts go to `build/` rather than `receipts/`.
Requirements are listed at the end.

## Receipt status at this revision (read this first)

Each claim below names its receipt. Where a receipt is not in this
revision, the claim is marked **PENDING** and does not count as evidence
until a later revision ships the receipt.

| receipt in `receipts/` | what it contains at this revision |
|---|---|
| `z3-R.json` | model R **+ queue order**, z3 4.16.0: UNSAT at T_opt − 1 and SAT at T_opt for **N = 1..7**, with the witnesses re-validated |
| `sat-lane.jsonl` | model R **+ queue order**, CNF: **N = 1..7**. The UNSAT side is accepted by cake_lpr, lrat-check and drat-trim, and the SAT witness is re-validated. For every N the CNF, LRAT and SAT-CNF hashes are identical to those in the earlier revision's receipts (compared; same CaDiCaL build) |
| `fifo-exchange.json` | the executable check of the exchange lemma: 60 witnesses, 33 of them non-FIFO before normalisation |
| `originals-z3.txt` | the original instrument (a stricter model, not a net bound), N = 1..7, family table and mutants. The strengthened wrapper's checker accepts this file and rejects doctored copies; the file itself came from the earlier wrapper |
| `glc-replay.jsonl` | the decoded-container replay: N = 84 sealed (π\* 21,795; WC 21,875; two expected refusals with their causes and container hashes), and π\* instance-adapted at N = 1..12 |
| `lean.txt` | the strict script: BUILD ok, 0 sorry, audit 6/6 with {propext, Classical.choice, Quot.sound}, both audit-checker negatives rejected, control copy compiles, four mutants refused at exactly the named lines, `LEAN LEG OK` |
| `sat-lane-nofifo.jsonl`, `z3-R-nofifo.json` | the queue-order-free lane (`recheck.sh` leg 6): model R with **no queue-order constraint**, N = 1..5, two-sided in both encodings. Each UNSAT is accepted by cake_lpr, lrat-check and drat-trim, and each SAT witness is re-validated |
| `provenance.jsonl` | which run produced which receipt, with dates: legs 0–4 come from one full run of the previous revision's scripts; legs 5–6 come from a separate rerun after that run's leg-5 abort (fixed) |

## Scope in one table

| N | lower side (no schedule at T_opt − 1) | upper side (a schedule at T_opt) |
|---|---|---|
| 1..7 | model R + queue order: **certified** by CNF with LRAT (cake_lpr, lrat-check) and DRAT (drat-trim), plus z3. This bounds every net run **only via** R ⊇ net and the exchange lemma (§R), both pen-and-paper | **net replay**: π\* against the decoded container, instance-adapted (below) |
| 1..5 without queue order | **certified** on model R alone, which does not need the exchange lemma (CNF + LRAT/DRAT, and z3). This still rests on R ⊇ net (pen-and-paper) | — |
| 6, 7 without queue order | **not shipped**. It runs outside the default target (`NOFIFO_MAX_N=7`) and is compute-bound | — |
| 8..12 | **open** | net replay of π\*, instance-adapted (N = 10: 2925) |
| ≥ 8 in general | **open for the net**. The Lean floor 255N + 375 is proved only for the abstract `MFSchedule`, onto which the net does not project (two gaps, Findings 2) | — |
| 84 | open (as above) | net replay of π\* against the sealed bytes as they are: 21,795 |

The `--fifo` lane (rows with `"fifo": true`) reproduces the original
archive's evidence. Its CNF hashes match the earlier receipts for
N = 1..7, as compared above. It is labelled as R + queue order
everywhere, and it is never cited as a net bound without the lemma.

## What is checked, and by what

| side | N | route | checker / trust | receipt |
|---|---|---|---|---|
| lower | 1..7 | model **R** (below) with queue order (exchange lemma, §R), CNF order encoding `cnf_floor.py --fifo` | **cake_lpr**, lrat-check, drat-trim | `receipts/sat-lane.jsonl` |
| lower | 1..7 | model R with queue order, integer SMT `z3_floor.py --fifo` | z3 4.16.0 | `receipts/z3-R.json` |
| lower, without any queue-order constraint | 1..5 | model R, both encodings without `--fifo` | as above | `receipts/sat-lane-nofifo.jsonl`, `receipts/z3-R-nofifo.json` |
| exchange lemma, executable test | 2..4 | non-FIFO R-witnesses normalised and re-validated | `fifo_exchange_check.py` | `receipts/fifo-exchange.json` |
| upper | 1..12 | **decoded container** `carved/minifab-supervised.glc` + named transcribed remainder, π\* table, holds only on start1/start5, **instance-adapted** | public qtsl reader @ `a9bdb62`, container sha = GLC-MANIFEST | `receipts/glc-replay.jsonl` (`pi-star-instance` rows) |
| upper | 84 | the same, on the sealed bytes as they are | as above | `receipts/glc-replay.jsonl` |
| R is tight (not a net claim) | 1..7 | SAT model of R at T_opt, decoded and re-validated | `decode_cnf_witness.py` → `z3_floor.validate` | `receipts/sat-lane.jsonl` |
| original instrument, reproduced | 1..7 | `originals/.../minifab_idt.py` (verbatim) | z3 4.16.0 | `receipts/originals-z3.txt` |
| floor 255N + 375, N ≥ 4 (255N + 373, N ≥ 3), **`MFSchedule` only** | ≥ 3 | original `lean/IDT/FloorContiguity.lean` (verbatim; see its `FloorContiguity.ERRATUM.md`) | Lean 4.32.0-rc1 kernel, mathlib `6f60478`, axioms {propext, Classical.choice, Quot.sound}, 0 sorry, mutants M-A..M-D refuse at the named lines | `receipts/lean.txt` |
| IDT collapse to makespan | — | original `lean/IDT/DeterministicCollapse.lean` (+ import closure) | Lean kernel | `receipts/lean.txt` |

**The instance adaptation.** The sealed container is the N = 84 instance.
For the N = 1..12 rows the replay substitutes exactly two sealed facts in
the decoded net: the pending-lot count (the initial marking of q_pend, sealed
84) and the unlock threshold (the q6 multiplicity in the unlock row's
pre-set, sealed 84). Nothing else is changed (`replay.rs::adapt_instance`,
test `instance_adaptation_is_narrow`). These rows are "the sealed net with N
pending lots". They are **not** byte-faithful replays of a sealed N-lot
container, and each receipt row records the substitution
(`instance_adaptation`).

**The original instrument is not a bound for the net.** `minifab_idt.py`
uses strict-start tick windows (`s + 1`) and closed-interval machine
exclusion. Both moves of Findings 2 are therefore outside it, so its UNSAT
rows reproduce the original archive's result about a stricter model. They
are not independent evidence for the net's lower side.

## What this does and does not establish

* **N ≤ 7, both sides, with the receipts as listed above.** The lower side
  is decided on model R + queue order, by z3 and by CNF with LRAT/DRAT
  certificates, for N = 1..7. R contains every net
  run (argument in §R). The UNSAT proofs certify the generated DIMACS. The **trust base** for "no net run meets T_opt − 1" is
  therefore cake_lpr, plus the encoder `cnf_floor.py` (unverified Python,
  cross-checked by the separately written `z3_floor.py`), plus two
  pen-and-paper arguments: R ⊇ net and the queue-order exchange lemma (§R).
  The upper side is a net-level replay of π\* (instance-adapted). The values
  are 626, 881, 1138, 1395, 1650, 1905 and 2160.
* **N = 8..12: upper side only.** π\* replays at T_opt (2415, 2670, 2925,
  3180, 3435). Since a net run projects to an R-schedule, this also shows R
  is SAT at T_opt there. No lower bound is shipped for N ≥ 8. Earlier z3
  runs at N = 8, 9 (two-sided) and N = 10 (lower side) were made outside
  the recheck target, so they are not shipped as receipts.
* **N ≥ 8: the Lean floor is not a proof for the net.** `FloorContiguity.lean`
  proves 255N + 375 ≤ makespan for every N ≥ 4, but about `MFSchedule`. The
  net allows two moves that `MFSchedule` excludes (Findings 2). The
  docstring sentence "every run of the carved net projects onto an
  `MFSchedule`" is false and is superseded by `lean/IDT/FloorContiguity.ERRATUM.md`.
* **The replays use the sealed fraction of the container plus a named
  transcribed remainder.** The `.glc` bytes do not carry the whole net
  (Findings 1). These parts are transcribed, not decoded:
  * the produced sides (post-sets) of the release, start, finish, ship and
    unlock rows;
  * the zero-work guards consumed by finish_k and ship;
  * the start colour (initial work T_k on p_k);
  * the step→station map `STATION`;
  * the process times `T` (each of T_1..T_6 is then cross-checked against
    the sealed tick image);
  * the **round semantics itself**: phase order finish/ship → release →
    start/unlock → tick, dueness at the end of the previous round, and
    holds. This is an independent reimplementation of mtk
    `carved_script.rs`, and model R's soundness argument relies on the same
    phase order;
  * the closed-form π\* and work-conserving schedule families.

  All of this is in `glc-replay/src/replay.rs` (`T`, `STATION`, `compile`,
  the replay loop and the schedule-family section).
* **Attainment, not optimality among work-conserving policies.** At N = 84
  π\* replays at 21,795, and the work-conserving family `wc_lots` replays
  at 21,875. The replay shows only that these schedules are lawful and
  where they land. It does not show that `wc_lots` is the best
  work-conserving schedule. The replay also stops at the goal round, so it
  establishes the goal prefix only, not later firings.
* **What "makespan" means here.** Every number above is the round of the
  **N-th arrival into q6** (the N-th `finish5`). It is *not* the round at
  which all N lots have shipped: step 6 (litho, T_6 = 10) and ship come
  after it and are deliberately outside the measured quantity.

## Model R (the relaxation the SAT/SMT lanes decide)

For each lot i < N and each step k = 1..5 there are three integer rounds:
release `rel_i`, start `s_ik` and finish `f_ik`.

* **Step 6 deleted.** The goal is the N-th `finish5`. start6, tick6 and ship
  only take the litho token and give it back. All arcs are consuming and
  there are no inhibitors, so deleting those firings from a lawful run leaves
  every other firing due and pickable.
* **Meter law.** Each schema fires at most once per round: the `rel_i` are
  distinct, the `s_·k` are distinct, the `f_·k` are distinct, and at most
  one `tick_k` fires per round.
  This is the net's own rule, not a relaxation, and it is where the model
  departs from Kempf (see the model caveat at the top).
* **Dueness.** A row fires at r only if it was enabled at the end of r − 1.
  So `s_i1 ≥ rel_i + 1` and `s_ik ≥ f_i,k−1 + 1`. Queue tokens are colourless
  (unit colour), so the lot identity of a start can always be traced to a
  token that was already queued at the end of r − 1. Some such token exists,
  because the start was due.
* **Ticks.** Lot i's T_k ticks lie in `[a_ik, f_ik − 1]`, with one tick_k per
  round across lots. This is encoded exactly by Hall's condition on
  intervals (convex bipartite matching). Normally `a_ik = s_ik + 1`, but
  `a_ik = s_ik` is allowed whenever some other lot j's step-k service spans
  the end of `s_ik − 1` (`s_jk ≤ s_ik − 1 ≤ f_jk − 2`). This is the
  **same-round fresh-token tick** (Findings 2). The side condition is
  *weaker* than the net's "j's token still has work ≥ 1": it only asks that
  j be in service. That enlarges R, which is the safe direction for a lower
  bound. But it means an R-witness validated by `z3_floor.validate` is not
  thereby a net run. The upper side comes from the net replay. The encoding
  assumes T_k ≥ 2 (nonempty windows), which is asserted.
* **Machines** (station σ, capacity c). A step holds a machine at the end of
  rounds s..f−1. For pickability, occupancy after the start phase is ≤ c.
  For dueness, occupancy at the end of the previous round is ≤ c − 1. This
  admits the **same-round machine handover** of Findings 2.
* **No forcing.** Any due row may be left unfired, so the hold axis is the
  whole roster.
* **Lot relabelling.** Every constraint above is invariant under permuting
  lot indices, and the releases are distinct. So `rel_0 < rel_1 < …` is a
  pure relabelling, and the encodings impose it unconditionally.
* **Queue order (`--fifo`; the exchange lemma).** The `--fifo` lanes also
  impose `rel_i < rel_j ⇒ s_i1 < s_j1` and
  `f_i,k−1 < f_j,k−1 ⇒ s_ik < s_jk` for k = 2..5. This is **not** argued from
  net semantics. It is a property of R itself:

  > *Lemma.* If R has a schedule meeting horizon H, it has one that also
  > satisfies queue order.
  >
  > *Proof.* Call lot i's steps k..5 its *k-suffix*. Every R constraint
  > except the dueness chain mentions a lot only through the multiset of its
  > per-step intervals:
  > * distinctness is per step;
  > * Hall's condition is per step, over the multiset of step-k windows;
  > * the early-tick condition for a step-k service refers to the other
  >   step-k services;
  > * machine occupancy is per station, over the multiset of intervals;
  > * the horizon applies to every f_i5.
  >
  > So reassigning k-suffixes among lots preserves all of these. Process
  > k = 1, …, 5 in turn. At stage k, reassign the k-suffixes so that the
  > j-th earliest arrival into q_k (by `rel` for k = 1, by `f_·,k−1`
  > otherwise; these values are distinct) receives the k-suffix with the
  > j-th earliest `s_k`.
  >
  > The chain `s_k ≥ arrival + 1` survives this. A matching of sorted
  > arrivals to sorted starts that satisfies `start ≥ arrival + 1` exists
  > whenever any such matching exists: if a ≤ a′ and b ≥ b′ with b ≥ a + 1
  > and b′ ≥ a′ + 1, then b′ ≥ a′ + 1 ≥ a + 1 and b ≥ b′ ≥ a′ + 1, so the
  > crossing pair can be uncrossed. Stage k leaves steps < k untouched, and
  > the step-k pairs it creates are sorted, so queue order at q_1..q_k holds
  > afterwards. ∎

  So UNSAT of R + queue order at H implies UNSAT of R at H. The lemma is
  pen-and-paper, not machine-checked. `fifo_exchange_check.py` tests it: it
  samples non-FIFO R-witnesses from the encoding *without* queue order
  (N = 2..4, several horizons and seeds), normalises them by the stage-wise
  sort above, and re-validates them round by round. Its receipt counts how
  many raw witnesses violated FIFO, so the test is visibly non-vacuous. The
  `*-nofifo` receipts decide R with no queue-order constraint at all, for
  N = 1..5, and agree. That search is 10–60× slower. N = 6, 7 without the
  reduction are not shipped.

**Why R bounds the net.** Every run of the carved net, under any
supervision, gives an R-schedule once lot identities are traced as in
"Dueness". So `UNSAT(R, T_opt − 1)` refutes every policy. R is looser than
the net: it drops forcing, guards and the release/finish details, and it
weakens the early-tick condition. That is the safe direction for a lower
bound, and it is why the upper side comes from the net replay, not from R.

The CNF lane (`cnf_floor.py`) is a second encoding of R, written separately
from `z3_floor.py`. The two share the model text above and nothing else. It
uses order literals `[x ≤ t]`. Each indicator emits only its "forced true"
direction and sits only on the side of a constraint that it tightens, so a
spurious TRUE can only hurt the solver. Setting each indicator exactly when
its antecedent holds extends any R-schedule to a CNF model. That is why the
CNF is equisatisfiable with R. *Observed during development, not shipped as
a receipt* (z3 at N = 4, H = 1394, with queue order): the floor stays UNSAT
without the impl/litho capacities and without the tick2–4 meters, and it
becomes SAT if the diffusion capacity, the tick1 meter, the tick5 meter, or
diffusion dueness is dropped.

## Lean: what the theorem is about

`MFSchedule N` (FloorContiguity.lean) is an abstract schedule with these
parts:

* per-lot chains with one-round dwell;
* each step's ticks as a `Finset` of size T_k inside the **open** window
  `(start, finish)`;
* per-step cross-lot tick disjointness (the meter law);
* diffusion capacity 2 in machine-assignment form (`mach1`/`mach5 : Bool`,
  with same-machine services **strictly** separated, `finish < start`).

It has no release/start/finish meters, no impl/litho capacities and no
guards. `floor_contiguity : 4 ≤ N → (∀ l, finish l 4 ≤ B) → 255N + 375 ≤ B`.

Relation to the net: if every net run had an `MFSchedule` image, the floor
would transfer a fortiori. Chains and meters do project directly. The two
places where the projection fails are named in Findings 2, and both are
covered by model R but not by the Lean floor.

## Findings

1. **The public-artifact gap, corrected.** The proofs existed in the
   original MAITRIA archive (2026-07/08; `ORIGINALS.sha256`; vendored here
   verbatim) but had never reached a public repository. A second point is
   new: the sealed `.glc` containers do **not** carry the net needed for a
   byte-level replay.
   * Per transition, the bytes hold the clock, the grain (consume/read
     counts per place) and, for the six tick rows only, a same-place scale
     view (reset w ↦ w − 1 and landing enclosure [0, T_k − 1]).
   * They hold no post-sets for release/start/finish/ship/unlock, no
     consumed-side guards for finishes and ship, and no start colour. This
     is the encoder's own stated honesty line (mtk `content.rs`
     `PlatedTransitionDecl`).
   * The original replay's "container bytes" were the in-memory
     `PlatedModel`.

   `glc-replay/` now decodes everything that *is* sealed, through the public
   checker's reader (container grammar, signatures, body hash and baked
   vocabulary, all from qtsl @ `a9bdb62`). It checks the transcribed
   remainder against every sealed fact it touches (T_k from the tick images,
   pre-sets, clocks, marking, and the unlock threshold from the grain), and
   then replays. The book's sentence "π\*'s explicit firing table replays
   against the sealed exhibit bytes" is therefore true only of the sealed
   fraction. mtk itself cannot be built from public sources (its pins are
   `*-dev` repositories). The native-read `carved/minifab-gated.glc` (17
   plates, read arcs) belongs to a different lane from the replayed
   flattened `minifab-supervised` (18 plates, one-shot key), and the replay
   refuses it by name.
2. **Two projection gaps between the net and `MFSchedule` (no number
   changes).** Under the round semantics, a row is due when *some* token
   enables it, and which token a firing takes is the scheduler's choice.
   * **(a) Same-round fresh-token tick.** Lot B may start step k at round r
     and take tick_k at r, when another lot's p_k token made tick_k due.
     Test `same_round_fresh_token_tick_is_lawful` replays this against the
     decoded container and reaches p1 = {217, 224}. That marking is
     unreachable if ticks must follow starts. `MFSchedule.hwin` (`Ioo`) and
     `minifab_idt.py` (`s + 1`) both exclude it.
   * **(b) Same-round machine handover** (found by the astra and opus
     reviews). A start is due if a diffusion machine was free at the end of
     r − 1, and in phase 2 it may take a machine that a phase-0 finish freed
     in round r. start1 and start5 are different schemas, so both can fire
     in one round, and then one of them must take the just-freed machine.
     Test `same_round_machine_handover_is_lawful` replays a three-lot run
     (last arrival 1219). In it the closed diffusion intervals [3,453],
     [453,679] and [453,709] pairwise intersect, so no Bool labelling meets
     the strict `hcap11`/`hcap15`/`hcap55`. `minifab_idt.py`'s closed-interval
     machine exclusion also excludes it.

   Model R contains both moves (early window; pickability counts occupancy
   after phase-0 finishes). For N ≥ 8, the Lean floor's coverage of such
   runs is **open**. Closing it needs either a normalisation lemma that
   removes both moves from some optimal run, or a restated `MFSchedule`
   (`Ico` window with the due-by-another-lot side condition, *and* a
   non-strict handover with the two-starts-in-one-round case), followed by a
   re-proof. `Ico` alone is not enough.
3. **The Lean mutants** refuse at exactly the lines the original receipts
   name. `recheck_lean.sh` fails if they refuse anywhere else or for an
   environment reason (receipt `lean.txt`):

   | mutant | patch | what it breaks | refuses at |
   |---|---|---|---|
   | M-A | `mutant-A-cap3.patch` (diffusion capacity 3) | the two Bool-pigeonhole forcing steps | 343, 452 |
   | M-B | `mutant-B-t5-254.patch` (T_5 = 254, statement held at 255N + 375) | `finish5_ge`, `entrant_bound` | 145, 212 |
   | M-C | `mutant-C-drop-cap55.patch` (drop `hcap55`) | both 257-round handover derivations | 345, 455 |
   | M-D | `mutant-D-drop-tick1-meter.patch` | `window_count`'s meter use | 184 |

   The docstrings of the vendored originals cite archive notes
   (preregistration and findings files) that are not part of this release.
   The statements, proofs and receipts here stand without them.

## Receipts: what is byte-reproducible

* `sat-lane*.jsonl`: one row per N, with the sha256 of the CNF, LRAT and DRAT
  and the three checkers' verdict lines. The LRAT/DRAT proofs (N = 7: about
  1 GB) are **not retained**. The CNF is a pure function of (N, H, flag), so
  its hash reproduces anywhere. The LRAT/DRAT hashes reproduce with the
  **same CaDiCaL build** (the pinned rev, built with the same compiler and
  flags, single-threaded) and can differ across builds. To compare, rerun
  `sat_lane.sh N DIR [--fifo]`.
* `z3-R*.json`: results and witnesses only. Wall times go to
  `build/minifab-floor/z3-R*.log`. The witnesses are reproducible with the
  pinned z3 4.16.0.
* `glc-replay.jsonl`: only replay rows (deterministic). The cargo test
  summary is checked, not stored.
* `lean.txt`: deterministic given the pinned toolchain and mathlib.
* A reduced run (`MAX_N < 7` or `NOFIFO_MAX_N < 5`) writes to
  `build/minifab-floor/receipts-quick/`, never into `receipts/`.
  `LEGS="5 6"` reruns individual legs and appends dated rows to
  `provenance.jsonl`.
* Checker self-tests that run on every recheck:
  * `z3_floor.py --selftest`: the witness validator must reject backwards,
    too-short, missing-lot, four-step and over-horizon witnesses. It
    checks shape and domains, and that zero work is left after the EDF
    tick assignment.
  * `recheck_originals_z3.sh` parses the original instrument's output and
    rejects `unknown`, `FAIL`, missing or duplicated rows; it is tested on
    doctored copies.
  * The replay must refuse for the *named* reason. A missing gated
    container is a WRONG REFUSAL (rc ≠ 0), and leg 5 checks this.
  * Leg 5 also asserts the sealed N = 84 values and the exact refusal set.

## Requirements

x86-64 Linux (`cake_lpr.S` is CakeML's x86-64 build; upstream ships a
separate ARM build that is not wired in here). bash, GNU coreutils
(`sha256sum`), gcc, make, git. python3 with the z3 bindings at exactly
4.16.0, e.g. `pip install z3-solver==4.16.0.0`. `recheck.sh` refuses other
versions unless `Z3_VERSION_POLICY=any`, in which case it records the
version it used. cargo (builds use `--locked`). elan/lake for Lean
v4.32.0-rc1. `lake exe cache get` needs the network; without it, mathlib
builds from source. The SAT lane writes about 1 GB under `build/` at N = 7.
The vendored originals check with `cd semicon/minifab-floor && sha256sum -c
ORIGINALS.sha256`.

## Files

* `z3_floor.py`, `cnf_floor.py`, `sat_lane.sh`, `decode_cnf_witness.py`:
  model R in two encodings. `fifo_exchange_check.py` tests the
  queue-order lemma.
* `recheck_originals_z3.sh`, `recheck_lean.sh`, `lean/AxiomAudit.lean`:
  rechecks of the original artifacts.
* `glc-replay/`: the decoder, an independent round-semantics replay, the
  π\*/WC tables (canonical dump checked byte-identical to the original
  `rust-tables.txt` by `recheck.sh`), and 9 mutant/witness tests (including
  both projection gaps and the instance adaptation).
* `lean/IDT/FloorContiguity.ERRATUM.md`: supersedes the vendored file's
  false projection sentence.
* `RELEASE-CHECK.md`: the release check (licence and provenance).
* `originals/`, `lean/IDT/*.lean`, `lean/BernsteinLogicAux/Imprecise/Credal.lean`,
  `lean/lean-toolchain`, `lean/lake-manifest.json`: original files,
  verbatim, each checked against `ORIGINALS.sha256`.
