// glc-recheck.ts — re-verify the supermartingale campaign's
// certificates from their sealed containers through the geolog
// fast-path checker, positives and mutants both, across both
// committed container lanes:
//
//   synthetic-chain lane (out/glc/) — the original containers on the
//     staged BLstack/BLCert reproduction vocabulary, kept checkable
//     forever (versioning law: a committed lane never dies). Checked
//     bare; the obligation-count gate compares the verdict line's
//     enumerated obligations against the manifest.
//
//   baked lane (out/glc-baked/) — the same certificates re-sealed on
//     the baked qc closure: the checker derives the vocabulary index
//     from the pinned theory-pair chain's own bytes and discharges the
//     containers' explicit claim rows through the exact-rational
//     ground provider (`glc-check --baked-cert --expect-claims
//     name=N,...`, expectations from the manifest's per-relation claim
//     counts). This lane's positives include the corpus's committed
//     FALSIFIER LISTINGS (sec3/mutants/**) sealed as containers: the
//     driver expects those to FAIL claim discharge — except the two
//     M3_byte_flip listings, which legitimately ACCEPT, because their
//     class was a container-byte tamper and a listing-level re-seal
//     erases it (the sealed-byte tamper class is covered by the baked
//     lane's own designed REFUSE mutants, m5–m9).
//
// The sealed container (.glc) is the canonical binary form: a signed
// geolog commit envelope whose body is the certificate's object rows —
// byte-identical in content to the committed reference listings (the
// producer proves this at emission by re-rendering the sealed body and
// comparing against the stored `.rows.txt`). The checker (glc-check,
// built from the pinned qtsl checkout's qtsl-geolog-provider crate)
// verifies container + envelope integrity, then ENUMERATES the
// Bernstein-cone obligations from the PBernCone nodes and re-derives
// each through the exact-rational ground provider — coefficient tables
// recomputed from (simplex, S, degree, polynomial, predicate), nothing
// trusted from the certificate.
//
// The obligation-count gate is load-bearing and lives HERE: node
// deletion leaves every surviving obligation true (mutant m4 witnesses
// it), so every positive's reported count must equal the committed
// manifest's. Structural-rule checking (ax/ande/ore formula
// equalities) is the reference lane's and the certified checker's
// stratum today — see GLC-CERT.md for the coverage split.
//
//   positives: every manifest positive must ACCEPT (exit 0 for the
//              batch) with obligations exactly k/k, k = manifest;
//   mutants:   every manifest mutant must land in its expected class —
//              REFUSE (integrity), FAIL (obligation discharge), or
//              ACCEPT_UNDERCOUNT (the count-gate witness: ACCEPTs with
//              fewer obligations than its base).
//
// Sets: out/glc/sec1 + out/glc/sec3 (the per-row published-table
// certificate sets, one .glc per stored listing) and — when present —
// the wave-B corpus out/glc/sec2-glc.tar.zst (4,105 containers,
// unpacked to build/, checked against the manifest packed beside them
// in the tarball).
//
// Trust posture: this driver is UNTRUSTED orchestration — it walks the
// committed artifacts, invokes the checker, and tabulates verdicts.
// The manifest states the producer's claims; the checker binary is the
// instrument that verifies the bytes against them.
//
// Contract: glc-check <file.glc>...; exit 0 = all ACCEPT; one verdict
// line per file on stdout:
//   `<name> ACCEPT commits=1 ops=N obligations allForwardDiffBernCoeffsSatisfyP=k/k`
// env GLC_CHECK names the checker binary (built from the pinned qtsl
// checkout: cargo build --release --bin glc-check in
// crates/qtsl-geolog-provider); there is no default — an unset path is
// an honest refusal, never a silent skip.
//
// Run: GLC_CHECK=... node --experimental-strip-types supermartingales/glc-recheck.ts

import { spawnSync } from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const REPO = path.dirname(HERE);
const GLCDIR = path.join(HERE, "out", "glc");
const MANIFEST = path.join(GLCDIR, "GLC-MANIFEST.json");
const SEC2_TAR = path.join(GLCDIR, "sec2-glc.tar.zst");
const SEC2_DIR = path.join(REPO, "build", "sec2-glc");
const PERSIST2D_DIR = path.join(HERE, "out", "glc-baked", "persist2d");

