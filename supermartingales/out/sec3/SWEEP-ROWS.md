# sweep-table fragment — wave A (section-3 loop family + twins)

Columns per the campaign plan. House bounds are certified exact-$\mathbb{Q}$
values (display: ceiling at 7 decimals — an upper bound rounds up; the
one lower bound rounds down); at authoring time every row's certificate
was on-wire double-checked (python reference on read-back bytes +
current-source fastcheck — campaign-era format vocabulary; see the
format note in `../sweep/SWEEP.md`), with the mutant battery receipts
in receipts.json. Certification tier, all rows: fastcheck-Trusted +
reference-Checked; Lean glue = wave 2, pending.

| benchmark | their best neural | their Farkas | house certified bound | true $p$ (exact) | their learn+verify (s) | house synth (s) | house check (s) | provenance |
|---|---|---|---|---|---|---|---|---|
| fixed_loop | 0.0094 (PW) | 0.0091 | **0.0089651** | 0.0089641 | 5.8 | 0.01 | 0.04 | reconstructed+fingerprint |
| faulty_loop | 0.0184 (PW) | 0.0181 | **0.0178400** | 0.0178390 | 6.4 | 0.01 | 0.02 | listed+adjudicated |
| faulty_loop__as_listed | 0.0184 (PW) | 0.0181 | **0.0198024** | 0.0198014 | 6.4 | 0.01 | 0.01 | listed |
| faulty_uniform | 0.0221 (PW) | 0.0181 | **[0.0184458, 0.0185556]** | — | 8.1 | 0.21 | 0.20 | reconstructed+fingerprint |
| faulty_rare | 0.0022 (PW) | 0.0019 | **0.0017994** | 0.0017984 | 6.7 | 0.01 | 0.03 | reconstructed+fingerprint |
| faulty_easy1 | 0.0865 (PW) | 0.0801 | **0.0768847** | 0.0768837 | 12.5 | 0.03 | 0.09 | reconstructed+fingerprint |
| faulty_ndecr | 0.063 (PW) | 0.0561 | **0.0544619** | 0.0544609 | 10.0 | 0.02 | 0.06 | reconstructed+fingerprint |
| faulty_walk | 0.0166 (PW) | 0.0121 | **0.0119293** | 0.0119283 | 10.3 | 0.00 | 0.02 | reconstructed+fingerprint |
| faulty_walk__det12 | 0.0166 (PW) | 0.0121 | **0.0119353** | 0.0119343 | 10.3 | 0.01 | 0.03 | reconstructed+fingerprint |
| faulty_concave | 0.1356 (PW) | 0.1819 | **0.1183897** | 0.1183887 | 21.3 | 0.01 | 0.03 | listed |
| faulty_varying | 0.0864 (PA) | 0.1819 | **0.0694720** | 0.0694710 | 26.8 | 0.01 | 0.03 | reconstructed+fingerprint |
| faulty_varying__last_step | 0.0864 (PA) | 0.1819 | **0.0353614** | 0.0353604 | 26.8 | 0.01 | 0.02 | reconstructed+fingerprint |

## Row notes

- **faulty_loop__as_listed**: the adjudication row — its exact true $p = 0.0198014$ EXCEEDS all three of their published bounds for faulty_loop (PA 0.0195, PW 0.0184, Farkas 0.0181); the violation is the finding (adjudication note, workshop-side). The house certificate for the as-listed chain is itself sound: bound 0.0198024.
- **faulty_uniform**: certified two-sided bracket $p \in [0.0184458, 0.0185556]$ (supermartingale upper + submartingale lower, both exact-$\mathbb{Q}$-checked). Their published Farkas value $0.0181 = \lceil 2/111 \rceil_4$ lies BELOW the certified lower bound (margin $\approx 4.28\times 10^{-4}$): for the frozen continuous-advance reconstruction, their baseline column entry is NOT a sound bound — the exit-hole convention (linear surface negative on the reachable overshoot strip $(10, 11]$) is the mechanism, the same non-inductiveness their own published cliff certificate surface exhibits. Independent witness: kit MC pooled two-seed $\hat p = 0.01851 \pm 6.7\times 10^{-5}$ sits inside the bracket and $7.3\sigma$ above $2/111$. (Lower-leg soundness seam, producer-side: a.s. absorption — fault probability $\ge 1/1000$ per step everywhere.)

## Comparability caveats (carried verbatim from the campaign plan)

- Their neural bounds are averages over successful stochastic runs (success 0.8–1.0); house synthesis is deterministic, 12/12.
- Their Z3 check quantifies over $\mathbb{R}^n$ globally; house rows check over the invariant box with the guard/region geometry made explicit per cell (drift rows are anchored at every active lattice state; the cell discipline is recorded in the leg's workshop notes).
- Their timings: A40 GPU + Z3, learn+verify. House timings: workstation CPU, synthesis (exact chain/LP + proof-term assembly + emission) and check (reference on-wire + fastcheck) reported separately.
- Uniform additive slack $\delta = 10^{-6}$: every discrete-row house bound is exactly $p_{true} + \delta$ — the certified-bound column is tight to the truth column by construction, not by luck.
