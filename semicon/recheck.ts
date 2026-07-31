// recheck.ts — re-check every stored semicon fluid-drift certificate against
// the independent Rust checker (fluidcheck), positives and mutants both.
//
//   positive sets: every non-mutant *.cbor under out/minifab/ and out/mimac7/
//                  must be ACCEPTED (exit 0);
//   mutant sets:   every *.cbor under out/minifab/mutants/ and
//                  out/mimac7/mutants/ must be REFUSED (exit 1).
//
// Trust posture: this driver is UNTRUSTED orchestration — it only walks the
// committed certificate tree, invokes the checker, and tabulates verdicts.
// The checker binary is the instrument; producer-side receipts
// (producer_receipts_fluid.json / producer_receipts_cover.json) are
// producer artefacts and are not rewritten here.
//
// Contract (mirrors the retired python producers' run_checker):
//   fluidcheck <cert.cbor> [--bench N]; exit 0 = ACCEPT, 1 = REFUSE;
//   one verdict line on stdout (ACCEPT modes=... / REFUSE locus=... detail=...).
// env FLUIDCHECK overrides the checker path
// (default <repo>/target/release/fluidcheck).
// Exit: 0 iff every positive ACCEPTs and every mutant REFUSEs.
//
// Run: node --experimental-strip-types semicon/recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const OUT = path.join(HERE, "out");
const REPO = path.resolve(HERE, "..");
const FLUIDCHECK = process.env.FLUIDCHECK ||
  path.join(REPO, "target", "release", "fluidcheck");

const SETS = ["minifab", "mimac7"];

function runChecker(certPath: string): [number, string] {
  const p = spawnSync(FLUIDCHECK, [certPath], { encoding: "utf8" });
  if (p.error) throw p.error;
  return [p.status ?? -1, p.stdout.trim()];
}

function cborsIn(dir: string): string[] {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir).filter((f) => f.endsWith(".cbor")).sort()
    .map((f) => path.join(dir, f));
}

function main(): number {
  let ok = true;
  let nPos = 0;
  let nMut = 0;

  for (const set of SETS) {
    const dir = path.join(OUT, set);
    if (!fs.existsSync(dir)) {
      console.log(`MISSING SET: ${dir}`);
      ok = false;
      continue;
    }

    for (const certPath of cborsIn(dir)) {
      const rel = path.relative(OUT, certPath);
      const [rc, line] = runChecker(certPath);
      nPos += 1;
      if (rc === 0) {
        console.log(`ACCEPT ${rel}: ${line}`);
      } else {
        console.log(`UNEXPECTED REFUSE: ${rel}: ${line}`);
        ok = false;
      }
    }

    for (const certPath of cborsIn(path.join(dir, "mutants"))) {
      const rel = path.relative(OUT, certPath);
      const [rc, line] = runChecker(certPath);
      nMut += 1;
      if (rc === 1) {
        console.log(`REFUSE  ${rel}: ok (${line})`);
      } else {
        console.log(`FAILED-TO-REFUSE: ${rel}: rc=${rc} (${line})`);
        ok = false;
      }
    }
  }

  console.log(`\nsemicon recheck: ${nPos} positives + ${nMut} mutants — ` +
    (ok ? "ALL VERDICTS AS EXPECTED" : "WRONG VERDICTS PRESENT"));
  return ok ? 0 : 1;
}

process.exit(main());
