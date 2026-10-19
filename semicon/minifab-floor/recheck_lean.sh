#!/usr/bin/env bash
# Build the vendored Lean originals on their pinned toolchain + mathlib,
# audit axioms, and rerun the four original mutants M-A..M-D (each must
# make FloorContiguity.lean fail to compile, at the named lines, for a
# proof reason). Writes receipts/lean.txt. FAILS (nonzero exit, receipt
# moved to lean.txt.failed) unless ALL of these hold:
#   * the build succeeds and the sorry count is 0;
#   * the axiom-audit command exits 0, prints each of the six expected
#     theorems exactly once, each with axioms within {propext,
#     Classical.choice, Quot.sound}, and prints nothing else (no sorryAx,
#     no error line);
#   * the audit checker itself rejects two broken audits (negative tests:
#     a failing audit command, and an audit that includes a sorry-backed
#     theorem);
#   * an unmutated control copy of FloorContiguity.lean compiles, and each
#     mutant fails only with errors inside FloorContiguity.lean at exactly
#     the expected lines (so an environment/import failure is never
#     counted as a refusal).
# Mutants run on temporary copies; the vendored source is never edited.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd); L=$HERE/lean; OUT=${1:-$HERE/receipts/lean.txt}
MUT=$HERE/originals/maitria/w6xx-receipts/w676-contiguity
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT INT TERM
cd "$L"

# audit_check STATUS OUTFILE: the positive check of one axiom-audit run.
audit_check() {
  python3 - "$1" "$2" <<'PY'
import sys, re
status, text = int(sys.argv[1]), open(sys.argv[2]).read()
want = ["BernsteinLogic.MiniFab.MFSchedule.floor_contiguity",
        "BernsteinLogic.MiniFab.MFSchedule.floor_contiguity_three",
        "BernsteinLogic.MiniFab.MFSchedule.anchor_four",
        "BernsteinLogic.MiniFab.MFSchedule.anchor_seven",
        "BernsteinLogic.IDT.isMWERChoice_makespan_iff",
        "BernsteinLogic.IDT.wregret_makespan"]
ok_axioms = {"propext", "Classical.choice", "Quot.sound"}
errs, seen = [], {}
if status != 0: errs.append(f"audit command exited {status}")
for line in text.splitlines():
    if not line.strip(): continue
    m = re.fullmatch(r"'([^']+)' depends on axioms: \[(.*)\]", line.strip())
    if not m: errs.append(f"unexpected audit line: {line.strip()[:120]}"); continue
    ax = {a.strip() for a in m.group(2).split(",") if a.strip()}
    seen[m.group(1)] = seen.get(m.group(1), 0) + 1
    if not ax <= ok_axioms: errs.append(f"{m.group(1)}: axioms {sorted(ax - ok_axioms)} not permitted")
for w in want:
    if seen.get(w, 0) != 1: errs.append(f"{w}: printed {seen.get(w, 0)} times, want 1")
for k in seen:
    if k not in want: errs.append(f"{k}: not an expected theorem")
if errs:
    print("AUDIT REJECTED: " + "; ".join(errs)); sys.exit(1)
print("AUDIT ok: 6/6 expected theorems, axioms within {propext, Classical.choice, Quot.sound}")
PY
}

# expected refusal lines per mutant (original receipts; README Findings 3)
declare -A WANT=([mutant-A-cap3.patch]="343 452" [mutant-B-t5-254.patch]="145 212" \
                 [mutant-C-drop-cap55.patch]="345 455" [mutant-D-drop-tick1-meter.patch]="184")

