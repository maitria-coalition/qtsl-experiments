#!/usr/bin/env node
/* gen_lean_route.ts — re-derive the Lean-route section's numbers from
 * the repository's receipt files.
 *
 * Usage:  node --experimental-strip-types tables/gen_lean_route.ts [--no-verify]
 *
 * Inputs (paths relative to the repository root):
 *
 *   lean-route/out/results.json        timed-run receipts (one row per
 *                                      certificate x route x pass, with
 *                                      host identity embedded)
 *   lean-route/fixtures/MANIFEST.tsv   per-rung structural counts
 *   lean-route/species.json            campaign species census (byte
 *                                      ranges + fragment status)
 *
 * Outputs, into chapter/generated/:
 *
 *   lean-route-table.tex       the ladder table (one row per rung)
 *   lean-route-points.dat      accepted compiled-route points (pgfplots)
 *   lean-route-censored.dat    budget-censored rungs, at the budget line
 *   lean-route-glc-points.dat  fast-path engine route, accepted rungs
 *   lean-route-qtslab-points.dat    certified-checker route, accepted
 *   lean-route-qtslab-censored.dat  certified route, cut at budget
 *   lean-route-fit.tex         \providecommand macros: fitted exponent,
 *                              CI, r^2, admission threshold, host string,
 *                              per-route counts for the figure caption
 *   lean-route-admission.tex   the species admission table
 *
 * The two production-route point files re-derive from their own receipt
 * files (lean-route/out/glc-times.json, lean-route/out/qtslab-times.json
 * — same row discipline as results.json: one row per certificate x pass,
 * host and binary identity embedded; tools/time-routes.py re-measures).
 *
 * and, into lean-route/:
 *
 *   ADMISSION.md               the same admission table as markdown (the
 *                              campaign-runner-facing form)
 *
 * Every number in every emitted file is computed from the receipt files;
 * nothing numeric is hard-coded here.  The PIN block at the bottom is
 * the regression pin (same discipline as gen_tables.ts): the fitted
 * exponent and threshold are re-checked against their pinned values so
 * that a receipts regeneration that moves them is noticed, never silent.
 */

import { readFileSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const GEN = path.join(ROOT, "chapter", "generated");
const noVerify = process.argv.includes("--no-verify");

function fail(msg: string): never {
  console.error(`gen_lean_route: ${msg}`);
  process.exit(1);
}

// ---------------------------------------------------------------- inputs

type Row = {
  name: string;
  container_bytes: number;
  route: "compiled" | "kernel";
  pass: number;
  budget_s: number;
  outcome: string;
  t_e2e_ms?: number | null;
  t_wall_s?: number | null;
  maxrss_kb?: number | null;
  host: { host: string; cpu: string; threads: number; mem_gib: number; arch: string };
};

const rows: Row[] = JSON.parse(
  readFileSync(path.join(ROOT, "lean-route", "out", "results.json"), "utf8"),
);
if (rows.length === 0) fail("results.json is empty");

const hosts = new Set(rows.map((r) => `${r.host.host} (${r.host.cpu})`));
if (hosts.size !== 1)
  fail(`receipts span ${hosts.size} hosts — the ladder is a one-box experiment`);
const host = rows[0].host;
const budgetS = rows[0].budget_s;
if (!rows.every((r) => r.budget_s === budgetS)) fail("mixed budgets in receipts");

const manifest = new Map<string, { ops: number; nodes: number; species: string }>();
{
  const lines = readFileSync(
    path.join(ROOT, "lean-route", "fixtures", "MANIFEST.tsv"),
    "utf8",
  ).trimEnd().split("\n");
  const cols = lines[0].split("\t");
  const idx = (c: string) => {
    const i = cols.indexOf(c);
    if (i < 0) fail(`MANIFEST.tsv lacks column ${c}`);
    return i;
  };
  const [iName, iOps, iNodes, iSpec] = [
    idx("name"), idx("n_ops"), idx("n_proof_nodes"), idx("species_family"),
  ];
  for (const l of lines.slice(1)) {
    const f = l.split("\t");
    manifest.set(f[iName], {
      ops: Number(f[iOps]), nodes: Number(f[iNodes]), species: f[iSpec],
    });
  }
}

const speciesCensus = JSON.parse(
  readFileSync(path.join(ROOT, "lean-route", "species.json"), "utf8"),
);

// ------------------------------------------------------------ aggregate

type Rung = {
  name: string;
  bytes: number;
  ops: number;
  nodes: number;
  compiled:
    | { kind: "accept"; medianMs: number; passes: number[] }
    | { kind: "timeout" }
    | { kind: "refuse" }
    | { kind: "error" };
  kernel:
    | { kind: "accept"; wallS: number; rssKb: number }
    | { kind: "timeout"; rssKb: number | null }
    | { kind: "oom"; rssKb: number | null }
    | { kind: "refuse" }
    | { kind: "error"; rssKb: number | null }
    | { kind: "not_attempted" };
};

const byName = new Map<string, Row[]>();
for (const r of rows) {
  byName.set(r.name, [...(byName.get(r.name) ?? []), r]);
}

const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
};

