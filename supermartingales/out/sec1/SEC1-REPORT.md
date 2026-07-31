# Section-1 rows (wave C) — certified bounds + procurement findings

Wave C of the first verification wave: the three envelope-validated
SS1 reconstruction candidates, plus the CAV'25-artefact procurement leg.
All numbers below are exact $\mathbb{Q}$, decimalized for display. At
authoring time every certificate row passed BOTH the reference checker
(`bl_checker`) and a second, native checker on the emitted certificate
bytes (an earlier binary certificate format, since retired; the
certificates ship in this repository as geolog listing rows — per-file
pins in `../../MANIFEST.jsonl` — and `make supermartingales` re-verifies
them through the pinned qtsl reference checker).

## Per-row certified bounds

Provenance for all three rows: **reconstructed-per-description** (envelope
+ negative fingerprint validated; NOT fingerprint-pinned — and see the
procurement result below: no public artefact exists to pin them against).

| row | p_true (exact, dec.) | house bound (VI-export) | house bound (poly-LP) | their PW / PA | tightness (p_true/PW) | rows checked | verdict |
|---|---|---|---|---|---|---|---|
| faulty_marbles | 0.0623603 | **0.0623603** (+6.2e-11) | 0.0624239 (deg 4) | 0.0649 / 0.0739 | 96.1% | 23 + 11 | CERTIFIED |
| faulty_unreliable | 0.0516560 | **0.0516560** (+1.0e-10) | 0.0517600 (deg 4) | 0.0536 / 0.0553 | 96.4% | 29 + 11 | CERTIFIED |
| faulty_regions | 0.0406530 | **0.0406530** (+5.1e-11) | 0.0562584 (deg 6) | 0.0411 / 0.0473 | 98.9% | 29 + 11 | CERTIFIED |

- **VI-export route** (primary): piecewise-bilinear lift
  $V(x,e) = (1-e)L_k(x) + e$ of the slack-inflated exact value vector
  $h + \varepsilon T$ ($\varepsilon = 10^{-12}$; $T$ = exact expected
  absorption time, drift exactly $-\varepsilon$ at active nodes). The
  bound equals $p_{\mathrm{true}} + \varepsilon T(s_0)$ — indistinguishable
  from truth at 7 decimals, and **beats their best neural bound on all
  three rows**. Known cautions honoured: Park-inductive witness exported
  (not a trace), slack applied before any checker, cell rows are exact.
- **poly-LP route** (secondary, decorrelated synthesis): one GLOBAL
  polynomial $P(x)$, float LP proposes / exact $\mathbb{Q}$ ratifies
  (rationalize + constant-shift repair, which can only improve drift).
  Beats their neural bounds on marbles + unreliable at degree 4.
- Mutant battery per row (two-thermostat pattern): false-$p$ claim,
  scaled certificate (drift rows refuse), lying leaf (checker REJECT) —
  all refused, all three rows.
- Receipts: `<row>/receipts.json` (VI-export), `<row>/receipts-poly.json`
  (poly-LP). At authoring time the 92 byte-format certificate files all
  passed the native checker (92/92 ACCEPTs); the shipped forms are the
  listing rows under `<row>/rows{,-poly}/`.

## The representation finding (input to the sweep table)

On **faulty_regions** the global-polynomial route does NOT beat their
2-layer (3,1) network: degree ladder 6/8/10 gives 0.0563/0.0508/0.0498
against their 0.0411, converging slowly (at degree 12 the float LP
proposal degrades and the exact pipeline certifies a sound-but-useless
bound — the system fails safe; exactness was never at risk). The
region-dependent hazard makes the true value function non-monotone with a
kink structure that a single polynomial pays degree for, while their
2-layer net and the house piecewise-linear VI-export both express it
cheaply — the house piecewise route still wins the row outright
(0.0406530 < 0.0411). So their architecture claim for this row
(2-layer-worthiness) is *corroborated* by the house degree ladder, and
the honest sweep-table sentence is: piecewise representation (ours or
theirs) is what this row rewards; the house version of it is exact and
certified.

Comparability caveat (carried through from the campaign plan): their neural bounds
average over successful stochastic runs (success 0.8–1.0); house numbers
are deterministic, exact, and checker-refereed.

## Procurement leg: the CAV'25 artefact does NOT pin anything

Zenodo **15222907** ("Quantitative Supermartingale Certificates", CAV'25,
same group) = one 1.17 GB file, *named* .zip but actually an OCI/Docker
image tar. Full census of all 11 layers (file lists recorded during
procurement): the artefact's benchmark set is
{becomingRichOnce, gamblersRuin d2–d5, reactivityOne/Two, repeatedCoin},
Handelman-template synthesis, sets 1–4 under `/usr/src/app`. **Zero
occurrences of any CONCUR'23 benchmark name in any layer.** So: no
provenance upgrades are available from this artefact for the SS1 trio,
and no persist_2d definition either. The spec headers now carry this as a
negative procurement receipt.

### persist_2d verdict: noise class remains UNPINNED

- CAV'25 artefact: refuted as a source (above).
- ChakarovS13 (CAV'13), fetched and read:
  its only 2D candidate is **2DWALK** ("Random walk on R2", |X|=4), with
  details only in an offline TR (systems.cs.colorado.edu, dead link).
- The group's actual persistence paper — Chakarov–Voronin–Sankaranarayanan,
  **TACAS'16** (`tacas16-persistence.pdf`) — carries 2D persistence
  exemplars in BOTH noise classes: uniform U(−0.01,0.01) vs Gaussian
  N(0,0.25) variants of the two-room controller, and an
  additional-systems table with N(−1,1), N(0,1), U(−1,1) 2D dynamics.
- The CONCUR'23 grammar names only Bernoulli and Gaussian (wave-0
  finding) — still the strongest hint, still not a pin.

**Requirement sharpening for the conditional Gaussian leg** (not built
in this wave): with a GLOBAL polynomial certificate, the drift expectation
has exact rational coefficients (even central moments of N(0,σ²) with
rational σ² are rational) — **no truncation machinery needed at all**;
what changes is the nonnegativity side: global-domain rows need PSD/SOS
(house psdcert) instead of BernCone-on-box. Truncation + tail-mass is
required only if a *piecewise* certificate is wanted (incomplete Gaussian
moments are non-rational and need enclosures). This halves the
anticipated scope of that leg in the global-certificate case.

## Named next legs

1. **persist_2d candidate synthesis** (noise class to be decided): freeze
   twin candidates (U and N variants) and envelope-check both against
   0.1026 — the TACAS'16 exemplars give natural dynamics templates; the
   Gaussian variant needs the psdcert global-nonneg wiring above.
2. **Wave 2 Lean glue**: the SS1 rows are now concrete `reach_avoid`
   instances; the wave-2 glue-lemma leg can cite these
   receipts as its worked pair.
3. **Optional**: regions-row ReLU demonstrator (Route-A′) to complete the
   representation story from the neural side — cuttable, per wave 1b.
