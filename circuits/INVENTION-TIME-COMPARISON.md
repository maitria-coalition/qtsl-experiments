# Invention-time comparison: their 5 h/task budget vs our end-to-end [invention + production + check]

*2026-07-20. The producers here are task-specific, so the honest
end-to-end comparison must charge us the wall-clock of **inventing**
the method and writing every producer — not just running them. The
entire pipeline (route design, spec census, Lean lemma, certificate
format, checker, producers, full sweep) was authored by an AI agent in
three back-to-back sessions on 2026-07-20; the sessions' timestamped
event logs are the invention-time instrument.*

*Sources: the two authoring-session event logs (JSONL, one timestamp
per event — extracted by streaming, methodology below);
`SWEEP-RECEIPTS.md`; the route-design notes; the paper's own
per-task appendix table (`figs/appendixTable.tex` in their arXiv
source, config (8,5) — the canonical 194-row table; parse cross-checked
against the paper's summary table `figs/tab_cmp.tex`, recovering their 180/194
exactly, including three starred-footnote rows a naive parse drops).*

**Typographic convention, load-bearing:** plain numbers are receipts
(transcript timestamps, receipt files, the paper's published tables);
*italic bracketed numbers are this document's estimates* — every estimate
states what it interpolates between.

## The comparison table

Their side: paper's config (8,5), Intel Xeon 2.5 GHz / 8 threads / 32 GB
(their §Experimental Setup, quoted verbatim in Methodology), 5 h/task
limit; "total" is the appendix table's per-task "our (8,5)" column,
summed over completed tasks (whether it includes the separately-tabled
training time is not explicit in the TeX; their Limitations names the
SMT check as dominating wall-clock, and the train column sums are small
against the totals either way). Our side: one GB10 (aarch64), single
process; produce/check walls from `SWEEP-RECEIPTS.md`; invention walls
derived from the session logs.

| family | tasks | theirs: done | theirs: total wall | theirs: mean/task | ours: done | ours: marginal invention (session logs) | ours: produce wall | ours: check wall | artefact class |
|---|---|---|---|---|---|---|---|---|---|
| delay (Da) | 16 | 16 | 16 649 s | 1 041 s | 16 | *[1–2.5 min]* (template family; also the route session's worked instance) | 0.52 ms | 2.35 ms | QTSL cert (415–435 B) vs bare unsat |
| gray (Ga) | 11 | 11 | 8 743 s | 795 s | 11 | *[0.5–1.5 min]* | 0.44 ms | 1.98 ms | QTSL cert vs bare unsat |
| PWM (P) | 12 | 11 (P₁₂ dnt) | 20 377 s | 1 853 s | **12** | *[1.5–3 min]* (8-way input-value box split) | 2.13 ms | 17.43 ms | QTSL cert vs bare unsat |
| seven_seg (7a) | 15 | 15 | 4 780 s | 319 s | 15 | *[0.5–1.5 min]* | 0.46 ms | 2.10 ms | QTSL cert vs bare unsat |
| delay2 (Db, X-spec) | 16 | 16 | 30 752 s | 1 922 s | 16 | *[0.5–1.2 min]* (one-step-unfold note; cert identical to spec 1) | 0.17 ms | 0.85 ms | QTSL cert vs bare unsat |
| gray2 (Gb, X-spec) | 11 | 11 | 22 276 s | 2 025 s | 11 | *[0.2–0.8 min]* | 0.20 ms | 0.87 ms | QTSL cert vs bare unsat |
| gray3 (Gc, 2-pair) | 11 | 11 | 20 119 s | 1 829 s | 11 (22 certs) | *[0.3–0.9 min]* | 0.26 ms | 1.20 ms | QTSL cert vs bare unsat |
| **covered subtotal** | **92** | **91** | **123 696 s ≈ 34.4 h** | 1 359 s | **92** | see amortized headline | **4.18 ms** | **26.78 ms** | — |
| seven_seg spec-2 (7b) | 15 | 15 | 7 183 s | 479 s | **15** (30 certs) | 2.7 min (0.2 + 2.5, both pairs) | 0.80 ms | 4.20 ms | QTSL cert vs bare unsat |
| lcd (L) | 14 | 14 | 13 739 s | 981 s | **14** | 3.7 min (1.2 + 2.5) | 0.93 ms | 6.96 ms | QTSL cert vs bare unsat |
| i2c (I) | 20 | 18 | 6 196 s | 344 s | **20** | 3.8 min (1.3 + 2.5) | 1.92 ms | 15.09 ms | QTSL cert vs bare unsat |
| load_store (Ls) | 16 | 15 | 23 915 s | 1 594 s | **16** | 3.5 min (1.0 + 2.5) | 0.62 ms | 4.40 ms | QTSL cert vs bare unsat |
| thermocouple (T) | 17 | 17 | 32 051 s | 1 885 s | **17** | 3.0 min (0.5 + 2.5) | 0.69 ms | 5.16 ms | QTSL cert vs bare unsat |
| uart (U) | 10 | 10 | 3 550 s | 355 s | **10** | 3.2 min (0.7 + 2.5) | 0.24 ms | 1.19 ms | QTSL cert vs bare unsat |
| vga (V) | 10 | **0** (training local minima; '24 as published — their '25 MILP successor solves vga 10/10, see the NeurIPS'25 re-anchor below) | — | — | **10** | 4.0 min (1.5 + 2.5) — the route design's lex two-counter prediction held | 1.39 ms | 11.25 ms | QTSL cert (lex two-counter) vs no '24 result |
| **suite total** | **194** | **180** | **210 330 s ≈ 58.4 h** | 1 169 s | **194** (220 certs) | see fill note | **10.0 ms** | **68.6 ms** | — |

**Fill note (2026-07-20, from `SWEEP-RECEIPTS.md`'s measurement-rider
table):**
the seven formerly-pending rows are transcript-stamped per-family
invention walls plus produce/check walls from the sweep-2 re-run. Three
reading conventions: (a) each row's decomposition $(a + b)$ is quoted
verbatim from the receipts table; the second term is a uniform 2.5 min
across all seven families, so summing the column over-counts the window —
the bounding receipt is the session's commit-to-commit wall,
**29 m 08 s** (authoring-repo commit stamps, 09:09:50 → 09:38:58 BST). (b) The sweep-2 run re-measured the *whole*
suite, so the suite-total produce/check walls (10.0 / 68.6 ms, 220
certificates) are one coherent run; the covered-subtotal row above keeps
the earlier run's walls (run-to-run variance is visible, e.g. PWM check
17.43 → 13.63 ms — same certificates, same checker). (c) The per-mode
LP synthesis this table's earlier caveats priced for the FSM families
was never needed: every family fell to closed-form per-region
steps-to-target rankings, margin exactly 0 (sweep-2 README, trust
ledger).

Their per-task *budget* was 5 h (uniform limit); the totals above are
what they actually spent on completed tasks. Their 14 timeouts/failures
(10 VGA + 2 i2c + 1 PWM + 1 load_store) additionally consumed up to
5 h/task of budget each — not included in their totals above.

## The amortized headline, computed honestly

**End-to-end, all invention charged to the 92 covered tasks:**

- Ours: route + spec census 17 m 13 s (07:25:10→07:42:23 Z) +
  species + Lean lemma + checker + producers + 103 certs
  26 m 28 s (07:43:45→08:10:13 Z) = **43 m 41 s** wall (sum of the two
  session spans; calendar span including the 38 s gap between sessions
  and the ~1 m of session startup before the self-clocked start: 07:24:05→08:10:45
  = 46 m 40 s).
- Theirs, same 92 tasks: 123 696 s ≈ **34.4 h** of pipeline compute
  (91 completed; the 92nd, PWM₁₂, did not train — our side completes it).
- **Ratio ≈ 47x (43.7 min vs 34.4 h), with the entire method invented
  inside our number.** Using the calendar span instead: 44x.

**Marginal per task, after invention (the number that governs every
future ladder instance):** produce + check = 30.96 ms for 103
certificates over 92 tasks ≈ **0.34 ms/task**, vs their completed-task
mean 1 359 s — a ratio of *≈ $4 \times 10^6$* (estimate only in the
sense that the two machines differ; both terms are receipts).

**Amortization is real and forward-looking, and the transcript exhibits
it directly:** the three spec-2/3 families (delay2, gray2, gray3 — 38
tasks, 49 certificates) entered the codebase at 08:08:02 (first
appearance of the token `delay2` anywhere in the build session's log, as a
`produce.py` edit) and were fully swept with receipts by 08:09:35 —
**≈ 93 s of agent wall for 38 tasks** riding the already-built species,
checker, and ground-truth harness, i.e. *≈ 2.4 s/task marginal invention*
once the species exists. Their side spent 73 147 s ≈ 20.3 h of compute
on the same 38 tasks. The shared strata (census, demonic-recurrence
lemma, species schema, checker) amortize further across the 102 pending
tasks: the sweep-2 session inherits them at zero additional invention cost,
so its per-family numbers (merge slot above) should be read against
producer-writing time only.

**Against their 5 h/task budget:** our *entire
end-to-end effort for 92 tasks* — inventing the route, the census
over all 194 tasks, the Lean lemma, the certificate species, the
checker, the producers, and running everything — fits inside **15 % of
their budget for one single task**.

## Methodology: how spans were attributed

**Session-log extraction.** Both authoring-session logs are JSONL with
a `timestamp` per event. They were streamed (never loaded whole): each
assistant event's tool calls were reduced to (timestamp, tool,
first 160 bytes of file_path/command). The reduced timelines are 47 and
75 lines; every phase boundary below is one of those receipt lines.
(The logs are records of the authoring runs; they are not included in
this repository. Producer file names quoted from them — `produce.py`,
`species.py`, and kin — are transcript records of the original python
producer layer, which is migrating to TypeScript on the modelling
toolkit's public API; see `README.md`. They are not paths in this
tree.)

**Composition-interval convention (estimate-defining).** A file-write's
composition time is the interval from the previous transcript event to
the write event (the agent composes, then emits). This is the sole
convention behind the per-family *italic* brackets; it is exact for the
interval totals and fuzzy only in how an interval splits across the
families whose code it contains — hence brackets, not points.

**Phase table (receipts).** All times UTC (Z), 2026-07-20:

| span | wall | content | attribution |
|---|---|---|---|
| 07:25:10–07:42:23 | 17 m 13 s | route session: paper + vendor pin, design recon, spec census (all 194 tasks reduced to $\mathsf{GF}\,\gamma$ forms), route design, delay₁ certificate + 3 mutants by hand, route notes written | shared — all 14 families, incl. the 102 pending tasks |
| 07:43:45–07:59:12 | 15 m 27 s | build session boot, route digest, Lean-library recon, mathlib name pinning, `StreettDemonic.lean` (169 lines) written + committed + machine-check-verified (build exit 0, 3.9 s) | shared — the lemma discharges every family incl. FSM ones |
| 07:59:19–08:01:44 | 2 m 25 s | SV ladder extraction: delay, gray, PWM, seven_seg sources + runner scripts | base-4 families |
| 08:01:44–08:04:22 | 2 m 38 s | `species.py` (schema), `produce.py` (4 family producers), `ck_manifest.py` (standalone checker), 54 fixtures produced + smoke check | species + checker shared; producers base-4 (*interleaved — bracketed*) |
| 08:04:22–08:06:14 | 1 m 52 s | `groundtruth.py` (dual-simulator cross-check), `mutants.py` (2 controls + 8 attacks) | base-4 models + shared gate harness (*interleaved*) |
| 08:06:43–08:08:02 | 1 m 19 s | `sweep.py` harness, first full sweep | shared |
| 08:08:02–08:09:35 | 1 m 33 s | **spec-trio addition**: delay2/gray2/gray3 producers (one-step-unfold note inline), groundtruth extension, re-sweep → 103 certs | delay2 + gray2 + gray3 (38 tasks) |
| 08:09:35–08:10:13 | 38 s | README, final receipts, commit, final verification | shared |

Base-4 marginal invention (the family-attributable windows: SV
extraction + produce.py composition + groundtruth + mutants + two
receipt-fix edits at 08:08:30–08:08:47) totals ≈ 5 m 14 s across four
families; the per-family brackets in the main table split this unevenly
by observed content (PWM's input-split design note is the longest,
delay was partly pre-paid in the route session's worked instance). The
spec-trio window is crisp: no token of any spec-2/3 family appears in
the build session's log before 08:08:02.

**What is receipt vs estimate, summarized:** session walls, phase
boundaries, the 93 s spec-trio window, produce/check milliseconds, their
per-task seconds and completion counts — receipts. The per-family
splits *within* the base-4 windows and the ± on "≈ 5 m 14 s" — estimates
by the composition-interval convention, bracketed.

## Honest caveats

1. **Platform-relativity, symmetric.** Our 43.7 min is marginal
   invention *given our platform* (the Bernstein/QTSL calculus and its
   Lean lemma library, prior checker patterns, a Lean build farm) — not
   from-nothing. Their 5 h/task likewise rides their platform (Spot,
   EBMC, Bitwuzla, PyTorch) and their *own* invention time — building
   the neuralmc pipeline itself, a NeurIPS-paper engineering effort — is
   unpriced and not in their totals. The honest comparison is
   marginal-vs-marginal: their per-task pipeline cost vs our
   [per-suite invention + per-task production + check].
2. **Hardware differs**: Xeon 2.5 GHz/8 threads/32 GB (theirs, 2024) vs
   one GB10 (ours). *A single-digit factor at most* — noise against
   ratios of 47x and $4 \times 10^6$.
3. **Artefact class is not the same good.** Their per-task output is a
   solver verdict (Bitwuzla "unsat" — no checkable artefact in their
   Figure-6 toolchain); every task on our side ships a 415–2 632 B certificate
   replayed by a standalone checker in exact $\mathbb{Q}$, with the
   soundness lemma machine-checked in Lean. The time comparison
   *understates* the assurance asymmetry.
4. **Coverage is 194/194 as of sweep-2** (this caveat originally read
   "92/194 until sweep-2 lands"). The named risks resolved cleanly:
   the FSM+counter families fell to closed-form rankings (no per-mode
   synthesis needed), and VGA — the '24 paper's systematic failure —
   certified on the predicted lex two-counter shape. Per-family completion is
   reported unblended in the table, as required.
5. **Their totals' train-time inclusion is ambiguous** in the TeX (see
   table preamble); the ambiguity moves their totals by at most the
   train-column sums (e.g. 462 s across all 16 delay tasks), well under
   4 % of any family total used here.
6. **Paper-internal discrepancy, resolved in their favour:** a naive
   parse of their appendix yields 177 completions vs the summary
   table's 180; the three missing are starred-footnote entries
   (7b₁₁, 7b₁₄, Ls₁₅) restored here — plus one duplicated label
   (Ls₁₅ appears twice; the second, oot, row is read as their
   16th load_store task). Our counts use the recovered 180.


## NeurIPS'25 re-anchor (read before quoting the table above)

The table above compares against Neural Model Checking **as published
at NeurIPS 2024**. The same group's NeurIPS 2025 follow-up (*Let a
Neural Network Be Your Invariant*, NeurIPS 2025, pp. 74713–74740 — a
MILP-based successor method, Gurobi-synthesized integer nets, same
Bitwuzla check) moves both the suite and the baselines; the pinned
upstream tree (`neuralmc-pin/`, fetched by `make fetch-pins`;
provenance: `neuralmc-pin/VENDOR-PIN.md`) carries its experiment
subset, whose hard-coded runtime arrays were cross-validated
cell-for-cell against the published Table 1.

