#!/bin/sh
# fetch-pins.sh — materialize the third-party trees this repository
# builds against, from their upstream repositories at pinned commits.
#
# The repository deliberately carries PINS, not copies: a reproducer who
# audits our comparisons against neuralmc (NeurIPS'24/'25) should not
# have to trust our copy of the competitor's code, and the Atari core's
# TIA.v/PIA.v lineage carries no explicit upstream license grant that
# would cover our redistributing them. Git's own content addressing
# makes the fetched trees' integrity independent of this repository:
# each fetch checks out an exact commit hash, and every deliberate house
# modification is a visible patch file applied AFTER checkout (see
# gymnasium/atari/vendor/house-patches/ — nothing is ever edited
# silently).
#
#   tools/fetch-pins.sh          # fetch anything missing
#   tools/fetch-pins.sh --force  # refetch everything
#
# Network: github.com. Idempotent: a tree whose .pin-stamp matches its
# pinned hash is left alone.

set -eu

REPO="$(cd "$(dirname "$0")/.." && pwd)"
FORCE="${1:-}"

NEURALMC_URL=https://github.com/aiverification/neuralmc
NEURALMC_SHA=f2bd71cbc2976a00897c22301f3aafbfa77a0658
ATARI_URL=https://github.com/rejunity/tiny-atari-2600
ATARI_SHA=4682d2f60b97fdd8346e5ace65ea7b1bf481d7a5

fetch_commit() { # url sha dest-worktree
  rm -rf "$3" && mkdir -p "$3"
  git -C "$3" init -q
  git -C "$3" remote add origin "$1"
  git -C "$3" fetch -q --depth 1 origin "$2"
  git -C "$3" -c advice.detachedHead=false checkout -q FETCH_HEAD
  got="$(git -C "$3" rev-parse HEAD)"
  [ "$got" = "$2" ] || { echo "fetch-pins: $1 pinned $2, got $got" >&2; exit 1; }
}

stamp_ok() { # dir sha
  [ "$FORCE" != "--force" ] && [ -f "$1/.pin-stamp" ] && [ "$(cat "$1/.pin-stamp")" = "$2" ]
}

# ---------------------------------------------------------------- neuralmc
#
# Layout transform (this list IS the former vendored subset, kept so the
# consuming code's paths are stable; see circuits/neuralmc-pin/VENDOR-PIN.md
# for the provenance rationale and the pruning reasons):
#   Benchmarks/            <- Benchmarks/ (top level: the combined '25-era set;
#                             see VENDOR-PIN.md's benchmark-version caveat)
#   NeurIPS24_Experiments/ <- "NeurIPS24/NeurIPS'24_Experiments/" (apostrophe dropped)
#   NeurIPS25_Experiments/ <- NeurIPS25/README.md + NeurIPS25/Experiments/*
#                             (minus plot_ablation.py + __pycache__; Tools/ binaries
#                             and the .deb never fetched)
#   neuralmc-tool/         <- NeurIPS24/Tools/neuralmc/{nuR.py,nrf.py,
#                             bitwuzlaEncoderBV.py,old_bitwuzlaEncoderBV.py}
#                             (generated experiment data pruned)
#   README.md, LICENSE     <- upstream top level

NMC="$REPO/circuits/neuralmc-pin"
if stamp_ok "$NMC" "$NEURALMC_SHA"; then
  echo "fetch-pins: neuralmc pin up to date ($NEURALMC_SHA)"
else
  echo "fetch-pins: fetching neuralmc @ $NEURALMC_SHA"
  scratch="$REPO/build/fetch-neuralmc"
  fetch_commit "$NEURALMC_URL" "$NEURALMC_SHA" "$scratch"
  # never remove our own tracked provenance docs
  find "$NMC" -mindepth 1 -maxdepth 1 ! -name 'VENDOR-PIN.md' -exec rm -rf {} +
  mkdir -p "$NMC/NeurIPS25_Experiments" "$NMC/neuralmc-tool"
  cp -R "$scratch/Benchmarks" "$NMC/Benchmarks"
  cp -R "$scratch/NeurIPS24/NeurIPS'24_Experiments" "$NMC/NeurIPS24_Experiments"
  cp "$scratch/NeurIPS25/README.md" "$NMC/NeurIPS25_Experiments/README.md"
  for d in Liveness Safety "Safety+Liveness"; do
    cp -R "$scratch/NeurIPS25/Experiments/$d" "$NMC/NeurIPS25_Experiments/$d"
  done
  cp "$scratch/NeurIPS25/Experiments/plot_cmp.py" "$scratch/NeurIPS25/Experiments/run_exp.py" \
     "$NMC/NeurIPS25_Experiments/"
  for f in nuR.py nrf.py bitwuzlaEncoderBV.py old_bitwuzlaEncoderBV.py; do
    cp "$scratch/NeurIPS24/Tools/neuralmc/$f" "$NMC/neuralmc-tool/$f"
  done
  cp "$scratch/README.md" "$scratch/LICENSE" "$NMC/"
  rm -rf "$scratch"
  echo "$NEURALMC_SHA" > "$NMC/.pin-stamp"
  echo "fetch-pins: neuralmc materialized"
fi

# ------------------------------------------------------- tiny-atari-2600
#
# Full upstream tree at the pinned commit, minus repository furniture
# (.github/.devcontainer/.vscode/.gitignore) and the bulky die-shot
# images + GDS macros the flows never read; then the house patches
# (P0-P3, each marked "HOUSE PATCH" in-file and documented in
# gymnasium/atari/CORE-LEDGER.md) applied as one visible patch file.

TAT="$REPO/gymnasium/atari/vendor/tiny-atari-2600"
if stamp_ok "$TAT" "$ATARI_SHA"; then
  echo "fetch-pins: tiny-atari-2600 pin up to date ($ATARI_SHA)"
else
  echo "fetch-pins: fetching tiny-atari-2600 @ $ATARI_SHA"
  scratch="$REPO/build/fetch-tiny-atari"
  fetch_commit "$ATARI_URL" "$ATARI_SHA" "$scratch"
  rm -rf "$TAT" && mkdir -p "$TAT"
  (cd "$scratch" && git archive HEAD | tar -x -C "$TAT")
  rm -rf "$scratch" \
         "$TAT/.github" "$TAT/.devcontainer" "$TAT/.vscode" "$TAT/.gitignore"
  find "$TAT/docs" -type f ! -name 'info.md' -delete
  find "$TAT/macro" -type f ! -name '*.lef' ! -name '*.lib' -delete
  (cd "$REPO/gymnasium/atari/vendor" && patch -p1 -s < house-patches/house.patch)
  echo "$ATARI_SHA" > "$TAT/.pin-stamp"
  echo "fetch-pins: tiny-atari-2600 materialized + house patches applied"
fi

echo "fetch-pins: done"
