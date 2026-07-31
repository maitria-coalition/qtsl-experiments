/**
 * recheck.ts — re-verify the published-table campaign's shipped
 * certificates through the reference checker lane of the pinned qtsl
 * checkout (`checkers/supermartingales`: `read_rows.ts` reads the stored
 * listing form directly, `bl_check.ts` re-derives every obligation in
 * exact arithmetic; this script is orchestration only and carries no
 * checking logic of its own).
 *
 * Positive sets must ACCEPT wholesale; the mutant battery must REFUSE
 * wholesale. Receipts land in out/recheck_receipts.json. Exit is nonzero
 * on any wrong verdict, so this doubles as the suite's regression gate.
 *
 * The certificates are stored as geolog object rows in the reference
 * listing notation (`*.rows.txt`; see MANIFEST.jsonl for the per-file
 * pins, including the sha256 of the retired byte-format sources each
 * listing was derived from — listing headers and manifest `source`
 * fields keep the authoring tree's relative layout, in which the
 * wave-A directory shipped here as `out/sec3/` was named `wave1a/`).
 * The 22 `M3_byte_flip`/`M4_truncate`-class
 * mutants of the retired byte format have no row twin — their refusal
 * duty was byte-format-level and is documented in the manifest
 * (`disposition: "byte-negative"`); the reader-level malformed-input
 * battery lives with the checker (`test_read_rows.ts`).
 *
 * The full campaign corpus (28,569 certificates, including the
 * persist_2d twin reconstructions) lives in the authoring workshop; this
 * repository ships the per-row certificate sets behind the published
 * table. Checker location: $QTSL_CHECKERS, or ../build/qtsl/checkers/
 * supermartingales (the Makefile's pinned checkout).
 *
 *   node --experimental-strip-types supermartingales/recheck.ts
 */

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(new URL(import.meta.url).pathname);
const REPO = path.dirname(HERE);
const CHECKERS =
  process.env.QTSL_CHECKERS ?? path.join(REPO, "build", "qtsl", "checkers", "supermartingales");
const SEC1 = path.join(HERE, "out", "sec1");
const SEC3 = path.join(HERE, "out", "sec3");
const SEC2_TAR = path.join(HERE, "out", "sec2", "rows-certs.tar.zst");
const SEC2_DIR = path.join(REPO, "build", "sec2-certs");
const OUT = path.join(HERE, "out", "recheck_receipts.json");

function sec2Extracted(): string {
  if (!fs.existsSync(SEC2_DIR)) {
    fs.mkdirSync(SEC2_DIR, { recursive: true });
    const r = spawnSync("bash", ["-c", `zstd -d --stdout ${SEC2_TAR} | tar -x -C ${SEC2_DIR}`]);
    if (r.status !== 0) {
      fs.rmSync(SEC2_DIR, { recursive: true, force: true });
      throw new Error("need the zstd CLI + tar to unpack the sec2 certificates");
    }
  }
  return SEC2_DIR;
}

function walkRows(d: string): string[] {
  const out: string[] = [];
  for (const e of fs.readdirSync(d, { withFileTypes: true, recursive: true }))
    if (e.isFile() && e.name.endsWith(".rows.txt")) out.push(path.join(e.parentPath, e.name));
  return out.sort();
}

/** Run the reference lane over `files`; verdict line per file. */
function checkFiles(files: string[]): Map<string, string> {
  const out = new Map<string, string>();
  const stack = path.join(CHECKERS, "theories", "BLstack.rows.txt");
  const cert = path.join(CHECKERS, "theories", "BLCert.rows.txt");
  for (let i = 0; i < files.length; i += 400) {
    const chunk = files.slice(i, i + 400);
    const r = spawnSync(
      process.execPath,
      [
        "--experimental-strip-types",
        path.join(CHECKERS, "check_rows.ts"),
        "--stack", stack,
        "--cert", cert,
        ...chunk,
      ],
      { encoding: "utf8", maxBuffer: 64 << 20 },
    );
    if (r.error) throw r.error;
    for (const line of r.stdout.split("\n")) {
      if (!line) continue;
      const sp = line.indexOf(" ");
      const rest = line.slice(sp + 1);
      const sp2 = rest.indexOf(" ");
      const p = sp2 < 0 ? rest : rest.slice(0, sp2);
      out.set(p, line.slice(0, sp) + (sp2 < 0 ? "" : " " + rest.slice(sp2 + 1)));
    }
  }
  for (const f of files) if (!out.has(f)) out.set(f, "CRASH no verdict line");
  return out;
}

