// run_checks.ts — checker sweep for the FrozenLake tier: run flcheck over
// every stored certificate (+ the mutant battery) listed in
// out/producer_receipts.json, collect verdicts and bench timings into
// out/checker_receipts.json (same schema as the retired python driver).
//
// Trust posture: this driver is UNTRUSTED orchestration — it only walks the
// committed receipts, invokes the independent Rust checker (flcheck), and
// records what the checker said. The checker binary is the instrument; a bug
// here can at worst mislabel a verdict, never manufacture one.
//
// Contract: flcheck <cert.cbor> [--bench N]; exit 0 = ACCEPT, 1 = REFUSE;
// one verdict line on stdout (ACCEPT rows=... / REFUSE locus=... detail=...).
// env FLCHECK overrides the checker path (default <repo>/target/release/flcheck).
// Exit: 0 iff every certificate ACCEPTs and every mutant REFUSEs.
//
// Run: node --experimental-strip-types gymnasium/run_checks.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const OUT = path.join(HERE, "out");
const REPO = path.resolve(HERE, "..");
const FLCHECK = process.env.FLCHECK || path.join(REPO, "target", "release", "flcheck");
const BENCH_N = 200;

function run(certPath: string, bench?: number): [number, string] {
  const args = [certPath];
  if (bench) args.push("--bench", String(bench));
  const p = spawnSync(FLCHECK, args, { encoding: "utf8" });
  if (p.error) throw p.error;
  const line = (p.stdout !== "" ? p.stdout : p.stderr).trim();
  return [p.status ?? -1, line];
}

interface AcceptFields {
  rows?: number;
  decode_us?: number;
  verify_us?: number;
  bench_n?: number;
  verify_min_ns?: number;
  verify_median_ns?: number;
  bound?: string;
}

function parseAccept(line: string): AcceptFields {
  const d: AcceptFields = {};
  for (const k of ["rows", "decode_us", "verify_us", "bench_n",
                   "verify_min_ns", "verify_median_ns"] as const) {
    const m = line.match(new RegExp(`${k}=(\\d+)`));
    if (m) d[k] = parseInt(m[1], 10);
  }
  const m = line.match(/bound=(\d+)\/(\d+)/);
  if (m) d.bound = `${m[1]}/${m[2]}`;
  return d;
}

function main(): number {
  const prod = JSON.parse(
    fs.readFileSync(path.join(OUT, "producer_receipts.json"), "utf8"));
  const receipts: {
    flcheck: string; bench_n: number;
    certs: Record<string, AcceptFields>;
    mutants: Array<Record<string, unknown>>;
  } = {
    // record the checker's repo-relative path; an out-of-tree build's
    // absolute location is its own business, not the receipt's
    flcheck: path.relative(REPO, path.resolve(FLCHECK)).startsWith("..")
      ? "flcheck (out-of-tree build)"
      : path.relative(REPO, path.resolve(FLCHECK)),
    bench_n: BENCH_N,
    certs: {},
    mutants: [],
  };
  let ok = true;

  for (const row of Object.values(prod.rows) as Array<{ certs: string[] }>) {
    for (const rel of row.certs) {
      const certPath = path.join(OUT, rel);
      const [rc, line] = run(certPath, BENCH_N);
      if (rc !== 0) {
        console.log(`UNEXPECTED REFUSE: ${rel}: ${line}`);
        ok = false;
        continue;
      }
      const d = parseAccept(line);
      receipts.certs[rel] = d;
      const nsRow = d.rows ? (d.verify_median_ns ?? 0) / d.rows : null;
      if (nsRow) {
        console.log(`ACCEPT ${rel}: rows=${d.rows} ` +
          `verify_median=${((d.verify_median_ns ?? 0) / 1e3).toFixed(1)}us ` +
          `(${nsRow.toFixed(0)} ns/row)`);
      } else {
        console.log(`ACCEPT ${rel}`);
      }
    }
  }

  for (const m of prod.mutants as Array<{ name: string; cert: string; expected_locus: string }>) {
    const certPath = path.join(OUT, m.cert);
    const [rc, line] = run(certPath);
    const lm = line.match(/locus=(\S+)/);
    const locus = lm ? lm[1] : null;
    receipts.mutants.push({
      name: m.name, expected_locus: m.expected_locus,
      refused: rc === 1, locus, line,
    });
    const verdict = rc === 1 ? "ok" : "FAILED-TO-REFUSE";
    console.log(`mutant ${m.name}: ${verdict} (${line})`);
    if (rc !== 1) ok = false;
  }

  const outPath = path.join(OUT, "checker_receipts.json");
  fs.writeFileSync(outPath, JSON.stringify(receipts, null, 1));
  console.log("receipts ->", outPath);
  return ok ? 0 : 1;
}

process.exit(main());
