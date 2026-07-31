# SPDE moment certificates — the sealed reproduction route

Exact-rational expected-energy certificates for the 1-D stochastic
heat and stochastic Burgers equations on the $2\pi$-torus with
additive finite-rank $Q$-Wiener noise,

$$du = (\nu\,u_{xx} - u\,u_x)\,dt + dW_Q,$$

certified at the stochastic weak-energy rung: for
$\bar E(t) = \mathbb{E}\bigl[\int u_t^2\,dx\bigr]$, the claims

- $\alpha = 0$: $\bar E(t) \le \bar E(s) + \mathrm{tr}(Q)\,(t-s)$;
- $\alpha = 2\nu$: $\bar E(t) \le e^{-2\nu t}\,\bar E(0) +
  \tfrac{\mathrm{tr}(Q)}{2\nu}\,(1 - e^{-2\nu t})$,

with non-degeneracy witnesses (the noise demonstrably injects energy:
per-mode second-moment trajectories and a quantitative floor-mass
attainment) and two-sided summed-floor enclosures whose **floor tie**
$F + \mathrm{deficit} = \beta/\alpha$ pins the certificate's
correction constant against independently recomputed mode-law
content.

## Instances

| artifact | equation | noise ($\sin$ modes) | $\mathrm{tr}(Q)$ | floor $\mathrm{tr}(Q)/2\nu$ |
|---|---|---|---|---|
| `stoch-heat-1d-periodic` | heat, $\nu = 1/10$ | $1\!:\tfrac15,\ 2\!:\tfrac3{50}$ | $\tfrac{13}{50}$ | $\tfrac{13}{10}$ |
| `stoch-burgers-1d-periodic` | Burgers, $\nu = 1/10$ | $1\!:\tfrac15,\ 2\!:\tfrac3{50}$ | $\tfrac{13}{50}$ | $\tfrac{13}{10}$ |
| `stoch-heat-1d-periodic-sharp` | heat, mode-1 only (sharp) | $1\!:\tfrac15$ | $\tfrac15$ | $1$ |

## The sealed route

Each certificate is one signed geolog commit in a `.glc` container
(`out/glc/`), produced by the `mtk` toolkit: the system is
reconstructed through the SPDE catalogue profile (jet variables +
differential-polynomial drift + a noise spectrum whose tail form is a
required constructor argument — never read as zero), and the
covariance is **derived, never stored** — the producer's trace comes
from the profile's own derived-covariance surface, the checker's from
the sealed driver rows, and they meet at a number neither stores.

The container carries primary data only (spectrum, claims,
correction node, witness trajectories, enclosure tables) plus claim
rows in three relations:

- `spdeClaimValid` — q-table well-formedness (mode 0 silent, modes
  within the declared rank, no duplicates, nonnegative weights),
  the Itô-correction recompute
  $\mathrm{value} = \tfrac12\,\mathrm{hess}\cdot\mathrm{tr}(Q)$, the
  beta discipline $\beta = \beta_{\mathrm{jet}} + \mathrm{value}$,
  and the floor row $\beta/\alpha$;
- `spdeTierWitnessValid` — every live noise mode carries a
  second-moment trajectory whose enclosures are replayed VERBATIM
  from independent alternating exponential brackets
  ($S_{\mathrm{odd}} \le e^{-x} \le S_{\mathrm{even}}$ on
  $0 < x \le 1$), monotone, confined below the mode floor; the summed
  last lower brackets attain the declared fraction of the partial
  floor;
- `spdeSummedFloorValid` — common-time two-sided sums with powered
  brackets $e^{-x} = (e^{-x/n})^n$ at the canonical minimal
  $n = \max(\lceil x\rceil, 1)$, strictly monotone, confined below
  $F = \sum_k q_k/(2\nu k^2)$, width-controlled, attaining the
  declared fraction of $F$ — and the floor tie, recomputed termwise.