type SetResult = {
  n: number;
  expect: string;
  accepts: number;
  refusals: number;
  errors: number;
  wall_s: number;
  detail: [string, string][];
};

const results: { meta: Record<string, string>; sets: Record<string, SetResult> } = {
  meta: {
    date: new Date().toISOString(),
    checker:
      "qtsl checkers/supermartingales rows lane — read_rows.ts + bl_check.ts (pinned by the Makefile)",
  },
  sets: {},
};

let ok = true;

function checkSet(name: string, files: string[], expect: "accept" | "refuse" | "recorded") {
  const t0 = Date.now();
  const verdicts = checkFiles(files);
  let acc = 0, rej = 0, err = 0;
  const detail: [string, string][] = [];
  for (const f of files) {
    const v = verdicts.get(f)!;
    if (v.startsWith("ACCEPT")) {
      acc += 1;
      if (expect === "refuse") detail.push([path.basename(f), "ACCEPTED but battery expects refusal"]);
    } else if (v.startsWith("REJECT") || v.startsWith("REFUSED")) {
      rej += 1;
      if (expect === "accept") detail.push([path.basename(f), v.slice(0, 240)]);
    } else {
      err += 1;
      detail.push([path.basename(f), v.slice(0, 240)]);
    }
  }
  const wall = (Date.now() - t0) / 1000;
  if (expect === "accept" && (rej || err)) ok = false;
  if (expect === "refuse" && acc) ok = false;
  // expect == "recorded": verdicts are receipts, never gate failures
  results.sets[name] = {
    n: files.length, expect, accepts: acc, refusals: rej, errors: err,
    wall_s: Math.round(wall * 1000) / 1000, detail: detail.slice(0, 10),
  };
  console.log(`${name}: ${acc} accept / ${rej} refuse / ${err} err over ${files.length} (expect ${expect}) ${wall.toFixed(2)}s`);
}

function main(): number {
  for (const bench of fs.readdirSync(SEC1).sort()) {
    const d = path.join(SEC1, bench);
    if (!fs.statSync(d).isDirectory()) continue;
    for (const [sub, suffix] of [
      ["rows", ""],
      ["rows-poly", "__poly"],
    ] as const) {
      const sd = path.join(d, sub);
      if (fs.existsSync(sd)) checkSet(`${bench}${suffix}`, walkRows(sd), "accept");
    }
  }
  const certs = path.join(SEC3, "certs");
  for (const bench of fs.readdirSync(certs).sort())
    checkSet(bench, walkRows(path.join(certs, bench)), "accept");
  const mutants = path.join(SEC3, "mutants");
  if (fs.existsSync(mutants)) {
    const allm = walkRows(mutants);
    // M3_byte_flip is a recorded-only oracle in the battery's own
    // convention (a random byte flip may land on a don't-care byte and
    // legitimately still accept); it is reported, never gated on.
    checkSet("sec3-mutants", allm.filter((f) => !f.includes("M3_byte_flip")), "refuse");
    checkSet("sec3-mutants-recorded-only", allm.filter((f) => f.includes("M3_byte_flip")), "recorded");
  }
  const sec2 = sec2Extracted();
  for (const bench of fs.readdirSync(sec2).sort()) {
    const d = path.join(sec2, bench);
    if (fs.statSync(d).isDirectory()) checkSet(`sec2/${bench}`, walkRows(d), "accept");
  }
  fs.writeFileSync(OUT, JSON.stringify(results, null, 1));
  console.log(ok ? "OK" : `WRONG VERDICTS — see ${OUT}`);
  return ok ? 0 : 1;
}

process.exitCode = main();