/** The run's machine identity for the timing receipt.  GLC_MACHINE_ID
 * names a row of the repository's machine registry (MACHINES.md) and
 * routes the published receipt to that machine's committed path,
 * out/glc/GLC-TIMINGS.<id>.json; unset, the identity is recorded as
 * "unknown" and the receipt keeps the legacy bare path (which the
 * table build consumes untagged, and only while no per-machine
 * receipt exists).  The id is never guessed from the host: uname
 * tells an architecture, not which registry row this box is.  A set
 * but unregistered id fails HERE, at startup, rather than after an
 * hour of checking (and rather than at the table build). */
const MACHINE_ID = ((): string => {
  const id = process.env.GLC_MACHINE_ID;
  if (id === undefined || id === "") return "unknown";
  const ids = new Set<string>();
  for (const line of fs
    .readFileSync(path.join(REPO, "MACHINES.md"), "utf8")
    .split("\n")) {
    const m = /^\|\s*`([A-Za-z0-9_-]+)`\s*\|/.exec(line);
    if (m !== null) ids.add(m[1]);
  }
  if (!ids.has(id)) {
    console.error(
      `glc-recheck: GLC_MACHINE_ID=${id} is not a machine id in MACHINES.md; ` +
        "add the machine's registry row first (or unset GLC_MACHINE_ID for " +
        "an untagged scratch run).",
    );
    process.exit(2);
  }
  return id;
})();

const GLC_CHECK = process.env.GLC_CHECK;
if (!GLC_CHECK) {
  console.error(
    "glc-recheck: GLC_CHECK is not set. Build the checker from the pinned qtsl " +
      "checkout (cargo build --release --bin glc-check in crates/qtsl-geolog-provider) " +
      "and pass its path: GLC_CHECK=/path/to/glc-check node --experimental-strip-types " +
      "supermartingales/glc-recheck.ts",
  );
  process.exit(2);
}

/** Batch the checker over files; returns per-path verdict lines. */
function runChecker(files: string[]): Map<string, string> {
  const out = new Map<string, string>();
  for (let i = 0; i < files.length; i += 400) {
    const chunk = files.slice(i, i + 400);
    const p = spawnSync(GLC_CHECK, chunk, { encoding: "utf8", maxBuffer: 64 << 20 });
    if (p.error) throw p.error;
    const lines = p.stdout.split("\n").filter((l) => l.length > 0);
    // One verdict line per input, in order (the checker's contract);
    // keyed positionally because verdict lines carry basenames and
    // basenames repeat across benchmark directories.
    if (lines.length !== chunk.length) {
      throw new Error(
        `checker emitted ${lines.length} verdict lines for ${chunk.length} inputs`,
      );
    }
    for (let j = 0; j < chunk.length; j++) out.set(chunk[j], lines[j]);
  }
  return out;
}

function obligationCount(line: string): number | null {
  // Not end-anchored: the checker's BL roster may grow relations after
  // the cone entry's position is no longer guaranteed line-final
  // (the F6 widening keeps it last today, but this parse must not
  // depend on that).
  const m = line.match(/allForwardDiffBernCoeffsSatisfyP=(\d+)\/(\d+)/);
  if (!m || m[1] !== m[2]) return null;
  return Number(m[1]);
}

function sec2Extracted(): string | null {
  if (!fs.existsSync(SEC2_TAR)) return null;
  if (!fs.existsSync(SEC2_DIR)) {
    fs.mkdirSync(SEC2_DIR, { recursive: true });
    const r = spawnSync("bash", ["-c", `zstd -d --stdout ${SEC2_TAR} | tar -x -C ${SEC2_DIR}`]);
    if (r.status !== 0) {
      fs.rmSync(SEC2_DIR, { recursive: true, force: true });
      throw new Error("need the zstd CLI + tar to unpack the sec2 containers");
    }
  }
  return SEC2_DIR;
}

let ok = true;

