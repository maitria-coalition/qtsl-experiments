// glc-recheck.ts — guarded-specification certificates over Petri
// nets, through the geolog fast-path checker.
//
// Each certificate seals, in one container: a place/transition net as
// a bag-mechanism system (dimensions, sparse flow and marking rows —
// primary data, untouched); a specification whose body is an
// always-invariant formula (the conjunction of a ≤ and a ≥ atom at
// the net's first P-invariant charge) and whose GUARD is a region —
// the degenerate box at the initial marking; a region-satisfaction
// warrant; the satisfaction claim wired through the guard-warrant
// route; and the well-formedness/membership claim rows the checker
// discharges. The checker derives its vocabulary index from the baked
// theory closure embedded in its own binary, resolves the container's
// prerequisite pins against it, and re-evaluates every claim row
// through the exact-rational ground provider — guard membership is
// closed-interval arithmetic over ℚ, per marking wire.
//
// This family's certificates derive from Model Checking Contest 2026
// model files, which this repository does not redistribute (no
// explicit upstream license statement; clarification with the
// organizers is pending — see ../README.md). The committed artifact
// is GLC-MANIFEST.expected.json — sha256 + exact per-relation claim
// counts per container — and this driver verifies a MATERIALIZED
// copy against it: byte identity first (sha256; sealing is
// deterministic, so regeneration from the model files reproduces
// every container bit-for-bit), then the checker from the bytes.
//
// Verdict discipline:
//   positives: sha256 must match the manifest AND the checker must
//              ACCEPT at the exact per-relation count table;
//   specimens: each lands in its expected class —
//     ACCEPT                  the unmutated control;
//     FAIL_AT                 refuses with the named relation at 0/1
//                             (guard violation, region disagreement,
//                             route duplication, alien component);
//     ACCEPT_BARE_FAIL_COUNTS whole-row omission: per-obligation
//                             discharge ACCEPTs the bare file, the
//                             count gate (--expect-claims) FAILs it —
//                             both verdicts are asserted;
//     ACCEPT_TIE_BLIND        the root-formula tie-break: a theory-
//                             stratum equality the claim walk never
//                             sees, accepted BY DESIGN and asserted
//                             so, keeping the reach boundary
//                             machine-checked (the theory engine's
//                             own chase refuses this specimen; see
//                             README.md).
//
// env GLC_CHECK names the checker binary (built from the pinned qtsl
// checkout: crates/qtsl-geolog-provider, cargo build --release --bin
// glc-check). env GUARD_GLC_DIR names the materialized container
// directory (the six positive *.glc files plus a specimens/
// subdirectory); default petri/guard/out. When the directory is
// absent the driver prints the fence message and exits 0 — the
// family's reproduction is gated on corpus materialization, and an
// honest no-op is not a failure.
//
// Run: GLC_CHECK=... node --experimental-strip-types petri/guard/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(new URL(import.meta.url).pathname);
const DIR = process.env.GUARD_GLC_DIR ?? path.join(HERE, "out");
const EXPECTED = path.join(HERE, "GLC-MANIFEST.expected.json");

if (!fs.existsSync(DIR)) {
  console.log(
    "petri/guard glc-recheck: no materialized containers at " +
      DIR +
      " — this family's model inputs (MCC 2026 corpus) are not redistributed here; " +
      "regenerate per petri/guard/README.md (fetch the model archives, run the producer, " +
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
type Specimen = {
  file: string;
  expected: string;
  fails_at?: string;
  glc_sha256: string;
};
const manifest = JSON.parse(fs.readFileSync(EXPECTED, "utf8"));
const positives: Positive[] = manifest.positives;
const specimens: Specimen[] = manifest.specimens;
const baseExpect: string = manifest.specimen_base.expect_claims;

let failed = false;
const fail = (msg: string) => {
  console.error("FAIL " + msg);
  failed = true;
};

function sha256(file: string): string {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function run(file: string, expectClaims?: string): { line: string; status: number } {
  const args = expectClaims ? ["--expect-claims", expectClaims, file] : [file];
  const r = spawnSync(GLC_CHECK!, args, { encoding: "utf8" });
  if (r.error) throw r.error;
  const line = (r.stdout ?? "").trim().split("\n").pop() ?? "";
  return { line, status: r.status ?? -1 };
}

// ---- positives: byte identity, then ACCEPT at the exact count table.
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
  const { line, status } = run(f, p.expect_claims);
  if (status !== 0 || !line.includes(" ACCEPT ")) {
    fail(`${p.file}: expected ACCEPT at the manifest count table, got: ${line}`);
    continue;
  }
  console.log("ok  " + line);
}

// ---- specimens: each in its expected class, checked at the BASE
// count table (the count gate is part of the instrument).
for (const m of specimens) {
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
  switch (m.expected) {
    case "ACCEPT": {
      const { line, status } = run(f, baseExpect);
      if (status !== 0 || !line.includes(" ACCEPT ")) {
        fail(`${m.file}: control expected ACCEPT, got: ${line}`);
      } else {
        console.log("ok  [control] " + line);
      }
      break;
    }
    case "FAIL_AT": {
      const { line, status } = run(f, baseExpect);
      const marker = `${m.fails_at}=0/`;
      if (status === 0 || !line.includes(" FAIL ") || !line.includes(marker)) {
        fail(`${m.file}: expected FAIL at ${m.fails_at} (0/…), got: ${line}`);
      } else {
        console.log(`ok  [want FAIL at ${m.fails_at}] ` + line);
      }
      break;
    }
    case "ACCEPT_BARE_FAIL_COUNTS": {
      // Whole-row omission: both halves of the demonstration are the
      // assertion — per-obligation discharge alone cannot see a
      // deleted claim row (bare ACCEPT), the count table can (FAIL).
      const bare = run(f);
      if (bare.status !== 0 || !bare.line.includes(" ACCEPT ")) {
        fail(`${m.file}: expected bare ACCEPT (per-obligation blindness half), got: ${bare.line}`);
        break;
      }
      const gated = run(f, baseExpect);
      const marker = `${m.fails_at}=0/`;
      if (gated.status === 0 || !gated.line.includes(" FAIL ") || !gated.line.includes(marker)) {
        fail(`${m.file}: expected count-gate FAIL at ${m.fails_at}, got: ${gated.line}`);
      } else {
        console.log(`ok  [bare ACCEPT, count gate FAIL at ${m.fails_at}] ` + gated.line);
      }
      break;
    }
    case "ACCEPT_TIE_BLIND": {
      const { line, status } = run(f, baseExpect);
      if (status !== 0 || !line.includes(" ACCEPT ")) {
        fail(`${m.file}: tie-break specimen expected ACCEPT (documented reach boundary), got: ${line}`);
      } else {
        console.log("ok  [tie-blind BY DESIGN — theory-stratum tooth covers it] " + line);
      }
      break;
    }
    default:
      fail(`${m.file}: unknown expected class ${m.expected}`);
  }
}

if (failed) {
  console.error("petri/guard glc-recheck: verdicts diverged from the manifest");
  process.exit(1);
}
console.log(
  `petri/guard glc-recheck: ${positives.length} positives + ${specimens.length} specimens ` +
    `through glc-check; all verdicts as expected`,
);
