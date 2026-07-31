# qtsl-experiments

Reproduction artefacts for a set of certified-verification experiment
tables — and for the book chapter they appear in. Everything here is
one of four kinds of object:

- **producers** synthesize certificates (untrusted by construction).
  The python producers that built this repository's certificates are
  migrating to TypeScript on the modelling toolkit's public API and
  return with that migration; a producer is never part of any trust
  story, so the certificates stand on their own in the meantime;
- **fixtures** are the certificates actually published, committed
  byte-for-byte;
- **checkers** replay certificates in exact arithmetic and are the only
  trusted components — they are *imported*, not vendored: the Rust
  checkers come from the companion
  [`qtsl`](https://github.com/maitria-coalition/qtsl) repository via
  Cargo git references (with the shared kernels from
  [`kernels`](https://github.com/maitria-coalition/kernels)), and the
  reference checker lane from a pinned checkout of the same repository
  (TypeScript, except that the circuits checker at the current pin is
  still python — its TypeScript replacement is in progress upstream);
- **receipts** are the measured results (JSON + markdown) from which
  the chapter's tables are re-derived mechanically.

The trust model throughout: a certificate is a small self-contained
object; a checker rebuilds every obligation from the certificate and
the declared model data, importing nothing from any producer. Every
suite ships a mutant battery (deliberately broken certificates that
must refuse at the attacked layer) beside its positive controls.

## One command per chapter section

<!-- Count provenance (regenerate from receipts, never hand-carry):
     17/17 improved rows — chapter/sections/supermartingales.tex +
       chapter/generated/sweep-table.tex, derived from
       supermartingales/out/results.json (17 base benchmarks; the
       4 additional rows there are this repository's own variants).
     245 certificates / 219 tasks — `ls circuits/fixtures/*.cbor | wc -l`
       = 245, over the 219-task pure-liveness class of the '25 suite (the
       '24 suite's 194 tasks plus the blink ladder); see circuits/README.md
       and circuits/SWEEP-RECEIPTS.md.
     28,569-file corpus — chapter/sections/supermartingales.tex and
       supermartingales/recheck.ts (its module docstring). -->
```
make supermartingales   # re-certifying a published table (17/17 rows improved)
make gymnasium          # certified rows for the Gymnasium RL suite
make circuits           # hardware liveness: 245 certificates over all 219 benchmark tasks
make semicon            # fluid-stability certificates for semiconductor fab models
make variadic           # transfer-theorem countermodel batteries (exact rationals)
make chapter            # all of the above + the full typeset chapter PDF
```

Each target runs the full pipeline: `cargo build --release` (checkers,
through the git-reference imports) → the stored certificates and their
mutant batteries are re-verified through the independent checkers →
the tables are re-derived from the receipt files
(`tables/gen_tables.ts`) → `lualatex` typesets the section standalone
into `build/sections/`.

Gymnasium family subtargets: `make gymnasium-text` (FrozenLake,
CliffWalking, Taxi, Blackjack — re-check of the committed certificate
sets and their mutant batteries),
`make gymnasium-box2d` (LunarLander e-process ledgers — replay of the
committed certificates), `make gymnasium-atari` (exact-replay rows;
see the ROM note below), `make gymnasium-classic` and
`make gymnasium-mujoco` (reserved; each prints its honest status).

## Layout

```
checkers/          thin CLI wrappers (Cargo git-deps on the qtsl repo)
gymnasium/         fixtures + receipts (out/), re-check drivers, atari/ subfamily
circuits/          neural-model-checking benchmark suite: fixtures, re-check
                   driver, pinned upstream benchmarks (make fetch-pins)
semicon/           Mini-Fab + MIMAC fluid ladder: models, certificates, receipts
petri/             P-invariant and guarded-specification certificate families
                   over MCC 2026 nets: expected manifests + re-check drivers
                   (the MCC-derived containers themselves are not
                   redistributed — see petri/README.md)
supermartingales/  published-table campaign: specs, certificate listings +
                   manifest, receipts, sealed-container lanes (out/glc,
                   out/glc-baked)
variadic/          transfer-theorem countermodel batteries for plated
                   certificates (exact-rational fixtures; kernel twins in
                   the qtsl repo's Qtsl/Plate/TTransfer.lean)
lean-route/        the Lean-route ladder: fixtures, committed timing receipts,
                   admission gate, sealed-container lanes
spde/              SPDE moment certificates: sealed witness + spectrum
                   containers and their mutant batteries (SPDE-CERT.md)
chapter/           the Experiments chapter: section sources + generated tables
tables/            receipts → LaTeX table generator (gen_tables.ts)
tools/             fetch-pins.sh (pinned third-party trees)
```

## Cloning and building

Requirements: Rust (stable), Node ≥ 22 (the drivers are TypeScript,
zero npm packages), GNU make, git, `zstd`, and a TeX Live with
`lualatex` for the PDF steps. One residue: Python ≥ 3.10 with `cbor2`
and `sympy` (`pip install cbor2 sympy`) is needed only by the pinned
qtsl checkout's circuits checker at the current pin — its TypeScript
replacement is in progress upstream.

Two steps fetch from the companion repositories:

1. **Cargo git dependencies** — the Makefile exports
   `CARGO_NET_GIT_FETCH_WITH_CLI=true`, which makes Cargo fetch
   through your git (and therefore your credentials, should any be
   needed) instead of its built-in fetcher. Nothing else to configure.
2. **The pinned qtsl checkout** — `make` clones the `qtsl` repository
   into `build/qtsl` at the revision pinned in the Makefile
   (`QTSL_REV`), for the reference checker lane and every
   sealed-container lane; all such corpora — the
   guarded-specification lane included — are sealed at the current
   baked revision event and share the one pin.

Public clones need no auth at all; private mirrors work through any
GitHub auth your git already has (the `gh` CLI's credential helper, an
SSH key, or a PAT). The committed `Cargo.lock` pins the exact revisions
of the imported checker crates; bumping them is a deliberate act
(`cargo update` + re-run), never a side effect.

## The Atari ROM note

The `gymnasium/atari` family certifies exact bit-level replay through
a synthesized circuit of the Atari 2600 console (the FOSS
`tiny-atari-2600` core, fetched from its upstream at a pinned commit
by `make fetch-pins`, never copied into this repository) with the game
cartridge as part of the circuit. Game ROM images are copyrighted and
are **not** in this repository, and neither are artefacts that embed
their contents (the synthesized netlists and the RAM traces). What
ships: the claim certificates with their pinned trace witnesses, the
checker receipts, the mutant-battery receipts, and the synthesis and
simulation flow (yosys + Verilator) for rebuilding the circuit from
your own legally obtained ROM image (`gymnasium/atari/README.md`).
The python trace/claim harness that produced the claim bundles is
part of the producer migration described above and returns with it.

## What is deliberately absent

- **The producers, temporarily.** The python programs that synthesized
  these certificates are migrating to TypeScript on the modelling
  toolkit's public API and return with that migration. Everything the
  trust story rests on ships now: the certificates byte-for-byte, the
  independent checkers, the mutant batteries, the receipts, and the
  mechanical receipt-to-table derivation.
- **Invention.** The method-authoring process behind the certificates
  is a record, not a rerunnable artefact; producers re-synthesize
  certificates, they do not re-discover methods.
- **The continuous-state (Pendulum/CartPole) reproduction spine** —
  still graduating toward this repository; the chapter prose covers
  the campaign, `make gymnasium-classic` says so honestly.
- **The full published-table certificate corpus** (28,569 files):
  this repository ships the per-row certificate sets behind the
  published table plus the mutant battery; `supermartingales/recheck.ts`
  is the regression gate over the shipped sets.

## The wider system

This repository is one of six:

- **geolog-alpha** — a local-first database whose tables are logical
  consequences; independently interesting as a principled replacement
  for event-sourcing spines. Certificates live there as ordinary
  database content.
- **qtsl** — the Quantitative Temporal Specification Logic: the
  theory-as-data half of the system and the home of everything
  proof-coupled — the rule catalogue, the machine-checked soundness
  developments (Lean for the rule lemmas, with the HOL4 development
  behind the certified checker growing beside them), and the certified
  checkers this repository imports.
- **kernels** — the shared multi-architecture fast paths, implemented
  once per architecture behind one conformance discipline.
- **mtk** — the modelling toolkit: the producer side, untrusted by
  construction.
- **qtsl-experiments** (here) — the reproduction spine under the
  published experiment tables.
- **maitria.org** — the front door: the narrative site, the guided
  reading paths, and the book.

## Licensing

Code and text: MIT OR Apache-2.0 (see `LICENSING.md`). Third-party
comparison trees are not copied into this repository: `make
fetch-pins` materializes them from their upstreams at pinned commits,
and they keep their own licenses (provenance and license ledgers:
`circuits/neuralmc-pin/VENDOR-PIN.md`,
`gymnasium/atari/CORE-LEDGER.md`).