/** Per-benchmark wall-clock record — the receipt behind the chapter's
 * wall-clock comparison table's geolog-fast-path column.  Written to
 * a committed path under out/glc/ only under GLC_PUBLISH_TIMINGS=1
 * (a run whose numbers the published tables should carry) — the
 * per-machine GLC-TIMINGS.<id>.json when GLC_MACHINE_ID names a
 * registry machine, the legacy bare GLC-TIMINGS.json otherwise;
 * without the publish flag, to the gitignored build/ area, so casual
 * verification runs leave the tree clean instead of overwriting the
 * published receipt with run-local noise (timestamp, load, walls).
 * Group key: the certificate set's benchmark
 * (sec1/<bench>/rows, sec1/<bench>/rows-poly as <bench>__poly,
 * sec3/certs/<bench>, and the sec2 family directories). */
const groupWalls: Record<string, { n: number; wall_s: number }> = {};

function groupKey(rel: string): string {
  const parts = rel.split("/");
  if (parts[0] === "sec1") {
    return parts[2] === "rows-poly" ? `${parts[1]}__poly` : parts[1];
  }
  if (parts[0] === "sec3") return parts[2];
  return parts[0]; // sec2 family directories
}

/** Check one manifest's positives (batched per benchmark group, each
 * group timed around its own batches) + mutants (per-file). */
function checkManifest(label: string, dir: string, manifestPath: string) {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const t0 = Date.now();
  const groups = new Map<string, { file: string; obligations: number }[]>();
  for (const pos of manifest.positives) {
    const k = groupKey(pos.file);
    if (!groups.has(k)) groups.set(k, []);
    groups.get(k)!.push(pos);
  }
  let pass = 0;
  const detail: string[] = [];
  for (const [key, poss] of groups) {
    const files = poss.map((p) => path.join(dir, p.file));
    const g0 = process.hrtime.bigint();
    const verdicts = runChecker(files);
    const gWall = Number(process.hrtime.bigint() - g0) / 1e9;
    const prev = groupWalls[key] ?? { n: 0, wall_s: 0 };
    groupWalls[key] = {
      n: prev.n + poss.length,
      wall_s: prev.wall_s + gWall,
    };
    for (const pos of poss) {
      const line = verdicts.get(path.join(dir, pos.file))!;
      const n = obligationCount(line);
      const good = line.includes(" ACCEPT ") && n === pos.obligations;
      if (good) pass++;
      else {
        ok = false;
        detail.push(`FAIL positive ${pos.file} (expect ${pos.obligations} obligations): ${line}`);
      }
    }
  }
  for (const d of detail.slice(0, 10)) console.log(d);
  const wall = ((Date.now() - t0) / 1000).toFixed(2);
  console.log(
    `${label}: ${pass}/${manifest.positives.length} positives ACCEPT with exact obligation counts (${wall}s)`,
  );

  const base = new Map<string, number>(
    manifest.positives.map((p: { source: string; obligations: number }) => [
      p.source,
      p.obligations,
    ]),
  );
  let mutPass = 0;
  for (const mut of manifest.mutants ?? []) {
    const file = path.join(dir, mut.file);
    const p = spawnSync(GLC_CHECK, [file], { encoding: "utf8" });
    if (p.error) throw p.error;
    const line = p.stdout.trim();
    let good: boolean;
    if (mut.expect === "ACCEPT_UNDERCOUNT") {
      const n = obligationCount(line);
      const b = base.get(mut.base);
      good = p.status === 0 && n !== null && b !== undefined && n < b;
    } else {
      good = p.status !== 0 && line.includes(` ${mut.expect} `);
    }
    if (good) mutPass++;
    else {
      ok = false;
      console.log(`FAIL mutant ${mut.file} (expected ${mut.expect}): ${line.slice(0, 140)}`);
    }
  }
  const nMut = (manifest.mutants ?? []).length;
  if (nMut) console.log(`${label}: ${mutPass}/${nMut} mutants land in their expected classes`);
}

/** Write the per-benchmark timing receipt (only after a fully-green
 * run: a timing receipt for a run with wrong verdicts would be a
 * receipt about nothing). */
