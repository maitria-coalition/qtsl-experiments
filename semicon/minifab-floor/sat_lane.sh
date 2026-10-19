#!/usr/bin/env bash
# SAT lane for one N: CNF of model R at H = T_opt-1 (must be UNSAT, with an
# LRAT proof checked by cake_lpr (CakeML-verified) and lrat-check, and a
# DRAT proof checked by drat-trim) and at H = T_opt (must be SAT, witness
# decoded and re-validated). Usage: sat_lane.sh N OUTDIR [--fifo]
#   --fifo: add queue-order clauses (sound by the exchange lemma, README §R).
# Proof retention: the LRAT and DRAT files are large (N = 7: ~1 GB) and are
# not kept; the receipt records the sha256 of the CNF, the LRAT and the
# DRAT. The CNF hash is reproducible anywhere; the LRAT/DRAT hashes are
# reproducible with the SAME CaDiCaL build (pinned rev, same compiler and
# flags; single-threaded), and may differ across builds.
set -euo pipefail
case ${1:-} in ''|-h|--help) echo "usage: sat_lane.sh N OUTDIR [--fifo]" >&2; exit 2;; esac
N=$1; OUT=$2; FIFO=${3:-}; HERE=$(cd "$(dirname "$0")" && pwd); TOOLS=${TOOLS:-$HERE/../../build/minifab-floor/tools}
[ -z "$FIFO" ] || [ "$FIFO" = --fifo ] || { echo "sat_lane.sh: unknown argument $FIFO" >&2; exit 2; }
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
CAD=$TOOLS/cadical/build/cadical; DT=$TOOLS/drat-trim/drat-trim; LC=$TOOLS/drat-trim/lrat-check; CAKE=$TOOLS/cake_lpr/cake_lpr
cd "$OUT"
case $N in 1|2) D=0;; 3) D=2;; *) D=4;; esac
TOPT=$((255*N+371+D)); U=u$N; S=s$N
python3 "$HERE/cnf_floor.py" $N $((TOPT-1)) $U.cnf $FIFO 2> $U.gen
python3 "$HERE/cnf_floor.py" $N $TOPT $S.cnf $FIFO 2> $S.gen
set +e
$CAD -q --lrat=true --binary=false $U.cnf $U.lrat > $U.solve; rc=$?
set -e; [ $rc -eq 20 ] || { echo "N=$N: expected UNSAT at $((TOPT-1)), cadical rc=$rc"; exit 1; }
$CAKE $U.cnf $U.lrat > $U.cake 2>&1 || true; grep -q '^s VERIFIED UNSAT' $U.cake
$LC $U.cnf $U.lrat > $U.lratcheck 2>&1 || true; grep -q 'c VERIFIED' $U.lratcheck
set +e
$CAD -q $U.cnf $U.drat > /dev/null; rc=$?   # binary DRAT, separate run, hint-free check
set -e; [ $rc -eq 20 ]
$DT $U.cnf $U.drat -t 50000 > $U.drattrim 2>&1 || true; tr '\r' '\n' < $U.drattrim > $U.drattrim.txt; grep -q '^s VERIFIED$' $U.drattrim.txt
set +e
$CAD -q $S.cnf > $S.sol; rc=$?
set -e; [ $rc -eq 10 ] || { echo "N=$N: expected SAT at $TOPT, rc=$rc"; exit 1; }
(cd "$HERE" && python3 decode_cnf_witness.py "$OUT/$S.cnf.map.json" "$OUT/$S.sol") > $S.witness.json
printf '{"n":%d,"fifo":%s,"t_opt":%d,"unsat_at":%d,"cnf_sha256":"%s","lrat_sha256":"%s","drat_sha256":"%s","cake_lpr":"%s","lrat_check":"%s","drat_trim":"%s","sat_at":%d,"sat_cnf_sha256":"%s","witness_last_arrival":%s}\n' \
  $N "$([ -n "$FIFO" ] && echo true || echo false)" $TOPT $((TOPT-1)) "$(sha256sum $U.cnf|cut -c1-64)" "$(sha256sum $U.lrat|cut -c1-64)" "$(sha256sum $U.drat|cut -c1-64)" \
  "$(grep '^s ' $U.cake)" "$(grep 'c VERIFIED' $U.lratcheck|head -1)" "$(tr '\r' '\n' < $U.drattrim | grep '^s ')" \
  $TOPT "$(sha256sum $S.cnf|cut -c1-64)" "$(python3 -c "import json;print(json.load(open('$S.witness.json'))['last_arrival'])")"
rm -f $U.drat  # large; re-derivable
