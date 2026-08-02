# Order/interleave effects (sec1): findings

**VERDICT: NULL — the preregistered expectations E1–E3 all hold.**
Benchmark order within the campaign process and repeat interleaving
(round-robin vs blocked) do not move the Section-1 medians above the
±3% stability band. The incumbent protocol's fixed blocked roster is
not shaping the published cells.

## Conditions

All four arms were admitted in one quiet window on the **first**
whole-sequence gate attempt (2026-08-02, ~10:34Z): starting 1-minute
load averages 3.16 / 3.31 / 3.44 / 3.44 against the 4.0 gate, maximum
observed at any sample 3.44, **zero** per-row load transients, nice 19,
CPUs 5-9,15-16, runner pinned at mtk `b414ba5ba` (binary sha in each
receipt). The anti-selection commitment was honored trivially: the
first attempt passed and published.

## Numbers (median `wall_s` over 10 measured rows, seconds)

| arm | roster | marbles | regions | unreliable |
|---|---|---|---|---|
| order-mru | (m, r, u) blocked — canonical | 0.041799 | 0.071622 | 0.063109 |
| order-rum | (r, u, m) blocked | 0.041818 | 0.070411 | 0.062292 |
| order-umr | (u, m, r) blocked | 0.041821 | 0.070574 | 0.062157 |
| interleave-rr | 11×(m, r, u) round-robin, round 1 discarded | 0.042372 | 0.071606 | 0.063058 |

- **E1** (validity): 132/132 rows byte-identical to the trusted
  incumbents with clean geolog acceptance, across all arms.
- **E2** (order): cross-arm median spread 0.05% (marbles), 1.72%
  (regions), 1.53% (unreliable) — all within the band. The residual
  spread patterns by **arm run-order**, not roster position: the first
  arm ran uniformly slightly hot and the ambient load was decaying
  through the window (starts 3.16 → 3.44 are 1-minute averages lagging
  a falling instantaneous load); no position trend is consistent
  across benchmarks (regions pos1 < pos3 < pos2, marbles flat,
  unreliable's apparent monotonicity coincides exactly with arm order).
- **E3** (interleave): deltas vs blocked +1.37% / −0.02% / −0.08% —
  within the band.
- Control replication: arm order-mru reproduces the landed
  `gb10-quiet-retime` medians (0.042205 / 0.071516 / 0.063250) within
  1.0% — same conduct, different session, both under the gate.

## Consequence

Per the preregistered decision rule: **no chapter motion** (the chapter
does not narrate every null), no table motion (structurally impossible
from this directory, and `make tables` verified 7/7 MATCH after the
campaign). The order/interleave lane of the 2026-08-02 08:26Z timing
interpretation audit is executed, closing the audit menu.

Analysis artifact: `ANALYSIS.json` (written by
`supermartingales/order-interleave-analyze.py`, the preregistered
analysis); raw receipts per arm in `*/campaign.json`.
