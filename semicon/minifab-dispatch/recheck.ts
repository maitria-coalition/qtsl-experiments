// semicon/minifab-dispatch/recheck.ts — re-verify the Mini-Fab
// dispatch-synthesis certificate family from its sealed containers, and
// re-run the producer-tier batteries; print one verdict table for the
// whole family.
//
// The family (out/GLC-MANIFEST.json; one .glc = one sealed geolog
// commit carrying every certificate of its campaign as claim rows):
//
//   minifab.glc            the closed baseline — the twelve-certificate
//                          safety/deadline family over the Mini-Fab net
//                          (ranking, lot count, six machine conservation
//                          bounds, congestion, ceiling, and the
//                          arrival-potential + step-budget deadline pair).
//   minifab-gated.glc      the same plant as an OPEN system: a comb with
//                          one policy hole; the hole count is a checked
//                          claim, and the gated deadline dictionary
//                          certifies T* = fold_D(m0).
//   minifab-supervised.glc the synthesized supervisor closed composite —
//                          the latest-safe-unlock winner with certified
//                          completion and the 626N arrivals bound.
//   minifab-simplex.glc    the Simplex composite — a certified switching
//                          envelope filling the same hole, bounding the
//                          downside of an uncertified advanced policy.
//
// Two verification legs, honestly partitioned:
//
//   containers (this file, via GLC_CHECK) — everything the sealed bytes
//     witness: envelope integrity, then per-relation claim discharge
//     through the exact-rational ground provider, with the manifest's
//     per-relation counts as a load-bearing gate (a deleted relation
//     leaves every surviving claim true; the count gate is what refuses
//     it). Committed integrity mutants must REFUSE.
//
//   producer batteries (via MTK_MANIFEST) — everything the container
//     bytes cannot witness: campaign re-derivation in exact rational
//     arithmetic, exact-tier sequent kills, encoder-mirror kills, and
//     the strategy-tier replays (synthesis grid, winner replay, envelope
//     runs). These run as the pinned mtk checkout's semicon test suite;
//     the manifest pins a minimum test count so an over-narrow filter
//     cannot pass vacuously.
//
// Trust posture: this driver is UNTRUSTED orchestration — it walks the
// committed artifacts, invokes the checker and the batteries, and
// tabulates verdicts. The checker binary re-derives every claim from
// the container bytes; nothing here is taken from the producer's word.
//
// Artifact freshness: every .glc is sha256-verified against the
// manifest before checking, so a stale or hand-edited artifact reports
// as STALE rather than laundering a verdict.
//
// env:
//   GLC_CHECK     the checker binary, built from the pinned qtsl
//                 checkout (cargo build --release --bin glc-check in
//                 crates/qtsl-geolog-provider). No default — unset is an
//                 honest refusal, never a silent skip.
//   MTK_MANIFEST  path to the pinned mtk checkout's Cargo.toml for the
//                 battery leg. No default — unset is an honest refusal.
//
// Run (the make target wires both): make minifab-dispatch

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const OUTDIR = path.join(HERE, "out");

const GLC_CHECK = process.env.GLC_CHECK;
const MTK_MANIFEST = process.env.MTK_MANIFEST;
if (!GLC_CHECK || !MTK_MANIFEST) {
  console.error(
    "minifab-dispatch recheck: GLC_CHECK and MTK_MANIFEST must both be set.\n" +
      "  GLC_CHECK    = glc-check built from the pinned qtsl checkout\n" +
      "                 (cargo build --release --bin glc-check in crates/qtsl-geolog-provider)\n" +
      "  MTK_MANIFEST = <pinned mtk checkout>/Cargo.toml\n" +
      "The make target `minifab-dispatch` wires both from the pinned checkouts.",
  );
  process.exit(2);
}

const manifest = JSON.parse(
  fs.readFileSync(path.join(OUTDIR, "GLC-MANIFEST.json"), "utf8"),
);

function sha256(p: string): string {
  return createHash("sha256").update(fs.readFileSync(p)).digest("hex");
}

function runChecker(certPath: string, expectClaims: string): [number, string] {
  const p = spawnSync(
    GLC_CHECK!,
    ["--baked-cert", "--expect-claims", expectClaims, certPath],
    { encoding: "utf8" },
  );
  if (p.error) throw p.error;
  return [p.status ?? -1, (p.stdout + p.stderr).trim().split("\n")[0]];
}

