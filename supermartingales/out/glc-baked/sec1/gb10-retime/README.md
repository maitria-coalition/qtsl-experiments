# Section 1 checked-production timing receipt

This directory records the `gb10/host` re-conduct of the three Section 1
published-table families. `PROVENANCE.json` is the human entry point: it pins
the source revisions and binary digests, states the one-warmup/ten-repeat
protocol, and gives the exact `supermart-sec1 retime` invocation. Replace
`<qtsl-experiments>` in that invocation with the absolute path to this
checkout.

The producer source is the `mtk` repository at revision
`6ccb1d863cc79f1cc8da5e87e62039bd20442e38`. Build the recorded executable
with:

```
git clone https://github.com/maitria-coalition/mtk
cd mtk
git checkout 6ccb1d863cc79f1cc8da5e87e62039bd20442e38
cargo build --release -p mtk-strategies --bin supermart-sec1
```

The unsuffixed URL is the coalition's intended public release endpoint; until
the private-preview flip, this is a complete provenance recipe rather than a
claim that an unauthenticated clone is available.

`campaign.json` is a version-2 timing receipt. Its per-repeat `wall_s` is
unchanged in meaning from version 1: the authoritative whole elapsed
checked-production wall for that repeat, including orchestration overhead;
`produce_s`, `check_s`, and `seal_s` remain informative subphases whose sum is
bounded above by `wall_s`, and the timing table consumes `wall_s` only.
Version 2 additionally makes the campaign's own accounting explicit, so the
distinction between the outer campaign wall and the summed per-repeat walls
rests on receipt fields rather than prose:

- `process_wall_s` is a monotonic whole-process wall (scope declared in
  `process_wall_scope`; a receipt cannot include its own final write).
- Each row carries `setup_s` (per-row producer construction), `invoke_s` (the
  outer bracket around the conductor, with `invoke_s >= wall_s` enforced),
  `validation_s` (the independent byte/digest gate against the trusted
  incumbents), and `retain_copy_s` (the retained-container copy).
- The `attribution` block re-derives from the rows: `accounted_s` sums
  preflight, spec reads, the four per-row phase totals, and the monitor join,
  and `residual_s` is exactly `process_wall_s - accounted_s`. The receipt
  validator recomputes these identities and refuses on disagreement.

Every measured output is required to match the committed incumbent bytes and
checked claim count.
