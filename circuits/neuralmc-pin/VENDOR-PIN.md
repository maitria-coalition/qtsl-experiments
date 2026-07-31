# neuralmc vendor pin

Upstream: `github.com/aiverification/neuralmc`, **BSD-3-Clause** (the
upstream LICENSE is materialized beside this file by `make fetch-pins`).

Pinned commit: `f2bd71cbc2976a00897c22301f3aafbfa77a0658`
(2025-11-02 13:33:34 +0000, "Moved licence to top level") — first
cloned and extracted 2026-07-20; refetched at the same commit by every
`make fetch-pins`.

Pinned subset (912K of a 345M repo; the tree is fetched from upstream,
never copied into this repository), chosen for what the comparisons
here depend on:

- `Benchmarks/` — the full combined SystemVerilog benchmark set (the
  circuit-design side of the paper's 194 LTL liveness tasks; the task
  list = these designs x the spec scripts below).
- `NeurIPS24_Experiments/` — the NeurIPS'24 runner scripts (directory
  renamed from `NeurIPS'24_Experiments` to drop the apostrophe; contents
  unmodified). `Benchmark_Run/*.py` carry the `specTXT` LTL
  specifications and per-instance parameter ladders — the authoritative
  task definitions used by this campaign's spec census.
- `neuralmc-tool/` — their producer's python core (`nuR.py` training
  loop, `nrf.py` network, `bitwuzlaEncoderBV.py` + `old_` variant: the
  quantized-net SMT encoders). Kept for the neural-certificate-import
  lane (census item F7) and producer reproduction.
- `README.md` — upstream top-level readme (citations for both papers).

- `NeurIPS25_Experiments/` — the NeurIPS'25 artefact's experiment
  subset (directory renamed from `NeurIPS25/{README.md,Experiments/}`;
  contents unmodified; added 2026-07-20 by the upstream delta check —
  see the benchmark-version caveat below). Carries the '25
  runner scripts for all three property classes (Liveness / Safety /
  Safety+Liveness) and `plot_cmp.py`, whose **hard-coded runtime
  arrays are the paper's Table 1 + Figure 3 data** (per its README:
  "Timings are hard-coded to avoid the multi-day cost") — the
  authoritative their-side numbers for any public comparison, incl.
  the rIC3 1.5.1 baseline rows. Cross-validated cell-for-cell against
  the published Table 1 (NeurIPS 2025, pp. 74713–74740).

Pruned, with reasons: `.git` (nested repo), `NeurIPS25/` binaries and
tools (`ebmc_5.6_amd64.deb`, `Tools/{abc_mc,ebmc,neuralmc,nuXmv,rIC3}`
— refetch by pin if needed) and `NeurIPS25/Experiments/plot_ablation.py`
(151K of ablation-study data: learning engines, architectures — not
comparison-load-bearing), `NeurIPS24/Tools/{nuXmv,ebmc,abc_mc}`
(proprietary/binary baselines), `neuralmc-tool/{SMT_benchmark,
Model-SMT,Netlists,Traces,Verilog-Sampler}` (56M of generated
experiment data / empty run dirs).

**Benchmark-version caveat (load-bearing for every comparison):** the pinned `Benchmarks/` is the *combined* ('25-era)
circuit set. Upstream commit `5007492` ("restructure with NeurIPS'25
artefact") **semantically modified 142 of the '24-era `.sv` files**
(delay's counter/reset logic rewritten, PWM's pulse-width constant
changed, load_store's saturation behaviour, gray's `sig` semantics,
i2c_1's parameter N 500 → 12, …) besides adding the 25 blink files.
The '24-era circuits live at upstream commit `37fcae7` only. The
certifications in this repository against this pin (the 194/194 sweep — receipt: its
i2c_1 row reads `divider=3, CBITS=4`, the '25 file) are therefore
apples-to-apples with the **'25 paper's Table 1 re-runs**, and only
approximately comparable to the '24 paper's `tab_cmp` numbers.
