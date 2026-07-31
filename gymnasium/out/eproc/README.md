# Box2D e-process certificates as sealed containers (`gym-eproc-v0`)

The four LunarLander / LunarLanderContinuous statistical certificates
(anytime-valid lower confidence bounds on the expected episodic return
of the pinned heuristic policy, via a betting-martingale mixture with a
randomized last look) re-expressed as sealed geolog commit containers,
checked from the bytes alone by `glc-check --gym-eproc`.

## What a container carries

Primary data only, over the pinned `gymeproc-v0` vocabulary:

- **protocol constants** — the mixture level (`alphaInv`), the scale
  pair `kLo < kHi`, the claim class as an explicit constructor row
  (min-cap, which consumes the producer's range lemma `R >= kLo` a.s.;
  or clip, the truncated-mean claim with a tight scale), the claimed
  boundary `m*` as the 96-bit `[hi, lo]` pair the certificate schema
  draws on, and the declared episode count;
- **the rollout ledger, verbatim** — every episode return as an exact
  rational (each finite IEEE-754 double is a rational; the conversion
  is value-preserving and invertible), chunked into 2048-entry inline
  rank-1 tensors;
- **the claimed bounds in raw units** — tie-checked exactly against
  `kLo + m·(kHi − kLo)/2^96` at every discharge (in the incumbent
  format the bound was checker *output*; here it is a checked claim);
- **the randomized last look**, when made — the improved boundary
  `mRand >= mDet`, the drawn `u` as an exact rational (demanded
  binary64-representable in the guarded range), and its
  drawn-after-and-independent protocol sentence carried verbatim as a
  declared string row. That declaration is a protocol record, not a
  machine-checkable fact — which is why the deterministic verdict is a
  separate claim relation discharged alongside.

The checker recomputes the entire mixture wealth walk in Q32.96 fixed
point with every rounding directed down (a fixed-point accept implies
the exact-rational accept), from these rows, at every discharge. Wealth
values, scaled observables, and the 16-entry dyadic bet grid are never
stored: the grid lives in checker code on both sides of the
differential, so a grid tamper is unrepresentable in the container
rather than merely detected.

## Reproduction

The producer is `gym-eproc-glc` (mtk). Per certificate:

```
gym-eproc-glc seal --cert <incumbent.cbor> --ledger <ledger.cbor> \
    --out <name.glc>
```

The producer cross-ties the certificate's returns bit-exactly against
the committed rollout ledger, re-verifies through a vendored port of
the incumbent checker's fixed-point lane (no byte is sealed for a
certificate that lane rejects), seals deterministically (public
reproduction seed, zero timestamp), and asserts double-emission
byte-identity. Published numbers move zero: the `claim` column of
`GLC-MANIFEST.json` is byte-for-byte the incumbent checker's printed
bound for the same certificate.

The mutant battery regenerates with:

```
gym-eproc-glc battery --cert <n20k.cbor> --ledger <ledger.cbor> \
    --rand-cert <a100.cbor> --out-dir mutants/
```

## The battery (12 classes, 17 sealed files)

Threshold near-miss (the minimal representable boundary inflation),
bound-relevant ledger perturbation, range-lemma violation
(refuse-with-witness), randomized-boundary inflation, drawn-`u`
perturbation against an untouched deterministic verdict, `u` outside
its guards, randomized-below-deterministic, claim-class confusion
(non-tight clip scale), episode-count lie and bound-tie lie (rows the
incumbent format cannot even express), five envelope-integrity loci,
and the coherent-forgery control — an internally true certificate over
a corpus-foreign ledger, accepted by design on both sides, whose
falsifier is provenance rather than arithmetic (its manifest entry
names the reach boundary). Wherever the incumbent CBOR schema can
express a perturbation, a differential twin rides in
`mutants/cbor-twins/` and the incumbent checker's verdict is recorded
beside the sealed lane's in `DIFFERENTIAL.md`.

## Timing

The sealed-lane checker is the reference-grade exact path: the two
1,000,000-episode containers verify in ~5 s each (two full wealth
walks, 32 million fixed-point multiplies, plus two million
rational-to-fixed-point conversions), against ~0.14 s for the
incumbent checker's SIMD lane on the same claims. Cost may change;
verdicts never — the containers are the same bytes for every checker
tier.
