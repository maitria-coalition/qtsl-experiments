# Atari section — delta ledger (re-run 2026-07-22)

One row per published number: `number | published | re-derived | cause-class`,
cause ∈ {producer-improvement, measurement-noise, REGRESSION, control-match,
not-re-runnable}. Pins: `PIN-MANIFEST.txt`. Raw receipts:
`receipts/{gb10,epyc,fidelity}/`.

**Regression count: 0.** Every certified return, bound, divergence class,
and mutant refusal reproduces; both claim generations ACCEPT on both
silicons.

**Scope headline: the section is NOT replay-blocked upstream.** This
repository is partial by construction (ROM license law), but the upstream
producer tree holds ROMs, netlists, traces, and the replaycheck lane; the
full replay re-ran there on two boxes and two claim generations. This
repository's status is unchanged (replay still requires
synth-from-your-own-ROM); receipts here reference netlists/traces by sha256
only, never committed.

## The netlist-generation finding (read first; it scopes every row below)

The upstream producer tree re-synthesized both game netlists AFTER the
export's claims were produced (the VIDEO_OUT emission-lane change), and
re-produced + re-checked the claim bundles against the
new AIGs. Two coexisting generations, both fully checked in this re-run:

| generation | pong AIG (I/L/O/A) | freeway AIG (I/L/O/A) | pinned by |
|---|---|---|---|
| export (= published scale row) | 52 / 1454 / 19 / 20400 | 52 / 1454 / 19 / 20377 | this repo's committed claims |
| latest tips (VIDEO_OUT) | 52 / **1491** / 26 / **20821** | 52 / **1491** / 26 / **20798** | upstream claims (AIG sha256s in PIN-MANIFEST) |

Cause-class: producer-improvement (deliberate emission-lane change; +37
latches = video pipeline state, +7 outputs, ~+420 ANDs). Consequence for any
future republication at tips: the scale row becomes 1491 FFs and the
"$2^{1454}$ states" sentence becomes $2^{1491}$ (the 1024-bit RIOT RAM
sentence is unchanged). Behaviorally the generations are equivalent on the
claim surface: identical RAM streams, rewards, returns, and total_ticks;
the latest-tips checks compare bit-exactly against the SAME Verilator trace
witnesses both claim generations pin (the Verilator sim was not changed by
the re-synth — "plain path stays byte-identical" held, now with replay
receipts).

## Certified rows (the regression-law rows)

| number | published | re-derived | cause |
|---|---|---|---|
| Pong U1: $R_k = -6$ for all 31 offsets, $\min = -6$ | -6 uniformly | **-6 uniformly, both claim generations, both silicons** (4 ACCEPTs) | control-match |
| Freeway U1: per-offset span $\{2,3,4\}$, offsets 6–7 = 4, nine offsets = 2, $\min = 2$ | as published | **identical 31-vector both generations both silicons**; span $\{2,3,4\}$, min 2 < modal 3 | control-match |
| 31/31 traces bit-exact vs AIG replay (artefact-pair) | zero divergence, 31,465 frames / 4,027,520 RAM bytes per row | **zero divergence, same counts, all four checks** — and the pair now ALSO spans netlist generations (old AIG and new AIG each bit-exact vs the same Verilator witnesses) | control-match (strengthened) |
| total netlist clocks | 87,303,218 (pong) / 87,386,010 (freeway) | identical, all lanes | control-match |

## Check wall-clocks (end-to-end, named machine, single core)