const rungs: Rung[] = [];
for (const [name, rs] of byName) {
  const m = manifest.get(name);
  if (!m) fail(`receipt rung ${name} missing from MANIFEST.tsv`);
  const comp = rs.filter((r) => r.route === "compiled");
  const kern = rs.filter((r) => r.route === "kernel");
  const bytes = rs[0].container_bytes;

  let compiled: Rung["compiled"];
  const accepts = comp.filter((r) => r.outcome === "accept");
  if (accepts.length > 0) {
    const passes = accepts.map((r) => r.t_e2e_ms!) as number[];
    compiled = { kind: "accept", medianMs: median(passes), passes };
  } else if (comp.some((r) => r.outcome === "timeout")) compiled = { kind: "timeout" };
  else if (comp.some((r) => r.outcome === "refuse")) compiled = { kind: "refuse" };
  else compiled = { kind: "error" };

  let kernel: Rung["kernel"] = { kind: "not_attempted" };
  if (kern.length > 0) {
    const k = kern[0];
    const rssKb = (k.maxrss_kb ?? null) as number | null;
    if (k.outcome === "accept")
      kernel = { kind: "accept", wallS: k.t_wall_s!, rssKb: rssKb! };
    else if (k.outcome === "timeout") kernel = { kind: "timeout", rssKb };
    else if (k.outcome === "oom") kernel = { kind: "oom", rssKb };
    else if (k.outcome === "not_attempted") kernel = { kind: "not_attempted" };
    else if (compiled.kind === "refuse") kernel = { kind: "refuse" };
    else kernel = { kind: "error", rssKb };
  }
  rungs.push({ name, bytes, ops: m.ops, nodes: m.nodes, compiled, kernel });
}
rungs.sort((a, b) => a.bytes - b.bytes);

// ------------------------------------------------------------------ fit
// OLS on (ln bytes, ln seconds) over accepted compiled rungs.

const pts = rungs
  .filter((r) => r.compiled.kind === "accept")
  .map((r) => ({
    x: Math.log(r.bytes),
    y: Math.log((r.compiled as { medianMs: number }).medianMs / 1000),
    bytes: r.bytes,
    tS: (r.compiled as { medianMs: number }).medianMs / 1000,
  }));
if (pts.length < 4) fail(`only ${pts.length} accepted points — no stable fit`);

const n = pts.length;
const mx = pts.reduce((s, p) => s + p.x, 0) / n;
const my = pts.reduce((s, p) => s + p.y, 0) / n;
const sxx = pts.reduce((s, p) => s + (p.x - mx) ** 2, 0);
const sxy = pts.reduce((s, p) => s + (p.x - mx) * (p.y - my), 0);
const b = sxy / sxx;                    // the exponent
const a = my - b * mx;                  // ln coefficient
const yhat = (x: number) => a + b * x;
const ssRes = pts.reduce((s, p) => s + (p.y - yhat(p.x)) ** 2, 0);
const ssTot = pts.reduce((s, p) => s + (p.y - my) ** 2, 0);
const r2 = 1 - ssRes / ssTot;
const seB = Math.sqrt(ssRes / (n - 2) / sxx);
// two-sided 95% t critical values, df = n-2
const T95: Record<number, number> = {
  2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365,
  8: 2.306, 9: 2.262, 10: 2.228, 11: 2.201, 12: 2.179, 13: 2.160,
};
const tCrit = T95[n - 2] ?? 2.0;
const ciLo = b - tCrit * seB;
const ciHi = b + tCrit * seB;