set +e  # errexit is ignored inside an if-condition, so run the subshell plainly
(
  set -e
  echo "# toolchain $(cat lean-toolchain) :: $(lean --version)"
  echo "# mathlib rev $(python3 -c "import json;print([p['rev'] for p in json.load(open('lake-manifest.json'))['packages'] if p['name']=='mathlib'][0])")"
  echo "# sources sha256"; sha256sum IDT/*.lean BernsteinLogicAux/Imprecise/Credal.lean
  SORRY=$(cat IDT/*.lean BernsteinLogicAux/Imprecise/Credal.lean | grep -cw sorry || true)
  echo "# sorry count: $SORRY"
  [ "$SORRY" = 0 ] || { echo "FAIL: sorry count $SORRY"; exit 1; }
  lake exe cache get >/dev/null 2>&1 || echo "# (mathlib cache fetch unavailable; lake build compiles from source)"
  if lake build >"$TMP/build.log" 2>&1; then echo "BUILD ok"; else echo "BUILD FAILED"; tail -20 "$TMP/build.log"; exit 1; fi

  echo "# axiom audit"
  set +e; lake env lean "${AXIOM_AUDIT_FILE:-AxiomAudit.lean}" >"$TMP/audit.out" 2>&1; st=$?; set -e
  cat "$TMP/audit.out"
  audit_check "$st" "$TMP/audit.out"

  echo "# audit-checker negative tests (each must be REJECTED)"
  : >"$TMP/empty.out"
  if audit_check 1 "$TMP/empty.out" >"$TMP/neg1"; then echo "FAIL: checker accepted a failing audit command"; exit 1; fi
  echo "neg 1 (audit command fails, no output): $(cut -c1-100 "$TMP/neg1")"
  { cat AxiomAudit.lean; printf '\ntheorem audit_negative_probe : False := sorry\n#print axioms audit_negative_probe\n'; } >"$TMP/BrokenAudit.lean"
  set +e; lake env lean "$TMP/BrokenAudit.lean" >"$TMP/broken.out" 2>&1; st=$?; set -e
  grep -q sorryAx "$TMP/broken.out" || { echo "FAIL: negative probe did not exercise sorryAx"; exit 1; }
  if audit_check "$st" "$TMP/broken.out" >"$TMP/neg2"; then echo "FAIL: checker accepted a sorry-backed audit"; exit 1; fi
  echo "neg 2 (sorry-backed theorem added to the audit): REJECTED ($(grep -o "audit_negative_probe: axioms \[[^]]*\]" "$TMP/neg2" | head -1))"

  echo "# mutants (temporary copies; vendored source untouched)"
  mkdir -p "$TMP/ctl"; cp IDT/FloorContiguity.lean "$TMP/ctl/FloorContiguity.lean"
  lake env lean "$TMP/ctl/FloorContiguity.lean" >"$TMP/ctl.log" 2>&1 \
    || { echo "FAIL: unmutated control copy does not compile (environment problem)"; tail -5 "$TMP/ctl.log"; exit 1; }
  echo "control (unmutated copy): COMPILES"
  for p in "$MUT"/mutant-*.patch; do
    b=$(basename "$p"); d="$TMP/$b"; mkdir -p "$d"; cp IDT/FloorContiguity.lean "$d/FloorContiguity.lean"
    # explicit target file: the archive-relative a/... headers inside the vendored patches are ignored
    patch -s "$d/FloorContiguity.lean" < "$p" || { echo "FAIL: $b did not apply"; exit 1; }
    if lake env lean "$d/FloorContiguity.lean" >"$d/log" 2>&1; then echo "FAIL: $b COMPILED (mutant NOT refused)"; exit 1; fi
    lines=$(grep -o 'FloorContiguity.lean:[0-9]*:[0-9]*: error' "$d/log" | cut -d: -f2 | sort -nu | tr '\n' ' ' | sed 's/ $//' || true)
    nerr=$(grep -c ': error' "$d/log" || true); nfc=$(grep -c 'FloorContiguity.lean:[0-9]*:[0-9]*: error' "$d/log" || true)
    [ "$nerr" = "$nfc" ] || { echo "FAIL: $b: $((nerr-nfc)) error(s) outside FloorContiguity.lean (environment?)"; exit 1; }
    [ "$lines" = "${WANT[$b]}" ] || { echo "FAIL: $b refused at lines [$lines], expected [${WANT[$b]}]"; exit 1; }
    echo "mutant $b: REFUSED: $nerr error(s) at lines $lines (as expected)"
  done
  echo "# vendored source sha256 $(sha256sum IDT/FloorContiguity.lean | cut -c1-64)"
  echo "LEAN LEG OK"
) > "$OUT.tmp"
rc=$?; set -e
if [ $rc -ne 0 ] || [ "$(tail -1 "$OUT.tmp")" != "LEAN LEG OK" ]; then
  cat "$OUT.tmp"; mv "$OUT.tmp" "$OUT.failed"; echo "LEAN LEG FAILED (rc=$rc; receipt kept at $OUT.failed)" >&2; exit 1
fi
mv "$OUT.tmp" "$OUT"
echo "receipt $OUT sha256 $(sha256sum "$OUT" | cut -c1-64)"