type Row = { kind: string; name: string; expected: string; verdict: string; ok: boolean; detail: string };
const rows: Row[] = [];

// ------------------------------------------------------------ positives
for (const pos of manifest.positives) {
  const file = path.join(OUTDIR, pos.file);
  if (sha256(file) !== pos.glc_sha256) {
    rows.push({
      kind: "container", name: pos.cert, expected: "ACCEPT",
      verdict: "STALE", ok: false,
      detail: "artifact sha256 does not match the manifest pin",
    });
    continue;
  }
  const [status, line] = runChecker(file, pos.expect_claims);
  const accepted = status === 0 && line.includes(" ACCEPT ");
  rows.push({
    kind: "container", name: pos.cert, expected: "ACCEPT",
    verdict: accepted ? "ACCEPT" : "REJECTED", ok: accepted,
    detail: line.replace(`${pos.file} `, ""),
  });
}

// -------------------------------------------------------------- mutants
for (const mut of manifest.mutants) {
  const file = path.join(OUTDIR, mut.file);
  if (sha256(file) !== mut.glc_sha256) {
    rows.push({
      kind: "mutant", name: mut.mutant, expected: mut.expected,
      verdict: "STALE", ok: false,
      detail: "artifact sha256 does not match the manifest pin",
    });
    continue;
  }
  const base = manifest.positives.find((p: any) => p.cert === mut.base);
  const [status, line] = runChecker(file, base.expect_claims);
  const rejected = status !== 0 && line.includes(` ${mut.expected}`);
  rows.push({
    kind: "mutant", name: mut.mutant, expected: mut.expected,
    verdict: rejected ? mut.expected : `UNEXPECTED(exit ${status})`, ok: rejected,
    detail: line.slice(0, 100),
  });
}

// -------------------------------------------------------------- battery
{
  const b = manifest.battery;
  console.log(
    `battery: cargo test -p ${b.package} ${b.filter} (pinned mtk checkout; ` +
      `producer-tier re-derivation + semantic mutant kills) ...`,
  );
  const p = spawnSync(
    "cargo",
    ["test", "--manifest-path", MTK_MANIFEST!, "-p", b.package, b.filter],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  );
  let passed = 0;
  let failed = 0;
  for (const m of (p.stdout ?? "").matchAll(
    /test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed/g,
  )) {
    passed += Number(m[1]);
    failed += Number(m[2]);
  }
  const ok = p.status === 0 && failed === 0 && passed >= b.min_tests;
  rows.push({
    kind: "battery", name: `${b.package}::${b.filter}`,
    expected: `>=${b.min_tests} pass`,
    verdict: `${passed} passed, ${failed} failed`, ok,
    detail:
      p.status === 0
        ? passed >= b.min_tests
          ? "producer batteries green"
          : "suite green but below the pinned minimum test count (filter drift?)"
        : `cargo exit ${p.status}` +
          ((p.stderr ?? "").trim() ? ` — ${(p.stderr ?? "").trim().split("\n").slice(-1)[0]}` : ""),
  });
}

// ---------------------------------------------------------------- table
const w = (s: string, n: number) => (s.length >= n ? s : s + " ".repeat(n - s.length));
console.log("");
console.log("Mini-Fab dispatch-synthesis family — reproduction verdicts");
console.log("-".repeat(72));
console.log(w("kind", 10) + w("artifact", 26) + w("expected", 12) + "verdict");
console.log("-".repeat(72));
for (const r of rows) {
  console.log(
    w(r.kind, 10) + w(r.name, 26) + w(r.expected, 12) + `${r.ok ? "PASS" : "FAIL"}  ${r.verdict}`,
  );
  if (!r.ok) console.log(`          ${r.detail}`);
}
console.log("-".repeat(72));
for (const pos of manifest.positives) {
  console.log(`  ${pos.cert}: ${pos.role}`);
}
const allOk = rows.every((r) => r.ok);
console.log("");
console.log(
  `minifab-dispatch: ${rows.length} verdicts through ${path.basename(GLC_CHECK!)} + the pinned batteries; ` +
    (allOk ? "all as expected" : "MISMATCHES above"),
);
process.exit(allOk ? 0 : 1);
