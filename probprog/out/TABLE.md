# Guaranteed posterior bounds: three-way comparison

Box: GB10 (NVIDIA Grace) (aarch64, 20 cores, Cortex-X925 (measured runs pinned to one X925 core @3.9 GHz, nice 19 with /proc read-back)). All rows same box; walls are end-to-end process medians; tools' own internally reported times are context, never the comparator.

## geometric counter — posterior masses P[Y=n], comparison configuration (quality-matched set)

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | geo.spcf, depth 500 | 1.1e-15 on Z; per-bin at the matched quality (Z-interval width (per-bin bounds in raw .bounds receipts)) | 0.8800 | 0.701 | — | — |
| diabolo-residual | residual geo.sgcl -u 100 (artifact comparison command) | 7.89e-31 (residual mass bound (p(n) upper minus exact lower, uniform)) | 0.0027 | 0.00156 | — | — |
| house | probprog-produce spec geo-counter 100 -> dabs-emit -> glc-check | 7.889e-31 (exact: 2^-100 over evidence+residual) (posterior bracket width, class-uniform R/(S+R), exact rational (the absorb claims are exact points at event 8: layered frontier, per-class reachable residual identically zero)) | 0.2982 | — | 0.2152 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (201 claims; the per-class reachability recompute is the dominant check cost at 100 classes) |

## asymmetric random walk — posterior masses P[C=n], comparison configuration (quality-matched set)

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | asym_rw.spcf, depth 100 | 0.639 on Z ([0.9927, 1.6314]) (Z-interval width (per-bin bounds in raw .bounds receipts)) | 51.6200 | 51.4 | — | — |
| diabolo-residual | residual asym_rw.sgcl -u 14 (artifact comparison command) | 7.40e-3 (residual mass bound (float, outward-rounded)) | 0.0018 | 0.00074 | — | — |
| house | probprog-produce spec asym-rw 14 -> dabs-emit -> glc-check | 7.342e-3 (exact: 123181/16777216 over evidence+residual) (posterior bracket width, class-uniform R/(S+R), exact rational (absorb claims exact points at event 8, layered frontier)) | 0.1076 | — | 0.0542 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (15 claims) |

