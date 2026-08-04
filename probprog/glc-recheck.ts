// glc-recheck.ts — re-verify the probprog certificates from their
// sealed containers (out/glc/*.glc) through the geolog fast-path
// checker, positives and mutants both.
//
// BAKED LANE: the certificate species (absorption brackets for
// stored-row chains, and the contraction/EV witness family) sealed
// at theory revision event 8 — the dac closure (BernsteinLogic +
// Qtsl + Dtmc + DtmcAbsorb) and the dcc closure (dac +
// DtmcContraction) are in the checker's baked roster. The
// certificates' prerequisite tables name those revisions and the
// checker resolves them by roster, no flags. (Historical note, as
// the staged-lane paragraph that stood here promised: before the
// event sealed, every check rode --staged-chain against the staged
// closure extract; the baked dac revision is byte-identical to the
// staged one, so the committed containers crossed the event without
// re-emission — the flip was mechanical.)
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
// Contract: glc-check [--expect-claims name=N,...] <file.glc>;
// exit 0 = ACCEPT; one verdict line per file on stdout. env
// GLC_CHECK names the checker binary (cargo build --release --bin
// glc-check in crates/qtsl-geolog-provider of the pinned qtsl
// checkout). Unset is an honest refusal, never a skip.
//
// Run: GLC_CHECK=... node --experimental-strip-types
//      probprog/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const GLCDIR = path.join(HERE, "out", "glc");
const MANIFEST = path.join(GLCDIR, "GLC-MANIFEST.json");

const GLC_CHECK = process.env.GLC_CHECK;
if (!GLC_CHECK) {
  console.error(
    "glc-recheck: GLC_CHECK must be set (checker binary from the pinned " +
      "qtsl checkout). Unset is a refusal, never a silent skip.",
  );
  process.exit(2);
}

function runChecker(certPath: string, expect: string | null): [number, string] {
  const args = [
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
  `== positives (baked closures: ${manifest.closure ?? "see manifest"}) ==`,
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
