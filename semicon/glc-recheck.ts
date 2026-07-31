// glc-recheck.ts — re-verify the semicon fluid-drift certificates from
// their sealed containers through the geolog fast-path checker,
// positives and mutants both, across every committed container lane.
//
// The sealed container (.glc) is the canonical binary form: signed
// geolog commit envelopes whose body carries every sign mode's
// obligation rows plus the per-mode side-condition claim rows. Two
// lanes are committed, and this driver discovers and checks both by
// walking every GLC-MANIFEST.json under semicon/out/, dispatching on
// the manifest's own shape:
//
//   baked lane (out/minifab/glc-baked/) — the reproduction route.
//     Containers whose prerequisite table pins the baked theory-pair
//     closure revision; the checker derives the vocabulary index from
//     the pinned chain's own bytes and discharges every claim row
//     through the exact-rational ground provider. Manifest positives
//     carry `expect_claims` (per-relation counts); invocation is
//     `glc-check --baked-cert --expect-claims name=N,...`. The baked
//     lane's claim roster is derived from the revision, so
//     `--expect-modes` refuses there by design; the per-relation count
//     gate is load-bearing (whole-mode deletion leaves every surviving
//     claim true — mutant m4 witnesses it).
//
//   legacy lane (out/minifab/glc/) — the original containers against
//     the synthetic v0 reproduction vocabulary, kept checkable
//     forever (versioning law: a committed lane never dies). Manifest
//     positives carry `modes` only; invocation is
//     `glc-check --expect-modes N`.
//
// Mutants inherit their base positive's expectation (the baked lane's
// per-relation string, or the legacy lane's base_modes) and must be
// rejected with the manifest's expected verdict class — REFUSE
// (integrity) or FAIL (claim discharge / count).
//
// Trust posture: this driver is UNTRUSTED orchestration — it walks the
// committed artifacts, invokes the checker, and tabulates verdicts.
// The manifest states the producer's claims; the checker binary is the
// instrument that verifies the bytes against them.
//
// Contract: glc-check [--baked-cert] [--expect-modes N |
// --expect-claims name=N,...] <file.glc>; exit 0 = all ACCEPT; one
// verdict line per file on stdout.
// env GLC_CHECK names the checker binary (built from the pinned qtsl
// checkout: cargo build --release --bin glc-check in
// crates/qtsl-geolog-provider); there is no default — an unset path is
// an honest refusal, never a silent skip.
//
// Run: GLC_CHECK=... node --experimental-strip-types semicon/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const OUTDIR = path.join(HERE, "out");

const GLC_CHECK = process.env.GLC_CHECK;
if (!GLC_CHECK) {
  console.error(
    "glc-recheck: GLC_CHECK is not set. Build the checker from the pinned qtsl " +
      "checkout (cargo build --release --bin glc-check in crates/qtsl-geolog-provider) " +
      "and pass its path: GLC_CHECK=/path/to/glc-check node --experimental-strip-types " +
      "semicon/glc-recheck.ts",
  );
  process.exit(2);
}

function findManifests(dir: string): string[] {
  const found: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) found.push(...findManifests(p));
    else if (entry.name === "GLC-MANIFEST.json") found.push(p);
  }
  return found.sort();
}

function runChecker(certPath: string, args: string[]): [number, string] {
  const p = spawnSync(GLC_CHECK, [...args, certPath], { encoding: "utf8" });
  if (p.error) throw p.error;
  return [p.status ?? -1, p.stdout.trim()];
}

// The expectation arguments for one manifest entry. Baked-lane
// positives state per-relation claim counts; legacy positives state a
// mode count. Mutants inherit from their named base positive.
function expectArgs(manifest: any, entry: any, baked: boolean): string[] {
  if (baked) {
    const claims =
      entry.expect_claims ??
      manifest.positives.find((p: any) => p.cert === entry.base)?.expect_claims;
    if (!claims) {
      throw new Error(
        `no expect_claims resolvable for ${entry.file} — baked-lane manifests ` +
          `must state per-relation counts (the count gate is load-bearing)`,
      );
    }
    return ["--baked-cert", "--expect-claims", claims];
  }
  return ["--expect-modes", String(entry.modes ?? entry.base_modes)];
}

function checkManifest(manifestPath: string): boolean {
  const dir = path.dirname(manifestPath);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  // Lane dispatch is data-driven: baked-lane manifests state
  // per-relation claim expectations on their positives.
  const baked = manifest.positives.every((p: any) => typeof p.expect_claims === "string");
  const lane = baked ? "baked-cert" : "expect-modes";
  console.log(`lane ${lane}: ${path.relative(HERE, manifestPath)}`);

  let ok = true;
  let nPos = 0;
  let nMut = 0;

  for (const pos of manifest.positives) {
    const file = path.join(dir, pos.file);
    const [status, line] = runChecker(file, expectArgs(manifest, pos, baked));
    const accepted = status === 0 && line.includes(" ACCEPT ");
    if (!accepted) ok = false;
    console.log(`${accepted ? "PASS" : "FAIL"} positive ${pos.file}: ${line}`);
    nPos++;
  }

  for (const mut of manifest.mutants) {
    const file = path.join(dir, mut.file);
    const [status, line] = runChecker(file, expectArgs(manifest, mut, baked));
    const rejected = status !== 0;
    const classMatch = line.includes(` ${mut.expected} `) || line.includes(` ${mut.expected}`);
    const good = rejected && classMatch;
    if (!good) ok = false;
    console.log(
      `${good ? "PASS" : "FAIL"} mutant ${mut.file} (expected ${mut.expected}): ${line.slice(0, 140)}`,
    );
    nMut++;
  }

  console.log(
    `  ${nPos} positives + ${nMut} mutants; ${ok ? "all verdicts as expected" : "MISMATCHES above"}`,
  );
  return ok;
}

function main(): number {
  const manifests = findManifests(OUTDIR);
  if (manifests.length === 0) {
    console.error(`glc-recheck: no GLC-MANIFEST.json found under ${OUTDIR}`);
    return 2;
  }
  let ok = true;
  for (const m of manifests) ok = checkManifest(m) && ok;
  console.log(
    `glc-recheck: ${manifests.length} container lane(s) through ${path.basename(GLC_CHECK!)}; ${
      ok ? "all lanes green" : "MISMATCHES above"
    }`,
  );
  return ok ? 0 : 1;
}

process.exit(main());