## die paradox — posterior masses P[T=n | even throws], comparison configuration; GuBPI column is unnormalized-lower-only per its own artifact README

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | die_paradox.spcf, polytopes, depth 80 | unbounded (Z upper = NaN; normalized bounds vacuous — upstream-documented); unnormalized lower bounds usable (n/a (baseline defect, its artifact's own caveat)) | 74.7600 | 74.56 | — | — |
| diabolo-residual | residual die_paradox.sgcl -u 6 (artifact comparison command) | 1.65e-2 (normalized total-mass interval [0.9837, 1.0165]) (normalized residual bound (interval-quotient assembly)) | 0.0012 | 0.00028 | — | — |
| house | probprog-produce spec die-paradox 6 -> dabs-emit -> glc-check | 5.464e-3 (exact: 1/183; correlated-ratio form, 3.0x tighter than the interval quotient at the same truncation) (posterior bracket width, class-uniform R/(S+R), exact rational (absorb claims exact points at event 8, layered frontier)) | 0.1048 | — | 0.0534 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (13 claims) |

## tightness rows (same wall class, deeper truncation — house only; no baseline ran these configs in the comparison) — posterior masses at deeper horizons

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| house | asym-rw horizon 70 | 3.012e-7 (posterior bracket width, exact rational) | 0.6848 | — | — | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (71 claims) |
| house | die-paradox horizon 40 | 3.290e-19 (posterior bracket width, exact rational) | 0.1239 | — | — | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (81 claims) |

## burglar alarm (PSI discrete suite) — posterior P[burglary | called] (hard conditioning); Diabolo: n/a — the case is not in its benchmark suite

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | burglarAlarm.spcf, depth 200 (header config) | 0 at the .bounds print precision (validated float; the Z interval it derives from has width 6.8e-16) (normalized bin bounds for burglary in {0,1} (validated-float intervals)) | 0.2600 | 0.08 | — | — |
| house | probprog-produce spec burglar-alarm 4 -> dabs-emit -> glc-check (full unrolling of the finite program) | exactly 0 (R = 0; posterior = 2969983/992160802 for burglary=1, the PSI exact result, as a point interval in Q) (posterior bracket width, class-uniform R/(S+R), exact rational) | 0.1182 | — | 0.0700 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (5 claims; the container crossed the event without re-emission — the baked dac revision is byte-identical to the staged one) |

## Beauquier-Gradinariu-Johnen token ring, 3 processes — P[count < 1] (score-free; truth exactly 1/2); Diabolo: n/a — the case is a GuBPI-suite member with no published Diabolo row (absence stated, not interpolated). Headline, event 8: the per-class reachability sharpening landed in the baked closure, and the house class-0 absorb bracket is EXACTLY [1/2, 1/2] — width 0 as an exact rational point, at every horizon, with strictly weaker hypotheses than the baseline row (no termination assumption; the sharpening is recomputed by the checker, not trusted from the producer) and a replayable certificate. The former near-miss (trailing GuBPI's score-free float-point printout by the sound residual) is closed to a tie-or-better.

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | beauquier-etal-3-Q1.spcf, depth 50 (artifact config; method Polytopes, splits 10000) | 0 at the printed precision — [0.5, 0.5] normalized and unnormalized (exact-point floats leaning on score-free Z = 1) (printed bounds for the count<1 bucket (validated-float intervals)) | 7.8700 | 7.67 | — | — |
| house | probprog-produce spec beauquier-3 51 -> dabs-emit -> glc-check (matched-depth class; time-homogeneous stored rows: 35 states / 91 rows at any horizon) | exactly 0 — class-0 absorb bracket [1/2, 1/2], exact rational point (event-8 reachability sharpening: class 0 is fed only by the start row, its co-reachable residual is zero; the normalized posterior form keeps the uniform residual on both sides of its shared normalizer by design) (class-0 absorb bracket width, exact rational) | 0.1320 | — | 0.0681 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (5 claims) |
| house | probprog-produce spec beauquier-3 1000 -> dabs-emit -> glc-check (tight row, same 91 stored rows) | exactly 0 — class-0 absorb bracket [1/2, 1/2] at horizon 1000, same exact point (class-0 absorb bracket width, exact rational) | 0.8034 | — | 0.7065 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless (5 claims) |

## coupon collector, 5 coupons (EV query) — first moment of the return value (number of draws to collect all five; exact truth 137/12) + tail asymptotics

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| diabolo-geobound-ev | geobound coupon-collector5.sgcl --objective ev -u 80 --optimizer ipopt --optimizer linear (the source file's own flag header; the bench harness drops its -u 30 default when the header overrides) | 1.426e-1 (EV in [11.416659157566642, 11.55925880204251]; matches the published [11.41, 11.56]; IPOPT returned its initial solution in every run on this box, recorded in the raw stdout — the resulting bound equals the published one) (EV bracket width, float) | 12.6500 | 12.64 | — | — |
| diabolo-geobound-tail | geobound coupon-collector5.sgcl --objective tail -u 1 (tail-objective context cell, bench-harness default flags) | tail envelope 4409806508.567278 * 0.8001525531256914^n for n >= 2 (matches the published O(0.8002^n) row) (tail asymptotics (coefficient * rate^n)) | 14.9500 | 14.94 | — | — |
| house | probprog-produce spec coupon-collector5 80 --with-contraction -> dabs-emit -> glc-check (ONE container: absorption + contraction/EV claims) | 4.417e-7 (EV in [11.41666622..., 11.41666666...], exact rationals; contains 137/12; 3.2e5 x tighter than the baseline at the matched config). Tail envelope exactly 5 * (4/5)^n for n >= 80 — the rate is the chain's spectral radius, so no smaller rate exists; strictly tighter than both baseline tail lines in rate and coefficient (EV bracket width, exact rational (decimal rendering)) | 0.1400 | — | 0.0802 | glc-check, baked event-8 roster (dcc revision 70b31e69…: dac + DtmcContraction), flagless — wf=1, absorb=1, posterior=1, genform=1, partial-lo=1, ev=1, tail=1 in one verdict; the dev-oracle EV lane retired at event 8 (its data is container content now, recomputed by the fast-path checker) |

## Israeli-Jalfon self-stabilization, ring of 4 (EV query) — first moment of the return value (loop iterations to reach <= 1 token; exact truth 373/256 by in-crate linear-solve oracle) + tail asymptotics; GuBPI: case not in its suite (checked: no israeli-jalfon file under its benchmarks tree) - absence stated, never interpolated

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| diabolo-geobound-ev | geobound israeli-jalfon4.sgcl -u 30 --objective ev --optimizer ipopt --optimizer linear (bench default unroll 30 + the source file's flags(geobound) header) | 3.983e-4 (EV in [1.4570312269497663, 1.457429486540848]; the published table's [1.457, 1.458] at 4 digits; contains the exact truth 373/256 = 1.45703125) (EV bracket width, float) | 19.1400 | 19.13 | — | — |
| diabolo-geobound-tail | geobound israeli-jalfon4.sgcl -u 1 --objective tail --optimizer ipopt --optimizer linear (the paper's tail protocol: unrolling limit 1) | tail envelope 41187393.48318438 * 0.5010816216979803^n for n >= 2 (matches the published O(0.5011^n) row); the EV objective's own asymptotics line carries only 0.7288 (tail asymptotics (coefficient * rate^n)) | 0.2100 | 0.21 | — | — |
| house | probprog-produce spec israeli-jalfon4 31 --with-contraction -> dabs-emit -> glc-check (ONE container: absorption + contraction/EV claims, anchor-offset 1) | 1.441e-9 (EV in [1.4570312471..., 373/256], exact rationals; the upper endpoint IS the exact truth — the >= 3-token states are transient in three iterations, so the geometric tail bound is achieved; 2.8e5 x tighter than the baseline at matched depth). Tail envelope exactly (99/64) * (1/2)^n for n >= 31 — the rate is the chain's spectral radius, strictly tighter than the baseline tail cell in rate (0.5 < 0.50108) and coefficient (1.5 vs 4.1e7) (EV bracket width, exact rational (decimal rendering)) | 0.1155 | — | 0.0548 | glc-check, baked event-8 roster (dcc revision 70b31e69…: dac + DtmcContraction), flagless — wf=1, absorb=1, posterior=1, genform=1, partial-lo=1, ev=1, tail=1 in one verdict; the dev-oracle EV lane retired at event 8 |

## 3-D asymmetric random walk (EV query; the baseline suite's slowest solved row) — raw first moment of the return value (steps until (x,y,z) = (0,0,0) from (1,1,1); axis uniform 1/3, +1 w.p. 1/4 else monus-decrement) + tail asymptotics + termination mass

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| diabolo-geobound-ev | geobound 3d-asym-rw.sgcl --objective ev -u 30 -d 3 --optimizer adam-barrier --optimizer linear (the source file's flag header + bench default unroll 30) | 1.192e6 (1st raw moment in [9.443844608015025, 1191688.869263926], byte-stable across all 5 runs; the published row reads [9.443, 6.85e5] at the same config, hardware unstated — the optimizer landed a looser polytope on this box) (EV (raw first moment) bracket width, float) | 151.4100 | 151.22 | — | — |
| diabolo-geobound-tail | geobound 3d-asym-rw.sgcl --objective tail -u 3 -d 3 (the source file's tail flags) | tail envelope 396.12864306866237 * 0.9814846813716037^n for n >= 4 (published row: O(0.9812^n)); same cell's 0-th raw moment (termination mass) in [0.09375, 20228.1] (tail asymptotics (coefficient * rate^n) on the return-value density) | 7.0800 | 7.08 | — | — |
| house (u200 flagship) | probprog-produce spec 3d-asym-rw 60 -> dabs-emit -> glc-check (container: stored window max<=11, absorb/Z rows) + evcert 3d-asym-rw 200 -> probprog-evcheck (EV rows: pattern DP horizon 200 + product-profile contraction template, dev-oracle lane) | 1.291e-2 (EV in [16.690028724..., 16.702938972...], exact rationals; 9.2e7 x tighter than the baseline's same-box face). Tail envelope 0.5617 * 0.965^n for n >= 200 on the SURVIVAL mass P[T > n] (which dominates the density): rate 0.965 vs the baseline's 0.98148 — TIGHTEN, coefficient 396 -> 0.56. Termination mass certified exactly 1 (c < 1 forces P[T = infinity] = 0) vs the baseline's [0.094, 20228] (EV bracket width, exact rational (decimal rendering)) | 3.9441 | — | 2.3800 | glc-check, baked event-8 roster (dac revision 6f1b2c7e…), flagless for the container (wf=1, absorb=1, posterior=1; 0.93 s) + probprog-evcheck for the EV rows (partial=1, ev=1, tail=1; 1.45 s) — the 3-D rows are the PATTERN face (finitely presented infinite chain), descoped from event 8 to its own design leg and dev-oracle pending that face's own revision event; spec-sha256 binds both artifacts to the same stored rows |
| house (u150, fast cell) | same pipeline, evcert horizon 150 | 1.117e-1 (EV in [16.681534894..., 16.793210510...], exact rationals; 1.1e7 x tighter); same tail row family at coefficient 0.818 (EV bracket width, exact rational (decimal rendering)) | 1.8413 | — | 1.3300 | same two-checker pipeline (glc baked 0.93 s + evcheck 0.40 s, pattern face dev-oracle pending its own event) |

## herman3 (Herman's self-stabilization, 3 processes) — EV of the self-stabilization time E[count] (truth 4/3 exact; the suite's first case run by BOTH baselines — GuBPI compares on the Q1 case below, its suite's query for this program)

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| diabolo-geobound-ev | geobound psi/herman3.sgcl -u 30 --objective ev --optimizer ipopt --optimizer linear (bench.py defaults + file flags) | 4.750e-4 ([1.3333333109815915, 1.33380835187633], byte-stable across 5 runs, contains 4/3; published row [1.333, 1.334] @ 62.08 s) (1st-raw-moment interval width) | 49.3700 | 49.37 | — | — |
| house | probprog-produce spec herman3 31 --with-contraction -> dabs-emit -> glc-check (ONE container: absorption + contraction/EV claims, anchor-offset 1; matched depth: Diabolo unroll 30 + the prologue transition) | 1.397e-9 exact (claim ev [L, L + 3*2^-31], contains 4/3; upper exceeds truth by exactly (1/6)4^-30 — recurrent above-floor states, the sound-not-achieved tail) (EV bracket width, exact Q) | 0.1331 | — | 0.0539 | glc-check, baked event-8 roster (dcc revision 70b31e69…: dac + DtmcContraction), flagless — wf=1, absorb=2, posterior=2, genform=1, partial-lo=1, ev=1, tail=1 in one verdict; the dev-oracle EV lane retired at event 8 |

## herman3 Q1 (Herman's self-stabilization, 3 processes) — P[count < 1] (truth 3/8 exact; GuBPI suite member ProbEstimation/herman-3, depth 90; Diabolo's suite queries this program as the EV case above)

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | herman-3-Q1.spcf (header: depth 90, discretization -0.1 0.1 0.2) | 0 at print precision ([0.375, 0.375] printed, norm + unnorm — score-free Z=1 exact-point printout, the beauquier-3 mechanism; contains 3/8) (printed posterior-bin interval width) | 54.6500 | 54.44 | — | — |
| house | herman3-u31.glc class-0 absorb/posterior bracket (same container as the EV case's pipeline) | exactly 0 — class-0 absorb bracket [3/8, 3/8], exact rational point (event-8 reachability sharpening: class 0 is START-only, its co-reachable residual is zero — exactly the closure this cell predicted pre-mint; the normalized posterior form keeps the uniform residual, [3/8, 3/8 + R31], by design) (class-0 absorb bracket width, exact Q) | 0.1331 | — | 0.0539 | glc-check, baked event-8 roster (dcc revision 70b31e69…: dac + DtmcContraction), flagless |

