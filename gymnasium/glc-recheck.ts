// glc-recheck.ts — the Gymnasium certificate families through the
// geolog fast-path checker, manifest-driven and family-generic.
//
// Discovery: every GLC-MANIFEST.json under gymnasium/out/ (recursive)
// is one family; `file` fields resolve relative to the manifest's own
// directory. The driver dispatches on each manifest's `mode_flag`
// (the literal checker flag for that family's vocabulary), so a new
// family is a dropped-in manifest plus its containers — zero driver
// edits.
//
// Verdict discipline, per manifest:
//   positives: sha256 must match the manifest AND the checker must
//              ACCEPT at the manifest's exact per-relation obligation
//              counts (--expect-claims);
//   mutants:   each lands in its expected class — FAIL / REFUSE at
//              the designed locus; ACCEPT_UNDERCOUNT (self-consistent
//              shrinkage: FAILs the base count gate with fewer
//              obligations held than expected — the count gate is the
//              witness); ACCEPT_BY_DESIGN (a coherent-forgery CONTROL:
//              in-lane a true statement about a different system; the
//              manifest's reach_note names the falsifier with that
//              reach, byte identity against regeneration).
//   a family whose manifest carries `materialize` (bytes not
//   committed) skips honestly when its files are absent.
//
// env GLC_CHECK names the checker binary (built from the pinned qtsl
// checkout: crates/qtsl-geolog-provider, cargo build --release --bin
// glc-check). For pre-landing verification against an unlanded
// checker branch, point GLC_CHECK at that branch's build — the
// binary, not the pin, is the contract here.
//
// Run: GLC_CHECK=... node --experimental-strip-types gymnasium/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(new URL(import.meta.url).pathname);
const OUT = process.env.GYM_GLC_OUT ?? path.join(HERE, "out");

const GLC_CHECK = process.env.GLC_CHECK;
if (!GLC_CHECK) {
  console.error(
    "glc-recheck: GLC_CHECK is not set. Build the checker from the pinned qtsl " +
      "checkout (crates/qtsl-geolog-provider, cargo build --release --bin glc-check) " +
      "and pass its path.",
  );
  process.exit(2);
}

type Positive = {
  file: string;
  expect_claims: string;
  glc_sha256: string;
  expected: string;
  claim?: string;
};
type Mutant = {
  mutant: string;
  base: string;
  file: string;
  glc_sha256: string;
  expected: string;
  locus?: string;
  reach_note?: string;
};
type Manifest = {
  family: string;
  mode_flag: string;
  positives: Positive[];
  mutants: Mutant[];
  materialize?: string;
};

function findManifests(dir: string): string[] {
  const out: string[] = [];
  const stack = [dir];
  while (stack.length) {
    const d = stack.pop()!;
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) stack.push(p);
      else if (e.name === "GLC-MANIFEST.json") out.push(p);
    }
  }
  return out.sort();
}

let failed = false;
const fail = (msg: string) => {
  console.error("FAIL " + msg);
  failed = true;
};

function sha256(file: string): string {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function run(
  modeFlag: string,
  expectClaims: string | undefined,
  file: string,
): { line: string; status: number } {
  const args = [modeFlag];
  if (expectClaims) args.push("--expect-claims", expectClaims);
  args.push(file);
  const r = spawnSync(GLC_CHECK!, args, { encoding: "utf8" });
  if (r.error) throw r.error;
  const line = (r.stdout ?? "").trim().split("\n").pop() ?? "";
  return { line, status: r.status ?? -1 };
}

/// Some relation's held-count sits strictly below its expected count
/// on the verdict line (the ACCEPT_UNDERCOUNT witness).
function someUndercount(line: string): boolean {
  for (const m of line.matchAll(/[A-Za-z_]+=(\d+)\/(\d+)/g)) {
    if (Number(m[1]) < Number(m[2])) return true;
  }
  return false;
}

let families = 0;
let positives = 0;
let mutants = 0;

for (const mpath of findManifests(OUT)) {
  const mdir = path.dirname(mpath);
  const manifest: Manifest = JSON.parse(fs.readFileSync(mpath, "utf8"));
  const relm = path.relative(OUT, mpath);
  const entries = [...manifest.positives, ...manifest.mutants];
  if (manifest.materialize && entries.some((e) => !fs.existsSync(path.join(mdir, e.file)))) {
    console.log(
      `skip ${relm} (${manifest.family}): containers not materialized here — ` +
        `regenerate per the manifest's materialize note; committed sha256 lines ` +
        `verify a regeneration bit-for-bit. Skipping honestly.`,
    );
    continue;
  }
  families += 1;
  console.log(`family ${manifest.family} (${relm}, mode ${manifest.mode_flag})`);

  for (const p of manifest.positives) {
    const f = path.join(mdir, p.file);
    if (!fs.existsSync(f)) {
      fail(`${relm}: ${p.file} missing`);
      continue;
    }
    const got = sha256(f);
    if (got !== p.glc_sha256) {
      fail(`${relm}: ${p.file}: sha256 ${got} != manifest (regeneration divergence)`);
      continue;
    }
    const { line, status } = run(manifest.mode_flag, p.expect_claims, f);
    if (status !== 0 || !line.includes(" ACCEPT ")) {
      fail(`${relm}: ${p.file}: expected ACCEPT at ${p.expect_claims}, got: ${line}`);
      continue;
    }
    positives += 1;
    console.log("ok  " + line);
  }

  for (const m of manifest.mutants) {
    const f = path.join(mdir, m.file);
    if (!fs.existsSync(f)) {
      fail(`${relm}: ${m.file} missing`);
      continue;
    }
    const got = sha256(f);
    if (got !== m.glc_sha256) {
      fail(`${relm}: ${m.file}: sha256 ${got} != manifest`);
      continue;
    }
    // Mutants run at the BASE certificate's count expectation: the
    // count gate is part of the instrument (undercount witnesses).
    const base = manifest.positives.find(
      (p) => p.file === m.base + ".glc" || p.file.endsWith("/" + m.base + ".glc"),
    );
    const { line } = run(manifest.mode_flag, base?.expect_claims, f);
    let ok: boolean;
    switch (m.expected) {
      case "FAIL":
        ok = line.includes(" FAIL ");
        break;
      case "REFUSE":
        ok = line.includes(" REFUSE ");
        break;
      case "ACCEPT_UNDERCOUNT":
        ok = line.includes(" FAIL ") && someUndercount(line);
        break;
      case "ACCEPT_BY_DESIGN":
        ok = line.includes(" ACCEPT ") && typeof m.reach_note === "string";
        break;
      default:
        ok = false;
    }
    if (!ok) {
      fail(`${relm}: ${m.file}: expected class ${m.expected}, got: ${line}`);
    } else {
      mutants += 1;
      console.log(`ok  [want ${m.expected}] ${line}`);
    }
  }
}

if (families === 0) {
  fail(`no GLC-MANIFEST.json found under ${OUT}`);
}
if (failed) {
  console.error("gymnasium glc-recheck: verdicts diverged from the manifests");
  process.exit(1);
}
console.log(
  `gymnasium glc-recheck: ${families} families, ${positives} positives + ` +
    `${mutants} mutants through glc-check; all verdicts as expected`,
);
