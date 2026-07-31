# circuits — certificates for the neural-model-checking benchmark

Measured rows of the QTSL certificate route against Neural Model
Checking's benchmark suite (Giacobbe–Kroening–Pal–Tautschnig, NeurIPS
2024, arXiv 2410.23790, extended at NeurIPS 2025; pinned upstream tree
`neuralmc-pin/`, fetched at an exact commit by `make fetch-pins` —
provenance: `neuralmc-pin/VENDOR-PIN.md`), at **full coverage of the
combined 2025 suite: 634/634 tasks across all three property classes**
— pure liveness 219/219 (245 certificates, committed here), safety
206/206 (222 certificates), safety+liveness 209/209 (418 certificates:
one safety-window + one recurrence certificate per task). The
published field's own arrays (see the caveats box) give the strongest
single tool 554/634 and the per-task union of every published baseline
617/634: seventeen tasks — twelve pure-liveness, five blink
safety+liveness — are completed by nothing published, and certified
here. Where their per-task verdict is Bitwuzla's bare "unsat" on a
query about a quantized network, each task here ships a certificate —
a piecewise-affine rational ranking and/or a safety-window obligation
set, a handful of Bernstein rows — replayed exactly by a standalone
checker, with the consuming lemmas machine-checked in Lean and every
spec-rewrite rule machine-checked in both Lean and HOL4.

**The vga row, stated carefully:** vga is the one family the
NeurIPS'24 pipeline systematically fails (training local minima — their
own Limitations section); this route certifies all 10 vga instances
with a closed-form lexicographic two-counter ranking, 30 rows each,
~1 ms per certificate, sim-tight. Their NeurIPS'25 successor method
solves vga 10/10, so the surviving completion contrast is elsewhere
(their '25 method: load_store liveness 0/16, blink liveness 14/25,
blink SL 11/25) — see the caveats box below before quoting any
comparison.

## run it

From the repository root:

```
make circuits   # fetch the benchmark pin, re-check all 245 committed
                # liveness certificates + the mutant battery, rebuild
                # tables + PDF
```

The driver is `recheck.ts` (node ≥ 22, no packages): an accept sweep
over every committed fixture, then the ported mutant battery — 4
positive controls + 17 layer-targeted mutants, generated in-memory
from pristine committed fixtures, each required to REJECT with the
attacked check named. The reference checker it drives comes from the
pinned qtsl checkout (`checkers/circuits/ck_manifest.py` — python at
the current pin, with a replacement in progress upstream behind the
same CLI contract). To check one instance by hand:

```
python3 build/qtsl/checkers/circuits/ck_manifest.py circuits/fixtures/vga_1.cbor
```

## trust ledger

1. **Lemma (machine-checked)**: the demonic ($\forall$-input) recurrence/Streett
   ranking lemma —
   `lean/BernsteinLogicAux/StreettDemonic.lean` in the qtsl repository
   (`frequently_target_of_ranking`, plus the general pair form and the
   multi-pair conjunction; no measure theory, imports = Archimedean +
   Linarith). Lean build receipt: exit 0. (The module ships in the qtsl
   repository at that relative path, with a minimal Lake manifest.)
