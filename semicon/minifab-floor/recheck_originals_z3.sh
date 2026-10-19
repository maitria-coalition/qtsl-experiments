#!/usr/bin/env bash
# Rerun the original z3 instrument minifab_idt.py (originals/, verbatim) for N=1..7
# at T_opt-1 and T_opt, plus its family table and mutant battery; write a
# receipt with tool versions and output hashes. The verbatim instrument exits 0
# even when it prints `unknown` or `FAIL`, so this wrapper PARSES the receipt
# and fails unless every expected row is present exactly once with the expected
# verdict (check_receipt below). A self-test feeds the checker a doctored
# receipt and requires a rejection. The leg reproduces the archive's result
# about a stricter model; it is not part of the net lower-bound argument.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd); P=$HERE/originals/maitria/w6xx-receipts/w676-idt/minifab_idt.py
OUT=${1:-$HERE/receipts/originals-z3.txt}
check_receipt() { python3 - "$1" <<'PY'
import sys, re
lines = [l.rstrip("\n") for l in open(sys.argv[1]) if not l.startswith("#")]
t_opt = lambda n: 255 * n + 371 + (0 if n <= 2 else 2 if n == 3 else 4)
want = []
for n in range(1, 8):
    t = t_opt(n)
    want += [f"N={n} T<={t-1}: unsat", f"N={n} T<={t}: sat", f"  witness validated, makespan={t}"]
for n in (1, 2, 3, 4, 5, 6, 7, 8, 12, 30, 84):
    d = t_opt(n) - 255 * n - 371
    want.append(f"family N={n}: makespan={t_opt(n)} = 255N+371+{d}")
# the exact, named mutant battery: each row must appear exactly once, verbatim
want += [
    "M2 t5=254 N=1: opt=625? sat@625=sat unsat@624=unsat -> PASS",
    "M2 t5=254 N=2: opt=879? sat@879=sat unsat@878=unsat -> PASS",
    "M2 t5=254 N=3: opt=1135? sat@1135=sat unsat@1134=unsat -> PASS",
    "M3 no-tick5-meter N=2 sat@880=sat -> PASS (meter law is load-bearing)",
    "M6 no-machines N=1 unsat@625=unsat -> PASS (B0 rides meters+chains)",
    "M6 no-machines N=2 unsat@880=unsat -> PASS (B0 rides meters+chains)",
]
errs = [f"missing or duplicated: {w!r}" for w in want if lines.count(w) != 1]
errs += [f"bad verdict: {l!r}" for l in lines if re.search(r"\b(unknown|FAIL)\b", l)]
errs += [f"unexpected line: {l!r}" for l in lines if l not in want]
if errs: print("ORIGINALS RECEIPT REJECTED: " + "; ".join(errs[:5])); sys.exit(1)
print(f"originals receipt ok: all {len(want)} expected rows (incl. the 6 named mutants PASS), each exactly once, nothing else")
PY
}
{
  echo "# minifab_idt.py sha256 $(sha256sum "$P" | cut -c1-64)"
  echo "# $(python3 -c 'import z3;print("z3",z3.get_version_string())') python $(python3 -c 'import sys;print(sys.version.split()[0])')"
  for N in 1 2 3 4 5 6 7; do
    case $N in 1|2) D=0;; 3) D=2;; *) D=4;; esac; T=$((255*N+371+D))
    python3 "$P" search $N $((T-1)); python3 "$P" search $N $T
  done
  python3 "$P" familytable 1 2 3 4 5 6 7 8 12 30 84
  python3 "$P" mutants
} > "$OUT.tmp"
check_receipt "$OUT.tmp" || { mv "$OUT.tmp" "$OUT.failed"; exit 1; }
# self-test: the checker must reject a doctored receipt
TMPF=$(mktemp); trap 'rm -f "$TMPF"' EXIT
sed '0,/: sat$/s//: unknown/' "$OUT.tmp" > "$TMPF"
if check_receipt "$TMPF" >/dev/null; then echo "self-test FAILED: checker accepted an 'unknown' row"; exit 1; fi
sed 's/-> PASS/-> FAIL/' "$OUT.tmp" > "$TMPF"
if check_receipt "$TMPF" >/dev/null; then echo "self-test FAILED: checker accepted FAIL mutants"; exit 1; fi
# six copies of one mutant row (an invented / substituted battery) must be rejected
{ grep -v '^M' "$OUT.tmp"; for i in 1 2 3 4 5 6; do grep -m1 '^M' "$OUT.tmp"; done; } > "$TMPF"
if check_receipt "$TMPF" >/dev/null; then echo "self-test FAILED: checker accepted a duplicated mutant battery"; exit 1; fi
{ grep -v '^M6' "$OUT.tmp"; echo "M7 invented N=1 -> PASS"; echo "M7 invented N=2 -> PASS"; } > "$TMPF"
if check_receipt "$TMPF" >/dev/null; then echo "self-test FAILED: checker accepted substituted mutants"; exit 1; fi
echo "checker self-test: doctored receipts (unknown row; FAIL mutants; duplicated battery; substituted mutants) rejected"
mv "$OUT.tmp" "$OUT"
echo "receipt $OUT sha256 $(sha256sum "$OUT" | cut -c1-64)"
