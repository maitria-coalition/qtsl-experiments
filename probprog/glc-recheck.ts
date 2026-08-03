// glc-recheck.ts — re-verify the probprog absorption-bracket
// certificates from their sealed containers (out/glc/*.glc) through
// the geolog fast-path checker, positives and mutants both.
//
// STAGED-CLOSURE LANE, stated loudly: the certificate species
// (absorption brackets for stored-row chains) is staged for the next
// theory revision event and is in no baked closure yet. Every check
// therefore rides the checker's --staged-chain lane: the closure chain
// container (sealed from the staged theory set, double-mint
// determinism verified at production) is loaded explicitly, and the
// certificates' prerequisite tables name that staged revision. When
// the revision event seals and the QTSL_REV pin advances past it, the
// containers re-emit against the baked closure and this driver drops
// the STAGED_CHAIN requirement — tracked in the manifest's `closure`
// field so the flip is mechanical.
//
//   positives: must ACCEPT (exit 0) with the manifest's exact
//              per-relation claim counts;
//   mutants:   land in the manifest's expected class — REFUSE
//              (integrity / prerequisite / no-evidence), FAIL (claim
//              discharge), or ACCEPT (positive controls).
//
// Trust posture: this driver is UNTRUSTED orchestration — it walks
// the committed artifacts, invokes the checker, and tabulates
// verdicts. The checker binary is the instrument.
//
// Contract: glc-check --staged-chain <chain.glc> --staged-cert
// [--expect-claims name=N,...] <file.glc>; exit 0 = ACCEPT; one
// verdict line per file on stdout. env GLC_CHECK names the checker
// binary (built from the qtsl checkout that carries the staged lane:
// cargo build --release --bin glc-check in
// crates/qtsl-geolog-provider); env STAGED_CHAIN names the closure
// chain container. Both unset is an honest refusal, never a skip.
//
// Run: GLC_CHECK=... STAGED_CHAIN=... node --experimental-strip-types
//      probprog/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const GLCDIR = path.join(HERE, "out", "glc");
const MANIFEST = path.join(GLCDIR, "GLC-MANIFEST.json");

const GLC_CHECK = process.env.GLC_CHECK;
const STAGED_CHAIN = process.env.STAGED_CHAIN;
if (!GLC_CHECK || !STAGED_CHAIN) {
  console.error(
    "glc-recheck: GLC_CHECK and STAGED_CHAIN must both be set (checker binary " +
      "from the qtsl checkout carrying the staged lane; the staged closure " +
      "chain container). Unset is a refusal, never a silent skip.",
  );
  process.exit(2);
}

function runChecker(certPath: string, expect: string | null): [number, string] {
  const args = [
    "--staged-chain",
    STAGED_CHAIN!,
    "--staged-cert",
    ...(expect ? ["--expect-claims", expect] : []),
    certPath,
  ];
  const p = spawnSync(GLC_CHECK!, args, { encoding: "utf8" });
  if (p.error) throw p.error;
  return [p.status ?? -1, (p.stdout + p.stderr).trim()];
}

const manifest = JSON.parse(fs.readFileSync(MANIFEST, "utf8"));
let bad = 0;

console.log(
  `== positives (staged closure: ${manifest.closure ?? "see manifest"}) ==`,
);
for (const pos of manifest.positives) {
  const [status, line] = runChecker(
    path.join(GLCDIR, pos.file),
    pos.expect_claims,
  );
  const ok = status === 0 && line.includes(" ACCEPT ");
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} ${line.slice(0, 200)}`);
}

console.log("== mutants (expect the manifest's class) ==");
for (const mut of manifest.mutants ?? []) {
  const [status, line] = runChecker(
    path.join(GLCDIR, mut.file),
    mut.expect_claims ?? null,
  );
  let ok: boolean;
  if (mut.expected === "ACCEPT") {
    ok = status === 0 && line.includes(" ACCEPT ");
  } else {
    const classOk =
      mut.expected === "REFUSE"
        ? line.includes("REFUSE") || line.includes("refus")
        : line.includes("FAIL");
    ok = status !== 0 && classOk;
  }
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} [want ${mut.expected}] ${line.slice(0, 200)}`);
}

if (bad > 0) {
  console.error(`glc-recheck: ${bad} verdict(s) off expectation`);
  process.exit(1);
}
console.log("glc-recheck: all verdicts as expected");
