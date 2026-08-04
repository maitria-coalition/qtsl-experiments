// measure.ts — the family's measurement protocol, mechanized.
//
// Runs a labeled command N times (1 warmup + N measured), reniced to
// an ABSOLUTE niceness with the achieved value read back from /proc
// (a claimed niceness without read-back is a command line, not a
// measurement), and appends a receipt row to a JSON receipts file:
// per-run walls, median, the read-back niceness, load average before
// the block, and box facts. Comparison tables are regenerated from
// receipts, never hand-edited.
//
// Usage:
//   node --experimental-strip-types probprog/measure.ts \
//     --label geo-counter/producer --runs 5 --out probprog/out/receipts.json \
//     -- ./target/release/probprog-produce eval geo-counter 30
//
// Wall clock is process end-to-end (spawn to exit), the honest
// like-for-like unit for tool comparisons; tools' own internally
// reported timings are recorded by the caller as separate context
// fields, never conflated with walls.

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import { parseSubst, sanitizeCmd, type Subst } from "./sanitize-cmd.ts";

function fail(msg: string): never {
  console.error(`measure: ${msg}`);
  process.exit(2);
}

const argv = process.argv.slice(2);
const sep = argv.indexOf("--");
if (sep < 0) fail("missing `--` before the command");
const opts = argv.slice(0, sep);
const cmd = argv.slice(sep + 1);
if (cmd.length === 0) fail("empty command");

function opt(name: string, dflt?: string): string {
  const i = opts.indexOf(`--${name}`);
  if (i < 0) {
    if (dflt !== undefined) return dflt;
    fail(`missing --${name}`);
  }
  return opts[i + 1];
}

const label = opt("label");
const runs = parseInt(opt("runs", "5"), 10);
const outPath = opt("out");
const targetNice = parseInt(opt("nice", "19"), 10);

// --subst NAME=/abs/prefix (repeatable): declared path prefixes rewrite
// to $NAME in the recorded cmd. Receipts are committed, so the recorded
// command carries the argument shape, never machine-local paths — see
// sanitize-cmd.ts for the full normalization (cwd → `.`; any remaining
// absolute path collapses to its final component).
const substs: Subst[] = [];
for (let i = 0; i < opts.length; i++) {
  if (opts[i] === "--subst") {
    const s = parseSubst(opts[i + 1] ?? "");
    if (s === null) fail(`bad --subst ${opts[i + 1]} (want NAME=/abs/prefix)`);
    substs.push(s);
    i++;
  }
}

// Absolute renice of this process; children inherit. Read back.
try {
  os.setPriority(0, targetNice);
} catch (e) {
  fail(`renice to ${targetNice} failed: ${e}`);
}
const stat = fs.readFileSync("/proc/self/stat", "utf8").split(" ");
const readbackNice = parseInt(stat[18], 10); // field 19, 0-indexed 18
if (Number.isNaN(readbackNice)) fail("could not read back niceness");

const loadBefore = fs
  .readFileSync("/proc/loadavg", "utf8")
  .trim()
  .split(" ")
  .slice(0, 3)
  .join(" ");

function once(): { wall_s: number; status: number } {
  const t0 = process.hrtime.bigint();
  const p = spawnSync(cmd[0], cmd.slice(1), { encoding: "utf8" });
  const t1 = process.hrtime.bigint();
  if (p.error) fail(`spawn failed: ${p.error}`);
  return { wall_s: Number(t1 - t0) / 1e9, status: p.status ?? -1 };
}

// warmup (uncounted), then measured runs; any nonzero exit voids the block
const warm = once();
if (warm.status !== 0) fail(`warmup exited ${warm.status}`);
const walls: number[] = [];
for (let i = 0; i < runs; i++) {
  const r = once();
  if (r.status !== 0) fail(`run ${i} exited ${r.status}`);
  walls.push(r.wall_s);
}
const sorted = [...walls].sort((a, b) => a - b);
const median =
  sorted.length % 2 === 1
    ? sorted[(sorted.length - 1) / 2]
    : (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]) / 2;

const cpuModel =
  fs
    .readFileSync("/proc/cpuinfo", "utf8")
    .split("\n")
    .find((l) => /model name|Model/i.test(l))
    ?.split(":")[1]
    ?.trim() ?? "unknown";

const row = {
  label,
  cmd: sanitizeCmd(cmd.join(" "), process.cwd(), substs),
  runs,
  walls_s: walls,
  median_s: median,
  nice_readback: readbackNice,
  loadavg_before: loadBefore,
  box: { arch: os.arch(), cpus: os.cpus().length, cpu: cpuModel },
  at: new Date().toISOString(),
};

let doc: { rows: unknown[] } = { rows: [] };
if (fs.existsSync(outPath)) {
  doc = JSON.parse(fs.readFileSync(outPath, "utf8"));
}
doc.rows.push(row);
fs.mkdirSync(outPath.substring(0, outPath.lastIndexOf("/")), {
  recursive: true,
});
fs.writeFileSync(outPath, JSON.stringify(doc, null, 2) + "\n");
console.log(
  `${label}: median ${median.toFixed(6)} s over ${runs} runs ` +
    `(nice ${readbackNice}, load ${loadBefore})`,
);
