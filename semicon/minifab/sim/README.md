# minifab-sim — deterministic-core DES (comprehension check)

Scope, exclusions, and why this exists: crate docs in `src/lib.rs`.
This simulator is the "reference DES" of the campaign's model-gap
analysis, whose finding labels (F1–F4, W3, the G1/G2/G4 vocabulary
gaps) are cited below and in `../FLUID-CERT.md`; that analysis is part
of the campaign record, not of this export. One binary, bare
invocation: `cargo run -q -- --weeks 52 [--mix 3,51,30]`.

Reference run (52 weeks, Kempf mix 3/51/30 = 84 lots/week):

| KPI | value | check |
|---|---|---|
| throughput | 83.77 lots/week | tracks release; stable (end WIP 15) |
| mean flow time | 1525 min | (deterministic core; thesis full-model FIFO: 4580 min at 57.5/wk) |
| time-avg WIP | 12.7 lots | Little: $84/10080 \times 1525 = 12.7$ ✓ |
| litho occupancy | 95.2% | handling+processing 87.4% vs capacity arithmetic $84 \times 105/10080 = 87.5\%$ ✓; +7.85% FIFO setup burden |
| diffusion occupancy | 84.8 / 81.7% | mean 83.2% vs $28 \times (285+315)/2/10080 = 83.3\%$ ✓ |
| implantation occupancy | 73.4 / 43.2% | mean 58.3% vs $84 \times 140/2/10080 = 58.3\%$ ✓ (asymmetry = index-order machine pick) |
| max buffers | 7 / 3 / 6 | within spec caps 18/12/12 (unbounded here) |

Overload probe (`--mix 4,54,32` = 90/week): litho saturates (99.8%),
throughput caps at **87.0 lots/week**, WIP grows without bound — the
deterministic-core FIFO ceiling, litho-bound as the capacity arithmetic
predicts ($10080 / (105 + \text{FIFO setup} \approx 9.4) \approx 88$).

Differential verdict, stated honestly: every deterministic capacity
number agrees with the TU/e thesis's own Appendix-H arithmetic to three
digits, and the FIFO setup burden (7.85% vs ~3.3% under their optimized
schedule) reproduces their diagnosis that setup sequencing is a
first-order control lever. The thesis's headline FULL-model numbers
($\delta_{\max} = 57.5$/week, $\varphi = 4580$ min) are NOT reproduced
here and are not claimed — the ~30-lot/week gap between 87 and 57.5 is
carried by exactly the layers this core excludes (personnel + breaks +
maintenance + failures + transport), i.e. by the G1/G2/G4 vocabulary
gaps; a full-fidelity differential is named successor work in that
analysis.

Tests (`cargo test`): lot conservation, determinism (the W3 tie-break
pinned reproducible), horizon-clamped occupancy $\le$ wall clock
(proptest); plus the Kempf-mix regression pin (stability, bottleneck
identity, litho occupancy band). Single-threaded with stated reason:
a DES event loop is a serial dependency chain and this instance is far
under a core-minute.