1. **The circuits were edited upstream.** The '25-era restructure
   semantically modified 142 of the '24-era `.sv` files (delay's
   counter/reset logic, PWM's pulse-width constant, load_store's
   saturation, gray's `sig` semantics, i2c_1's parameter N 500 → 12).
   **The fixtures in this repository are the post-edit versions**, so
   our numbers are apples-to-apples with the NeurIPS'25 Table 1
   re-runs, and only approximately comparable to the '24-as-published
   totals quoted above.

2. **vga is no longer anybody's systematic failure.** Their '25 MILP
   method solves vga **10/10** (0.107–0.247 s/task); their re-run '24
   method solves 9/10. The vga row above is true only of the '24
   method as published in 2024.

3. **Corrected their-side column** (NeurIPS'25 Table 1, pure liveness,
   restricted to the 194 tasks certified here): their '25 method
   **178/194** (with **load_store 0/16**, their own regression);
   NMC'24-rerun **172/194**; nuXmv-BDD **172/194**; rIC3 1.5.1
   **104/194**; ABC **73/194**. The virtual best-of-all-five-tools
   portfolio completes **193/194**.

4. **Surviving claims, stated against the '25 numbers**: 194/194 here
   vs 193/194 for the virtual best portfolio (and ≥ 16 tasks over
   every single tool); load_store **16/16 vs 0/16** for their current
   method; the assurance axis is unchanged (their '25 verdict is still
   a bare "unsat" with no checkable artefact — every task here ships
   an exact-$\mathbb{Q}$ certificate replayed by a standalone checker); and the
   speed story here is the **check side** (whole-suite replay ~80 ms),
   not synthesis wall-clock supremacy — their '25 method is fast
   (sub-second on most families it completes).