2. **Rows (recomputed)**: the reference checker (`ck_manifest.py`,
   imported from the pinned qtsl checkout's `checkers/circuits`) —
   standalone, zero producer imports, own tensor-Bernstein in exact
   $\mathbb{Q}$. Check ledger C0–C6 in its
   docstring; an ACCEPT = the lemma's hypotheses hold for the manifest's
   step system. Bernstein certifies over the real box $\supseteq$ integer states
   (sound; exact for these unit-descent certificates — every family's
   worst-case bound is sim-tight, see groundtruth output).
3. **Manifest = SV semantics (producer seam, narrowed)**: hand
   transcription, cross-checked at production time by a dual-simulator
   instrument (`groundtruth.py`, in the migrated producer layer — see
   the files table below): two independent
   simulators (imperative SV transcription vs manifest-cell
   interpretation) agree **exhaustively** on the full state $\times$ input box of
   the smallest instance of every family, sampled (20k points, seeded) on
   the largest. The FSM families exercise this seam hardest — three
   blocking-cascade subtleties are load-bearing (i2c's reset cascades
   into the increment and lands cnt = 1; lcd's state-0 completion lands
   (1,1) and its enable-entry lands (3,1)) and the certificates are
   exact, so a one-step transcription error refuses a row (mutant M13
   demonstrates). The shared-author residue this item used to declare
   FIRED and was repaired (2026-07-21): the safety campaign's
   independent re-transcription of every design caught a shared
   misreading of the lcd counter's wrap point, latent in the
   already-swept liveness manifests — exactly the failure class the
   declared residue named. The lcd manifests and certificates were
   regenerated and the full battery re-run green within the hour; the
   committed fixtures are the repaired ones. Remaining residue: the
   named successor checks are a yosys-normalized netlist read, then a
   Lean netlist bridge.
4. **Spec reduction (producer trust, documented per fixture)**: LTL →
   $\mathsf{GF}\,\gamma$ by the three classical rewrites (spec census); for
   the X-specs the one-step unfold is additionally witnessed by the
   manifest's own reset cells; atom maps taken from the runners' nuXmv
   LTLSPEC lines (i2c's `enable` = `scl_not_ena = FALSE`; load_store's
   `vol = N` vs their `sig` encoding is GF-tail-equivalent since
   $sig' = (vol' = N)$ on every step).

## coverage (pure-liveness class: 219 of 219 tasks; 100% of every family)

The 2025 edition grew the pure-liveness class 194 → 219 by adding the
blink design; this table is the class certified by the fixtures
committed here. (The 2024 paper's original suite is these families
minus blink: covered 194/194.)

| family | tasks | certs | shape |
|---|---|---|---|
| delay 1–16 | 16 | 16 | 1 ranking piece, 2 rows |
| gray 1–11 | 11 | 11 | wrap → 2 pieces, 4 rows |
| PWM 1–12 | 12 | 12 | input-split cells (8×swv), 2 pieces, 34 rows |
| seven_seg 1–15 | 15 | 15 | blocking-order rst; 3 pieces, 6 rows |
| 2_delay (X-spec) | 16 | 16 | X-atom collapses; identical rows |
| 2_gray (X-spec) | 11 | 11 | X-atom collapses; identical rows |
| 3_gray (conjunction) | 11 | 22 | two independent pairs per task |
| vga 1–10 | 10 | 10 | **NMC'24's systematic failure**; lex two-counter, 9+1 pieces, 30 rows |
| load_store 1–16 | 16 | 16 | triangle wave; two-mode pieces, 12 rows |
| thermocouple 1–17 | 17 | 17 | 4-state FSM (junk state 3); 6 pieces, 14 rows |
| i2c 1–20 | 20 | 20 | stretch-freeze + junk-scl lattice; 8 pieces, 22 rows |
| lcd 1–14 | 14 | 14 | double blocking-cascade; 8 pieces, 20 rows |
| uart 1–10 | 10 | 10 | closed-form sketch mechanized; 2–3 pieces, 4–6 rows |
| 2_seven_seg (conjunction) | 15 | 30 | pair 1 = spec-1 cert; pair 2 flips ds polarity |
| blink 1–25 ('25 addition) | 25 | 25 | led lags mode by a blocking read; 4 pieces, 12 rows |

blink is the '25 class's hardest family for published tools (their
MILP method 14/25, NMC'24-rerun 0/25, nuXmv 8/25, rIC3 6/25, ABC
5/25); the closed-form species takes all 25 at N up to 2^32 with no
change of certificate shape.

Every ranking is a CLOSED-FORM per-region steps-to-target function —
the per-mode LP synthesis provisionally priced for the FSM families was
never needed (search-free; every ranking row discharges with margin
exactly 0). uart_transmit_11.sv is a NeurIPS'25 spare, unused by both
papers' task lists.

The **safety class (206/206, 222 certificates + 206 chain sidecars)**
and the **safety+liveness class (209/209, 418 certificates + 209 chain
sidecars)** are certified by the same calculus — safety windows over
the doubled wire space with obligations on the exact one-step image;
for safety+liveness one window fixture plus one recurrence fixture per
task, chained from the verbatim spec text by six until-introduction
rewrite rules proved in both Lean and HOL4. Their receipt tables are
vendored here (`SAFETY-RECEIPTS.md`, `SL-RECEIPTS.md`, each carrying
its trust ledger); their fixtures, chain sidecars, and standalone
checkers land with the producer layer's migration.

Per-instance + per-family receipts: `SWEEP-RECEIPTS.md` (generated;
carries the measurement-rider columns — producer wall with device, check
wall, family-invention wall). End-to-end timing comparison incl.
invention walls: `INVENTION-TIME-COMPARISON.md`. Their published
comparison context is quoted, not re-run.

## files

| file | role |
|---|---|
| `fixtures/` | 245 CBOR certificates (0.4–2.7 KB each), committed byte-for-byte — the full pure-liveness class |
| `recheck.ts` | re-check driver: accept sweep over `fixtures/` + the ported mutant battery (untrusted orchestration; the checker is the instrument) |
| `neuralmc-pin/` | the upstream benchmark tree at a pinned commit (`make fetch-pins`); provenance + pruning rationale: `neuralmc-pin/VENDOR-PIN.md` |
| `SWEEP-RECEIPTS.md` | per-instance + per-family receipts from the pure-liveness production sweep |
| `SAFETY-RECEIPTS.md` | per-instance + per-family receipts from the safety-class sweep (206/206; artefacts migrate with the producer layer) |
| `SL-RECEIPTS.md` | per-instance + per-family receipts from the safety+liveness sweep (209/209; artefacts migrate with the producer layer) |

The producer layer — `species.py` (schema + poly vocabulary),
`produce.py` (all 16 family producers, ladder params verbatim from the
pin), `sweep.py` (produce → check → receipt tables), `mutants.py`
(mutant emission), `groundtruth.py` (extraction cross-check +
recurrence-gap sims) — was python and is migrating to TypeScript on
the modelling toolkit's public API; it returns with that migration.
Its mutant battery was ported into `recheck.ts`, so every refusal
duty stays exercised against the committed fixtures; the reference
checker (imported, see above) never depended on any of it.

## Caveats — read before quoting any comparison

1. **Two papers, two goalposts.** The '24-as-published numbers (93%
   completion, vga 0/10) are superseded by the same group's NeurIPS'25
   follow-up (*Let a Neural Network Be Your Invariant*, MILP-based):
   their '25 method solves vga 10/10; rIC3 1.5.1 — the strong recent
   model checker added as a '25 baseline — completes 104/194 on this
   suite. Full re-anchored their-side column:
   `INVENTION-TIME-COMPARISON.md`, "NeurIPS'25 re-anchor."
