#!/usr/bin/env bash
# Mini-Fab optimum recheck: every leg of T_opt(N) = 255N + 371 + d(N).
#   MAX_N (default 7): largest N for the model-R legs (z3 and SAT/LRAT).
#   NOFIFO_MAX_N (default 5): largest N for the queue-order-free cross-check
#   lane (no FIFO constraint at all; slow beyond N = 5).
# Legs: (0) originals sha + completeness, tool-version policy;
# (1) model R, z3, two-sided, + the FIFO exchange-lemma check;
# (2) original z3 instrument rerun; (3) independent CNF + LRAT/DRAT checked
# by cake_lpr, lrat-check, drat-trim; (4) Lean build + axiom audit + mutants;
# (5) decoded-container replay (N = 84 sealed; pi* at N = 1..12 instance-adapted)
# + canonical-table recheck; (6) the queue-order-free cross-check lane. Receipts land in receipts/. Network use: only the
# pinned clones and the cargo/lake fetches.
# Requirements (x86-64 Linux; cake_lpr.S is the x86-64 build): bash, GNU
# coreutils (sha256sum), gcc, make, git, python3 with z3 bindings at exactly
# 4.16.0 (e.g. `pip install z3-solver==4.16.0.0`; Z3_VERSION_POLICY=any
# accepts another version and records it), cargo, elan/lake.
set -euo pipefail
case ${1:-} in -h|--help) sed -n 2,16p "$0"; exit 2;; '') ;; *) echo "recheck.sh: no arguments (use MAX_N=...)" >&2; exit 2;; esac
HERE=$(cd "$(dirname "$0")" && pwd); ROOT=$(cd "$HERE/../.." && pwd)
B=$ROOT/build/minifab-floor; T=$B/tools; MAX_N=${MAX_N:-7}; NOFIFO_MAX_N=${NOFIFO_MAX_N:-5}
LEGS=${LEGS:-0 1 2 3 4 5 6}   # e.g. LEGS="5 6" reruns only those legs (provenance is recorded)
# Receipts go to receipts/ only for the full-range run (MAX_N=7, NOFIFO_MAX_N>=5);
# a reduced quick run writes to build/ so it can never overwrite committed receipts.
if [ -n "${RECEIPTS_DIR:-}" ]; then R=$RECEIPTS_DIR
elif [ "$MAX_N" -ge 7 ] && [ "$NOFIFO_MAX_N" -ge 5 ]; then R=$HERE/receipts
else R=$B/receipts-quick; echo "NOTE: reduced run (MAX_N=$MAX_N, NOFIFO_MAX_N=$NOFIFO_MAX_N): receipts go to $R, not receipts/"; fi
JOBS=$(nproc 2>/dev/null || echo 4)
mkdir -p "$T" "$R"
pin() { # url dir rev: clone, check the remote, check out rev
  [ -d "$2/.git" ] || git clone -q "$1" "$2"
  [ "$(git -C "$2" remote get-url origin)" = "$1" ] || { echo "$2: origin is not $1" >&2; exit 1; }
  git -C "$2" cat-file -e "$3^{commit}" 2>/dev/null || git -C "$2" fetch -q origin
  git -C "$2" checkout -q "$3"
}
check_z3() { python3 - "$1" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
for r in d["rows"]:
    n, t = r["n"], r["t_opt_claimed"]
    assert r[f"H={t-1}"] == "unsat" and r[f"H={t}"] == "sat" and r["witness_revalidated"], r["n"]
    print(f"R{' +fifo' if d['fifo'] else ' (no fifo)'} N={n}: unsat@{t-1} sat@{t} (witness re-validated)")
PY
}
# provenance: one dated row per leg run, appended to receipts/provenance.jsonl
prov() { printf '{"leg":%s,"finished_utc":"%s","commit":"%s","recheck_sh_sha256":"%s","max_n":%s,"nofifo_max_n":%s}\n' \
  "$1" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$(git -C "$HERE" rev-parse HEAD 2>/dev/null || echo unknown)" \
  "$(sha256sum "$HERE/recheck.sh" | cut -c1-64)" "$MAX_N" "$NOFIFO_MAX_N" >> "$R/provenance.jsonl"; }
leg0() {
echo "== (0) vendored originals: sha256sum -c ORIGINALS.sha256, and nothing unlisted; script lint"
# lint (regression for the SIGPIPE/141 abort): no pipeline may end in `grep -q` under pipefail
GQ='grep -q'; if grep -nE "(^|[^|])\\|[^|;&]*$GQ" "$HERE"/recheck.sh "$HERE"/sat_lane.sh "$HERE"/recheck_lean.sh "$HERE"/recheck_originals_z3.sh; then
  echo "lint: a pipeline ends in grep -q (exits early -> SIGPIPE -> rc 141 under pipefail)"; exit 1; fi
(cd "$HERE" && sha256sum -c --strict --quiet ORIGINALS.sha256 && echo "17 listed originals ok")
(cd "$HERE" && comm -23 <(find originals -type f | sort) <(awk '{print $2}' ORIGINALS.sha256 | sort)) > "$B/unlisted"
[ ! -s "$B/unlisted" ] || { echo "files under originals/ not in ORIGINALS.sha256:"; cat "$B/unlisted"; exit 1; }
ZV=$(python3 -c 'import z3;print(z3.get_version_string())')
if [ "$ZV" != 4.16.0 ]; then
  [ "${Z3_VERSION_POLICY:-pinned}" = any ] || { echo "z3 $ZV found; receipts are pinned to 4.16.0 (pip install z3-solver==4.16.0.0, or Z3_VERSION_POLICY=any)"; exit 1; }
  echo "WARNING: z3 $ZV, not the pinned 4.16.0 (Z3_VERSION_POLICY=any)"
fi
echo "z3 $ZV"
}
leg1() {
echo "== (1) model R, z3 two-sided search N=1..$MAX_N (queue order by the exchange lemma) + the lemma check"
python3 "$HERE/z3_floor.py" "$(seq -s, 1 "$MAX_N")" 0 --fifo > "$R/z3-R.json" 2> "$B/z3-R.log"; check_z3 "$R/z3-R.json"
(cd "$HERE" && python3 fifo_exchange_check.py) > "$R/fifo-exchange.json"; cat "$R/fifo-exchange.json"
(cd "$HERE" && python3 z3_floor.py --selftest)   # validator must reject malformed/backwards witnesses
}
leg2() {
echo "== (2) original z3 instrument rerun"
"$HERE/recheck_originals_z3.sh" "$R/originals-z3.txt"
}
leg3() {
echo "== (3) SAT lane: tools at pinned commits"
pin https://github.com/arminbiere/cadical "$T/cadical" c60730422e758ef1cebe7aeddf2dda31c996bf04
pin https://github.com/marijnheule/drat-trim "$T/drat-trim" 2e3b2dc0ecf938addbd779d42877b6ed69d9a985
pin https://github.com/tanyongkiam/cake_lpr "$T/cake_lpr" a36874a8b750b43fe4b385b8ddbf5b033e46a3fa
CREV=$(git -C "$T/cadical" rev-parse HEAD)
if [ "$(cat "$T/cadical/build/.built-rev" 2>/dev/null)" != "$CREV" ]; then  # rebuild unless built from this rev
  (cd "$T/cadical" && ./configure >/dev/null && make -j"$JOBS" >/dev/null && echo "$CREV" > build/.built-rev)
fi
(cd "$T/drat-trim" && gcc -O2 -D_GNU_SOURCE -o drat-trim drat-trim.c && gcc -O2 -D_GNU_SOURCE -o lrat-check lrat-check.c)
(cd "$T/cake_lpr" && grep -q "$(sha256sum cake_lpr.S | cut -c1-64)" cake_lpr.sha256 && gcc -O2 basis_ffi.c cake_lpr.S -o cake_lpr -std=c99 2>/dev/null)
: > "$R/sat-lane.jsonl"
for N in $(seq 1 "$MAX_N"); do
  TOOLS=$T "$HERE/sat_lane.sh" "$N" "$B/cnf/n$N" --fifo | tee -a "$R/sat-lane.jsonl"
  rm -f "$B/cnf/n$N"/*.lrat "$B/cnf/n$N"/*.cnf "$B/cnf/n$N"/*.sol
done
}
leg4() {
echo "== (4) Lean originals (fails unless build, audit, audit negatives and all four mutants are as expected)"
"$HERE/recheck_lean.sh" "$R/lean.txt"
grep -q '^LEAN LEG OK$' "$R/lean.txt"
}
leg5() {
echo "== (5) decoded-container attainment replay"
(cd "$HERE/glc-replay" && cargo test --release --locked -q > "$B/cargo-test.log" 2>&1) || { tail -20 "$B/cargo-test.log"; exit 1; }
grep -q 'test result: ok. 9 passed' "$B/cargo-test.log"   # grep the captured log: never pipe into grep -q under pipefail
echo "cargo test --release --locked: 9 passed (log: $B/cargo-test.log)"
(cd "$HERE/glc-replay" && cargo build --release --locked -q --bin glc-replay)
(cd "$ROOT" && "$HERE/glc-replay/target/release/glc-replay" semicon/minifab-dispatch/out/carved) > "$R/glc-replay.jsonl"
"$HERE/glc-replay/target/release/glc-replay" --dump-tables \
  | cmp - "$HERE/originals/maitria/w6xx-receipts/w676-idt-replay/rust-tables.txt"
echo "canonical pi*/WC table dump: byte-identical to the vendored original rust-tables.txt"
# regression instrument: a missing gated container must NOT count as its expected refusal
MISS=$(mktemp -d); cp "$ROOT"/semicon/minifab-dispatch/out/carved/{minifab-supervised,minifab}.glc "$MISS"/
if "$HERE/glc-replay/target/release/glc-replay" "$MISS" > "$MISS/out" 2>&1; then echo "replay accepted a missing gated container as a refusal"; exit 1; fi
grep -q 'WRONG REFUSAL .*minifab-gated.glc' "$MISS/out"; rm -rf "$MISS"
echo "negative: missing gated container -> WRONG REFUSAL, rc != 0 (as required)"
python3 - "$ROOT/semicon/minifab-dispatch/out/GLC-MANIFEST.json" "$R/glc-replay.jsonl" <<'PY'
import json, sys
man = {}
def walk(o):
    if isinstance(o, dict):
        if "file" in o and "glc_sha256" in o: man[o["file"]] = o["glc_sha256"]
        for v in o.values(): walk(v)
    elif isinstance(o, list):
        for v in o: walk(v)
walk(json.load(open(sys.argv[1])))
t_opt = lambda n: 255 * n + 371 + (0 if n <= 2 else 2 if n == 3 else 4)
inst, sealed, refused = {}, {}, {}
for line in open(sys.argv[2]):
    r = json.loads(line)
    if "sha256" in r:
        f = "carved/" + r["container"].rsplit("/", 1)[1]
        assert man[f] == r["sha256"], f"{f}: replayed bytes are not the manifest's"
        if r["label"] == "pi-star-instance":
            assert r["last_arrival_round"] == r["t_opt"] == t_opt(r["n"]), r
            assert r["instance_adaptation"]["pending_lots"] == [84, r["n"]], r
            assert r["instance_adaptation"]["unlock_threshold"] == [84, r["n"]], r
            inst[r["n"]] = r["last_arrival_round"]
        else:
            sealed[(f, r["label"])] = r["last_arrival_round"]
            print(f"{f} [{r['label']}] arrival {r['last_arrival_round']} firings {r['pre_goal_firings']} holds {r['holds_by_schema']}  (sha = manifest)")
    else:
        f = "carved/" + r["container"].rsplit("/", 1)[1]
        assert man[f] == r["refused_sha256"], f"{f}: refused bytes are not the manifest's"
        refused[(f, r["label"])] = r["refused_as_expected"]
        print(f"{f} [{r['label']}] refused as expected")
assert sealed == {("carved/minifab-supervised.glc", "pi-star"): 21795, ("carved/minifab-supervised.glc", "wc-family"): 21875,
                  ("carved/minifab.glc", "pi-star"): 21795, ("carved/minifab.glc", "wc-family"): 21875}, sealed
assert set(refused) == {("carved/minifab-supervised.glc", "pi-star-under-forced-law"), ("carved/minifab-gated.glc", "pi-star")}, refused
assert "hold outside" in refused[("carved/minifab-supervised.glc", "pi-star-under-forced-law")]
assert "native read arcs" in refused[("carved/minifab-gated.glc", "pi-star")]
print("sealed N=84: pi* 21795, wc 21875 on both lanes; refusals exactly {forced-law hold, native-read lane}")
assert sorted(inst) == list(range(1, 13)) and inst[10] == 2925, inst
print("pi* instance-adapted replay, N=1..12, arrival = T_opt:", inst)
PY
}
leg6() {
echo "== (6) cross-check lane: model R with NO queue-order constraint, N=1..$NOFIFO_MAX_N (z3, then CNF + LRAT/DRAT)"
python3 "$HERE/z3_floor.py" "$(seq -s, 1 "$NOFIFO_MAX_N")" 0 > "$R/z3-R-nofifo.json" 2> "$B/z3-R-nofifo.log"; check_z3 "$R/z3-R-nofifo.json"
: > "$R/sat-lane-nofifo.jsonl"
for N in $(seq 1 "$NOFIFO_MAX_N"); do
  TOOLS=$T "$HERE/sat_lane.sh" "$N" "$B/cnf-nofifo/n$N" | tee -a "$R/sat-lane-nofifo.jsonl"
  rm -f "$B/cnf-nofifo/n$N"/*.lrat "$B/cnf-nofifo/n$N"/*.cnf "$B/cnf-nofifo/n$N"/*.sol
done
}
for leg in 0 1 2 3 4 5 6; do
  case " $LEGS " in *" $leg "*) "leg$leg"; prov "$leg";; esac
done
echo "== legs [$LEGS] green; receipts in $R"
