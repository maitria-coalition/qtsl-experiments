# Section 1 checker-protocol comparability cells (gb10)

Receipts in this directory; driver: `supermartingales/checker-compare.py`.
All child processes `nice -n 19`; every wall is a monotonic clock around
child spawn-to-exit; 10 measured repeats and median discipline per cell,
matching the retiming campaign's convention. Cells ran under heavy
ambient load (loadavg 12–25, heavy shared-box hours; recorded per receipt)
— within-session comparisons are paired; cross-session levels shift with
load but proportions replicated.

## Question

The Section 1 timing table draws on three distinct measurement
protocols, and public claims must only compare cells whose protocols
measure the same quantity:

1. **In-process check** (`gb10-retime/produce-host.json` `check_s`):
   the conductor's in-loop geolog check over the just-produced
   container, inside a warm process.
2. **External per-invocation check** (`gb10-retime/geolog-shared.json`):
   one `glc-check` process per repeat over the one shared benchmark
   container.
3. **Legacy batch check** (`out/glc/GLC-TIMINGS.*.json` groups): one
   checker process over a whole group of pre-migration per-obligation
   containers, wall around the group, an earlier checker binary.

## Measured decomposition

`external-shared.json` — blessed checker (sha `6e2f21…`, identical to
the geolog-shared receipt's) over the shared incumbent containers, at
1 file/invocation and the same file passed 10×/invocation. Medians,
seconds, with the marginal per-container cost `(k10 − k1)/9` and the
implied per-process intercept:

| benchmark | per-invocation | marginal/container | intercept | intercept share |
|---|---|---|---|---|
| faulty_marbles | 0.0682 | 0.0051 | 0.0630 | 92% |
| faulty_regions | 0.0821 | 0.0095 | 0.0726 | 88% |
| faulty_unreliable | 0.0743 | 0.0089 | 0.0653 | 88% |

A single pre-migration (BL-lane) container clears a whole checker
process in 2.6–4.2 ms (`external-perrow-bridge.json`, single cells), so
the intercept is not generic process startup: it is the baked-vocabulary
derivation the checker performs once per process on first contact with a
baked-lane certificate. It amortizes across any batch checked by one
process; a per-invocation protocol pays it on every repeat.

**Consequences.** (a) Roughly nine-tenths of each published
shared-container geolog cell (0.052/0.056/0.054 s) is per-process
vocabulary derivation, not obligation discharge. (b) The marginal
external re-check (5–9 ms) is the same magnitude as the in-process
`check_s` medians (7.0/13.7/11.3 ms in the landed campaign receipt) —
the two checkers agree on the cost of the actual checking; the
protocols differ in fixed overhead, and neither cell family may be read
as the other's quantity.

## Same-binary bridge to the legacy batch cells

`external-perrow-bridge.json` — the CURRENT blessed checker over the
same pre-migration per-obligation batches the legacy group-wall
receipts timed (23/29/29 containers): medians 0.0208/0.0309/0.0169 s
against the legacy gb10 group walls 0.019/0.020/0.020 s (older binary
`7b06ce…`), under much heavier ambient load. Binary generation is not a
material confounder; the deltas between legacy cells and
shared-container cells are protocol (batch amortization) and container
architecture, both now measured.

## Retain-vs-no-retain

`retain-vs-noretain.json` — two fresh full-roster campaigns with the
runner rebuilt at the campaign's exact source revision, identical but
for `--retain-containers`: outer walls 4.100 s (retain) vs 4.134 s
(no-retain), delta −0.034 s — **zero within noise**. Row-wall and
`check_s` medians are equal across arms (copies happen outside row
walls, confirmed). The retained-container copy hypothesis for the
campaign's untimed outer-minus-inner gap is dead: 33 copies / 160 KB
cost less than the session's noise floor (≲1% of the outer wall). The
untimed gap (≈2.1 s here, 2.15 s in the landed campaign) is per-row
setup (provider/family/strategy construction), independent validation,
receipt fsync, and monitor quantization — the per-row attribution
instrument (receipt v2) owns that split.

Both arms reproduced every container byte-identical to the trusted
incumbents; row-wall medians 0.0428/0.0724/0.0640 s replicate the
landed campaign's 0.0417/0.0708/0.0628 s within 2–3% under far heavier
load.

## Cold-process anchor

`cold-anchor.json` — 10 fresh-process single-repeat productions of
faulty_marbles: median process wall 1.238 s around a median row wall of
0.0439 s, spread 1.11–4.91 s under ambient load. Per-repeat cold
processes would be setup-dominated and load-fragile; the campaign's
warm-loop protocol is the right one, and the anchor quantifies what it
avoids.

VINTAGE CAUTION (found by the independent-machine replication): this
cell ran the version-1 runner, whose progress monitor slept out up to
1 s at join — a fixed inflation that dominates a cold single-row
process wall. Cold anchors are comparable only within one runner
vintage; in particular the apparent w2145-faster-than-gb10 cold
inversion (0.743 s vs 1.238 s) is runner vintage, not architecture. A
same-vintage retake is one verb: `make sec1-cold-anchor-v2` (quiet
conduct; installs `cold-anchor-v2.json` beside this cell; see
`../RESIDUES-FINDINGS.md` for the pinned recipe and the
preregistered expectation).

## Comparability verdict

- **Shared-container geolog cells ↔ legacy batch geolog cells: not
  directly comparable** (per-invocation derivation overhead vs
  batch-amortized startup, on top of the container-architecture
  difference the chapter already notes). The chapter's geolog-column
  note now states the protocol difference with these receipts behind
  it.
- **Geolog cells ↔ in-process `check_s`: not the same quantity.** The
  comparable pair is marginal external re-check (5–9 ms) vs in-process
  `check_s` (7–14 ms); the published geolog cells additionally carry
  the per-process derivation.
- **Produce cells: unaffected.** Whole checked-production row walls,
  one protocol across rows; the landed medians replicated here within
  2–3%.
- **No published number changes.** Every table cell stands as measured
  under its own protocol; the deliverable of these cells is the
  measured protocol delta and the prose note that keeps readers from
  ratioing across protocols.

## Directly measured amortization (checker self-timing)

`batch-amortized.json` — the checker's own timing decomposition
(`glc-check --timings`, monotonic in-process clock) replaces the
k-slope *inference* above with direct measurement, with batch
conformance asserted in the field: every batch verdict line is
byte-identical to the single-invocation reference line (the checker's
committed conformance battery holds the same equality on synthetic
ACCEPT/FAIL/REFUSE rosters; cost may change, verdicts may not). Ran at
loadavg ≈ 8–18, lighter than the cells above — levels shift with load,
proportions replicate.

| benchmark | derivation (direct) | marginal/item (direct) | marginal (k-slope above) |
|---|---|---|---|
| faulty_marbles | 0.0460 | 0.0049 | 0.0051 |
| faulty_regions | 0.0443 | 0.0094 | 0.0095 |
| faulty_unreliable | 0.0532 | 0.0077 | 0.0089 |

Three refinements over the inferred decomposition:

1. **The intercept splits.** The per-invocation intercept (63–73 ms
   above) is per-process *vocabulary derivation* (44–54 ms, directly
   timed around the lazy baked-kit build) plus generic process
   start — spawn, binary load, and the static roster builds (the
   roster builds themselves are ~0.5 ms by the same clock). The
   chapter clause attributing the whole balance to derivation is
   corrected to "most of the balance" with these numbers.
2. **Marginals confirmed.** Direct per-item walls (first-contact
   derivation excluded by construction) reproduce the k-slope
   marginals within noise. (The production campaign's receipt was
   since republished at version 2 with per-row attribution; its
   in-process check medians moved to 7.1/15.1/11.5 ms — the
   magnitude agreement with the 5--9 ms external marginal stands.)
3. **The roster batch is the demonstration.** One process checking
   all three shared containers clears the whole Section 1 roster in
   ≈ 81 ms median (derivation ≈ 54 ms paid once + 5.1/9.5/7.7 ms
   items), against ≈ 224 ms summed over the per-invocation protocol —
   the shared-container architecture's amortization, now an
   affordance any driver gets by passing more files per invocation.

## Receipt register note

Documented transformation (one field family, applied after landing): the
`path` under `checker_binary`/`runner_binary` in the four receipts here is
normalized to the bare binary name — the build-tree location on the
measurement box is not meaningful outside it; the `sha256` beside it is the
identity that matters and is byte-untouched, as is every measurement field.
This matches the `w2145-checker-compare/` convention, and the driver
(`supermartingales/checker-compare.py`) now emits bare names directly.
