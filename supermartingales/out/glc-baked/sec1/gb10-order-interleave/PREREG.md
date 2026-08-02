# Section 1 order/interleave protocol experiment — preregistration

This file is the preregistered protocol for the `gb10-order-interleave`
methods experiment. It is committed **before** any measured campaign was
run (the git history of this branch is the ordering witness), following
the same anti-selection pattern as `../QUIET-PROTOCOL.md`.

## Question

Do benchmark **order** within a campaign process, or **interleaving**
repeats across benchmarks (round-robin rather than blocked), move the
Section-1 per-benchmark median row walls?

The incumbent protocol (all published sec1 timing families) runs the
three benchmarks blocked, in one fixed order: for each benchmark in
`(faulty_marbles, faulty_regions, faulty_unreliable)`, one flagged
warmup then ten measured repeats. If position in the roster or blocked
repetition materially shaped the medians (cache/branch-predictor
carryover between benchmarks, frequency/thermal drift across the
campaign, once-per-process lazy initialization leaking past the warmup),
the published cells would partly reflect roster structure rather than
benchmark cost. This experiment measures that hypothesis directly.

## Conducts (all parameters declared a priori)

All four arms inherit the quiet protocol's conditions verbatim — load
gate `--max-load-1min 4.0`, CPU affinity `taskset -c 5-9,15,16`,
absolute `renice 19`, warm-by-protocol page cache, byte-identity
comparison against the trusted incumbents — and vary **only** the roster
structure:

| arm | label | roster | warmups/repeats |
|---|---|---|---|
| A | `order-mru` | (marbles, regions, unreliable) — canonical | 1 + 10 per bench |
| B | `order-rum` | (regions, unreliable, marbles) — rotation | 1 + 10 per bench |
| C | `order-umr` | (unreliable, marbles, regions) — rotation | 1 + 10 per bench |
| D | `interleave-rr` | 11 rounds of (marbles, regions, unreliable) as 33 single-repeat roster entries in ONE process | 0 + 1 per entry |

- Arm A is the incumbent conduct re-run in the same session: the
  same-session control against the landed `gb10-quiet-retime` campaign.
- Arms A–C are the three cyclic rotations: a Latin square on position —
  each benchmark occupies each roster position exactly once. Three arms
  suffice to detect a position effect at this granularity; the remaining
  three permutations add campaigns without adding positions.
- Arm D's **first round (first 3 rows) is declared warmup a priori** and
  discarded from analysis, leaving 10 measured rows per benchmark —
  matched to the blocked arms. (With `--warmups 0 --repeats 1` the
  runner flags no row as warmup; the discard is this preregistration's.)
  Rows are analyzed by position in the receipt's `rows` array, which
  preserves execution order.
- Mechanism note for arm D: each roster entry re-reads the spec and
  re-derives the provider/conductor per row exactly as the blocked
  protocol does per row (the leg-A finding: per-row setup is a priced
  cold-row protocol choice, ~47 ms/row) — so arm D differs from arm A
  in row *ordering and spec-read count* only, not in per-row setup
  structure. Retained containers for duplicate entries overwrite the
  same `run-0.glc` path (byte-identical content; noted, harmless).

The four arms run consecutively in one quiet window via
`make sec1-order-interleave`; each invocation independently re-checks
the load gate. If the gate refuses partway through the sequence, the
completed arms stand (their recorded conditions passed) and the leg
reports partial. Whole-sequence gate attempts are capped at 2, spread
over the leg (audit-menu discipline; the leg-F precedent governs the
busy-all-afternoon case: protocol + refusal receipts + armed make
target = honest partial).

## Analysis (declared)

- Per arm, per benchmark: **median of `wall_s`** over the 10 measured
  rows (arm D: after the round-1 discard).
- Position-effect check: for each benchmark, compare its median across
  arms A/B/C (the three roster positions).
- Interleave check: arm D medians against arm A medians.
- Per-row `loadavg_before_1min`/`loadavg_after_1min` inspected; a row
  bracketing a load excursion above the gate value is flagged in the
  findings (flagged, not dropped).

## Preregistered expectations

- **E1** (timing-independent): every row in every arm has
  `byte_identical: true` and a clean geolog acceptance. (Amendment,
  pre-campaign, field-shape only: the receipt's `geolog` field is an
  object — clean means `accept: true` with empty `refusals` and
  `violations` — observed in the mechanics smoke, not the string
  `"pass"` this file first guessed. The expectation is unchanged.)
- **E2** (primary null): each benchmark's median agrees across arms
  A/B/C within the ±3% stability band (the leg-B replication band, the
  same band `../QUIET-PROTOCOL.md` E2 uses), with no position trend
  consistent across all three benchmarks.
- **E3** (null): arm D medians agree with arm A medians within ±3%.
- Known-mechanism note: the once-per-process baked-vocabulary/chain
  derivation (the leg-B finding) is absorbed by the first roster entry's
  warmup row (blocked arms) or the discarded first round (arm D). If it
  were *not* absorbed, its signature would be one inflated first
  measured row, not median motion — a median of 10 is robust to a
  single inflated row by construction.

## Decision rule

An **effect** is declared iff some benchmark's cross-arm median spread
exceeds 3% AND the per-row load context does not attribute it to a load
transient AND the pattern is consistent with a position/interleave
mechanism (e.g. same-position inflation across benchmarks, or a
monotone within-campaign drift). Effect ⇒ chapter prose motion (the
protocol distinction becomes reader-relevant). Null ⇒ one FINDINGS
statement + wish-note; **no book motion** (the chapter does not narrate
every null).

## Anti-selection commitment

The first gate-passing execution of each arm is the one analyzed,
whatever its numbers. A rerun is licensed only by a byte-identity or
validation failure (properties independent of timing), never by the
timing values. Gate refusals are appended to
`build/sec1-order-interleave-refusals.log` and the log is committed
with the findings.

Mechanics smoke runs (validating duplicate-roster/reused-out-dir
behavior on a throwaway path, un-gated, timings unused and uncompared)
precede the measured campaigns and are not part of the sample.

## Publication

Receipts land beside this file:
`gb10-order-interleave/{order-mru,order-rum,order-umr,interleave-rr}/campaign.json`.

This directory is a **methods experiment, never a timing family**: its
name deliberately does not match the tables scanner's
`^<machine>-retime$` discovery pattern (`tables/gen_tables.ts`,
`retimeBestOf`), so no rendered timing cell can move from this leg's
receipts — regardless of outcome. The `machine` field says `gb10-quiet`
(same box, same quiet conditions as `../QUIET-PROTOCOL.md`); each arm's
`expected_benchmarks`, `protocol`, and `complete_roster` fields make its
conduct self-describing. Arm A alone reads `complete_roster: true`; it
lives here, not in `gb10-quiet-retime/`, because it is this
experiment's control, not a new publication of the quiet family.

Runner: `supermart-sec1` built from a pinned mtk worktree at
`b414ba5baab4ba4f894cc924d3a305ddb862c43f` (mtk main at dispatch; the
receipt's `mtk_revision` + `binary_sha256` fields pin what actually
ran).
