#!/usr/bin/env bash
# run-ladder.sh — timed ladder for the Lean-route measurement.
#
#   run-ladder.sh FIXTURE_DIR REFLECTION_PKG_DIR OUT_DIR [BUDGET_S]
#
# For every *.geolog in FIXTURE_DIR (ascending by size):
#
#   compiled route  the geolog_reflect executable, REPS passes; the
#                   executable prints per-stage and end-to-end
#                   wall-clock ms (its end-to-end figure is a second,
#                   fresh evaluation of the whole pipeline — the outer
#                   wall here is therefore ~2x the reported time, and
#                   the outer timeout is sized 2x budget + slack so a
#                   near-budget certificate still reports its number;
#                   the budget verdict is applied to the REPORTED
#                   end-to-end ms, never the outer wall)
#   kernel route    one generated receipt module elaborated with the
#                   whole pipeline inside the kernel via `decide`,
#                   under `timeout BUDGET` with peak-RSS recorded
#
# Verdict vocabulary (one row per route x pass, receipts.jsonl):
#   accept | refuse | timeout | oom | error
# A route that exceeds BUDGET_S (default 600 s) is a timeout row and
# renders as an em-dash in every derived table.
#
# Timing integrity: run this alone on a dedicated box; warm the build
# first (any prior lake build of the package); nothing else scheduled.
set -uo pipefail

FIXDIR=${1:?fixture dir}
PKG=${2:?lean/checkers/reflection package dir (built)}
OUT=${3:?output dir}
BUDGET=${4:-600}
REPS=${REPS:-3}
OUTER=$((BUDGET * 2 + 60))
mkdir -p "$OUT/receipts" "$OUT/kernel-modules"
JL="$OUT/receipts.jsonl"
: >"$JL"

REFLECT="$PKG/.lake/build/bin/geolog_reflect"
[ -x "$REFLECT" ] || { echo "missing built executable: $REFLECT" >&2; exit 2; }

HOST_JSON=$(python3 - <<'PY'
import json, platform, subprocess
model = ""
for line in open('/proc/cpuinfo'):
    if line.startswith('model name'):
        model = line.split(':', 1)[1].strip(); break
mem_kb = int([l for l in open('/proc/meminfo') if l.startswith('MemTotal')][0].split()[1])
print(json.dumps({"host": platform.node(), "cpu": model,
                  "threads": int(subprocess.run(['nproc'], capture_output=True, text=True).stdout),
                  "mem_gib": round(mem_kb / 2**20, 1),
                  "arch": platform.machine()}))
PY
)
echo "host: $HOST_JSON"

emit() { # name bytes route pass json-fragment
  printf '{"name":"%s","container_bytes":%s,"route":"%s","pass":%s,"budget_s":%s,%s,"host":%s}\n' \
    "$1" "$2" "$3" "$4" "$BUDGET" "$5" "$HOST_JSON" >>"$JL"
}

