# Guaranteed posterior bounds: three-way comparison

Box: GB10 (NVIDIA Grace) (aarch64, 20 cores, Cortex-X925 (measured runs pinned to one X925 core @3.9 GHz, nice 19 with /proc read-back)). All rows same box; walls are end-to-end process medians; tools' own internally reported times are context, never the comparator.

## geometric counter — posterior masses P[Y=n], comparison configuration (quality-matched set)

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | geo.spcf, depth 500 | 1.1e-15 on Z; per-bin at the matched quality (Z-interval width (per-bin bounds in raw .bounds receipts)) | 0.8800 | 0.701 | — | — |
| diabolo-residual | residual geo.sgcl -u 100 (artifact comparison command) | 7.89e-31 (residual mass bound (p(n) upper minus exact lower, uniform)) | 0.0027 | 0.00156 | — | — |
| house | probprog-produce spec geo-counter 100 -> dabs-emit -> glc-check | 7.889e-31 (exact: 2^-100 over evidence+residual) (posterior bracket width, class-uniform R/(S+R), exact rational) | 0.2650 | — | 0.0510 | glc-check --staged-chain (staged closure 6f1b2c7ef…; 201 claims) |

## asymmetric random walk — posterior masses P[C=n], comparison configuration (quality-matched set)

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | asym_rw.spcf, depth 100 | 0.639 on Z ([0.9927, 1.6314]) (Z-interval width (per-bin bounds in raw .bounds receipts)) | 51.6200 | 51.4 | — | — |
| diabolo-residual | residual asym_rw.sgcl -u 14 (artifact comparison command) | 7.40e-3 (residual mass bound (float, outward-rounded)) | 0.0018 | 0.00074 | — | — |
| house | probprog-produce spec asym-rw 14 -> dabs-emit -> glc-check | 7.342e-3 (exact: 123181/16777216 over evidence+residual) (posterior bracket width, class-uniform R/(S+R), exact rational) | 0.0998 | — | 0.0510 | glc-check --staged-chain (staged closure; 15 claims) |

## die paradox — posterior masses P[T=n | even throws], comparison configuration; GuBPI column is unnormalized-lower-only per its own artifact README

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| gubpi | die_paradox.spcf, polytopes, depth 80 | unbounded (Z upper = NaN; normalized bounds vacuous — upstream-documented); unnormalized lower bounds usable (n/a (baseline defect, its artifact's own caveat)) | 74.7600 | 74.56 | — | — |
| diabolo-residual | residual die_paradox.sgcl -u 6 (artifact comparison command) | 1.65e-2 (normalized total-mass interval [0.9837, 1.0165]) (normalized residual bound (interval-quotient assembly)) | 0.0012 | 0.00028 | — | — |
| house | probprog-produce spec die-paradox 6 -> dabs-emit -> glc-check | 5.464e-3 (exact: 1/183; correlated-ratio form, 3.0x tighter than the interval quotient at the same truncation) (posterior bracket width, class-uniform R/(S+R), exact rational) | 0.0981 | — | 0.0510 | glc-check --staged-chain (staged closure; 13 claims) |

## tightness rows (same wall class, deeper truncation — house only; no baseline ran these configs in the comparison) — posterior masses at deeper horizons

| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |
|---|---|---|---|---|---|---|
| house | asym-rw horizon 70 | 3.012e-7 (posterior bracket width, exact rational) | 0.6437 | — | — | glc-check --staged-chain (71 claims) |
| house | die-paradox horizon 40 | 3.290e-19 (posterior bracket width, exact rational) | 0.1164 | — | — | glc-check --staged-chain (81 claims) |

