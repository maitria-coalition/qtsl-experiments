// run_tt_checks.ts — checker sweep for the toy-text tier: flcheck over every
// reward-lane certificate + the hitting companion + the tt mutant battery
// (all listed in out/producer_receipts_tt.json), collecting verdicts and
// bench timings into out/checker_receipts_tt.json (same schema as the
// retired python driver).
//
// Bench count scales inversely with certificate size so the sweep stays
// minutes-scale: 200 passes < 300KB, 50 < 1.5MB, 10 beyond.
//
// Trust posture: this driver is UNTRUSTED orchestration — it only walks the
// committed receipts, invokes the independent Rust checker (flcheck), and
// records what the checker said. The checker binary is the instrument.
//
// Contract: flcheck <cert.cbor> [--bench N]; exit 0 = ACCEPT, 1 = REFUSE;
// one verdict line on stdout. env FLCHECK overrides the checker path
// (default <repo>/target/release/flcheck).
// Exit: 0 iff every certificate ACCEPTs and every mutant REFUSEs at its
// designed locus.
//
// Run: node --experimental-strip-types gymnasium/run_tt_checks.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const OUT = path.join(HERE, "out");
const REPO = path.resolve(HERE, "..");
const FLCHECK = process.env.FLCHECK || path.join(REPO, "target", "release", "flcheck");

function benchN(certPath: string): number {
  const sz = fs.statSync(certPath).size;
  if (sz < 300_000) return 200;
  if (sz < 1_500_000) return 50;
  return 10;
}

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
  const m = line.match(/bound=(-?\d+)\/(-?\d+)/);
  if (m) d.bound = `${m[1]}/${m[2]}`;
  return d;
}

function main(): number {
  const prod = JSON.parse(
    fs.readFileSync(path.join(OUT, "producer_receipts_tt.json"), "utf8"));
  const receipts: {
    flcheck: string;
    certs: Record<string, AcceptFields>;
    mutants: Array<Record<string, unknown>>;
  } = {
    // record the checker's repo-relative path; an out-of-tree build's
    // absolute location is its own business, not the receipt's
    flcheck: path.relative(REPO, path.resolve(FLCHECK)).startsWith("..")
      ? "flcheck (out-of-tree build)"
      : path.relative(REPO, path.resolve(FLCHECK)),
    certs: {},
    mutants: [],
  };
  let ok = true;

  const certs: string[] = [];
  for (const row of Object.values(prod.rows) as Array<{ certs: string[] }>) {
    certs.push(...row.certs);
  }
  certs.push(prod.hitting_companion.cert);

  for (const rel of certs) {
    const certPath = path.join(OUT, rel);
    const n = benchN(certPath);
    const [rc, line] = run(certPath, n);
    if (rc !== 0) {
      console.log(`UNEXPECTED REFUSE: ${rel}: ${line}`);
      ok = false;
      continue;
    }
    const d = parseAccept(line);
    receipts.certs[rel] = d;
    const nsRow = d.rows ? (d.verify_median_ns ?? 0) / d.rows : 0;
    console.log(`ACCEPT ${rel}: rows=${d.rows} ` +
      `verify_median=${((d.verify_median_ns ?? 0) / 1e6).toFixed(2)}ms ` +
      `(${nsRow.toFixed(0)} ns/row, bench_n=${n})`);
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
    const good = rc === 1 && locus === m.expected_locus;
    console.log(`mutant ${m.name}: ${good ? "ok" : "UNEXPECTED"} (${line})`);
    if (!good) ok = false;
  }

  const outPath = path.join(OUT, "checker_receipts_tt.json");
  fs.writeFileSync(outPath, JSON.stringify(receipts, null, 1));
  console.log("receipts ->", outPath);
  return ok ? 0 : 1;
}

process.exit(main());
