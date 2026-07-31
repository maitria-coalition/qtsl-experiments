# variadic — transfer-theorem countermodel batteries

Refutation-side probes for the plated-certificate **transfer theorem**:
for a certificate that folds per-token features over a marking, one
footprint row per transition schema bounds the whole marking at every
extent — *under hypotheses*, each of which these batteries knock out
one at a time, verifying at every move that the rows the would-be
certificate carries all HOLD while the transfer conclusion FAILS.

Two scripts, one numbering:

| script | cells | content |
|---|---|---|
| `transfer_countermodels.py` | positive trials + CM1–CM5 + φ-space + toy | fold locality / safety / gap transfer on randomized exact instances; strictness-without-gap (CM1); the mean's arity-dependent modulus (CM2); the max carousel (CM3); ℓ₂ aggregate-scale vanishing (CM4); the signed-birth descent (CM5); φ-space certification and verdict transport at p = 2; a two-station manufacturing-line toy with a tight 18 ≤ 18 bound |
| `normalized_head_countermodels.py` | CM6–CM11 | arity normalization breaks footprint locality (CM6); transformed gap ≠ aggregate-scale gap (CM7); polynomial heads over multiple folds (CM8/CM8b); expectation rows give no pathwise bound (CM9); guard scope (CM10); gap-failure needs a separated pair (CM11) |

Both are **fixtures, not grinders**: sub-second, Python `fractions`
only — no floats, no roots, no dependencies. Each exits nonzero on any
failed cell.

## Kernel-checked twins

The theorems and most countermodels are mechanized in the qtsl
repository, `lean/qtsl/Qtsl/Plate/TTransfer.lean`: safety transfer for
any monotone commutative-monoid fold, ranking transfer at
translated-sum scales with the telescope and count bounds, the max
carousel (CM3), the halving run (CM1), the signed-birth descent (CM5),
the mean birth (CM6), verdict transport with its domain edge, the
square-gap cell (CM7), and the factorized modulus witness
`δ/(p·B^(p−1))`. The cells **without** Lean twins — CM2, CM4, CM8–CM11,
and the randomized positive trials — are these batteries' unique cargo;
for the twinned cells the batteries are an independent, decorrelated
re-verification of the same exact instances.

## Running

```sh
make variadic          # from the repository root
# or directly:
python3 variadic/transfer_countermodels.py
python3 variadic/normalized_head_countermodels.py
```