function writeTimings() {
  const binary = fs.readFileSync(GLC_CHECK!);
  const receipt = {
    date: new Date().toISOString(),
    what:
      "per-benchmark wall-clock of the geolog fast-path checker (glc-check) " +
      "over this campaign's sealed containers; serial batches (<=400 files " +
      "per checker process), one group at a time, walls measured around " +
      "each group's batches",
    machine: {
      id: MACHINE_ID,
      arch: os.arch(),
      cpu: os.cpus()[0]?.model ?? "unknown",
      loadavg_at_end: os.loadavg().map((x) => Math.round(x * 100) / 100),
      note: "single checker process at a time",
    },
    checker_binary_sha256: crypto.createHash("sha256").update(binary).digest("hex"),
    groups: Object.fromEntries(
      Object.entries(groupWalls)
        .sort(([a], [b]) => (a < b ? -1 : 1))
        .map(([k, v]) => [k, { n: v.n, wall_s: Math.round(v.wall_s * 1000) / 1000 }]),
    ),
  };
  const publish = process.env.GLC_PUBLISH_TIMINGS === "1";
  // a registered machine id routes the receipt to its per-machine
  // file name; "unknown" keeps the legacy bare name (and so can never
  // shadow a per-machine committed receipt).
  const fname =
    MACHINE_ID === "unknown"
      ? "GLC-TIMINGS.json"
      : `GLC-TIMINGS.${MACHINE_ID}.json`;
  const p = publish
    ? path.join(GLCDIR, fname)
    : path.join(REPO, "build", "supermartingales", fname);
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, JSON.stringify(receipt, null, 1) + "\n");
  console.log(`wrote ${path.relative(REPO, p)}`);
  if (!publish) {
    console.log(
      "(out-of-tree timing receipt; set GLC_PUBLISH_TIMINGS=1 to refresh the committed one)",
    );
  }
}

// ------------------------------------------------------------ baked lane

const BAKED_DIR = path.join(HERE, "out", "glc-baked");

/** `--expect-claims` argument from a manifest record's per-relation
 * claim counts. */
function claimSpec(claims: Record<string, number>): string {
  return Object.entries(claims)
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([k, v]) => `${k}=${v}`)
    .join(",");
}

/** All `name=h/n` pairs from a baked verdict line. */
function claimCounts(line: string): { holds: number; total: number } {
  let holds = 0;
  let total = 0;
  for (const m of line.matchAll(/([A-Za-z_]+)=(\d+)\/(\d+)/g)) {
    holds += Number(m[2]);
    total += Number(m[3]);
  }
  return { holds, total };
}

/** Check the baked-lane manifest: positives batched by identical claim
 * spec; falsifier listings and designed mutants per-file. */
