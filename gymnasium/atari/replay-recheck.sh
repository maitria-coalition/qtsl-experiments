#!/bin/bash
# Replay re-verification of the committed AtariCircuit claim bundles from a
# user-supplied ROM image (see README.md — ROMs are never committed; this
# script re-derives the pinned witnesses from yours and compares).
#
# Usage: replay-recheck.sh <pong|freeway|montezuma> </path/to/rom.bin>
#
# Builds the Verilator sim (vendored pinned core — run `make fetch-pins` at
# the repo root first) and the simreplay comparator, then replays every
# committed claim bundle for the chosen game. If a synthesized netlist for
# the game exists at out/<game>.aig it is hash-compared against the claim's
# pin as well (byte-identity requires the pinned yosys — see README.md).
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
GAME="${1:-}"
ROM="${2:-}"

case "$GAME" in
  pong)      BUNDLES="pong-noop-T1000";      SIM_BIN=atari_circuit_sim ;;
  freeway)   BUNDLES="freeway-up-T1000";     SIM_BIN=atari_circuit_sim ;;
  montezuma) BUNDLES="montezuma-key-family-T828 montezuma-door-T1548 montezuma-key-T706-k31 montezuma-key-T706-pixels"
             SIM_BIN=atari_circuit_sim_e0 ;;
  *) echo "usage: replay-recheck.sh <pong|freeway|montezuma> </path/to/rom.bin>"; exit 2 ;;
esac
if [ ! -f "$ROM" ]; then
  echo "ROM image not found: '$ROM'"
  echo "usage: replay-recheck.sh <pong|freeway|montezuma> </path/to/rom.bin>"
  exit 2
fi
if [ ! -d "$HERE/vendor/tiny-atari-2600/src" ]; then
  echo "vendored core missing — run \`make fetch-pins\` at the repo root first"
  exit 2
fi

echo "== building the sim ($SIM_BIN) and the comparator =="
make -C "$HERE/sim" "$SIM_BIN"
(cd "$REPO" && cargo build --release -p simreplay)
SIMREPLAY="$REPO/target/release/simreplay"

AIG_ARG=()
if [ -f "$HERE/out/$GAME.aig" ]; then
  AIG_ARG=(--aig "$HERE/out/$GAME.aig")
fi

FAIL=0
for b in $BUNDLES; do
  echo
  echo "== $b =="
  "$SIMREPLAY" "$HERE/certs/$b" --rom "$ROM" --sim "$HERE/sim/$SIM_BIN" \
    ${AIG_ARG[@]+"${AIG_ARG[@]}"} || FAIL=1
done
echo
if [ "$FAIL" -ne 0 ]; then
  echo "replay-recheck: at least one bundle REFUSED (a wrong or differently-dumped ROM refuses at the first witness compare)"
  exit 1
fi
echo "replay-recheck: every $GAME bundle re-derived bit-identical from your ROM"