const predictS = (bytes: number) => Math.exp(a + b * Math.log(bytes));
// admission threshold: bytes at which predicted time = budget
const thresholdBytes = Math.exp((Math.log(budgetS) - a) / b);

// regressor comparison: the same OLS against each of the three natural
// size units, so the choice of container bytes is shown, not assumed.
function r2Of(xs: number[], ys: number[]): number {
  const k = xs.length;
  const mx2 = xs.reduce((s, v) => s + v, 0) / k;
  const my2 = ys.reduce((s, v) => s + v, 0) / k;
  const sxx2 = xs.reduce((s, v) => s + (v - mx2) ** 2, 0);
  const sxy2 = xs.reduce((s, v, i) => s + (v - mx2) * (ys[i] - my2), 0);
  const b2 = sxy2 / sxx2;
  const res = ys.reduce((s, v, i) => s + (v - (my2 + b2 * (xs[i] - mx2))) ** 2, 0);
  const tot = ys.reduce((s, v) => s + (v - my2) ** 2, 0);
  return 1 - res / tot;
}
const acceptRungs = rungs.filter((r) => r.compiled.kind === "accept");
const lnT = acceptRungs.map((r) => Math.log((r.compiled as { medianMs: number }).medianMs / 1000));
const r2Bytes = r2Of(acceptRungs.map((r) => Math.log(r.bytes)), lnT);
const r2Rows = r2Of(acceptRungs.map((r) => Math.log(r.ops)), lnT);
const r2Nodes = r2Of(acceptRungs.map((r) => Math.log(r.nodes)), lnT);

// --------------------------------------------------------------- format

const esc = (s: string) => s.replace(/_/g, "\\_");
const tt = (s: string) => `\\texttt{${esc(s)}}`;
const num = (x: number) =>
  x.toLocaleString("en-US").replace(/,/g, "{,}");

const sig2 = (x: number) => String(Number(x.toPrecision(2)));
function humaneS(s: number): string {
  if (s < 0.0995) return `${sig2(s * 1000)}\\,ms`;
  if (s < 99.5) return `${sig2(s)}\\,s`;
  if (s < 5970) return `${sig2(s / 60)}\\,min`;
  if (s < 143000) return `${sig2(s / 3600)}\\,h`;
  if (s < 4e6) return `${sig2(s / 86400)}\\,d`;
  const yr = s / 3.156e7;
  if (yr < 1000) return `${sig2(yr)}\\,yr`;
  return `$\\sim$$10^{${Math.round(Math.log10(yr))}}$\\,yr`;
}
function humaneMd(s: number): string {
  const yr = s / 3.156e7;
  if (yr >= 1000) return `~10^${Math.round(Math.log10(yr))} yr`;
  return humaneS(s).replace("\\,", " ").replace(/\\/g, "");
}
function gib(kb: number): string {
  const g = kb / 2 ** 20;
  return g >= 99.5 ? g.toFixed(0) : g.toPrecision(2);
}

// ------------------------------------------------------ ladder table tex

