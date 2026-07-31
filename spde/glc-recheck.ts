// glc-recheck.ts — re-verify the SPDE moment certificates from their
// sealed containers (out/glc/*.glc) through the geolog fast-path
// checker, positives and mutants both, across the family's SPLIT
// CONTAINER design:
//
//   witness containers (the manifest's `positives`) — the moment core
//     on the synthetic reproduction vocabulary, kept checkable forever
//     (versioning law): the noise spectrum (driver rows + a
//     constructor-required tail form), the moment claims with their
//     Itô-correction node, the trajectory witnesses, the summed-floor
//     enclosures. The checker re-derives every claim through the
//     exact-rational ground provider: rates, mode floors, the
//     covariance trace, and every exponential bracket are RECOMPUTED
//     from the declared spectrum — derived quantities are never stored
//     in the artifact, so forging one is unrepresentable.
//
//   spectrum containers (the manifest's `spectra`) — per certificate,
//     the noise spectrum re-homed onto the baked qc closure as its own
//     container (`glc-check --baked-cert`; a mixed prerequisite table
//     is refused by checker design — mutant sp5 demonstrates it). The
//     claim surface is amplitude well-formedness (qtsl_poly_canonical
//     per driver amplitude); the spectrum LAW stays checked on the
//     witness side plus the production-time derived-spectrum gate, and
//     the manifest's cross-tie records each spectrum container's
//     witness commit address. STATED LIMITATION, by design and
//     documented rather than hidden: no independent checker binds
//     spectrum content to the moment core today — the battery carries
//     a designed specimen (sp3_coeff_forged, expected ACCEPT) that
//     witnesses exactly this boundary, and the production gate's
//     refusal of the same forged content is witnessed in the
//     producer's own test suite. A baked spectrum-law side condition
//     is the named successor (it requires a new theory revision).
//
//   positives/spectra: must ACCEPT (exit 0) with the manifest's exact
//              per-relation claim counts;
//   mutants:   land in the manifest's expected class — REFUSE
//              (integrity / prerequisite), FAIL (claim discharge or
//              count), or ACCEPT (sp3, the documented-limitation
//              specimen above).
//
// The count gate (--expect-claims) is load-bearing: whole-block
// deletion leaves every surviving claim true (mutant m7 witnesses
// it), and the m3 mutant witnesses the family's falsifier — an
// internally-coherent forgery through the correction constant, the
// beta slot, and the floor row survives every claim EXCEPT the
// summed-floor tie, which pins the floor against checker-rederived
// mode-law content.
//
// Trust posture: this driver is UNTRUSTED orchestration — it walks
// the committed artifacts, invokes the checker, and tabulates
// verdicts. The manifest states the producer's claims; the checker
// binary is the instrument that verifies the bytes against them.
//
// Contract: glc-check [--baked-cert] [--expect-claims name=N,...]
// <file.glc>; exit 0 = all ACCEPT; one verdict line per file on
// stdout. env GLC_CHECK names the checker binary (built from the
// pinned qtsl checkout: cargo build --release --bin glc-check in
// crates/qtsl-geolog-provider); there is no default — an unset path
// is an honest refusal, never a silent skip.
//
// Run: GLC_CHECK=... node --experimental-strip-types spde/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const GLCDIR = path.join(HERE, "out", "glc");
const MANIFEST = path.join(GLCDIR, "GLC-MANIFEST.json");

const GLC_CHECK = process.env.GLC_CHECK;
if (!GLC_CHECK) {
  console.error(
    "glc-recheck: GLC_CHECK is not set. Build the checker from the pinned qtsl " +
      "checkout (cargo build --release --bin glc-check in crates/qtsl-geolog-provider) " +
      "and pass its path: GLC_CHECK=/path/to/glc-check node --experimental-strip-types " +
      "spde/glc-recheck.ts",
  );
  process.exit(2);
}

function runChecker(certPath: string, expect: string, baked: boolean): [number, string] {
  const args = [...(baked ? ["--baked-cert"] : []), "--expect-claims", expect, certPath];
  const p = spawnSync(GLC_CHECK, args, { encoding: "utf8" });
  if (p.error) throw p.error;
  return [p.status ?? -1, p.stdout.trim()];
}

const manifest = JSON.parse(fs.readFileSync(MANIFEST, "utf8"));
let bad = 0;

console.log("== witness positives (synthetic lane; expect ACCEPT, exact claim counts) ==");
for (const pos of manifest.positives) {
  const file = path.join(GLCDIR, pos.file);
  const [status, line] = runChecker(file, pos.expect_claims, false);
  const ok = status === 0 && line.includes(" ACCEPT ");
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} ${line}`);
}

console.log("== spectrum containers (baked lane; expect ACCEPT, exact claim counts) ==");
for (const spec of manifest.spectra ?? []) {
  const file = path.join(GLCDIR, spec.file);
  const [status, line] = runChecker(file, spec.expect_claims, true);
  const ok = status === 0 && line.includes(" ACCEPT ");
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} ${line}`);
}

console.log("== mutants (expect the manifest's class, per lane) ==");
const spectrumBases = new Set(
  (manifest.spectra ?? []).map((s: { file: string }) => s.file.replace(/\.glc$/, "")),
);
for (const mut of manifest.mutants) {
  const file = path.join(GLCDIR, mut.file);
  const baked = spectrumBases.has(mut.base);
  const [status, line] = runChecker(file, mut.expect_claims, baked);
  let ok: boolean;
  if (mut.expected === "ACCEPT") {
    // sp3_coeff_forged: the documented-limitation specimen (header) —
    // in-lane ACCEPT by design; the production gate refuses the same
    // content at emission time.
    ok = status === 0 && line.includes(" ACCEPT ");
  } else {
    const classOk =
      mut.expected === "REFUSE" ? line.includes(" REFUSE ") : line.includes(" FAIL ");
    ok = status !== 0 && classOk;
  }
  if (!ok) bad++;
  console.log(`${ok ? "ok " : "BAD"} [want ${mut.expected}] ${line.slice(0, 160)}`);
}

if (bad > 0) {
  console.error(`glc-recheck: ${bad} verdict(s) off expectation`);
  process.exit(1);
}
console.log("glc-recheck: all verdicts as expected");
