# Checker-protocol comparability cells, independent machine (w2145)

The `gb10-checker-compare/` cell family (see its `FINDINGS.md`), replicated
once on `w2145` (x86_64) with `supermartingales/checker-compare.py --machine
w2145 --out-dir <this dir>`, against the same committed incumbents and
per-obligation batches, with the box-built pinned binaries recorded in each
receipt.

Replication verdict, in one line each (numbers in `../w2145-independent/FINDINGS.md`):

- `external-shared.json` — the decomposition REPLICATES: per-invocation cells
  are dominated by a per-process intercept (~215–218 ms here, uniform across
  benchmarks; 63–73 ms on gb10), and the same-file k-slope marginal re-check
  is 25/45/38 ms — 0.82–0.87× the production loop's in-process `check_s`,
  matching gb10's 0.7–0.75× relationship.
- `external-perrow-bridge.json` — same-binary bridge to the pre-migration
  per-obligation batches, for cross-vintage protocol comparisons on this box.
- `cold-anchor.json` — VINTAGE CAUTION: this cell ran the version-2 runner
  (channel-wakeup monitor join); the gb10 cold anchor ran the version-1 runner,
  whose progress monitor slept out up to 1 s at join, inflating its cold
  process wall. The apparent w2145-faster-than-gb10 inversion (0.743 s vs
  1.238 s) is runner vintage, not architecture. Cold anchors are comparable
  only within one runner vintage. (The same-vintage gb10 retake is staged
  as `make sec1-cold-anchor-v2` — machinery and preregistered expectation
  in `../RESIDUES-FINDINGS.md`; its first quiet-gate attempt refused
  honestly under daytime load, so this comparison remains cross-vintage
  until that receipt lands.)
- `retain-vs-noretain-invalidated-overlap.json` — the retain A/B attempt,
  kept deliberately as the overlap specimen: a Lean build arrived mid-cell
  (the zero-co-tenant-window rule's named overlap case) and the delta read a
  spurious +1.5 s where the clean-window gb10 cell reads ~0 — the
  invalidation rule earning its keep, receipted. The cell was NOT re-taken
  on this machine: at the version-2 receipt vintage the A/B protocol is
  superseded for its own question — both campaign receipts measure the
  retained-copy cost inline (`rows_retain_copy_s`: ~5 ms across a whole
  33-row campaign here, ~1.5 ms on gb10), answering directly what the
  outer-wall A/B could only bound from outside, and the gb10 clean-window
  A/B already cross-checks the ≈ 0 verdict.

Documented transformation (one field family): the `path` under
`checker_binary`/`runner_binary` is normalized to the bare binary name —
the build-tree location on the measurement box is not meaningful outside
it; the `sha256` beside it is the identity that matters and is untouched.