const EMDASH = "---"; // typeset as an em-dash by TeX; the timeout mark
const rowsTex = rungs.map((r) => {
  const comp =
    r.compiled.kind === "accept" ? humaneS(r.compiled.medianMs / 1000)
    : r.compiled.kind === "timeout" ? EMDASH
    : r.compiled.kind === "refuse" ? "\\emph{o.o.f.}"
    : "\\emph{error}";
  const kern =
    r.kernel.kind === "accept" ? humaneS(r.kernel.wallS)
    : r.kernel.kind === "timeout" ? EMDASH
    : r.kernel.kind === "oom" ? "\\emph{o.o.m.}"
    : r.kernel.kind === "refuse" ? "\\emph{o.o.f.}"
    : r.kernel.kind === "not_attempted" ? "n.a."
    : "\\emph{error}";
  const rss =
    r.kernel.kind === "accept" ? `${gib(r.kernel.rssKb)}\\,GiB`
    : (r.kernel.kind === "timeout" || r.kernel.kind === "oom" || r.kernel.kind === "error") &&
      r.kernel.rssKb != null
    ? `$\\geq$${gib(r.kernel.rssKb)}\\,GiB`
    : "";
  return `${tt(r.name)} & ${num(r.bytes)} & ${num(r.nodes)} & ${comp} & ${kern} & ${rss} \\\\`;
});

// the manifest carries certificate-row counts too; the table shows the
// two units the section's prose uses (bytes: the fitted regressor;
// proof nodes: the certificate's own size measure).
writeFileSync(
  path.join(GEN, "lean-route-table.tex"),
  `{\\setlength{\\tabcolsep}{4pt}%
\\begin{tabular}{@{}lrrrrr@{}}
\\toprule
certificate & bytes & proof nodes & compiled & kernel & kernel peak \\\\
\\midrule
${rowsTex.join("\n")}
\\bottomrule
\\end{tabular}}
`,
);

// -------------------------------------------------------- pgfplots data

writeFileSync(
  path.join(GEN, "lean-route-points.dat"),
  "bytes tseconds\n" + pts.map((p) => `${p.bytes} ${p.tS}`).join("\n") + "\n",
);
writeFileSync(
  path.join(GEN, "lean-route-censored.dat"),
  "bytes tseconds\n" +
    rungs
      .filter((r) => r.compiled.kind === "timeout")
      .map((r) => `${r.bytes} ${budgetS}`)
      .join("\n") +
    "\n",
);

// -------------------------------------- production-route receipts (figure)
// The three-route figure adds the two production checkers on the same
// ladder: the fast-path engine route on the staged-chain sealed
// containers (out/glc), and the certified checker on the
// content-resolved mints of the same certificates (out/qtslab;
// tools/mint-qtslab-vintage.py states the container-only transform).
// Same aggregation as the compiled route: median of accepted passes,
// per rung; a budget-cut rung is drawn at the budget line; a rung the
// checker classifies outside its fragment gets no point at all (the
// caption says which, and why that is a scope datum, not a defect).

type RouteRow = {
  name: string;
  container_bytes: number;
  route: "glc" | "qtslab";
  pass: number;
  budget_s: number;
  outcome: string;
  t_e2e_ms?: number | null;
  verdict?: string | null;
  host: { host: string; cpu: string };
};

type RouteAgg = {
  accepted: { name: string; bytes: number; medianS: number }[];
  censored: { name: string; bytes: number }[];
  oof: string[];
  budgetS: number;
  host: { host: string; cpu: string };
};

function loadRoute(file: string, route: "glc" | "qtslab"): RouteAgg {
  const rr: RouteRow[] = JSON.parse(
    readFileSync(path.join(ROOT, "lean-route", "out", file), "utf8"),
  );
  if (rr.length === 0) fail(`${file} is empty`);
  const hosts2 = new Set(rr.map((r) => r.host.host));
  if (hosts2.size !== 1) fail(`${file}: receipts span ${hosts2.size} hosts`);
  const rBudget = rr[0].budget_s;
  if (!rr.every((r) => r.route === route && r.budget_s === rBudget))
    fail(`${file}: mixed routes or budgets`);
  const agg: RouteAgg = {
    accepted: [], censored: [], oof: [], budgetS: rBudget, host: rr[0].host,
  };
  const names = new Map<string, RouteRow[]>();
  for (const r of rr) names.set(r.name, [...(names.get(r.name) ?? []), r]);
  for (const [name, rs] of names) {
    if (!manifest.has(name)) fail(`${file}: rung ${name} missing from MANIFEST.tsv`);
    const bytes = rs[0].container_bytes;
    const acc = rs.filter((r) => r.outcome === "accept");
    if (acc.length > 0)
      agg.accepted.push({ name, bytes, medianS: median(acc.map((r) => r.t_e2e_ms!)) / 1000 });
    else if (rs.some((r) => r.outcome === "timeout")) agg.censored.push({ name, bytes });
    else if (rs.some((r) => r.outcome === "unsupported")) agg.oof.push(name);
    else fail(`${file}: rung ${name} has outcome ${rs[0].outcome} (${rs[0].verdict}) — not a class this figure knows`);
  }
  agg.accepted.sort((x, y) => x.bytes - y.bytes);
  agg.censored.sort((x, y) => x.bytes - y.bytes);
  return agg;
}