The checker (`glc-check`, from the pinned qtsl checkout's
`qtsl-geolog-provider` crate) verifies container + envelope integrity
and then discharges every claim row through the exact-rational ground
provider — the same plugin seam the geolog engine's chase drives.
Rates, mode floors, the trace, and every bracket are recomputed from
the sealed spectrum at every discharge.

## Verify

```sh
GLC_CHECK=/path/to/glc-check node --experimental-strip-types spde/glc-recheck.ts
```

Positives must ACCEPT with the manifest's exact per-relation claim
counts; the twelve committed mutants must be rejected in their
recorded classes. Designed loci worth singling out:

- `m3_consistent_forgery` — the correction constant, the beta slot,
  AND the floor row all forged coherently: every claim relation and
  the tier witness pass, and the summed-floor tie is the only lock
  that fails. The tie is the family's falsifier: it compares the
  floor against $F + \mathrm{deficit}$ recomputed from the q-table,
  so a laundered correction cannot survive it.
- `m5_mode_dropped` — a live mode's trajectory removed with all
  aggregates consistently recomputed: caught semantically (noise-
  silent vacuity), not by counting.
- `m7_sf_dropped` — the whole summed-floor block omitted: every
  surviving claim holds; only the claim-count gate catches it.

Sealing is deterministic (public reproduction seed, zero timestamp,
canonical body transform): re-producing the certificates yields
byte-identical containers, so each commit's content address is the
certificate's reproducible identity.

## Scope

This route covers the stochastic moment core: the expected-energy
claims, the Itô-correction discipline, both witness tiers, and the
summed-floor enclosures. The deterministic jet-layer rows the
certificates also carry in their workshop form (evolution-integrand
recomputes, divergence rows, remainder nonnegativity, spectral
discharges) and the probability rows (Chebyshev-on-certified-
variance, Kushner–Ville) are checked by the workshop producer/checker
pair at production time; their sealed-claim species are planned
successors, as are the absorbing-ball (KSE) and pathwise/forcing-box
(RPDE) siblings of this family.

## The spectrum containers (split-container design)

Each certificate's noise spectrum is ALSO sealed as its own container
on the **baked theory-pair closure** (`*-spectrum.glc`, the manifest's
`spectra` records) — the family's split-container design: a mixed
prerequisite table naming both vocabularies in one container is
refused by the checker outright (the battery's `sp5_prereq_mix`
demonstrates the exact refusal string), so the re-home is a second
container, cross-tied by recorded commit address, never a mixed one.
The witness containers above are untouched — same bytes, same lane,
checked forever.

In the spectrum container the synthetic q-table becomes baked
vocabulary: Fourier terms → driver rosters → a noise spectrum with an
explicit tail form, with each mode's power re-expressed as a sum of
rational squares over single-basis-element drivers (the derived
covariance stays exactly diagonal). Its claim surface today is
**amplitude well-formedness** (`qtsl_poly_canonical` per driver
amplitude), checked with `glc-check --baked-cert`.

**Stated limitation, documented rather than hidden**: no independent
checker yet binds spectrum content to the moment core — the derived-q
equality (Parseval recompute from the baked driver rows against the
synthetic q-table, tail-form and rank/support ties included) is a
production-time refuse-to-seal gate, mirrored by the producer's test
suite. The battery carries a designed specimen for exactly this
boundary: `sp3_coeff_forged` (a forged Fourier coefficient) is
**expected ACCEPT** in-lane, and the production gate's refusal of the
same content is witnessed producer-side. A baked spectrum-law side
condition is the named successor (it requires a new theory revision);
when it lands, sp3's class flips to FAIL and the cross-tie becomes
checker-side. Until then, all spectrum-law MATH stays checked on the
witness lane.

The remaining spectrum mutants land as recorded: amplitude
non-canonicity and zero-coefficient forms FAIL under `qtsl_poly_canonical`;
a zero-claim container REFUSES (the vacuous-accept class); byte-level
tampers REFUSE at the envelope.

Run both lanes: `make spde-glc` (or the direct invocation above —
the driver walks the whole manifest: witness positives, spectrum
containers, both batteries).
