# Semicon re-run — delta ledger (2026-07-22)

Pins: mtk `184fa3014389050289a6ec4af055aa517eec0af6` · geolog-alpha
`8c2c14a54da2e205a562ee96743d1571174c006e` · qtsl `c0099c932aa49ba8295c49a84245ce741d2dc8cb`
(the development tip at run time; not yet on the public remote) · kernels
`b0143fa3a27ec60bfefe2c992b5be0c8be7b0bd5` · qtsl-experiments as committed
here. Checker: `fluidcheck` built from the committed Cargo.lock pin
qtsl `c94af1f735254b91298202aa8212a2b9004542b8` (see "checker latest-tips lane" below —
the crate does not exist at any newer qtsl rev). Machines (see MACHINES.md): `epyc`
(n2d-highmem-16 spot, EPYC 7B13 class, europe-west1-b; batch CPU box) and
`gb10` (GB10 aarch64 — same box class as every published number; bridge lane,
nice'd). Timed runs on epyc rode the `/workspace/timing.lock` flock convention.

## The ledger

| number | published | re-derived | cause-class |
|---|---|---|---|
| Mini-Fab drain certs, LBFS+FBFS core, up to 95/96 lots/wk (99.0%) | 10 certs ACCEPT, eps per receipts | **10/10 ACCEPT, byte-identical certs, eps exact-match** (both boxes; producer re-run also byte-identical on both) | unchanged (bit-exact) |
| **the honest failure probe at 96 lots/wk** | producer: no certificate (robust LP infeasible); checker mutant M7 REFUSEs | **STILL FAILS CORRECTLY, both faces**: producer probe "no certificate (expected)" re-run on gb10 AND epyc; M7_overcapacity_release REFUSE locus=drift-bound | unchanged — **no regression** (the section's top-severity check) |
| setup-adversarial variant up to 99.8% of its reduced ceiling | LBFS setupadv 78/wk ACCEPT, 79/wk probe fails | ACCEPT byte-identical; **79/wk probe still fails correctly** (both boxes) | unchanged (bit-exact) |
| cert shape: 819 obligations, checks 1.5–3 ms | GB10 receipts, bench_n=20 | 819 obligations confirmed; gb10 verify-kernel median-of-200: **1.46–2.98 ms** (in the published band); e2e process wall 5.5–8.9 ms; epyc kernel 2.48–5.03 ms | measurement-noise (silicon delta 1.7–1.9× GB10-faster, evidenced by the bridge pair) |
| policy separation near capacity (exact policy-comparison number) | LBFS@95 eps=1/43200 vs FBFS@95 eps=1/38880 (per-receipt exact rationals) | **identical exact rationals reproduced** from the producer re-run, both boxes, certs byte-identical | unchanged (bit-exact) |
| MIMAC-7 at the cover: 172 cells, 13.2% of nominal (11.5% avail-adjusted) | ceiling exact `1000000/7563387` (core), avail exact per receipts; 6 certs + 2 over-ceiling probes + 6 mutants | 6/6 ACCEPT byte-identical (sha256 match), ceilings recomputed exactly equal, probes still correctly refuse, 6/6 mutants at designed loci | unchanged (bit-exact; the station-top number is the cover's proven ceiling — it cannot move without a finer cover) |
| MIMAC-7 refinement path ("binary sign-split trees in the format") | format capability, no certified tree row published | **BOUND IMPROVEMENT, certified**: rung-2.5 joint-LP tree cover `tree_core_th150` — 244 cells, 84,180 obligations, θ = 1964/9903 ≈ **19.83% of nominal, 1.50× the published 13.2%** — ACCEPTs under this re-run's committed-pin checker on both boxes (kernel ~174 ms GB10 / ~275 ms EPYC). Sibling row `tree_core_thc90` (θ≈19.61%) also ACCEPT. | **producer-improvement** (rung-2.5 joint sparse-LP producer, 2026-07-20 — postdates the chapter numbers; cert bytes retained upstream, shas below) |
| Mini-Fab near-capacity extension (same producer wave) | grid stopped at 95/96 | rung-2.5 rows **95.5 and 95.9 lots/wk (99.90% of fluid capacity) ACCEPT** at these pins, as fully-refined tree covers; ceiling bracket (95.999939, 96.000000) per JOINT-LP-CERT.md — the 95→96 gap was grid granularity, nothing left in the weights | producer-improvement (context: published claim "up to 95" remains true and at-capacity-tight; 96 still correctly fails) |

rung-2.5 cert sha256s (bytes deliberately not yet vendored into this
repository — folding them into the reproduction spine is a pending editorial
decision; timings for them ride
`bench_*.json` under `extras/*` and the census under `rung25/*`):

- `mimac7/tree_core_th150.cbor` `c9968852ff253799b9f47d542db1216f17c3ae6892b025bc02c4fdf83a46a4e2`
- `mimac7/tree_core_thc90.cbor` `03f1c214078ed21383ad046cbc6253eb3a7895f5da2d93c2d4b6cdcd28afb09f`
- `minifab/lbfs_core_191_2wk.cbor` `045f510cbf734215c3fcdb3190bb0b92c24be83284fc94be7e45fa349a8f20ef`
- `minifab/lbfs_core_959_10wk.cbor` `19c988ab12642da2c0f92b86d1d9cb0947f610359566a6edd3c439d98d7bbb24`

verdict lines (the committed-pin checker):
`ACCEPT modes=244 obligations=84180 eps=5750631/50000000 … theta=1964/9903 … (rung 2.5)`
and `ACCEPT modes=244 obligations=84180 eps=794413/6250000 … theta=1865/9509 …`.

## Producer lanes

- **CANNOT-YET-RE-RUN-THROUGH-MTK**: mtk at the
  pin has no fluid-drift/queueing-network producer species — no network→
  polytope compiler, no mode/cover enumeration, no drain-certificate emitter
  (`git ls-tree`/`git grep` at `184fa3014`: only generic LP tier-1 float Farkas
  proposals in `crates/mtk/src/lp1st.rs` + the class-F dual slot in
  `witness-types`). The complete port shopping list already exists
  upstream (9 items, HiGHS-duals down to CBOR emission). Same disposition
  as the supermartingales 17-family CANNOT list.
- **What DID re-run**: the section's own producers (the historical python
  instruments — `minifab_fluid_producer.py`, `mimac_cover_producer.py`,
  upstream), end-to-end on both
  boxes: **every emitted certificate byte-identical to the committed artifacts,
  across x86_64 and aarch64** (16/16 sha-match; receipts in this directory).
  Producer walls: minifab grid 2.69 s gb10 / 5.01 s epyc; mimac cover
  4.36 s / 7.67 s (whole-run walls; the cover producer does not instrument
  per-row produce time — fig6 mimac7 `t_produce_ms` is empty by honesty, not
  omission). Note: the minifab docstring's "~2 core-minutes" cost claim is
  ~25× above today's measured wall on either box — stale prior in a comment,
  not a regression.
- The rung-2.5 **long** iteration legs (graft-2 bisection etc.) were NOT
  resumed: a 2026-07-20 wind-down fenced long python-producer iteration
  pending the mtk port, and this re-run honors that fence (the upstream
  checkpoint remains resumable).

## Checker latest-tips lane

**CANNOT — the crate is absent above the committed pin, by upstream removal.**
qtsl `c94af1f` (07-21 11:25) restored flcheck+fluidcheck as workspace members;
`f147e95` (07-21 11:27, two minutes later) reverted that restore; no rev newer
than `c94af1f` on any branch contains `crates/fluidcheck` (`git log --all --
crates/fluidcheck`). At the qtsl pin `c0099c9`, `crates/` holds only
the two provider crates and `checkers/` only circuits+supermartingales. A
`cargo update --precise c0099c9` attempt additionally failed at the remote
("revision not found" — the pin postdates the last push). Same disposition as
the gymnasium flcheck finding; where fluidcheck development
lives now is an open upstream question. `fluidcheck`
has **no kernels dependency** (lock deps: ciborium, num-bigint, num-rational,
num-traits), so the kernels pin is vacuous for this section.

## Provider lanes (special question 2)

**NO-LANE, both CPU and GPU, re-checked at the GROWN roster** (a roster
union at `c0099c9` took it 43→51 after the gymnasium
evidence): the roster's `FarkasValid` family is the QTSL/BL proof-rule Farkas
over sequent rows — nothing in the 51 relations consumes the
`fluid-drift-v0`/`fluid-drift-cover-v1` CBOR species, and fluidcheck
certificates are standalone CBOR that never lower to geolog relations. No GPU
box was needed; the empty lane is a verified finding, not a skipped job.

## Lean-gate size banking (rider 1)

No Lean timings (gated upstream). Banked per instance in `fig6.tsv` +
`rational_census.json`: cert_bytes, obligation_count, and the optional
regressor pair `denominator_bits_max` / `rational_count`. The census
mechanizes JOINT-LP-CERT's checker-economics observation: LP-found weights
carry 256–301-bit denominators (tree rows) vs 18–27-bit for remaining-work
covers; the avail variant's availability fractions compound to 159 bits at
identical obligation count — `bytes-dense-in-rationals` is a real, separable
axis for the Lean-route fit, exactly what the addendum's optional fields exist
to attribute.