const glc = loadRoute("glc-times.json", "glc");
const qtslab = loadRoute("qtslab-times.json", "qtslab");
if (glc.oof.length + glc.censored.length > 0)
  fail(`glc route: unexpected non-accepts (${[...glc.oof, ...glc.censored.map((c) => c.name)].join(", ")}) — the staged lane is 15/15 by its own battery`);
if (qtslab.budgetS !== budgetS)
  fail(`qtslab budget ${qtslab.budgetS}s != Lean-route budget ${budgetS}s — the figure draws one budget line`);
if (glc.host.host !== qtslab.host.host)
  fail(`production routes span hosts (${glc.host.host} vs ${qtslab.host.host}) — the footnote states one Arm box`);
// The figure caption and the fragment subsection state the certified
// route's out-of-fragment set in prose; if a re-measurement moves it,
// this check makes the stale sentence a build failure, never a silent lie.
const OOF_STATED = ["enclosure_v1_golden_cos6"];
if (JSON.stringify([...qtslab.oof].sort()) !== JSON.stringify(OOF_STATED))
  fail(
    `qtslab out-of-fragment set is [${qtslab.oof.join(", ")}] but the section prose ` +
    `states [${OOF_STATED.join(", ")}] — update the figure caption, the fragment ` +
    `subsection, and OOF_STATED together`,
  );

// same log-log OLS as the Lean fit, as a helper (exponent + 95% CI + r^2)
function olsLogLog(ps: { bytes: number; medianS: number }[]) {
  const xs = ps.map((p) => Math.log(p.bytes));
  const ys = ps.map((p) => Math.log(p.medianS));
  const k = xs.length;
  const mx2 = xs.reduce((s, v) => s + v, 0) / k;
  const my2 = ys.reduce((s, v) => s + v, 0) / k;
  const sxx2 = xs.reduce((s, v) => s + (v - mx2) ** 2, 0);
  const sxy2 = xs.reduce((s, v, i) => s + (v - mx2) * (ys[i] - my2), 0);
  const slope = sxy2 / sxx2;
  const icept = my2 - slope * mx2;
  const res = ys.reduce((s, v, i) => s + (v - (icept + slope * xs[i])) ** 2, 0);
  const tot = ys.reduce((s, v) => s + (v - my2) ** 2, 0);
  const se = Math.sqrt(res / (k - 2) / sxx2);
  const t = T95[k - 2] ?? 2.0;
  return { a: icept, b: slope, ciLo: slope - t * se, ciHi: slope + t * se,
           r2: 1 - res / tot, n: k };
}
const glcFit = olsLogLog(glc.accepted);
if (qtslab.accepted.length < 4)
  fail(
    `qtslab route has only ${qtslab.accepted.length} accepted rungs — the figure ` +
    `draws a green fitted line (\\QtslabRouteFit*); remove it from the figure ` +
    `and this gate together if a re-measurement shrinks the series`,
  );
const qtslabFit = olsLogLog(qtslab.accepted);

const datOf = (ps: { bytes: number; medianS: number }[]) =>
  "bytes tseconds\n" + ps.map((p) => `${p.bytes} ${p.medianS}`).join("\n") + "\n";
