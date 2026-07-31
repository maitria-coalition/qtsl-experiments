# Machines

Every timing in this repository carries a machine tag (the `silicon`
column in the per-instance TSVs; the host fields in receipt JSONs).
The tags are short machine identifiers; this file pins what they mean,
so the timings stay interpretable without any reference to the private
hosts the measurements were taken on.

| id | hardware | notes |
|---|---|---|
| `gb10` | NVIDIA GB10 (Grace–Blackwell superchip): aarch64, 20 cores — 10× Cortex-X925 up to 3.9 GHz + 10× Cortex-A725 at 2.808 GHz; 128 GB unified LPDDR5X | CPython 3.12.3. Wall-bearing runs pinned with `taskset` to a Cortex-X925 core unless a row says A725. |
| `epyc` | GCP `n2d-highmem-16` (spot): AMD EPYC 7B13 (Milan), x86_64, 16 vCPU, 128 GiB | CPython 3.12.3, node 22.23.1. Two instances of this class were used: a batch box for the CPU re-runs (`taskset` core 15, wall-bearing runs serialized under a timing lock) and a dedicated box for the Lean-route ladder (`lean-route/`). |
| `rtx5090` | rented NVIDIA RTX 5090 (sm_120, driver 570.211.01) | GPU producer-bench rows only. |
| `a40` | rented NVIDIA A40 (48 GB, sm_86, driver 580.159.04); host Intel Xeon Gold 6342, cgroup quota ~7.65 vCPU / 50 GB | The supermartingale re-conduct's produce and geolog columns (the sweep wall-clock table) — the same GPU model the published pipeline reports. Receipts: `supermartingales/out/glc-baked/sec2/a40-retime/`. |
| `b200` | rented NVIDIA B200 | the `verify_kernel_median_ms_b200` column in the semiconductor tables. |

Tag syntax in the TSVs: `<id>` optionally followed by `/<run annotation>`
(core class, pinning, scheduler niceness) recorded at measurement time.

## Normalization convention

Committed receipts are verbatim instrument output up to one documented
transformation: private host names, rental-instance identifiers, and
absolute working-directory paths from the measurement runs are
normalized to the machine ids above and to neutral path stems. Numbers,
counts, verdicts, and orderings are never edited; re-running the
reproduction targets on your own hardware regenerates receipts with
your machine's identity in the same fields.
