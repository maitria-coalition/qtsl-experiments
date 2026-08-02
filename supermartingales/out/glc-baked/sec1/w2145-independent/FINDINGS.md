# Section 1 on an independent machine (w2145, x86_64)

The Section 1 checked-production campaign and the checker-protocol
comparability cells, replicated on a second machine and a second
architecture: `w2145` (see `MACHINES.md`) is x86_64 where the published
cells' machine (`gb10`) is aarch64. Producer: `supermart-sec1` built
`--locked` at mtk revision `6ccb1d86…` (the version-2 receipt vintage);
checker: `glc-check` at qtsl revision `a9b33d14…` (the same revision the
gb10 receipts record). Receipt families: `w2145-independent/` (campaign,
version-2 receipts) and `w2145-checker-compare/` (protocol cells).

These receipts are evidence about machine-dependence, not candidates for
the rendered table cells, and are deliberately outside the
`<machine>-retime/` best-of scan. Verified by transient staging: with a
copy staged scan-visible, the receipt validator accepts it, a
corrupted-sha mutant refuses, and all four generated timing tables stay
byte-identical (the rendered cells remain gb10's under best-of either way).

## Environment, measured not assumed

Co-tenant load ~19–20 of 16 threads throughout (seventeen nice-0 prover
processes at ~73% duty plus one nice-10 grinder). A nine-cell scheduler
calibration probe (single-threaded spin at nice 19/10/0, three repeats
each, under that load) read wall/cpu inflation 1.000–1.013 — the
co-tenants' sleep fraction leaves idle cores and the scheduler finds
them, so serial nice-19 measurement is scheduler-clean here; what
co-tenancy still costs is all-core-turbo frequency and cache pressure,
which no probe of this kind removes. Same-box stability: a second full
campaign agrees with the first within ~1–2% on every phase median;
within-campaign repeat spread is 3–4% (the gb10 campaign's: 0–2%). The
box's Lean job queue was empty before and during every kept cell; the
one cell a Lean build overlapped (the retain A/B) was invalidated per
the zero-co-tenant-window rule — it read a spurious +1.5 s delta where
the clean gb10 cell reads ~0, the overlap rule earning its keep in one
specimen — and was retired rather than re-taken, because the version-2
receipts measure the retained-copy cost inline and more directly (see
result 3).

## Cross-machine / cross-architecture results

1. **Byte-identity is architecture-independent (the machine-stable
   core).** All 66 rows across both campaign runs (2 × (3 warmup + 30
   measured)) emitted containers byte-identical to the committed
   incumbents, on x86_64, from the same source revisions — with
   identical provider-closure revision (`ac5fdd9c…`) and identical claim
   tallies. The corpus's determinism claim now has an
   independent-machine, cross-ISA witness at this vintage (Section 2
   already had one from the four-machine re-conduct).

2. **Warm walls scale by ~3.5–5×, and NOT by one scalar.** w2145/gb10
   median ratios, phase by phase: produce 3.5–3.6×, in-process check
   4.0–4.1×, seal 4.4–4.5×, whole row wall 3.8–3.9×, per-row
   closure-derivation setup 4.8×, preflight 3.1×, external checker
   marginal re-check 4.3–4.9×, checker per-process intercept 3.0–3.5×.
   A single conversion factor between machines would be off by up to
   ~60% depending on phase mix — the quantitative case for per-machine
   labels on every absolute number.

3. **The protocol decomposition replicates structurally.**
   Per-invocation external checker cells are intercept-dominated on
   both machines (w2145 intercept 215–218 ms, uniform across
   benchmarks; gb10 63–73 ms), and the same-file k-slope marginal
   re-check (25/45/38 ms) is 0.82–0.87× the production loop's
   in-process `check_s` — the same checkers-agree-at-the-margin
   relationship the gb10 cells measured (0.7–0.75×). The retained-copy cost
   is ≈ 0 on both machines — measured inline by the version-2 receipts
   (`rows_retain_copy_s`: ~1.5 ms and ~5 ms across whole 33-row
   campaigns) and cross-checked by gb10's clean-window A/B cell; the
   version-2 inline instrumentation supersedes the outer-wall A/B
   protocol for this question. These
   *relationships* are the machine-portable content of the timing story.

4. **A vintage confound, caught in the cold anchor.** The gb10
   cold-anchor cell rode the version-1 runner, whose progress monitor
   slept out up to 1 s at join; this replication rode the version-2
   runner (channel wakeup at join). Result: an apparent inversion
   (w2145 cold process 0.743 s vs gb10 1.238 s) that is runner vintage,
   not architecture — the version-2 attribution block accounts w2145's
   cold process wall to ~0.34 s preflight + ~0.22 s row setup + ~0.16 s
   row, leaving no second anywhere. The read-each-under-its-own-
   protocol rule therefore covers cold-protocol cells too, not only
   geolog container vintages. Named residue: the gb10 cold anchor
   deserves a version-2-runner re-take next time that family moves.

**Anchor robustness (added at the merge that brought `gb10-quiet-retime/`
in).** After these cells were taken, a preregistered quiet-conditions gb10
family landed (`QUIET-PROTOCOL.md`; version-3 receipts: load-gated ≤ 4.0,
X925-pinned). Its medians sit within ~1% of the loaded gb10 campaign's
(42.2/71.5/63.2 ms vs 41.7/70.8/62.8 ms), so every cross-machine ratio in
this document is robust to the choice of gb10 anchor (e.g. the marbles row
wall ratio is 3.86 against the loaded anchor, 3.81 against the quiet one).
That protocol is also now the natural vehicle for this directory's
promotion rule: a w2145 number bound for a rendered table would be re-taken
under a QUIET-PROTOCOL-style declared admission gate on its own box.

## The publication rule

- **Platform-neutral, no machine label needed**: semantic content
  (container bytes and digests, verdicts, claim tallies, ops counts,
  closure revision, roster completeness) — witnessed identical
  cross-ISA; and protocol *relationships* stated as relationships
  (per-invocation checker cells are intercept-dominated; marginal
  external re-check ≈ in-process check; retained-copy cost ≈ 0 (inline
  measurement, both machines); repeat
  protocols must be warm-loop, since cold process cost is several times
  the row wall) — each replicated on both machines.
- **Per-machine label mandatory**: every absolute wall and everything
  derived from one (medians, phase sums, intercepts, marginals, process
  walls). No honest unlabeled "typical" number exists: the
  cross-machine factor on this pair alone spans 3.0–4.9× by phase.
- **Single-vintage rule, extended**: absolute AND structural
  comparisons only within one runner/checker vintage; the cold-anchor
  inversion is the second specimen family after the geolog
  amortization one.

The chapter already labels cells per machine (machine tags via
`tables/gen_tables.ts`; "walls … will drift with hardware"). The one
prose sharpening landed with these receipts: the receipt-conventions
paragraph's "marginal re-check … is 5–9 ms" sentence now names the
machine those milliseconds belong to and cites this replication, so a
reader cannot lift 5–9 ms as a platform constant.

## Inventory and key numbers

- `produce-host.json` — version-2 campaign receipt, 1+10 roster, 33/33
  byte-identical, accounting identity closed at residual 0.060 s (0.40%).
- `produce-host-run2.json` — stability replication (residual 0.060 s).
- `PROVENANCE.json` — revisions, binary digests, environment, promotion
  rule (any w2145 number promoted to a rendered table must first be
  re-taken in a true zero-co-tenant window per its `MACHINES.md` row).
- `../w2145-checker-compare/` — the comparability cell family (see its
  README for the vintage caution and the retain re-run provenance).

Medians in ms; gb10 = the committed campaign receipts; benchmarks in
the order marbles / regions / unreliable:

| cell | gb10 | w2145 | ratio |
|---|---|---|---|
| row wall | 41.7 / 70.8 / 62.8 | 160.8 / 270.0 / 238.5 | 3.9 / 3.8 / 3.8 |
| in-process check_s | 7.0 / 13.7 / 11.3 | 28.9 / 55.1 / 45.4 | 4.1 / 4.0 / 4.0 |
| external marginal re-check | 5.1 / 9.5 / 8.9 | 25.3 / 45.1 / 38.1 | 4.9 / 4.8 / 4.3 |
| checker per-process intercept | 63 / 73 / 65 | 218 / 215 / 217 | 3.5 / 3.0 / 3.3 |
| per-row setup (closure derivation) | ~47 | ~226 | 4.8 |
| campaign process wall (s) | 3.65 (version-2, gb10-class box) | 15.20 | 4.2 |