writeFileSync(path.join(GEN, "lean-route-glc-points.dat"), datOf(glc.accepted));
writeFileSync(path.join(GEN, "lean-route-qtslab-points.dat"), datOf(qtslab.accepted));
writeFileSync(
  path.join(GEN, "lean-route-qtslab-censored.dat"),
  "bytes tseconds\n" +
    qtslab.censored.map((c) => `${c.bytes} ${budgetS}`).join("\n") + "\n",
);

// ------------------------------------------------------------ fit macros

const fitLine = (bytes: number) => predictS(bytes);
writeFileSync(
  path.join(GEN, "lean-route-fit.tex"),
  `\\providecommand{\\LeanRouteExponent}{${b.toFixed(2)}}
\\providecommand{\\LeanRouteExponentLo}{${ciLo.toFixed(2)}}
\\providecommand{\\LeanRouteExponentHi}{${ciHi.toFixed(2)}}
\\providecommand{\\LeanRouteRSquared}{${r2.toFixed(3)}}
\\providecommand{\\LeanRoutePointCount}{${n}}
\\providecommand{\\LeanRouteBudgetMin}{${Math.round(budgetS / 60)}}
\\providecommand{\\LeanRouteThresholdKB}{${(thresholdBytes / 1000).toPrecision(2)}}
\\providecommand{\\LeanRouteThresholdBytes}{${Math.round(thresholdBytes)}}
\\providecommand{\\LeanRouteHost}{${esc(host.cpu)}, ${host.threads} threads, ${Math.round(host.mem_gib)}\\,GiB}
\\providecommand{\\LeanRouteFitA}{${a.toFixed(6)}}
\\providecommand{\\LeanRouteFitB}{${b.toFixed(6)}}
\\providecommand{\\LeanRouteFitDomainLo}{${pts[0].bytes}}
\\providecommand{\\LeanRouteFitDomainHi}{${pts[pts.length - 1].bytes}}
\\providecommand{\\LeanRouteRSqBytes}{${r2Bytes.toFixed(3)}}
\\providecommand{\\LeanRouteRSqRows}{${r2Rows.toFixed(3)}}
\\providecommand{\\LeanRouteRSqNodes}{${r2Nodes.toFixed(3)}}
\\providecommand{\\LeanRouteCensoredCount}{${rungs.filter((r) => r.compiled.kind === "timeout").length}}
\\providecommand{\\LeanRouteGlcCount}{${glc.accepted.length}}
\\providecommand{\\LeanRouteGlcMedian}{${humaneS(median(glc.accepted.map((p) => p.medianS)))}}
\\providecommand{\\LeanRouteGlcMax}{${humaneS(Math.max(...glc.accepted.map((p) => p.medianS)))}}
\\providecommand{\\LeanRouteQtslabAcceptCount}{${qtslab.accepted.length}}
\\providecommand{\\LeanRouteQtslabCensoredCount}{${qtslab.censored.length}}
\\providecommand{\\LeanRouteQtslabOofCount}{${qtslab.oof.length}}
\\providecommand{\\LeanRouteArmHost}{${esc(glc.host.cpu)}}
\\providecommand{\\GlcRouteExponent}{${glcFit.b.toFixed(2)}}
\\providecommand{\\GlcRouteExponentLo}{${glcFit.ciLo.toFixed(2)}}
\\providecommand{\\GlcRouteExponentHi}{${glcFit.ciHi.toFixed(2)}}
\\providecommand{\\GlcRouteRSquared}{${glcFit.r2.toFixed(2)}}
\\providecommand{\\GlcRouteFitA}{${glcFit.a.toFixed(6)}}
\\providecommand{\\GlcRouteFitB}{${glcFit.b.toFixed(6)}}
\\providecommand{\\GlcRouteFitDomainLo}{${glc.accepted[0].bytes}}
\\providecommand{\\GlcRouteFitDomainHi}{${glc.accepted[glc.accepted.length - 1].bytes}}
\\providecommand{\\QtslabRouteExponent}{${qtslabFit.b.toFixed(2)}}
\\providecommand{\\QtslabRouteExponentLo}{${qtslabFit.ciLo.toFixed(2)}}
\\providecommand{\\QtslabRouteExponentHi}{${qtslabFit.ciHi.toFixed(2)}}
\\providecommand{\\QtslabRouteRSquared}{${qtslabFit.r2.toFixed(2)}}
\\providecommand{\\QtslabRouteFitA}{${qtslabFit.a.toFixed(6)}}
\\providecommand{\\QtslabRouteFitB}{${qtslabFit.b.toFixed(6)}}
\\providecommand{\\QtslabRouteFitDomainLo}{${qtslab.accepted[0].bytes}}
\\providecommand{\\QtslabRouteFitDomainHi}{${qtslab.accepted[qtslab.accepted.length - 1].bytes}}
`,
);

