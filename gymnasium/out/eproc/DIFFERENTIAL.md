# Two-checker differential: sealed lane vs incumbent format

Every artifact with a twin was run through both checkers; the verdicts
correspond under the fixed mapping (the incumbent checker has no FAIL
verdict — a decided threshold or tie violation is a REFUSE with a
threshold locus there, and a FAIL here):

| incumbent verdict (CBOR)            | sealed-lane verdict (.glc)  |
|-------------------------------------|-----------------------------|
| ACCEPT                              | ACCEPT (exact claim counts) |
| REFUSE locus=threshold-det / -rand  | FAIL (the obligation is decided false) |
| REFUSE locus=schema / ledger / range-witness / decode | REFUSE (undecidable or refuted structure) |

## Positives (4/4 + the two by-design accepts)

| artifact | incumbent | sealed lane |
|---|---|---|
| ll_u4_min_a100 | ACCEPT bound=18716128944937423588132437098496/2^96 | ACCEPT det=1/1 rand=1/1, bounds tie |
| ll_u4_fixture_n20k | ACCEPT bound=16083034807171798000576214597632/2^96 | ACCEPT det=1/1 |
| llc_u4_min_a100 | ACCEPT bound=22277794896801307395173835079680/2^96 | ACCEPT det=1/1 rand=1/1, bounds tie |
| llc_u4_fixture_n20k | ACCEPT bound=19511345266886683149035013931008/2^96 | ACCEPT det=1/1 |
| base_rand (battery base) | ACCEPT | ACCEPT det=1/1 rand=1/1 |
| m16_coherent_forgery | ACCEPT (true of its own ledger) | ACCEPT — falsifier is provenance (manifest reach note) |

## Mutants

| mutant | incumbent (twin) | sealed lane | in class |
|---|---|---|---|
| m01_mstar_past_boundary | REFUSE threshold-det (wealth 1584563250285286751870878679689 < 1584563250285286751870879006720) | FAIL det=0/1 | yes |
| m02_ledger_entry_lowered | REFUSE threshold-det | FAIL det=0/1 | yes |
| m03_range_violation | REFUSE range-witness (return -190382 below k_lo) | REFUSE range-witness at chunk 0 entry 0 | yes |
| m04_mrand_raised | REFUSE threshold-rand | FAIL det=1/1 rand=0/1 | yes |
| m05_u_raised | REFUSE threshold-rand | FAIL det=1/1 rand=0/1 | yes |
| m06_u_out_of_range | REFUSE schema (u 1.5 not in (0,1)) | REFUSE (u not below 1) | yes |
| m07_rand_below_det | REFUSE schema | REFUSE | yes |
| m08_class_confusion | REFUSE schema (tight scale) | REFUSE (tight scale, both constants named) | yes |
| m09_episode_count_lie | — no twin (the incumbent schema has no episode-count field) | REFUSE (declared 19999 vs 20000 entries) | yes |
| m10_bound_tie_lie | — no twin (the incumbent bound is checker output, not a row) | FAIL det=0/1 (the tie) | yes |
| m11_body_byte | — envelope loci have no incumbent analogue | REFUSE body_hash | yes |
| m12_sig_byte | — | REFUSE signature | yes |
| m13_truncated | — | REFUSE body_truncated | yes |
| m14_magic | — | REFUSE magic | yes |
| m15_version | — | REFUSE container_version | yes |

Notes.

- The m01 margin: the incumbent-published boundary for the n20k
  fixture sits 302231454859242877604683 grid steps (about 2^-18 in
  boundary units, or ~0.7 raw return units) below the certified
  fixed-point boundary — the original producer left slack. m01 is cut
  at the true boundary + one 2^-96 step, so its wealth misses the
  threshold by ~3.3e5 parts in 1.6e30 on both checkers.
- The two verdict-separating mutants (m04, m05) leave the
  deterministic claim standing while the randomized claim falls —
  visible on one verdict line as `gymEprocDetValid=1/1
  gymEprocRandValid=0/1`.
- A lambda-grid tamper is not in the battery because it is not in the
  representable space: no container row carries the bet grid on either
  side of this table.
