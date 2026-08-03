// ev-recheck.ts — re-verify the probprog EV-bracket certificates
// (out/ev/*.evc) through the dev-oracle checker, positives and
// mutants both.
//
// DEV-ORACLE LANE, stated loudly: the EV rows (weighted transfer
// contraction + geometric template tail + moment assembly) belong to
// the draft strengthened generator-form seam, staged for a future
// theory revision event and in no sealed closure yet. Until that
// revision seals, `probprog-evcheck` (built from this workspace) is
// the family's dev oracle for those rows — exact-rational,
// recomputing every quantity from the stored chain rows named by the
// certificate's spec-sha256 binding — and every verdict this driver
// tabulates is labeled pending that revision in the comparison
// receipts. The absorption rows of the SAME chains ride the fast-path
// checker (glc-recheck.ts); the shared spec bytes are the bridge.
//
//   positives: must ACCEPT (exit 0) with the manifest's claim counts;
//   mutants:   land in the manifest's expected class — REFUSE (shape,
//              side condition, or binding), FAIL (claim discharge), or
//              ACCEPT (positive controls).
//
// Trust posture: this driver is UNTRUSTED orchestration — it walks
// the committed artifacts, invokes the checker, and tabulates
// verdicts. The checker binary is the instrument.
//
// Contract: probprog-evcheck --spec <spec.json> <cert.evc>; exit 0 =
// ACCEPT, 3 = FAIL, 4 = REFUSE; one verdict line on stdout. env
// EVCHECK names the checker binary (cargo build --release -p
// probprog-producer). Unset is an honest refusal, never a skip.
//
// Run: EVCHECK=... node --experimental-strip-types probprog/ev-recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const EVDIR = path.join(HERE, "out", "ev");
const MANIFEST = path.join(EVDIR, "EV-MANIFEST.json");

const EVCHECK = process.env.EVCHECK;
if (!EVCHECK) {
  console.error(
    "ev-recheck: EVCHECK must be set (the probprog-evcheck binary, " +
      "cargo build --release -p probprog-producer). Unset is a refusal, " +
      "never a silent skip.",
  );
  process.exit(2);
}

function runChecker(certFile: string, specFile: string): [number, string] {
  const p = spawnSync(
    EVCHECK!,
    ["--spec", path.join(EVDIR, specFile), path.join(EVDIR, certFile)],
    { encoding: "utf8" },
  );
  if (p.error) throw p.error;
  return [p.status ?? -1, (p.stdout + p.stderr).trim()];
}

const manifest = JSON.parse(fs.readFileSync(MANIFEST, "utf8"));
let bad = 0;

console.log("== positives (dev-oracle lane: see manifest.lane) ==");
for (const pos of manifest.positives) {
  const [status, line] = runChecker(pos.cert, pos.spec);
  const ok =
    status === 0 &&
    line.includes(" ACCEPT ") &&
    (!pos.expect || line.includes(pos.expect));
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} ${line.slice(0, 200)}`);
}

console.log("== mutants (expect the manifest's class) ==");
for (const mut of manifest.mutants ?? []) {
  const [status, line] = runChecker(mut.cert, mut.spec);
  let ok: boolean;
  if (mut.expected === "ACCEPT") {
    ok = status === 0 && line.includes(" ACCEPT ");
  } else {
    const classOk =
      mut.expected === "REFUSE"
        ? status === 4 && line.includes("REFUSE")
        : status === 3 && line.includes("FAIL");
    ok = classOk;
  }
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} [want ${mut.expected}] ${line.slice(0, 200)}`);
}

if (bad > 0) {
  console.error(`ev-recheck: ${bad} verdict(s) off expectation`);
  process.exit(1);
}
console.log("ev-recheck: all verdicts as expected");