// -------------------------------------------------------- admission table

type Species = {
  name: string; section: string; n: number;
  bytes_min: number; bytes_med: number; bytes_max: number;
  bytes_basis: string; fragment: string;
  bytes_definition: "artifact_proxy" | "sealed_container";
};
const speciesRows: Species[] = speciesCensus.species;
for (const sp of speciesRows)
  if (sp.bytes_definition !== "artifact_proxy" && sp.bytes_definition !== "sealed_container")
    fail(`species ${sp.name}: unknown bytes_definition ${sp.bytes_definition}`);

// The fitted regressor is SEALED-CONTAINER bytes.  A row whose census
// bytes are committed-artifact sizes (artifact_proxy) gets the measured
// ~420 B fixed container overhead added; a row already stated in
// sealed-container bytes is used as-is.  One definition per row, never
// blended — receipts that carry the other definition update the row
// AND its bytes_definition together.
const CONTAINER_OVERHEAD_B = 420;
const asContainerBytes = (sp: Species, bytes: number) =>
  sp.bytes_definition === "artifact_proxy" ? bytes + CONTAINER_OVERHEAD_B : bytes;

function verdict(sp: Species): { verdict: string; md: string } {
  if (sp.fragment.startsWith("out"))
    return { verdict: "\\emph{out of fragment}", md: "OUT OF FRAGMENT" };
  const tMed = predictS(asContainerBytes(sp, sp.bytes_med));
  const tMax = predictS(asContainerBytes(sp, sp.bytes_max));
  if (tMax <= budgetS) return { verdict: "pass", md: "PASS" };
  if (tMed <= budgetS) return { verdict: "partial", md: "PARTIAL (median passes, max does not)" };
  return { verdict: "fail", md: "FAIL" };
}

const admTexRows = speciesRows.map((sp) => {
  const v = verdict(sp);
  const tMed = humaneS(predictS(asContainerBytes(sp, sp.bytes_med)));
  const tMax = humaneS(predictS(asContainerBytes(sp, sp.bytes_max)));
  const frag = sp.fragment.startsWith("out") ? "out"
    : sp.fragment.startsWith("expected-in") ? "exp.\\ in"
    : "untested";
  return `${esc(sp.name)} & ${num(sp.bytes_max)} & ${frag} & ${tMed} & ${tMax} & ${v.verdict} \\\\`;
});
writeFileSync(
  path.join(GEN, "lean-route-admission.tex"),
  `{\\setlength{\\tabcolsep}{4pt}%
\\begin{tabular}{@{}p{10.5em}rllll@{}}
\\toprule
species & max B & fragment & pred.\\ med. & pred.\\ max & verdict \\\\
\\midrule
${admTexRows.join("\n")}
\\bottomrule
\\end{tabular}}
`,
);

// ------------------------------------------------------- ADMISSION.md