Published protocol: GB10, single core, release, nice'd, 2–10 sibling
checkers sharing the box. This re-run: gb10 (GB10 Grace, nice'd, box in
normal shared service) and epyc (GCP n2d-highmem-16, EPYC 7B13,
otherwise idle, flock-serialized). Walls below from `checker_receipt`
`wall_s` (checker's own clock; /usr/bin/time receipts beside them).

| row | published (GB10) | gb10, export claim + old AIG | gb10, tips claim + new AIG | epyc, export | epyc, tips | cause |
|---|---|---|---|---|---|---|
| Pong noop-T1000 check wall | 1413.0 s (0.062 Mclk/s) | 1442.6 s (0.061 Mclk/s) | 1405.4 s (0.062 Mclk/s) | 2795.6 s (0.031 Mclk/s) | 2883.7 s (0.030 Mclk/s) | see notes |
| Freeway up-T1000 check wall | 1409.8 s (0.062 Mclk/s) | 1392.3 s (0.063 Mclk/s) | 1409.5 s (0.062 Mclk/s) | 2818.8 s (0.031 Mclk/s) | 2864.8 s (0.031 Mclk/s) | see notes |

Notes: (i) ADJUDICATED — a discriminator pair ran the identical laneA-pong
job concurrently on gb10 under two checker revisions: published-era source
`26f6fd631` → **1401.3 s (0.062 Mclk/s)**, current HEAD → **1442.6 s (0.061
Mclk/s)**; ratio 1.03. So the checker's post-publication evolution (pixel
lane threaded through `run_core`, montezuma/slicegraph machinery) costs
~3% on this claim family — and an earlier 2026-07-20 re-check's
receipts (3080.0 / 3082.7 s, 0.028 Mclk/s, same silicon class) were
shared-box load noise, not code. Published
walls REPRODUCE on the published silicon at tips within ~2–3%
(receipts: `receipts/gb10/pubrev-pong.{json,log}`). (ii) EPYC 7B13
single-core runs this serial bit-parallel AIG replay ~2× slower than GB10
Grace (epyc laneA-pong 2795 s vs gb10 1443 s, same job; bench
calibration beside it) — silicon, not code; matched pairs in the fig6
bank. (iii) the tips AIG carries ~2% more AND gates; expect a
proportional wall increase between claim generations on the same box.

## Fidelity tier (posture 2) — fully re-derived

All rows in `receipts/fidelity/FIDELITY-RERUN.md`. Headline: every published
signature reproduces exactly — returns -6/-6 and +3/+3 and 0/0; 140-frame
rally both sides; Pong events uniformly +2 frames; Freeway first crossing at
frame 171 on both sides, later crossings +1; persistent divergent sets
{8,21,49,50,54} and {1,2,14,18,19,121,122} exactly. One summary-convention
note: "116–123 of 128 bytes exact per frame" is the central band; strict
per-frame min observed 114–115 on 1–3 frames per run. cause: control-match
(band note: measurement-summary convention, not a delta).

Reproduction finding: the operative input calibration is the claim's
`paddle=[132,132,0,0]` tuple (README prose says only "pot=132"; the sim
default is 128; wrong tuples resurrect the paddle divergence family). The
claim data is the protocol record — banked as editorial residue for the
section README (this branch repairs a separate README-adjacent line: the
RESULTS.md receipts index claimed committed .atrc.gz witnesses that the
public export deliberately omits; now states they are sha256-pinned, not
committed).

## Mutant ceremony (latest-tips generation)

CEREMONY PASS: 2 controls ACCEPT (166.9 / 168.3 s, gb10, 4 concurrent
jobs), 11/11 mutants REFUSE. Witness classes and bytes match the published
table for m01–m08, m10, m11 (e.g. m04 `ram[8] 0x45 vs 0x44`, m06
`ram[4] 0x0f vs 0x0e`, m07 `lane 2 computed 0, claim says 1` — exact).

| number | published | re-derived | cause |
|---|---|---|---|
| m09 (netlist gate flip, AND 17422) | `trace-ram`: lane 0 frame 0 ram[7] 0x07 vs 0x02 | **`vsync-timeout`**: lane 0 await-first-vsync, 20M clks | producer-improvement side-effect: the re-synth renumbered gates, so the pinned index feeds different (frame-machinery) logic in the new AIG; refusal preserved, witness class moved within the tool's own expected set. If a future battery wants the original ram[14][0] locus on the tips netlist, the index needs recomputing from the new latch table — noted as residue |
| controls ~176 s each | ~176 s | 166.9 / 168.3 s | measurement-noise |

