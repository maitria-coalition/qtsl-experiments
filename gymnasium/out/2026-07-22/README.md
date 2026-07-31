# Gymnasium section re-run receipts (2026-07-22)

The re-run re-derives every published number of the Experiments chapter on
the latest repo tips, banking wall-clock and bound numbers per experiment.
This directory is the gymnasium section's bank.

- `delta-ledger.md` — the verdict, one row per published number.
  Headline: **0 regressions; 53/53 byte-identical certificates; zero bound
  drift; the chapter's GPU-lane sentence's null result CONFIRMED; big
  reward-table checks 2–3.8× faster on identical silicon** (the wave-3
  sweep lane is now the checker's default `auto` lane through the repo path).
- `PIN-MANIFEST.txt` — exact revs, machines, and measurement protocols.
- `fig6.tsv` — per-instance rows (instance × lane × silicon) per the
  Fig6 addendum schema; regenerate with `python3 build_artifacts.py`.
  Column notes: `t_produce_ms` = min-of-N DP span (wave-3 protocol);
  `t_check_ms` = median verify (bench 200; box2d bench 50); `t_total_ms`
  only where both ran on one box; box2d + blackjack-k16/k20 rows carry no
  produce span (replay-only / python-emitted — see ledger); gpu rows carry
  no check (checks are CPU-side); `obligation_count` = checker `rows=`;
  `state_space_size` = env state count (`ns`).
- `ledger-data.json` — the ledger's numeric core (generated).
- `receipts/{gb10,epyc,rtx5090}/` — raw receipts: producer JSONs
  (1-core r15 / 4-core r7 / cold), checker receipt JSONs for BOTH checker
  lanes (committed-pin replay + latest-tips), box2d bench lines, ref-vs-auto
  sweep-lane lines, silicon + exit codes.
- `Cargo.lock.tips` — the latest-tips checker lock (kernels `b0143fa3a`).

Canonical receipt files one directory up (`checker_receipts*.json`,
`producer_receipts_rust.json`, `gpu_bench_receipts.json`) were regenerated
by this run on the published protocol's silicon (GB10 single-core pinned /
rented 5090) at the latest-tips lane; `tables/gen_tables.ts` verifies
MATCH against its re-pinned reference (bounds cells unchanged, timing
cells re-pinned with provenance comment).