const admMdRows = speciesRows.map((sp) => {
  const v = verdict(sp);
  const tMed = humaneMd(predictS(asContainerBytes(sp, sp.bytes_med)));
  const tMax = humaneMd(predictS(asContainerBytes(sp, sp.bytes_max)));
  return `| ${sp.name} | ${sp.section} | ${sp.n} | ${sp.bytes_med.toLocaleString()} | ${sp.bytes_max.toLocaleString()} | ${sp.fragment.split(" (")[0]} | ${tMed} | ${tMax} | **${v.md}** |`;
});
writeFileSync(
  path.join(ROOT, "lean-route", "ADMISSION.md"),
  `# Lean-route admission gate (generated — \`make lean-route\` re-derives; do not hand-edit)

Fitted law (compiled reflection route, ${n} accepted rungs,
${pts[0].bytes.toLocaleString()}–${pts[pts.length - 1].bytes.toLocaleString()} container bytes, one box):

    t(seconds) = exp(${a.toFixed(3)}) * bytes^${b.toFixed(3)}
    exponent 95% CI [${ciLo.toFixed(2)}, ${ciHi.toFixed(2)}], r^2 = ${r2.toFixed(3)}
    box: ${host.cpu}, ${host.threads} threads, ${Math.round(host.mem_gib)} GiB (${host.host})

**Admission threshold at the ${Math.round(budgetS / 60)}-minute budget: ~${Math.round(thresholdBytes / 1000)} KB
of sealed container.** A species attempts a Lean-reflection timing only if
(a) its certificates sit under the threshold and (b) it is in the route's
accepted fragment (first-order pinned-vocabulary rule set; a
primitive-application certificate is out of fragment, and the route
declines it by design -- a completeness boundary, never a soundness
one). The kernel (\`decide\`) route admits **nothing**: its
smallest measured full acceptance anywhere (919 B) exceeded both the
budget and this box's memory; see the chapter section.

Byte basis: the fitted regressor is sealed-container bytes. Each census
row declares its \`bytes_definition\`: \`artifact_proxy\` rows (committed
artifacts' sizes) get the measured ~${CONTAINER_OVERHEAD_B} B container overhead added;
\`sealed_container\` rows (campaign-banked real container sizes) are used
as-is — one definition per row, never blended. When receipts land,
update the row's stats and flip its definition together, then recompute
(\`make lean-route\`). Predictions
beyond the fitted domain (${pts[pts.length - 1].bytes.toLocaleString()} B) are extrapolations of the
power law and are marked by being, frankly, enormous.

| species | section | n | med. bytes | max bytes | fragment | pred. (med.) | pred. (max) | verdict |
|---|---|---:|---:|---:|---|---|---|---|
${admMdRows.join("\n")}

Fragment rule: "expected in" rows share their rule mix with an accepted
ladder rung; a species' FIRST Lean attempt (smallest instance) settles
fragment membership cheaply before any batch run. "untested" rows should
run that one-instance probe first regardless of size verdict.

Verdicts are computed at the species' median and maximum. A FAIL species
whose *minimum* sits under the ~${Math.round(thresholdBytes / 1000)} KB threshold can still probe its
small end (the fragment probe doubles as that); the verdict governs
batch attempts, not curiosity.
`,
);

// ------------------------------------------------------------------ pin
// Regression pin (see gen_tables.ts for the discipline): filled from the
// first accepted receipts; a legitimate receipts regeneration that moves
// a pinned digit updates the pin in the same change, so moves are
// noticed, never silent.

const PIN: { exponent: string; thresholdKB: string } | null = {
  exponent: "3.58",
  thresholdKB: "28",
};
if (PIN && !noVerify) {
  const got = { exponent: b.toFixed(2), thresholdKB: String(Math.round(thresholdBytes / 1000)) };
  if (got.exponent !== PIN.exponent || got.thresholdKB !== PIN.thresholdKB)
    fail(
      `regression pin moved: exponent ${got.exponent} (pinned ${PIN.exponent}), ` +
      `threshold ${got.thresholdKB} KB (pinned ${PIN.thresholdKB} KB) — ` +
      `if the receipts were legitimately regenerated, update PIN alongside`,
    );
}

console.log(
  `lean-route: ${rungs.length} rungs, ${n} fit points; ` +
  `exponent ${b.toFixed(2)} [${ciLo.toFixed(2)}, ${ciHi.toFixed(2)}], r^2 ${r2.toFixed(3)}; ` +
  `threshold ${(thresholdBytes / 1000).toFixed(1)} KB at ${budgetS} s; ` +
  `predict(10 KB) = ${predictS(10000).toFixed(1)} s`,
);
