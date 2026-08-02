#!/usr/bin/env bash
# The four preregistered arms of the sec1 order/interleave methods
# experiment (out/glc-baked/sec1/gb10-order-interleave/PREREG.md): three
# roster rotations (blocked, 1 warmup + 10 repeats per bench) and one
# round-robin interleaved conduct (11 rounds of single-repeat entries in
# one process). Each arm independently re-checks the quiet load gate; a
# refusal (exit 3) stops the sequence and leaves already-completed arms
# installed — per the preregistration, completed arms stand.
#
# Containers are not retained here: byte-identity is asserted per-row
# against the committed incumbents, and retain_copy_s is timed outside
# each row's wall_s, so retention affects no analyzed number.
set -u
BIN=$1; EXPERIMENTS=$2; STAGE=$3; GATE=$4; CPUS=$5
DEST=$EXPERIMENTS/supermartingales/out/glc-baked/sec1/gb10-order-interleave
renice 19 -p $$ >/dev/null

M="--bench faulty_marbles"; R="--bench faulty_regions"; U="--bench faulty_unreliable"

run_arm() {
  local label=$1 warmups=$2 repeats=$3; shift 3
  mkdir -p "$STAGE/$label"
  taskset -c "$CPUS" "$BIN" retime \
    --experiments "$EXPERIMENTS" --machine gb10-quiet --lane host \
    --warmups "$warmups" --repeats "$repeats" --max-load-1min "$GATE" \
    --out "$STAGE/$label/campaign.json" \
    --compare "$EXPERIMENTS/supermartingales/out/glc-baked/sec1/incumbents" \
    --pretty "$@" || return $?
  mkdir -p "$DEST/$label"
  cp "$STAGE/$label/campaign.json" "$DEST/$label/campaign.json"
  echo "order-interleave: arm $label installed"
}

INTERLEAVE=""
for _ in $(seq 11); do INTERLEAVE="$INTERLEAVE $M $R $U"; done

run_arm order-mru      1 10 $M $R $U   || exit $?
run_arm order-rum      1 10 $R $U $M   || exit $?
run_arm order-umr      1 10 $U $M $R   || exit $?
run_arm interleave-rr  0 1  $INTERLEAVE || exit $?
echo "order-interleave: all four arms installed in $DEST"