function checkBakedManifest(dir: string, manifestPath: string) {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const t0 = Date.now();

  type Rec = { file: string; claims: Record<string, number> };
  const genuine: Rec[] = [];
  const falsifiers: Rec[] = [];
  for (const pos of manifest.positives) {
    (pos.file.startsWith("sec3/mutants/") ? falsifiers : genuine).push(pos);
  }

  // Genuine positives: exact ACCEPT under the manifest's claim counts,
  // batched per identical expectation (the expectation is an argument,
  // so a batch must share it).
  const bySpec = new Map<string, Rec[]>();
  for (const pos of genuine) {
    const spec = claimSpec(pos.claims);
    if (!bySpec.has(spec)) bySpec.set(spec, []);
    bySpec.get(spec)!.push(pos);
  }
  let pass = 0;
  for (const [spec, poss] of bySpec) {
    for (let i = 0; i < poss.length; i += 400) {
      const chunk = poss.slice(i, i + 400);
      const files = chunk.map((p) => path.join(dir, p.file));
      const p = spawnSync(GLC_CHECK, ["--baked-cert", "--expect-claims", spec, ...files], {
        encoding: "utf8",
        maxBuffer: 64 << 20,
      });
      if (p.error) throw p.error;
      const lines = p.stdout.split("\n").filter((l) => l.length > 0);
      if (lines.length !== chunk.length) {
        throw new Error(`checker emitted ${lines.length} verdict lines for ${chunk.length} inputs`);
      }
      for (let j = 0; j < chunk.length; j++) {
        const good = p.status === 0 && lines[j].includes(" ACCEPT ");
        if (good) pass++;
        else {
          ok = false;
          console.log(`FAIL positive ${chunk[j].file} (expect ${spec}): ${lines[j].slice(0, 160)}`);
        }
      }
    }
  }
  console.log(
    `baked: ${pass}/${genuine.length} genuine positives ACCEPT with exact claim counts`,
  );

  // Corpus falsifier listings: FAIL with zero holding claims — except
  // the M3_byte_flip pair, ACCEPT by re-scope (see the header).
  let falsPass = 0;
  for (const rec of falsifiers) {
    const file = path.join(dir, rec.file);
    const p = spawnSync(GLC_CHECK, ["--baked-cert", "--expect-claims", claimSpec(rec.claims), file], {
      encoding: "utf8",
      maxBuffer: 16 << 20,
    });
    if (p.error) throw p.error;
    const line = p.stdout.trim();
    const c = claimCounts(line);
    const good = rec.file.includes("M3_byte_flip")
      ? p.status === 0 && line.includes(" ACCEPT ")
      : p.status !== 0 && line.includes(" FAIL ") && c.holds === 0;
    if (good) falsPass++;
    else {
      ok = false;
      console.log(`FAIL falsifier-listing ${rec.file}: ${line.slice(0, 160)}`);
    }
  }
  console.log(
    `baked: ${falsPass}/${falsifiers.length} corpus falsifier listings in class ` +
      `(FAIL 0/n; the two M3_byte_flip re-seals ACCEPT by design)`,
  );

  // Designed mutants on the sealed bytes.
  const base = new Map<string, Record<string, number>>(
    manifest.positives.map((p: { source: string; claims: Record<string, number> }) => [
      p.source,
      p.claims,
    ]),
  );
  let mutPass = 0;
  for (const mut of manifest.mutants ?? []) {
    const file = path.join(dir, mut.file);
    let good: boolean;
    let line: string;
    if (mut.expect === "ACCEPT_UNDERCOUNT") {
      // Bare baked run: ACCEPTs, but some relation tallies fewer
      // claims than its base — the count gate's witness.
      const p = spawnSync(GLC_CHECK, ["--baked-cert", file], { encoding: "utf8" });
      if (p.error) throw p.error;
      line = p.stdout.trim();
      const b = base.get(mut.base);
      const c = claimCounts(line);
      good =
        p.status === 0 &&
        b !== undefined &&
        c.holds === c.total &&
        c.holds < Object.values(b).reduce((s, n) => s + n, 0);
    } else {
      const b = base.get(mut.base);
      const p = spawnSync(
        GLC_CHECK,
        ["--baked-cert", "--expect-claims", claimSpec(b ?? {}), file],
        { encoding: "utf8" },
      );
      if (p.error) throw p.error;
      line = p.stdout.trim();
      good = p.status !== 0 && line.includes(` ${mut.expect} `);
    }
    if (good) mutPass++;
    else {
      ok = false;
      console.log(`FAIL mutant ${mut.file} (expected ${mut.expect}): ${line.slice(0, 160)}`);
    }
  }
  const nMut = (manifest.mutants ?? []).length;
  if (nMut) console.log(`baked: ${mutPass}/${nMut} designed mutants land in their expected classes`);
  const wall = ((Date.now() - t0) / 1000).toFixed(2);
  console.log(`baked lane done (${wall}s)`);
}

function main(): number {
  checkManifest("sec1+sec3", GLCDIR, MANIFEST);
  const sec2 = sec2Extracted();
  if (sec2) {
    checkManifest("sec2", sec2, path.join(sec2, "GLC-MANIFEST.json"));
  }
  const bakedManifest = path.join(BAKED_DIR, "GLC-MANIFEST.json");
  if (fs.existsSync(bakedManifest)) {
    checkBakedManifest(BAKED_DIR, bakedManifest);
  }
  // The persist_2d re-conduct lane (out/glc-baked/persist2d): both
  // twins as sealed shared-subtree containers — the uniform twin whole,
  // the gaussian twin as four sorted-quarter containers (its merged
  // forest exceeds the envelope's per-commit body cap; the manifest's
  // `split` note carries the partition) — with the designed-mutant
  // battery beside them. Same manifest schema, same checker contract.
  const p2dManifest = path.join(PERSIST2D_DIR, "MANIFEST.json");
  if (fs.existsSync(p2dManifest)) {
    checkBakedManifest(PERSIST2D_DIR, p2dManifest);
  }
  if (ok) writeTimings();
  console.log(ok ? "OK" : "WRONG VERDICTS above");
  return ok ? 0 : 1;
}

process.exit(main());
