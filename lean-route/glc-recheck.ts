// glc-recheck.ts — the lean-route ladder's sealed containers through
// the geolog fast-path checker, across both committed container lanes:
//
//   staged-chain lane (out/glc/) — the fifteen ladder certificates
//     re-sealed via mtk on the staged BLstack/BLCert chain, kept
//     checkable forever (versioning law: a committed lane never dies).
//     The APPLICATIVE obligation roster is enumerated from the rule
//     nodes themselves and checked with exact per-relation counts.
//
//   baked lane (out/glc-baked/) — the same fifteen certificates
//     re-sealed on the baked qc closure: explicit claim rows,
//     discharged through the exact-rational ground provider against
//     the vocabulary index the checker derives from the pinned
//     theory-pair chain's own bytes (`glc-check --baked-cert`).
//     Requires a checker revision that classifies the reserved
//     predicate-hole wire spelling (a TVar row whose single argument
//     is the nat literal 4294967295 — the Lean model's own encoding,
//     transported to row grain); at an older pin every cone claim
//     refuses loudly rather than passing vacuously.
//
// Both lanes check Bernstein-cone sweeps, Farkas and syzygy
// combination replays, the syzygy conclusion-form tie, enclosure
// remainder bounds, and the promise rule's two wire obligations.
// Structural rules (ax/ande/ore/cut/eqelim/conv, promise formula
// plumbing) remain the reference-lane + certified-checker stratum
// (`make lean-route-recheck`); GLC-CERT.md states the split.
//
// Verdict discipline (per lane):
//   positives: every manifest positive must ACCEPT with EXACT
//              per-relation obligation counts (--expect-claims from
//              the manifest — zeros enforced too, so an obligation
//              appearing where the manifest says none is a failure);
//   mutants:   every manifest mutant lands in its expected class —
//              FAIL / REFUSE at the designed locus,
//              ACCEPT_UNDERCOUNT (count-gate witnesses: ACCEPT with
//              fewer obligations than the base in some relation),
//              ACCEPT_CLAIMSWAP (the coherent-reforge CONTROL: the
//              lane accepts a self-consistent TRUE statement of a
//              DIFFERENT claim; the production-time byte-identity
//              gate is the falsifier with that reach).
//
// env GLC_CHECK names the checker binary (built from the pinned qtsl
// checkout: crates/qtsl-geolog-provider, cargo build --release --bin
// glc-check).
//
// Run: GLC_CHECK=... node --experimental-strip-types lean-route/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(new URL(import.meta.url).pathname);

const GLC_CHECK = process.env.GLC_CHECK;
if (!GLC_CHECK) {
  console.error(
    "glc-recheck: GLC_CHECK is not set. Build the checker from the pinned qtsl " +
      "checkout (crates/qtsl-geolog-provider, cargo build --release --bin glc-check) " +
      "and pass its path: GLC_CHECK=/path/to/glc-check node --experimental-strip-types " +
      "lean-route/glc-recheck.ts",
  );
  process.exit(2);
}

type Counts = Record<string, number>;

/** Parse every `name=h/n` pair from a verdict line. */
function countsOf(line: string): { holds: Counts; totals: Counts } | null {
  const holds: Counts = {};
  const totals: Counts = {};
  let seen = 0;
  for (const m of line.matchAll(/([A-Za-z]+)=(\d+)\/(\d+)/g)) {
    holds[m[1]] = Number(m[2]);
    totals[m[1]] = Number(m[3]);
    seen++;
  }
  return seen ? { holds, totals } : null;
}

let ok = true;

/** One lane: the staged chain checked bare, the baked lane with
 * --baked-cert; identical expectation discipline. */
function checkLane(label: string, dir: string, baked: boolean) {
  const manifest = JSON.parse(fs.readFileSync(path.join(dir, "GLC-MANIFEST.json"), "utf8"));
  const roster: string[] = manifest.obligation_roster;
  const flag = baked ? ["--baked-cert"] : [];
  const t0 = Date.now();
  let pass = 0;

  for (const pos of manifest.positives) {
    const file = path.join(dir, pos.file);
    const expect = roster.map((r) => `${r}=${pos.obligations[r] ?? 0}`).join(",");
    const p = spawnSync(GLC_CHECK, [...flag, "--expect-claims", expect, file], {
      encoding: "utf8",
      maxBuffer: 16 << 20,
    });
    if (p.error) throw p.error;
    const line = p.stdout.trim();
    // The verdict line must itself carry every roster relation with the
    // expected exact count — a checker built at too old a pin would
    // print a narrower roster and pass the zero-obligation rungs
    // vacuously; this gate refuses that instead.
    const c = countsOf(line);
    const good =
      p.status === 0 &&
      line.includes(" ACCEPT ") &&
      c !== null &&
      roster.every(
        (r) =>
          c.holds[r] === (pos.obligations[r] ?? 0) &&
          c.totals[r] === (pos.obligations[r] ?? 0),
      );
    if (good) pass++;
    else {
      ok = false;
      console.log(`FAIL positive ${pos.file} (expect ${expect}): ${line.slice(0, 200)}`);
    }
  }
  const wall = ((Date.now() - t0) / 1000).toFixed(2);
  console.log(
    `${label}: ${pass}/${manifest.positives.length} positives ACCEPT with exact ` +
      `per-relation obligation counts (${wall}s)`,
  );

  const baseCounts = new Map<string, Counts>(
    manifest.positives.map((p: { source: string; obligations: Counts }) => [
      path.basename(p.source).replace(".rows.txt", ""),
      p.obligations,
    ]),
  );

  let mutPass = 0;
  const nMut = (manifest.mutants ?? []).length;
  for (const mut of manifest.mutants ?? []) {
    const file = path.join(dir, mut.file);
    const p = spawnSync(GLC_CHECK, [...flag, file], { encoding: "utf8", maxBuffer: 16 << 20 });
    if (p.error) throw p.error;
    const line = p.stdout.trim();
    let good: boolean;
    if (mut.expect === "ACCEPT_UNDERCOUNT") {
      const c = countsOf(line);
      const base = baseCounts.get(mut.base);
      good =
        p.status === 0 &&
        c !== null &&
        base !== undefined &&
        roster.some((r) => (c.holds[r] ?? 0) < (base[r] ?? 0)) &&
        roster.every((r) => (c.holds[r] ?? 0) === (c.totals[r] ?? 0));
    } else if (mut.expect === "ACCEPT_CLAIMSWAP") {
      // The control: counts equal the base exactly; the lane accepts by
      // design (the claim-swap is caught at production, not here).
      const c = countsOf(line);
      const base = baseCounts.get(mut.base);
      good =
        p.status === 0 &&
        c !== null &&
        base !== undefined &&
        roster.every((r) => (c.holds[r] ?? 0) === (base[r] ?? 0));
    } else {
      good = p.status !== 0 && line.includes(` ${mut.expect} `);
    }
    if (good) mutPass++;
    else {
      ok = false;
      console.log(`FAIL mutant ${mut.file} (expected ${mut.expect}): ${line.slice(0, 160)}`);
    }
  }
  if (nMut) console.log(`${label}: ${mutPass}/${nMut} mutants land in their expected classes`);
}

checkLane("lean-route (staged chain)", path.join(HERE, "out/glc"), false);
const bakedDir = path.join(HERE, "out/glc-baked");
if (fs.existsSync(path.join(bakedDir, "GLC-MANIFEST.json"))) {
  checkLane("lean-route (baked closure)", bakedDir, true);
}

console.log(ok ? "OK" : "FAILED");
process.exit(ok ? 0 : 1);
