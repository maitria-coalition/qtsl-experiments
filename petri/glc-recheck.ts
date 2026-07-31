// glc-recheck.ts — the Petri P-invariant certificates through the
// geolog fast-path checker.
//
// This family's certificates derive from Model Checking Contest 2026
// model files, which this repository does not redistribute (no
// explicit upstream license statement; clarification with the
// organizers is pending). The committed artifact is therefore
// GLC-MANIFEST.expected.json — sha256 + expected obligation counts
// per container — and this driver verifies a MATERIALIZED copy of
// the containers against it: byte identity first (sha256), then the
// checker from the bytes alone (envelope integrity, reference
// discipline, and the constructed obligation enumeration — per
// invariant y and transition t the conservation identity
// Σ_p y_p·(post[p,t] − pre[p,t]) = 0, plus one initial-charge
// identity per invariant, each discharged as constExactEvalTrue with
// exact counts). A certificate stores primary data only; the checker
// refuses zero-invariant containers and zero invariant vectors, so
// nothing hollow can green.
//
// Verdict discipline:
//   positives: sha256 must match the manifest AND the checker must
//              ACCEPT with the exact per-file obligation count;
//   mutants:   each lands in its expected class — FAIL / REFUSE at
//              the designed locus, ACCEPT_UNDERCOUNT (count-gate
//              witnesses: self-consistent shrinkage caught only by
//              the manifest's expected counts), ACCEPT_FORGED (the
//              coherent-forgery CONTROL: a phantom place's true
//              conservation on a net that is not the fixture's; the
//              falsifier with that reach is byte identity against
//              regeneration, i.e. the sha256 line above).
//
// env GLC_CHECK names the checker binary (built from the pinned qtsl
// checkout: crates/qtsl-geolog-provider, cargo build --release --bin
// glc-check). env PETRI_GLC_DIR names the materialized container
// directory (containing the *.glc files, mutants/ subdirectory, and
// GLC-MANIFEST.json as produced by the regeneration path in
// README.md); default petri/out/glc. When the directory is absent
// the driver prints the fence message and exits 0 — the family's
// reproduction is gated on corpus materialization, and an honest
// no-op is not a failure.
//
// Run: GLC_CHECK=... node --experimental-strip-types petri/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(new URL(import.meta.url).pathname);
const DIR = process.env.PETRI_GLC_DIR ?? path.join(HERE, "out/glc");
const EXPECTED = path.join(HERE, "GLC-MANIFEST.expected.json");

if (!fs.existsSync(DIR)) {
  console.log(
    "petri glc-recheck: no materialized containers at " +
      DIR +
      " — this family's model inputs (MCC 2026 corpus) are not redistributed here; " +
      "regenerate per petri/README.md (fetch the five model archives, run the producer, " +
      "then re-run this driver; committed sha256s verify the regeneration bit-for-bit). " +
      "Skipping honestly.",
  );
  process.exit(0);
}

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
};
type Mutant = {
  file: string;
  base_expect_claims: string;
  glc_sha256: string;
  expected: string;
};
const manifest = JSON.parse(fs.readFileSync(EXPECTED, "utf8"));
const positives: Positive[] = manifest.positives;
const mutants: Mutant[] = manifest.mutants;

let failed = false;
const fail = (msg: string) => {
  console.error("FAIL " + msg);
  failed = true;
};

function sha256(file: string): string {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function run(expectClaims: string, file: string): { line: string; status: number } {
  const r = spawnSync(GLC_CHECK!, ["--expect-claims", expectClaims, file], {
    encoding: "utf8",
  });
  if (r.error) throw r.error;
  const line = (r.stdout ?? "").trim().split("\n").pop() ?? "";
  return { line, status: r.status ?? -1 };
}

// ---- positives: byte identity, then ACCEPT at exact counts.
let obligations = 0;
for (const p of positives) {
  const f = path.join(DIR, p.file);
  if (!fs.existsSync(f)) {
    fail(`${p.file}: missing from ${DIR}`);
    continue;
  }
  const got = sha256(f);
  if (got !== p.glc_sha256) {
    fail(`${p.file}: sha256 ${got} != manifest ${p.glc_sha256} (regeneration divergence)`);
    continue;
  }
  const { line, status } = run(p.expect_claims, f);
  if (status !== 0 || !line.includes(" ACCEPT ")) {
    fail(`${p.file}: expected ACCEPT at ${p.expect_claims}, got: ${line}`);
    continue;
  }
  obligations += Number(p.expect_claims.split("=")[1]);
  console.log("ok  " + line);
}

// ---- mutants: each in its expected class, checked at the BASE
// expectation (the count gate is part of the instrument).
for (const m of mutants) {
  const f = path.join(DIR, m.file);
  if (!fs.existsSync(f)) {
    fail(`${m.file}: missing from ${DIR}`);
    continue;
  }
  const got = sha256(f);
  if (got !== m.glc_sha256) {
    fail(`${m.file}: sha256 ${got} != manifest ${m.glc_sha256}`);
    continue;
  }
  const { line } = run(m.base_expect_claims, f);
  let ok: boolean;
  switch (m.expected) {
    case "FAIL":
      ok = line.includes(" FAIL ");
      break;
    case "REFUSE":
      ok = line.includes(" REFUSE ");
      break;
    case "ACCEPT_UNDERCOUNT":
      // Self-consistent shrinkage: FAILs the base count gate while
      // every surviving obligation holds (x/y with x < y).
      ok = line.includes(" FAIL ") && /constExactEvalTrue=(\d+)\/(\d+)/.test(line)
        && (() => {
          const [, a, b] = line.match(/constExactEvalTrue=(\d+)\/(\d+)/)!;
          return Number(a) < Number(b);
        })();
      break;
    case "ACCEPT_FORGED":
      // The coherent-forgery control: MORE obligations than the base,
      // all holding — in-lane the container is a true statement about
      // a different net; the sha256 gate above is the falsifier with
      // that reach (its hash can never match a regenerated original).
      ok = /constExactEvalTrue=(\d+)\/(\d+)/.test(line)
        && (() => {
          const [, a, b] = line.match(/constExactEvalTrue=(\d+)\/(\d+)/)!;
          return Number(a) > Number(b);
        })();
      break;
    default:
      ok = false;
  }
  if (!ok) {
    fail(`${m.file}: expected class ${m.expected}, got: ${line}`);
  } else {
    console.log(`ok  [want ${m.expected}] ${line}`);
  }
}

if (failed) {
  console.error("petri glc-recheck: verdicts diverged from the manifest");
  process.exit(1);
}
console.log(
  `petri glc-recheck: ${positives.length} positives (${obligations} obligations) + ` +
    `${mutants.length} mutants through glc-check; all verdicts as expected`,
);