for f in $(ls -S -r "$FIXDIR"/*.geolog 2>/dev/null); do
  name=$(basename "$f" .geolog)
  bytes=$(stat -c%s "$f")
  echo "== $name ($bytes B) =="

  # ---- compiled route, REPS passes --------------------------------
  # Outer wall: 2x budget + slack in the borderline zone (a near-budget
  # certificate must still print its number through the double pass);
  # far above it (> BIG_BYTES) a single budget + slack suffices — if
  # even the first pass has not finished by then, the end-to-end time
  # exceeds the budget a fortiori, and the log carries the stage
  # evidence for that inference.
  outer=$OUTER
  [ "$bytes" -gt "${BIG_BYTES:-30000}" ] && outer=$((BUDGET + 30))
  for rep in $(seq 1 "$REPS"); do
    log="$OUT/receipts/${name}.compiled.$rep.log"
    tv="$OUT/receipts/${name}.compiled.$rep.time"
    /usr/bin/time -v -o "$tv" timeout "$outer" "$REFLECT" "$f" >"$log" 2>&1
    rc=$?
    rss=$(sed -n 's/.*Maximum resident set size (kbytes): //p' "$tv" 2>/dev/null | head -1)
    e2e=$(sed -n 's/.*end-to-end: ACCEPT (\([0-9.e+]*\) ms)$/\1/p' "$log" | head -1)
    tread=$(sed -n 's/.*reader: ok (\([0-9.e+]*\) ms)$/\1/p' "$log" | head -1)
    trows=$(sed -n 's/.*rows→wire: ok (\([0-9.e+]*\) ms)$/\1/p' "$log" | head -1)
    televval=$(sed -n 's/.*elevation: ok (\([0-9.e+]*\) ms); validity: ACCEPT (\([0-9.e+]*\) ms)$/\1 \2/p' "$log" | head -1)
    telev=${televval%% *}; tval=${televval##* }
    if [ -n "$e2e" ]; then
      over=$(python3 -c "print(1 if float('$e2e') > $BUDGET*1000 else 0)")
      verdict=$([ "$over" = 1 ] && echo timeout || echo accept)
      emit "$name" "$bytes" compiled "$rep" \
        "\"outcome\":\"$verdict\",\"t_e2e_ms\":$e2e,\"t_read_ms\":${tread:-null},\"t_rows_ms\":${trows:-null},\"t_elev_ms\":${telev:-null},\"t_valid_ms\":${tval:-null},\"maxrss_kb\":${rss:-null}"
    elif [ "$rc" -eq 124 ]; then
      emit "$name" "$bytes" compiled "$rep" \
        "\"outcome\":\"timeout\",\"t_e2e_ms\":null,\"outer_wall_s\":$outer,\"maxrss_kb\":${rss:-null}"
      break   # further passes would time out identically; one attempt suffices
    elif grep -q REFUSE "$log"; then
      emit "$name" "$bytes" compiled "$rep" "\"outcome\":\"refuse\",\"t_e2e_ms\":null"
      break
    else
      emit "$name" "$bytes" compiled "$rep" "\"outcome\":\"error\",\"rc\":$rc,\"t_e2e_ms\":null,\"maxrss_kb\":${rss:-null}"
      break
    fi
  done

  # ---- kernel route, one pass -------------------------------------
  # KERNEL_MAX_BYTES caps which rungs are attempted in the kernel; a
  # rung above the cap gets an explicit not_attempted row (distinct
  # from a measured timeout) so the table can say which is which.
  if [ "$bytes" -gt "${KERNEL_MAX_BYTES:-999999999}" ]; then
    emit "$name" "$bytes" kernel 1 "\"outcome\":\"not_attempted\",\"t_wall_s\":null,\"maxrss_kb\":null"
    echo "  kernel: not attempted (bytes > KERNEL_MAX_BYTES)"
    continue
  fi
  mod="$OUT/kernel-modules/Receipt_${name}.lean"
  [ -f "$mod" ] || node --experimental-strip-types "$(dirname "$0")/gen-receipt.ts" "$f" "$mod" >/dev/null
  log="$OUT/receipts/${name}.kernel.log"
  tv="$OUT/receipts/${name}.kernel.time"
  ( cd "$PKG" && /usr/bin/time -v -o "$tv" timeout "$BUDGET" lake env lean "$mod" ) >"$log" 2>&1
  rc=$?
  rss=$(sed -n 's/.*Maximum resident set size (kbytes): //p' "$tv" 2>/dev/null | head -1)
  wall=$(sed -n 's/.*Elapsed (wall clock) time (h:mm:ss or m:ss): //p' "$tv" 2>/dev/null | head -1)
  wall_s=$(python3 -c "
p='$wall'.split(':')
print(sum(float(x)*60**i for i,x in enumerate(reversed(p))) if p!=[''] else 'null')" 2>/dev/null || echo null)
  case "$rc" in
    0)   emit "$name" "$bytes" kernel 1 "\"outcome\":\"accept\",\"t_wall_s\":$wall_s,\"maxrss_kb\":${rss:-null}" ;;
    124) emit "$name" "$bytes" kernel 1 "\"outcome\":\"timeout\",\"t_wall_s\":null,\"maxrss_kb\":${rss:-null}" ;;
    *)   if [ "$rc" -eq 137 ] || grep -qi "out of memory\|std::bad_alloc\|Killed\|terminated by signal 9" "$log" "$tv" 2>/dev/null; then oc=oom; else oc=error; fi
         emit "$name" "$bytes" kernel 1 "\"outcome\":\"$oc\",\"rc\":$rc,\"t_wall_s\":${wall_s:-null},\"maxrss_kb\":${rss:-null}" ;;
  esac
  echo "  kernel: rc=$rc wall=${wall:-?} rss=${rss:-?}kB"
done

echo "receipts: $JL ($(wc -l <"$JL") rows)"