2. **The circuits were silently edited upstream** between the two
   papers (142 `.sv` files semantically modified). The fixtures here
   are the **post-edit** versions, so these numbers are comparable to
   their NeurIPS'25 Table 1 re-runs, and only approximately to the
   '24-as-published table.
3. **The standing claims**, against the '25 goalposts and on the '25
   artifact's own arrays (timeout 18000 s): **634/634** certified
   across the three classes vs **554/634** for the best single
   published tool (their '25 MILP method: liveness 192 + safety 190 +
   SL 172) and **617/634** for the per-task union of every published
   baseline (207 + 206 + 204) — seventeen portfolio-orphan tasks
   certified here; sharpest single-family cells: **load_store
   liveness 16/16 vs 0/16** for their '25 method, **blink 25/25 vs
   14/25** (liveness) and **25/25 vs the whole portfolio's 20/25**
   (SL); the **assurance axis** (an exact-arithmetic certificate
   replayed by a standalone checker, vs a solver verdict with no
   checkable artefact) and the **check-side speed axis** (all three
   classes' certificates + chain witnesses replay in well under a
   second) are unchanged by any of the goalpost motion.
4. **The two production cross-check simulators share one author**
   (trust-ledger item 3): that residue fired once and was repaired —
   the independent re-transcription instrument added by the safety
   campaign caught the shared lcd wrap misread (2026-07-21) — and the
   yosys-normalized read has still not run.
5. **Invention walls are session-log-derived** — see the typographic
   convention in `INVENTION-TIME-COMPARISON.md` (plain numbers are
   receipts; bracketed italics are estimates).