## Not re-run, with reasons (honest scope)

| item | reason | class |
|---|---|---|
| yosys synth flow (source → AIG) | published toolchain is yosys 0.67 (oss-cad-suite); not present on either measurement box during the timed window. The committed tips AIGs ARE the current producer output; headers re-read and banked both generations. Deterministic re-synth at pinned yosys = named residue | not-re-runnable (this leg) |
| claim-bundle producer (31-lane trace production + claim assembly) | the python producer layer is mid-migration to the mtk TS API by declared design (README); the committed trace witnesses are the pinned inputs both claim generations verify against. Single-trajectory Verilator trace production WAS re-run in the fidelity lane (sim rebuilt from source, Verilator 5.020) | not-re-runnable (migrating layer) |
| GPU lane | NO-LANE: replaycheck has no provider-gpu path at any tip (standalone crate, serial bit-parallel u64 replay by design; deps: ciborium/serde/sha2/flate2). The 31 init lanes already ride one machine word — the parallelism the GPU would offer is consumed by the algorithm's own bit-parallelism | no-lane |
| Lean column | gated behind a separate measurement effort. Sizes banked for the gate table: claim.cbor 5106 B (pong) / 5113 B (freeway) export, 5300 / 5307 B tips; obligations per row: 31 lanes, 31,465 frame-comparisons, 4,027,520 RAM-byte equalities + reward stream; `rational_count` 0 / `denominator_bits_max` n/a (integer/bit arithmetic only — no exact rationals anywhere in this cert family) | gated |

## Bridge / protocol deviations, stated

The standard "smallest certificate, median-of-200 on gb10" bridge is
infeasible verbatim for this family (smallest committed bundle = T=60
4-lane, ~90–170 s per check ⇒ median-of-200 ≈ 5–9 h/box). Substitute
banked instead, stronger where it matters: the SAME four T1000 checks ran
on BOTH silicons (matched jobs, full-size), plus bench-verb calibration
pairs on both boxes. Per-offset walls do not exist in this lane (all 31
offsets ride one bit-parallel pass) — fig6 rows are per (game × lane ×
machine × claim-generation), obligation unit = frames-compared.

## Republication at tips (2026-07-22, follow-up to the finding above)

The consequence paragraph above was enacted the same day: the committed
claims, checker receipts, mutant battery, synth summary, and chapter prose
now all state the latest-tips (VIDEO_OUT) generation. What moved, exactly:

- scale row + prose: 1454 -> 1491 FFs, $2^{1454}$ -> $2^{1491}$, AND
  counts 20400/20377 -> 20821/20798 (chapter section, section README,
  `receipts-summary.txt`, RESULTS.md);
- `certs/{pong-noop-T1000,freeway-up-T1000}/`: claim + producer receipt
  re-vendored at the tips pins; committed checker receipts are this
  re-run's gb10 tips checks (1405.4 s / 1409.5 s — the `laneB` receipts
  under `receipts/gb10/`);
- mutant battery re-run wholesale at tips with m09's locus DERIVED from
  the pinned AIG's latch table + symbol map instead of hardcoded (the
  residue noted in the m09 row above, repaired): tips locus is AND 17843,
  and its refusal witness is byte-identical to the published one
  (`trace-ram`: lane 0 frame 0 ram[7] 0x07 vs 0x02) — the re-synth had
  renumbered the same logical cone;
- m10's tampered AIG is no longer committed (it was a full game netlist
  modulo one bit, i.e. ROM-embedding content); its sha256 + one-line
  derivation are pinned in the mutant's receipt/README instead.

Certified returns, bounds, divergence classes: unchanged (control-match,
per the rows above). No published number moved other than the netlist
scale row and the check walls.
