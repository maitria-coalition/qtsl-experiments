#!/usr/bin/env node
/* gen_tables.ts — re-derive the Experiments chapter's tables from the
 * repository's receipt files.
 *
 * Writes, into chapter/generated/, one file per table, each containing
 * exactly one \begin{tabular}...\end{tabular} environment:
 *
 *   sweep-table.tex       the published-table sweep, all seventeen
 *                         published rows plus the receipted alternate
 *                         readings, win-tint ladder on the ratio column
 *   sweep-times-table.tex wall-clock comparison over the same row set
 *                         (best published / produce / geolog / qtslab)
 *   gym-table.tex         the Gymnasium tabular-tier master table
 *   ll-table.tex          the Box2D i.i.d.-tier (U5) rows
 *   nmc-table.tex         the hardware-liveness completion table
 *   atari-table.tex       the Atari-circuit certified replay rows
 *   certified-checker-table.tex
 *                         the stored supermartingale corpus replayed
 *                         through the certified checker build
 *
 * Every number in every emitted table is computed from a receipt file in
 * this repository; nothing numeric is hard-coded here.  The literals in
 * this script are display metadata (row labels, claim-class tags, column
 * formats) and the frozen reference tables used by the verification step
 * (the chapter draft's hand-typed tables, kept as a regression pin: the
 * generated numbers must reproduce them cell for cell; when receipts are
 * legitimately regenerated and a pinned digit moves, the pin is updated
 * alongside — the pin exists so that such moves are *noticed*, never
 * silent).
 *
 * Receipt sources (paths relative to the repository root):
 *
 *   supermartingales/out/results.json            the paper's published columns
 *   supermartingales/out/sec3/receipts.json      house bounds, sec. 3 rows
 *   supermartingales/out/sec1/summary.json       house bounds, sec. 1 rows
 *   supermartingales/out/sec2/sweep_rows.json    house bounds, sec. 2 rows
 *   supermartingales/out/glc-baked/sec2/a40-retime/produce-gpu.json
 *                                                re-conducted rows' produce walls
 *   supermartingales/out/glc-baked/sec2/a40-retime/geolog-shared.json
 *                                                re-conducted rows' geolog walls
 *   supermartingales/out/gen-faulty-uniform-receipts.json  two-sided row
 *   supermartingales/out/sweep/SWEEP.md          persist_2d bound fraction
 *                                                (its per-file receipts have
 *                                                not yet migrated here)
 *   supermartingales/out/sec2/certified-checker/census.tsv
 *                                                certified-checker replay
 *   supermartingales/out/sec2/certified-checker/seal-twin-verdicts.tsv
 *                                                fast-path twin at seal
 *   gymnasium/out/producer_receipts.json         hitting-tier bounds
 *   gymnasium/out/producer_receipts_tt.json      toy-text bounds + gaps
 *   gymnasium/out/checker_receipts.json          hitting-tier check medians
 *   gymnasium/out/checker_receipts_tt.json       toy-text check medians
 *   gymnasium/out/RESULTS.md                     registry thresholds (200)
 *   gymnasium/out/ll/ll_cert_receipts.json       LunarLander U5 bounds
 *   gymnasium/out/llc/llc_cert_receipts.json     LunarLanderContinuous U5
 *   circuits/SWEEP-RECEIPTS.md                   house completion count
 *   circuits/neuralmc-pin/NeurIPS25_Experiments/plot_cmp.py
 *                                                baseline completion counts
 *                                                (the NeurIPS'25 artefact's
 *                                                hard-coded Table-1 data)
 *   gymnasium/atari/certs/<cert>/checker_receipt.json  Atari replay rows
 *                                                (the glob is spelled <cert>
 *                                                here only because a literal
 *                                                star-slash would close this
 *                                                comment block)
 *
 * Display conventions (the chapter's "How to read the tables" section and
 * the sweep receipts' SWEEP.md):
 *
 *   - displayed lower bounds round toward -inf, upper bounds toward +inf
 *     (the claim-weakening direction), fixed-point at 7 decimals unless a
 *     row's spec says otherwise;
 *   - magnitudes below 1e-9 display in scientific notation at 4
 *     significant digits (nearest);
 *   - improvement ratios: 2 decimals (nearest) below 100, else scientific
 *     at 2 significant digits;
 *   - check wall-clocks: nearest microsecond below 1 ms in the us-styled
 *     rows; 2 significant digits in the ms-styled rows.
 *
 * Usage:  node --experimental-strip-types tables/gen_tables.ts [--no-verify]
 * Exit status: 0 on success (and verification pass), 2 on verification
 * divergence, 1 on missing receipts.
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT = path.join(ROOT, "chapter", "generated");

function fail(msg: string): never {
  console.error(msg);
  process.exit(1);
}

function assert(cond: boolean, msg?: unknown): asserts cond {
  if (!cond) {
    throw new Error(`assertion failed${msg === undefined ? "" : `: ${msg}`}`);
  }
}

// --------------------------------------------------------------------------
// exact-arithmetic display helpers
// --------------------------------------------------------------------------

function bgcd(a: bigint, b: bigint): bigint {
  while (b !== 0n) [a, b] = [b, a % b];
  return a < 0n ? -a : a;
}

/** Floor division on bigints (Python's // — rounds toward -inf, where the
 * native / truncates toward zero; the difference is exactly the negative
 * lower bounds this script rounds). */
function floorDiv(a: bigint, b: bigint): bigint {
  const q = a / b;
  return a % b !== 0n && (a < 0n) !== (b < 0n) ? q - 1n : q;
}

/** Exact rational, always reduced, denominator always positive — the
 * fractions.Fraction of the python original, over bigint. */
class Frac {
  readonly num: bigint;
  readonly den: bigint;

  constructor(num: bigint, den: bigint = 1n) {
    if (den === 0n) throw new Error("Frac: zero denominator");
    if (den < 0n) {
      num = -num;
      den = -den;
    }
    const g = bgcd(num, den);
    this.num = num / g;
    this.den = den / g;
  }

  mul(o: Frac): Frac {
    return new Frac(this.num * o.num, this.den * o.den);
  }

  div(o: Frac): Frac {
    return new Frac(this.num * o.den, this.den * o.num);
  }

  neg(): Frac {
    return new Frac(-this.num, this.den);
  }

  abs(): Frac {
    return this.num < 0n ? this.neg() : this;
  }

  /** Sign of (this - o): cross-multiplication, denominators positive. */
  cmp(o: Frac): number {
    const l = this.num * o.den;
    const r = o.num * this.den;
    return l < r ? -1 : l > r ? 1 : 0;
  }

  isZero(): boolean {
    return this.num === 0n;
  }
}

function fracInt(n: number | bigint): Frac {
  return new Frac(BigInt(n));
}

/** 10^e as an exact rational, e possibly negative. */
function pow10(e: number): Frac {
  return e >= 0 ? new Frac(10n ** BigInt(e)) : new Frac(1n, 10n ** BigInt(-e));
}

/** Parse a decimal string (optionally signed, optionally exponent-form:
 * '0.0649', '3e-14', '200') into an exact rational — the Fraction(str)
 * constructor of the python original. */
function parseDecimal(t: string): Frac {
  const m = /^([+-]?)(?:(\d+)(?:\.(\d*))?|\.(\d+))(?:[eE]([+-]?\d+))?$/.exec(
    t.trim(),
  );
  if (m === null) throw new Error(`parseDecimal: unparseable ${t}`);
  const sign = m[1] === "-" ? -1n : 1n;
  const intPart = m[2] ?? "";
  const fracPart = m[3] ?? m[4] ?? "";
  const exp = m[5] === undefined ? 0 : parseInt(m[5], 10);
  let num = sign * BigInt(intPart + fracPart === "" ? "0" : intPart + fracPart);
  let den = 10n ** BigInt(fracPart.length);
  if (exp > 0) num *= 10n ** BigInt(exp);
  else if (exp < 0) den *= 10n ** BigInt(-exp);
  return new Frac(num, den);
}

/** Parse 'p/q' or 'n' (receipt fraction strings) into Frac. */
function frac(s: unknown): Frac {
  const t = String(s).trim();
  if (t.includes("/")) {
    const [n, d] = t.split("/");
    return new Frac(BigInt(n), BigInt(d));
  }
  return new Frac(BigInt(t));
}

/** Published decimals arrive as JSON floats; String() recovers the
 * intended short decimal exactly (e.g. 0.0649).  JS String(number) and
 * python str(float) both emit the shortest round-trip decimal, so the
 * recovered rational is identical (checked by the differential against
 * the python original). */
function fromFloat(x: number): Frac {
  return parseDecimal(String(x));
}

/** Render scaled/10^n with exactly n decimals. */
function fixed(scaled: bigint, n: number): string {
  const sign = scaled < 0n ? "-" : "";
  const s = scaled < 0n ? -scaled : scaled;
  if (n === 0) return `${sign}${s}`;
  const p = 10n ** BigInt(n);
  return `${sign}${s / p}.${(s % p).toString().padStart(n, "0")}`;
}

/** Round toward -inf at n decimals (lower bounds). */
function floorAt(f: Frac, n: number): string {
  const scaled = f.mul(pow10(n));
  return fixed(floorDiv(scaled.num, scaled.den), n);
}

/** Round toward +inf at n decimals (upper bounds). */
function ceilAt(f: Frac, n: number): string {
  const scaled = f.mul(pow10(n)).neg();
  return fixed(-floorDiv(scaled.num, scaled.den), n);
}

/** e such that 10^e <= |f| < 10^(e+1), exactly. */
function decExponent(f: Frac): number {
  assert(!f.isZero());
  f = f.abs();
  let e = f.num.toString().length - f.den.toString().length;
  while (pow10(e).cmp(f) > 0) e -= 1;
  while (pow10(e + 1).cmp(f) <= 0) e += 1;
  return e;
}

/** [mantissa string, exponent] at `sig` significant digits, nearest
 * (half away from zero); exact arithmetic throughout. */
function sciNearest(f: Frac, sig: number): [string, number] {
  assert(f.cmp(fracInt(0)) > 0);
  let e = decExponent(f);
  const q = f.div(pow10(e - sig + 1)); // in [10^(sig-1), 10^sig)
  const scaled = q.mul(fracInt(2));
  let m = floorDiv(floorDiv(scaled.num, scaled.den) + 1n, 2n); // nearest
  if (m >= 10n ** BigInt(sig)) {
    m /= 10n;
    e += 1;
  }
  const ms = m.toString();
  const mantissa = sig === 1 ? ms[0] : `${ms[0]}.${ms.slice(1)}`;
  return [mantissa, e];
}

function fmtSci(f: Frac, sig: number): string {
  const [m, e] = sciNearest(f, sig);
  return String.raw`${m}\times 10^{${e}}`;
}

/** Improvement ratio: 2 decimals below 100, else scientific 2 sig. */
function fmtRatio(f: Frac): string {
  if (f.cmp(fracInt(100)) < 0) {
    const scaled = f.mul(fracInt(100)).mul(fracInt(2));
    const v = floorDiv(floorDiv(scaled.num, scaled.den) + 1n, 2n);
    return String.raw`$${fixed(v, 2)}\times$`;
  }
  return String.raw`$${fmtSci(f, 2)}\times$`;
}

/** Sweep-table upper bounds: >=1e-9 fixed-point (ceil), else sci. */
function fmtUpperBound(f: Frac, decimals: number = 7): string {
  if (f.abs().cmp(new Frac(1n, 10n ** 9n)) >= 0) {
    return ceilAt(f, decimals);
  }
  return fmtSci(f, 4);
}

/** Exact short decimal (or null): '7.93', '-13', '1'. */
function terminating(f: Frac, maxDec: number = 2): string | null {
  for (let n = 0; n <= maxDec; n += 1) {
    const scaled = f.mul(pow10(n));
    if (scaled.den === 1n) return fixed(scaled.num, n);
  }
  return null;
}

/** Nearest microsecond from nanoseconds. */
function fmtUs(ns: number): string {
  return ((BigInt(ns) + 500n) / 1000n).toString();
}

/** ns -> ms at 2 significant digits, fixed-point (e.g. 1.6, 0.14). */
function fmtMs2sig(ns: number): string {
  const f = new Frac(BigInt(ns), 10n ** 6n);
  const [m, e] = sciNearest(f, 2);
  const digits = m.replace(".", "");
  const nDec = Math.max(0, 1 - e); // decimals so that 2 significant digits show
  const scaled = new Frac(BigInt(digits), 10n).mul(pow10(e)).mul(pow10(nDec));
  assert(scaled.den === 1n);
  return fixed(scaled.num, nDec);
}

/** One-significant-digit rendering of a small float gap: '3', -14. */
function sig1(x: number): [string, number] {
  return sciNearest(fromFloat(x), 1);
}

/** Round half to even (python's round(float) — JS Math.round rounds half
 * up, which would bias the atari wall-clock cell on exact .5 seconds). */
function pyRound(x: number): number {
  const f = Math.floor(x);
  const diff = x - f;
  if (diff > 0.5) return f + 1;
  if (diff < 0.5) return f;
  return f % 2 === 0 ? f : f + 1;
}

// --------------------------------------------------------------------------
// receipt loading
// --------------------------------------------------------------------------

function loadJson(rel: string): any {
  const p = path.join(ROOT, rel);
  if (!fs.existsSync(p)) fail(`gen_tables: missing receipt file ${rel}`);
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

function readText(rel: string): string {
  const p = path.join(ROOT, rel);
  if (!fs.existsSync(p)) fail(`gen_tables: missing receipt file ${rel}`);
  return fs.readFileSync(p, "utf8");
}

// --------------------------------------------------------------------------
// Table 1: the published-table sweep
// --------------------------------------------------------------------------

// The paper's own grouping (SWEEP.md): the seventeen published
// benchmarks, each followed by any house-frozen alternate reading of
// its (ambiguous) definition that this repository carries receipts
// for.  An alternate row's published cells apply to its parent (the
// row above); `alt` carries the display label.  Parents and order are
// the sweep record's; the row universe is pinned to the campaign's
// master reconciliation table (SWEEP.md) — the one alternate WITHOUT
// receipts here (the persist_2d gaussian twin, envelope-passed but
// not yet certified) stays in prose, not in the table.
type SweepSpec = { key: string; alt?: string; parent?: string };
const SWEEP_ORDER: SweepSpec[] = [
  { key: "persist_2d" },
  { key: "faulty_marbles" },
  { key: "faulty_unreliable" },
  { key: "faulty_regions" },
  { key: "cliff_crossing" },
  { key: "repulse100" },
  { key: "repulse100_uniform" },
  { key: "repulse100_uniform__discrete", parent: "repulse100_uniform",
    alt: "discrete-uniform reading" },
  { key: "repulse100_2d" },
  { key: "faulty_concave" },
  { key: "faulty_varying" },
  { key: "faulty_varying__last_step", parent: "faulty_varying",
    alt: "last-step reading" },
  { key: "fixed_loop" },
  { key: "faulty_loop" },
  { key: "faulty_loop__as_listed", parent: "faulty_loop",
    alt: String.raw`as printed ($i_0{=}0$)` },
  { key: "faulty_uniform" },
  { key: "faulty_rare" },
  { key: "faulty_easy1" },
  { key: "faulty_ndecr" },
  { key: "faulty_walk" },
  { key: "faulty_walk__det12", parent: "faulty_walk",
    alt: "deterministic 12-step reading" },
];

interface SweepRow {
  bestPub: Frac;
  bound: Frac;
  twoSided: [Frac, Frac] | null;
  // 'refuted-listing': the certified truth of this (alternate) reading
  // exceeds every published column, so no improvement ratio exists.
  ratioKind: "normal" | "two-sided" | "refuted-listing";
}

function collectSweepRows(): Record<string, SweepRow> {
  const published: Record<string, any> = {};
  for (const r of loadJson("supermartingales/out/results.json")) {
    published[r.name] = r.published;
  }

  const house = new Map<string, Frac>(); // name -> tightest certified upper bound
  const truth = new Map<string, Frac>(); // name -> exact truth, where receipted

  const offer = (name: string, bound: Frac): void => {
    const cur = house.get(name);
    if (cur === undefined || bound.cmp(cur) < 0) house.set(name, bound);
  };

  for (const [name, row] of Object.entries<any>(
    loadJson("supermartingales/out/sec3/receipts.json").benchmarks,
  )) {
    offer(name, frac(row.bound));
    if (row.p_true != null) truth.set(name, frac(row.p_true));
  }
  for (const [name, row] of Object.entries<any>(
    loadJson("supermartingales/out/sec1/summary.json"),
  )) {
    offer(name, frac(row.bound));
  }
  for (const row of loadJson("supermartingales/out/sec2/sweep_rows.json")) {
    offer(row.benchmark, frac(row.house_bound_exact));
  }

  // persist_2d (uniform twin, the primary reading): its per-file receipt
  // bundle has not migrated into this repository yet; the sweep record
  // (supermartingales/out/sweep/SWEEP.md, discrepancy log item 7)
  // carries the exact certified bound fraction, which we parse rather
  // than transcribe.
  const sweepMd = readText("supermartingales/out/sweep/SWEEP.md");
  const m = /exact\s+bound\s+fraction\s*\$(\d+\/\d+)\$/.exec(sweepMd);
  if (m !== null) {
    offer("persist_2d", frac(m[1]));
  }

  // the two-sided faulty_uniform row: certified upper AND lower bounds.
  const fu = loadJson("supermartingales/out/gen-faulty-uniform-receipts.json");
  const fuUpper = frac(fu.bound);
  const fuLower = frac(fu.lower_bound);
  assert(
    fuUpper.cmp(house.get("faulty_uniform") ?? fuUpper) === 0,
    "faulty_uniform upper bound disagrees across receipts",
  );

  const rows: Record<string, SweepRow> = {};
  for (const spec of SWEEP_ORDER) {
    const name = spec.key;
    const bound = house.get(name);
    if (bound === undefined) {
      fail(`gen_tables: no certified house bound receipt for ${name}`);
    }
    const pub = published[spec.parent ?? name];
    // the minimum over the paper's *sound* columns; a published column
    // is excluded as refuted exactly when this repository carries a
    // certified lower bound strictly above it (the faulty_uniform
    // Farkas refutation).
    const cands: Frac[] = [];
    for (const key of ["farkas_bound", "pa_bound", "pw_bound"]) {
      let v = pub[key];
      if (v == null || v === "none") continue;
      v = fromFloat(v);
      if ((spec.parent ?? name) === "faulty_uniform" && v.cmp(fuLower) < 0) {
        continue; // certified-refuted: not a sound column
      }
      cands.push(v);
    }
    const bestPub = cands.reduce((a, b) => (b.cmp(a) < 0 ? b : a));
    // an alternate reading whose exact certified truth exceeds every
    // published column has no improvement ratio (the published numbers
    // are refuted for that reading — derived, not asserted: the exact
    // truth fraction is in the receipts).
    const pTrue = truth.get(name);
    const refuted =
      spec.parent !== undefined &&
      pTrue !== undefined &&
      cands.every((c) => pTrue.cmp(c) > 0);
    rows[name] = {
      bestPub,
      bound,
      twoSided: name === "faulty_uniform" ? [fuLower, fuUpper] : null,
      ratioKind: refuted
        ? "refuted-listing"
        : name === "faulty_uniform"
          ? "two-sided"
          : "normal",
    };
  }
  return rows;
}

function texttt(name: string): string {
  return String.raw`\texttt{${name.replaceAll("_", "\\_")}}`;
}

/** Row label: the benchmark name, or the alternate-reading form. */
function sweepLabel(spec: SweepSpec): string {
  if (spec.alt !== undefined) {
    return String.raw`\altrow\tdemph{${spec.alt}}`;
  }
  return spec.key === "persist_2d"
    ? texttt(spec.key) + String.raw` \tdemph{(uniform twin)}`
    : texttt(spec.key);
}

/** The win-tint ladder: \cellcolor for the ratio cell, keyed to the
 * order of magnitude of the improvement (10^e <= ratio < 10^(e+1)).
 * Color is decoration on top of the ratio text, never the only
 * carrier — the legend lives in the chapter's conventions section. */
function winTint(ratio: Frac): string {
  const e = ratio.cmp(fracInt(1)) < 0 ? 0 : decExponent(ratio);
  const tint =
    e < 1 ? "winA" : e < 10 ? "winB" : e < 25 ? "winC" : e < 45 ? "winD" : "winE";
  return String.raw`\cellcolor{${tint}}`;
}

function sweepCells(spec: SweepSpec, row: SweepRow): string[] {
  // alternate rows leave the published cell empty: the parent row's
  // published number (the cell above) applies.
  let pub = "";
  if (spec.parent === undefined) {
    const scaled = row.bestPub.mul(pow10(4)); // 4-dp as published
    assert(scaled.den === 1n);
    pub = `$${fixed(scaled.num, 4)}$`;
  }
  let bound: string;
  let ratio: string;
  if (row.twoSided !== null) {
    const [lo, hi] = row.twoSided;
    bound = String.raw`$[${floorAt(lo, 7)},\, ${ceilAt(hi, 7)}]$`;
    // tint by the certified upper end: the conservative reading
    ratio = winTint(row.bestPub.div(hi)) + "(two-sided; see text)";
  } else if (row.ratioKind === "refuted-listing") {
    bound = String.raw`$\le ${fmtUpperBound(row.bound)}$`;
    ratio = "--- (see note)";
  } else {
    bound = String.raw`$\le ${fmtUpperBound(row.bound)}$`;
    const r = row.bestPub.div(row.bound);
    ratio = winTint(r) + fmtRatio(r);
  }
  return [sweepLabel(spec), pub, bound, ratio];
}

function buildSweepTable(): string {
  const rows = collectSweepRows();
  const header = String.raw`benchmark & best published & certified bound & ratio \\`;
  const body = SWEEP_ORDER.map((s) => sweepCells(s, rows[s.key]));
  return tabular("l r r r", header, body);
}

// --------------------------------------------------------------------------
// Table 1b: wall-clock comparison over the same row set
// --------------------------------------------------------------------------

/* Column sources (all receipt files in this repository):
 *
 *   best published — supermartingales/manifest.toml, the transcription
 *     of the paper's timings table (per-column extraction provenance in
 *     that file's header): the wall of whichever published route
 *     attains the row's best sound published bound (Farkas' lemma
 *     time, or learn + verify for the neural routes) — the same
 *     bound-selection rule as the bounds table, applied to times.
 *   produce — for rows re-conducted onto the shared-subtree vintage,
 *     out/glc-baked/sec2/a40-retime/produce-gpu.json: median over the
 *     non-warmup repeats of the checked-production total (produce +
 *     in-loop check + seal), GPU-proposing lane, output byte-identical
 *     to the committed container on every repeat; for the remaining
 *     rows, the campaign legs' producer walls:
 *     out/sec2/sweep_rows.json ms_produce + out/sec3/receipts.json
 *     t_synth_s (summed where a row's corpus spans both legs).
 *   geolog check — for re-conducted rows,
 *     out/glc-baked/sec2/a40-retime/geolog-shared.json: median
 *     glc-check wall over the row's one shared-subtree container; for
 *     the remaining rows, out/glc/GLC-TIMINGS.json: the geolog
 *     fast-path checker (glc-check) over the row's sealed containers,
 *     total wall per row (machine + binary hash in each receipt).
 *   qtslab check — out/sec2/certified-checker/census.tsv: sum of the
 *     certified build's per-VC walls for the row's stored
 *     corpus (machine + protocol in provenance.json beside it).
 *
 * Missing-cell marks (the notes block in the section file states the
 * reasons in prose):
 *   a — the persist_2d VC bundle has not migrated into this
 *       repository (no house walls to report);
 *   b — producer wall not recorded in that leg's receipts;
 *   c — row not in the certified-checker replay's stored corpus;
 *   d — the certified-checker replay covers the row's stored sec-2
 *       subset (fewer files than the VCs column);
 *   e — (on a number, not a hole) the row's produce wall is its
 *       campaign leg's: its certificate is a synthesis genus the
 *       toolkit producer does not yet cover.
 */

/** display seconds: >=10 one decimal, >=1 two decimals, else 2
 * significant digits (below 0.1 s, trailing zeros drop rather than
 * overstate the receipts' millisecond granularity). */
function fmtSecs(s: number): string {
  assert(Number.isFinite(s) && s >= 0.0005, `fmtSecs: ${s}`);
  if (s >= 10) return s.toFixed(1);
  if (s >= 1) return s.toFixed(2);
  const t = s.toPrecision(2);
  return s < 0.1 ? t.replace(/(\.\d*?)0+$/, "$1").replace(/\.$/, "") : t;
}

/** The published times are short decimals (2 dp in the source table);
 * sum and render them exactly — in integer centiseconds — so that no
 * float artifact moves a transcribed digit (254.45 s must display
 * 254.5, not 254.4). */
function fmtSecsCenti(cs: bigint): string {
  if (cs >= 1000n) {
    const deci = (cs + 5n) / 10n; // nearest, half away from zero
    return fixed(deci, 1);
  }
  return fixed(cs, 2);
}

/** Minimal reader for manifest.toml's [[benchmark]] blocks: only the
 * flat name/bound/time keys this table consumes. */
function manifestBenchmarks(): Record<string, Record<string, string>> {
  const txt = readText("supermartingales/manifest.toml");
  const out: Record<string, Record<string, string>> = {};
  for (const block of txt.split(/^\[\[benchmark\]\]\s*$/m).slice(1)) {
    const kv: Record<string, string> = {};
    for (const raw of block.split("\n")) {
      if (/^\[/.test(raw.trim())) break; // next section
      const m = /^\s*([a-z0-9_]+)\s*=\s*(.+?)\s*$/.exec(raw);
      if (m === null) continue;
      let v = m[2];
      if (v.startsWith('"')) v = v.slice(1, v.indexOf('"', 1));
      else v = v.replace(/\s*#.*$/, "");
      kv[m[1]] = v;
    }
    if (kv.name !== undefined) out[kv.name] = kv;
  }
  return out;
}

/** A 2-dp decimal string as exact integer centiseconds. */
function centi(s: string): bigint {
  const f = parseDecimal(s).mul(fracInt(100));
  assert(f.den === 1n, `not a 2-dp decimal: ${s}`);
  return f.num;
}

/** [bound, centiseconds] of the published route attaining the row's
 * best sound published bound (the bounds table's selection rule). */
function bestPublishedRoute(
  mb: Record<string, string>,
  excludeFarkas: boolean,
): [Frac, bigint] {
  const routes: [string, string, () => bigint][] = [
    ["farkas", "farkas_bound", () => centi(mb.farkas_time_s)],
    ["pa", "pa_bound", () => centi(mb.pa_learn_s) + centi(mb.pa_verify_s)],
    ["pw", "pw_bound", () => centi(mb.pw_learn_s) + centi(mb.pw_verify_s)],
  ];
  let best: [Frac, bigint] | null = null;
  for (const [key, boundKey, time] of routes) {
    const v = mb[boundKey];
    if (v === undefined || v === "none") continue;
    if (key === "farkas" && excludeFarkas) continue;
    const b = parseDecimal(v);
    if (best === null || b.cmp(best[0]) < 0) best = [b, time()];
  }
  assert(best !== null, `no published route for ${mb.name}`);
  return best;
}

/** Per-family accepted-wall sums from the certified-checker census. */
function qtslabFamilySums(): Record<string, { n: number; wall_s: number }> {
  const out: Record<string, { n: number; wall_s: number }> = {};
  for (const line of readText(QTSLAB_DIR + "/census.tsv").trimEnd().split("\n")) {
    const [famName, , , verdict, wall] = line.split("\t");
    assert(verdict.startsWith("s QTSLAB ACCEPT"), `census non-accept: ${line}`);
    const fam = (out[famName] ??= { n: 0, wall_s: 0 });
    fam.n += 1;
    fam.wall_s += parseFloat(wall);
  }
  return out;
}

/** Median of a nonempty sample (even length: mean of the middle two —
 * these are wall-clock medians, so float arithmetic is the right
 * precision). */
function median(xs: number[]): number {
  assert(xs.length > 0);
  const s = [...xs].sort((a, b) => a - b);
  const mid = s.length >> 1;
  return s.length % 2 === 1 ? s[mid] : (s[mid - 1] + s[mid]) / 2;
}

// The sec-2 re-conduct's timing receipts, committed beside the
// shared-subtree containers they time.
const A40_DIR = "supermartingales/out/glc-baked/sec2/a40-retime";

/** Per-benchmark medians over the non-warmup repeats of an a40-retime
 * receipt file.  `check` guards what the wall is a wall OF: a produce
 * repeat must report byte-identity to the committed container, and a
 * check repeat must accept — a timing for a run that produced
 * different bytes (or refused) would time the wrong event. */
function a40Medians(
  rel: string,
  value: (row: any) => number,
  check: (row: any) => boolean,
): Record<string, number> {
  const byBench = new Map<string, number[]>();
  for (const row of loadJson(`${A40_DIR}/${rel}`).rows) {
    if (row.warmup === true) continue;
    assert(check(row), `${rel}: bad repeat ${row.bench}#${row.repeat}`);
    const xs = byBench.get(row.bench) ?? [];
    xs.push(value(row));
    byBench.set(row.bench, xs);
  }
  const out: Record<string, number> = {};
  for (const [bench, xs] of byBench) {
    assert(xs.length === 10, `${rel}: ${bench} has ${xs.length} repeats`);
    out[bench] = median(xs);
  }
  return out;
}

function buildSweepTimesTable(): string {
  const mbs = manifestBenchmarks();
  const rows = collectSweepRows(); // for the bounds cross-check below
  const glc = loadJson("supermartingales/out/glc/GLC-TIMINGS.json").groups as
    Record<string, { n: number; wall_s: number }>;
  const qtslab = qtslabFamilySums();
  const sec3 = loadJson("supermartingales/out/sec3/receipts.json").benchmarks;
  const sec2: Record<string, any> = {};
  for (const r of loadJson("supermartingales/out/sec2/sweep_rows.json")) {
    sec2[r.benchmark] = r;
  }
  // the sec-2 re-conduct (the a40-retime receipts): checked-production
  // medians (produce + in-loop check + seal, GPU-proposing lane) and
  // shared-container check medians
  const a40Produce = a40Medians(
    "produce-gpu.json",
    (r) => r.produce_s + r.check_s + r.seal_s,
    (r) => r.byte_identical === true,
  );
  const a40Geolog = a40Medians(
    "geolog-shared.json",
    (r) => r.wall_s,
    (r) => r.accept === true,
  );

  // superscript marks: appended INSIDE a closing math cell (a bare
  // `$...$$^{..}$` would tokenize its middle `$$` unpredictably)
  const markMath = (mathCell: string, m: string): string => {
    assert(mathCell.endsWith("$"), mathCell);
    return mathCell.slice(0, -1) + String.raw`^{\,\text{${m}}}$`;
  };
  const hole = (m: string): string => "---" + String.raw`$^{\,\text{${m}}}$`;

  const body: string[][] = [];
  for (const spec of SWEEP_ORDER) {
    const name = spec.key;
    const cells: string[] = [sweepLabel(spec)];

    const g = glc[name];
    cells.push(g !== undefined ? `$${g.n}$` : hole("a"));

    if (spec.parent !== undefined) {
      cells.push(""); // the parent row's published cells apply
    } else {
      const mb = mbs[name];
      assert(mb !== undefined, `manifest.toml missing benchmark ${name}`);
      const [bound, cs] = bestPublishedRoute(
        mb,
        rows[name].ratioKind === "two-sided", // the refuted Farkas column
      );
      // artifact pair: the timing transcription's bound agrees with the
      // bounds table's independent selection from results.json
      assert(
        bound.cmp(rows[name].bestPub) === 0,
        `best published bound disagrees between manifest.toml and results.json for ${name}`,
      );
      cells.push(`$${fmtSecsCenti(cs)}$`);
    }

    const rp = a40Produce[name];
    if (rp !== undefined) {
      cells.push(`$${fmtSecs(rp)}$`);
    } else {
      let produce = 0;
      if (sec2[name]?.ms_produce != null) produce += sec2[name].ms_produce / 1000;
      if (sec3[name]?.t_synth_s != null) produce += sec3[name].t_synth_s;
      if (produce === 0) {
        cells.push(hole(name === "persist_2d" ? "a" : "b"));
      } else {
        const cell = `$${fmtSecs(produce)}$`;
        // in the re-conduct's check set but not its produce set: a
        // synthesis genus the producer does not cover yet (mark e)
        cells.push(a40Geolog[name] !== undefined ? markMath(cell, "e") : cell);
      }
    }

    const rg = a40Geolog[name];
    cells.push(
      rg !== undefined
        ? `$${fmtSecs(rg)}$`
        : g !== undefined
          ? `$${fmtSecs(g.wall_s)}$`
          : hole("a"),
    );

    const q = qtslab[name];
    if (q === undefined) {
      cells.push(hole(name === "persist_2d" ? "a" : "c"));
    } else {
      const cell = `$${fmtSecs(q.wall_s)}$`;
      cells.push(g !== undefined && q.n < g.n ? markMath(cell, "d") : cell);
    }
    body.push(cells);
  }

  const header = String.raw`benchmark & VCs & best published (s) & produce (s) & geolog (s) & qtslab (s) \\`;
  return tabular("l r r r r r", header, body);
}

// --------------------------------------------------------------------------
// Table 2: the Gymnasium tabular tier
// --------------------------------------------------------------------------

// (label, claim tag, receipt family, receipt key, deadline, display
//  decimals, bound relation, check-time style)
const GYM_ROWS: [string, string, string, string, string, number, string, string][] = [
  ["FrozenLake 4x4 (slippery)",              "U3", "fl", "frozenlake-4x4",       "100", 7,  String.raw`\ge`, "us"],
  ["FrozenLake 8x8 (slippery)",              "U3", "fl", "frozenlake-8x8",       "200", 7,  String.raw`\ge`, "us"],
  [String.raw`FrozenLake 4x4 (succ.\ $3/4$)`, "U3", "fl", "frozenlake-4x4-sr34",  "100", 7,  String.raw`\ge`, "us"],
  ["FrozenLake 4x4 (not slippery)",          "U3", "fl", "frozenlake-4x4-det",   "100", 7,  "=",             "us"],
  ["CliffWalking",                           "U2", "tt", "cliffwalking-det",     "100", 7,  String.raw`\ge`, "us"],
  ["CliffWalking (slippery)",                "U3", "tt", "cliffwalking-slippery","100", 7,  String.raw`\ge`, "us"],
  ["Taxi",                                   "U2", "tt", "taxi-dry",             "200", 7,  String.raw`\ge`, "ms"],
  ["Taxi (rainy)",                           "U3", "tt", "taxi-rainy",           "200", 10, String.raw`\ge`, "ms"],
  ["Taxi (fickle passenger)",                "U3", "tt", "taxi-fickle",          "200", 10, String.raw`\ge`, "ms"],
  ["Taxi (rainy + fickle)",                  "U3", "tt", "taxi-rainy-fickle",    "200", 10, String.raw`\ge`, "ms"],
  [String.raw`Blackjack (S\&B rules)`,        "U3", "tt", "blackjack",            "22",  10, String.raw`\ge`, "ms"],
];

function buildGymTable(): string {
  const prodFl = loadJson("gymnasium/out/producer_receipts.json").rows;
  const prodTt = loadJson("gymnasium/out/producer_receipts_tt.json").rows;
  const chkFl = loadJson("gymnasium/out/checker_receipts.json").certs;
  const chkTt = loadJson("gymnasium/out/checker_receipts_tt.json").certs;

  const body: string[][] = [];
  for (const [label, claim, fam, key, T, dec, rel, style] of GYM_ROWS) {
    let gap: number;
    let bound: Frac;
    let med: number;
    if (fam === "fl") {
      // hitting tier: the exact full-deadline certificate is the
      // optimum of the truncated env (finite-horizon optimal DP,
      // tight by construction — RESULTS.md, wave 1).
      bound = frac(prodFl[key].exact[T].bound);
      gap = 0.0;
      const cert = `${key}/dyadic_k48_T${T}.cbor`;
      med = chkFl[cert].verify_median_ns;
    } else {
      const row = prodTt[key];
      const lanes: [number, Frac][] = []; // (gap, bound) over full-claim certificates
      for (const lane of ["exact", "tail"]) {
        for (const c of Object.values<any>(row[lane] ?? {})) {
          const g = c.gap_to_optimum;
          lanes.push([g == null ? 0.0 : g, frac(c.bound)]);
        }
      }
      for (const [kk, c] of Object.entries<any>(row.dyadic ?? {})) {
        if (kk === "48") {
          lanes.push([c.gap_to_optimum, frac(c.bound)]);
        }
      }
      let best = lanes[0];
      for (const t of lanes) {
        if (t[0] < best[0]) best = t;
      }
      [gap, bound] = best;
      if (gap === 0.0) {
        bound = frac(row.optimum.fraction);
      }
      const cert = `${key}/rwdyadic_k48_T${T}.cbor`;
      med = chkTt[cert].verify_median_ns;
    }

    const term = terminating(bound);
    const shown = term !== null ? term : floorAt(bound, dec);
    const boundCell = `$${rel} ${shown}$`;
    let vs: string;
    if (gap === 0.0) {
      // exact-cell tint: zero conservatism against the model (the
      // conventions section's legend; text carries the distinction)
      vs = String.raw`\cellcolor{exacttint}optimal, exactly`;
    } else {
      const [m, e] = sig1(gap);
      vs = String.raw`within $${m}{\times}10^{${e}}$`;
    }
    let check: string;
    if (style === "us") {
      check = String.raw`$${fmtUs(med)}\,\mu$s`;
    } else {
      check = String.raw`$${fmtMs2sig(med)}$\,ms`;
    }
    body.push([label, claim, boundCell, vs, check]);
  }

  const header = String.raw`environment & claim & certified bound & vs.\ optimum & check \\`;
  return tabular("l l r l r", header, body);
}

// --------------------------------------------------------------------------
// Table 3: the Box2D i.i.d. tier (U5)
// --------------------------------------------------------------------------

function buildLlTable(): string {
  const resultsMd = readText("gymnasium/out/RESULTS.md");
  const thresholds = [
    ...resultsMd.matchAll(/[Rr]egistry threshold[^0-9]*?(\d+)\s*:/g),
  ].map((m) => m[1]);
  if (thresholds.length < 2) {
    fail("gen_tables: registry thresholds not found in RESULTS.md");
  }

  const body: string[][] = [];
  for (const [label, rel, thr] of [
    ["LunarLander-v3", "gymnasium/out/ll/ll_cert_receipts.json", thresholds[0]],
    ["LunarLanderContinuous-v3", "gymnasium/out/llc/llc_cert_receipts.json", thresholds[1]],
  ] as [string, string, string][]) {
    const rec = loadJson(rel);
    const row = rec.rows.min_a100; // E[R] >= at alpha = 1/100, det. Ville
    const alpha = "1/100";
    const boundF = fromFloat(row.bound_det);
    const bound = floorAt(boundF, 1); // lower: round down
    // win-tint vs the registry threshold (the bound/threshold ratio's
    // order of magnitude — the same ladder as the sweep table)
    const tint = winTint(boundF.div(fracInt(parseInt(thr, 10))));
    body.push([
      label,
      String.raw`$\mathbb{E}[R] \ge$, $\alpha = ${alpha}$`,
      `${tint}$${bound}$`,
      `$${thr}$`,
    ]);
  }

  const header = String.raw`environment & claim (class U5) & bound & threshold \\`;
  return tabular("l l r r", header, body);
}

// --------------------------------------------------------------------------
// Table 4: hardware model-checking completion counts (all three classes)
// --------------------------------------------------------------------------

type ClassCounts = {
  perMeth: Record<string, number>; // completions per named baseline
  union: number;                   // per-task union over EVERY parsed method
  total: number;                   // tasks in the class
};

/** Parse one property-class section of the NeurIPS'25 artefact's
 * plot_cmp.py (the authoritative their-side numbers, per the vendor
 * pin: its hard-coded runtime arrays are the paper's Table 1 data) and
 * count completions per method.  The union column is computed over
 * every method array present in the section — including the linear
 * variant of their method (nuRLin), which gets no row of its own
 * below — so "virtual best" means the whole published portfolio. */
function classCounts(section: string, timeout: number): ClassCounts {
  const cnts: Record<string, number> = {};
  const perTask: Record<string, Record<string, number[]>> = {};
  for (const line of section.split("\n")) {
    let m = /^\s*(\w+_cnt)\s*=\s*(\d+)/.exec(line);
    if (m !== null) {
      cnts[m[1]] = parseInt(m[2], 10);
      continue;
    }
    m = /^\s*(\w+?_(nuX|nuRauto|abc|rIC3|nuRLin|NMC24))\s*=\s*expandTOs\(\[(.*)\]\s*,\s*(\w+_cnt)\s*\)/.exec(
      line,
    );
    if (m === null) continue;
    const [, name, meth, arr, cntvar] = m;
    const fam = name.slice(0, name.lastIndexOf("_")); // e.g. l1_dly, s2_vga
    const cnt = cnts[cntvar];
    const vals: number[] = [];
    for (const tokRaw of arr.split(",")) {
      const tok = tokRaw.trim();
      if (tok !== "") {
        const v = tok === "time_out_time" ? timeout : Number(tok);
        if (Number.isNaN(v)) throw new Error(`classCounts: bad token ${tok}`);
        vals.push(v);
      }
    }
    while (vals.length < cnt) vals.push(timeout);
    (perTask[fam] ??= {})[meth] = vals.map((v) => Math.min(v, timeout));
  }

  const methods = ["nuX", "nuRauto", "abc", "rIC3", "nuRLin", "NMC24"];
  const perMeth: Record<string, number> =
    Object.fromEntries(methods.map((m_) => [m_, 0]));
  let union = 0;
  let total = 0;
  for (const perM of Object.values(perTask)) {
    const cnt = Object.values(perM)[0].length;
    total += cnt;
    for (const m_ of methods) {
      if (perM[m_] !== undefined) {
        perMeth[m_] += perM[m_].filter((v) => v < timeout).length;
      }
    }
    for (let i = 0; i < cnt; i += 1) {
      if (methods.some((m_) => perM[m_] !== undefined && perM[m_][i] < timeout)) {
        union += 1;
      }
    }
  }
  return { perMeth, union, total };
}

/** The house completion cell for one class: emitted only if the class's
 * receipt file in this repository witnesses it (never transcribed);
 * a missing or unparseable receipt yields an honest "---" hole. */
function houseCell(file: string, re: RegExp, expectTotal: number):
    [string, boolean] {
  let txt: string;
  try {
    txt = readText(file);
  } catch {
    return ["---", false];
  }
  const m = re.exec(txt);
  if (m === null || parseInt(m[2], 10) !== expectTotal) return ["---", false];
  return [`$${parseInt(m[1], 10)}$`, true];
}

function buildNmcTable(): string {
  const txt = readText("circuits/neuralmc-pin/NeurIPS25_Experiments/plot_cmp.py");
  const tm = /time_out_time\s*=\s*(\d+)/.exec(txt);
  const timeout = parseInt(tm![1], 10);
  const iSaf = txt.indexOf("**** SAFETY DATA ****");
  const iLiv = txt.indexOf("**** LIVENESS DATA ****");
  const iSl = txt.indexOf("**** SAFETY + LIVENESS DATA ****");
  assert(iSaf >= 0 && iSaf < iLiv && iLiv < iSl, "plot_cmp.py section order");
  const liv = classCounts(txt.slice(iLiv, iSl), timeout);
  const saf = classCounts(txt.slice(iSaf, iLiv), timeout);
  const sl = classCounts(txt.slice(iSl), timeout);
  assert(liv.total === 219 && saf.total === 206 && sl.total === 209,
    `${liv.total},${saf.total},${sl.total}`);
  // the 2024 method is liveness-only: the artefact's safety and
  // safety+liveness arrays for it are empty (padded to all-timeout).
  assert(saf.perMeth.NMC24 === 0 && sl.perMeth.NMC24 === 0, "NMC24 empty");

  // house cells, each witnessed by a receipt file in this repository
  const [hLiv, okLiv] = houseCell(
    "circuits/SWEEP-RECEIPTS.md",
    /covering (\d+) of the suite's (\d+) pure-liveness tasks/,
    219,
  );
  const [hSaf, okSaf] = houseCell(
    "circuits/SAFETY-RECEIPTS.md",
    /House: (\d+)\/(\d+) certified/,
    206,
  );
  const [hSl, okSl] = houseCell(
    "circuits/SL-RECEIPTS.md",
    /House: (\d+)\/(\d+) certified/,
    209,
  );
  const houseTotalOk = okLiv && okSaf && okSl;
  const hAll = houseTotalOk
    ? `$${[hLiv, hSaf, hSl]
        .map((c) => parseInt(c.replace(/\$/g, ""), 10))
        .reduce((a, b) => a + b, 0)}$`
    : "---";

  const cell = (n: number) => `$${n}$`;

  // exact-cell tint on house counts that strictly exceed the whole
  // published portfolio's virtual best (ties untinted; the numbers
  // themselves carry the comparison).
  const unions = [liv.union, saf.union, sl.union,
    liv.union + saf.union + sl.union];
  const houseTinted = [hLiv, hSaf, hSl, hAll].map((c, i) => {
    const v = parseInt(c.replace(/\$/g, ""), 10);
    return Number.isFinite(v) && v > unions[i]
      ? String.raw`\cellcolor{exacttint}` + c
      : c;
  });

  const rows: string[][] = [
    ["this toolkit", ...houseTinted, "checkable certificate"],
    ["neural, NeurIPS'25 (MILP)",
      cell(liv.perMeth.nuRauto), cell(saf.perMeth.nuRauto),
      cell(sl.perMeth.nuRauto),
      cell(liv.perMeth.nuRauto + saf.perMeth.nuRauto + sl.perMeth.nuRauto),
      "solver verdict"],
    ["nuXmv (BDD)",
      cell(liv.perMeth.nuX), cell(saf.perMeth.nuX), cell(sl.perMeth.nuX),
      cell(liv.perMeth.nuX + saf.perMeth.nuX + sl.perMeth.nuX),
      "tool verdict"],
    ["rIC3 1.5.1",
      cell(liv.perMeth.rIC3), cell(saf.perMeth.rIC3), cell(sl.perMeth.rIC3),
      cell(liv.perMeth.rIC3 + saf.perMeth.rIC3 + sl.perMeth.rIC3),
      "tool verdict"],
    ["ABC",
      cell(liv.perMeth.abc), cell(saf.perMeth.abc), cell(sl.perMeth.abc),
      cell(liv.perMeth.abc + saf.perMeth.abc + sl.perMeth.abc),
      "tool verdict"],
    ["neural, NeurIPS'24 (re-run)",
      cell(liv.perMeth.NMC24), "n/a", "n/a", cell(liv.perMeth.NMC24),
      "solver verdict"],
  ];
  const virtual = [
    "virtual best of all baselines",
    cell(liv.union), cell(saf.union), cell(sl.union),
    cell(liv.union + saf.union + sl.union),
    "portfolio of verdicts",
  ];

  const header = String.raw`method & liveness ${liv.total} & safety ${saf.total} & s{+}l ${sl.total} & all 634 & per-task evidence \\`;
  const lines = [String.raw`\begin{tabular}{l r r r r l}`,
    String.raw`\toprule`, header, String.raw`\midrule`];
  if (!houseTotalOk) {
    lines.push(
      "% one or more toolkit cells: no receipt in this repository at " +
      "this preview; left blank rather than transcribed.");
  }
  for (const cells of rows) {
    lines.push(cells.join(" & ") + String.raw` \\`);
  }
  lines.push(String.raw`\midrule`);
  lines.push(virtual.join(" & ") + String.raw` \\`);
  lines.push(String.raw`\bottomrule`, String.raw`\end{tabular}`);
  return lines.join("\n") + "\n";
}

// --------------------------------------------------------------------------
// Table 5: Atari-circuit certified replay rows
// --------------------------------------------------------------------------

const ATARI_ROWS: [string, Record<string, string>][] = [
  ["gymnasium/atari/certs/pong-noop-T1000", { "noop-T1000": "NOOP script" }],
  ["gymnasium/atari/certs/freeway-up-T1000", { "up-T1000": "all-UP script" }],
  [
    "gymnasium/atari/certs/montezuma-key-family-T828",
    { "montezuma-key-family-T828": "per-offset key scripts" },
  ],
  [
    "gymnasium/atari/certs/montezuma-door-T1548",
    { "montezuma-door-T1548": "key+door script" },
  ],
];

function buildAtariTable(): string {
  const body: string[][] = [];
  for (const [rel, policyNames] of ATARI_ROWS) {
    const rec = loadJson(rel + "/checker_receipt.json");
    assert(rec.verdict === "ACCEPT", rel);
    const policy = policyNames[rec.policy];
    const init =
      rec.lanes === 1
        ? String.raw`$k = 0$`
        : String.raw`$k \in \{0..${rec.lanes - 1}\}$`;
    assert(rec.min_return === Math.min(...rec.returns));
    assert(rec.min_return === rec.claimed_bound);
    const wall = pyRound(rec.wall_s);
    // display: the common AtariCircuit- prefix is elided (stated in the
    // caption); the receipt carries the full env id.
    assert(rec.env.startsWith("AtariCircuit-"), rel);
    const env = rec.env.slice("AtariCircuit-".length);
    body.push([
      env, policy, `$${rec.horizon}$`, init,
      // exact-cell tint: the replay minimum is exact (sim-tight)
      String.raw`\cellcolor{exacttint}$${rec.min_return}$`,
      String.raw`$${wall}$\,s`,
    ]);
  }
  const header = String.raw`environment & policy & $T$ & init set & min return & check \\`;
  return tabular("l l r l r r", header, body);
}

// --------------------------------------------------------------------------
// Table: the stored corpus through the certified checker build
// --------------------------------------------------------------------------

/* Derived from supermartingales/out/sec2/certified-checker/census.tsv
 * (one row per VC file: family, stem, exit code, first verdict line,
 * wall seconds) cross-checked against seal-twin-verdicts.tsv (the
 * fast-path twin's verdict at seal).  Percentile convention matches the
 * census receipts: sorted[floor(p*n)], clamped (upper median at even n);
 * min and max are exact order statistics.  Walls are displayed in whole
 * milliseconds (the receipts carry 3-decimal seconds). */

const QTSLAB_DIR = "supermartingales/out/sec2/certified-checker";

function wallQ(sorted: number[], p: number): number {
  assert(sorted.length > 0);
  const i = Math.min(
    sorted.length - 1,
    Math.max(0, Math.floor(p * sorted.length) - (p === 1.0 ? 1 : 0)),
  );
  return sorted[i];
}

function fmtWallsMs(walls: number[]): string {
  const ws = [...walls].sort((a, b) => a - b);
  const ms = (s: number) => `$${Math.round(s * 1000)}$`;
  return (
    [ws[0], wallQ(ws, 0.5), wallQ(ws, 0.9), ws[ws.length - 1]]
      .map(ms)
      .join(" / ") + String.raw`\,ms`
  );
}

function buildCertifiedCheckerTable(): string {
  const twins = new Map<string, string>();
  for (const line of readText(QTSLAB_DIR + "/seal-twin-verdicts.tsv")
    .trimEnd()
    .split("\n")) {
    const [stem, verdict] = line.split("\t");
    assert(!twins.has(stem), `duplicate seal-twin stem: ${stem}`);
    twins.set(stem, verdict);
  }
  type Fam = { n: number; accept: number; diverge: number; walls: number[] };
  const fams = new Map<string, Fam>();
  const pooled: Fam = { n: 0, accept: 0, diverge: 0, walls: [] };
  for (const line of readText(QTSLAB_DIR + "/census.tsv")
    .trimEnd()
    .split("\n")) {
    const [famName, stem, rc, verdict, wall] = line.split("\t");
    assert(wall !== undefined, `short census row: ${line}`);
    let fam = fams.get(famName);
    if (!fam) {
      fam = { n: 0, accept: 0, diverge: 0, walls: [] };
      fams.set(famName, fam);
    }
    fam.n += 1;
    pooled.n += 1;
    const twin = twins.get(stem);
    assert(twin !== undefined, `census stem missing from seal twins: ${stem}`);
    const twinAccept = twin === "accept";
    const certAccept = verdict.startsWith("s QTSLAB ACCEPT");
    if (twinAccept !== certAccept) {
      fam.diverge += 1;
      pooled.diverge += 1;
    }
    if (certAccept) {
      assert(rc === "0", `ACCEPT with nonzero exit: ${stem}`);
      fam.accept += 1;
      pooled.accept += 1;
      const w = parseFloat(wall);
      assert(Number.isFinite(w), `bad wall for ${stem}`);
      fam.walls.push(w);
      pooled.walls.push(w);
    }
  }
  assert(twins.size === pooled.n, "seal-twin rows do not cover the census 1:1");
  // The columns state what is semantically distinct: how many of the
  // family's VC files the certified build accepted (as a fraction
  // of the family's size), and on how many VC files the certified
  // build's verdict differs from the fast-path twin's — the zero
  // divergence is the finding, stated once, instead of as three
  // numerically identical count columns.
  const cellsFor = (d: Fam): string[] => [
    (d.accept === d.n ? String.raw`\cellcolor{exacttint}` : "") +
      String.raw`$${d.accept}/${d.n}$`,
    `$${d.diverge}$`,
    fmtWallsMs(d.walls),
  ];
  const body: string[][] = [];
  for (const famName of [...fams.keys()].sort()) {
    body.push([texttt(famName), ...cellsFor(fams.get(famName)!)]);
  }
  const header = String.raw`family & accepted, certified build & diverging vs.\ fast path & wall per VC file (ms), min/med/$p_{90}$/max \\`;
  const main = tabular("l r r r", header, body);
  // append the pooled row under its own rule, inside the tabular
  const total = ["all families", ...cellsFor(pooled)].join(" & ") + String.raw` \\`;
  return main.replace(
    "\\bottomrule",
    "\\midrule\n" + total + "\n\\bottomrule",
  );
}

// --------------------------------------------------------------------------
// emission + verification
// --------------------------------------------------------------------------

const BANNER =
  "% generated by tables/gen_tables.ts --- do not edit;\n" +
  "% every number is re-derived from receipt files in this repository.\n";

function tabular(colspec: string, header: string, body: string[][]): string {
  const lines = [`\\begin{tabular}{${colspec}}`, String.raw`\toprule`, header, String.raw`\midrule`];
  for (const cells of body) {
    lines.push(cells.join(" & ") + String.raw` \\`);
  }
  lines.push(String.raw`\bottomrule`, String.raw`\end{tabular}`);
  return lines.join("\n") + "\n";
}

// The chapter draft's hand-typed tables (source: the book chapter,
// 2026-07-20 preview), frozen here as the verification reference.
// gym-table timing column re-pinned 2026-07-22 (the dated re-run): receipts
// regenerated on the same GB10 single-core protocol with the checker's
// current default lane (auto = maitria-kernels sweep slabs where the
// overflow envelope permits; was reference-lane-only when first pinned).
// Bounds column unchanged — byte-identical certificates, zero bound
// drift, 53/53 (see gymnasium/out/2026-07-22/delta-ledger.md).
const REFERENCE: Record<string, string> = {
  // re-pinned 2026-07-27 (the all-rows + win-tint revision: every
  // published row, receipted alternate readings as \altrow subrows,
  // win-tint ladder on the ratio column)
  "sweep-table": String.raw`
\texttt{persist\_2d} \tdemph{(uniform twin)} & $0.1026$ & $\le 0.1000992$ & \cellcolor{winA}$1.02\times$ \\
\texttt{faulty\_marbles} & $0.0649$ & $\le 0.0623604$ & \cellcolor{winA}$1.04\times$ \\
\texttt{faulty\_unreliable} & $0.0536$ & $\le 0.0516561$ & \cellcolor{winA}$1.04\times$ \\
\texttt{faulty\_regions} & $0.0411$ & $\le 0.0406530$ & \cellcolor{winA}$1.01\times$ \\
\texttt{cliff\_crossing} & $0.0553$ & $\le 8.604\times 10^{-10}$ & \cellcolor{winB}$6.4\times 10^{7}\times$ \\
\texttt{repulse100} & $0.0268$ & $\le 9.556\times 10^{-20}$ & \cellcolor{winC}$2.8\times 10^{17}\times$ \\
\texttt{repulse100\_uniform} & $0.0344$ & $\le 6.157\times 10^{-61}$ & \cellcolor{winE}$5.6\times 10^{58}\times$ \\
\altrow\tdemph{discrete-uniform reading} &  & $\le 1.470\times 10^{-35}$ & \cellcolor{winD}$2.3\times 10^{33}\times$ \\
\texttt{repulse100\_2d} & $0.0541$ & $\le 9.556\times 10^{-20}$ & \cellcolor{winC}$5.7\times 10^{17}\times$ \\
\texttt{faulty\_concave} & $0.1356$ & $\le 0.1183888$ & \cellcolor{winA}$1.15\times$ \\
\texttt{faulty\_varying} & $0.0864$ & $\le 0.0694711$ & \cellcolor{winA}$1.24\times$ \\
\altrow\tdemph{last-step reading} &  & $\le 0.0353604$ & \cellcolor{winA}$2.44\times$ \\
\texttt{fixed\_loop} & $0.0091$ & $\le 0.0089651$ & \cellcolor{winA}$1.02\times$ \\
\texttt{faulty\_loop} & $0.0181$ & $\le 0.0178400$ & \cellcolor{winA}$1.01\times$ \\
\altrow\tdemph{as printed ($i_0{=}0$)} &  & $\le 0.0198024$ & --- (see note) \\
\texttt{faulty\_uniform} & $0.0221$ & $[0.0184458,\, 0.0185556]$ & \cellcolor{winA}(two-sided; see text) \\
\texttt{faulty\_rare} & $0.0019$ & $\le 0.0017994$ & \cellcolor{winA}$1.06\times$ \\
\texttt{faulty\_easy1} & $0.0801$ & $\le 0.0768847$ & \cellcolor{winA}$1.04\times$ \\
\texttt{faulty\_ndecr} & $0.0561$ & $\le 0.0544619$ & \cellcolor{winA}$1.03\times$ \\
\texttt{faulty\_walk} & $0.0121$ & $\le 0.0119293$ & \cellcolor{winA}$1.01\times$ \\
\altrow\tdemph{deterministic 12-step reading} &  & $\le 0.0119353$ & \cellcolor{winA}$1.01\times$ \\
`,
  // pinned 2026-07-27 (first emission): published column transcribed
  // (manifest.toml); produce/geolog/qtslab columns are TIMING_COLS
  // (re-benched walls drift, reported not failed).  Re-pinned
  // 2026-07-30 (the sec-2 re-conduct): produce cells for re-conducted
  // rows are the toolkit producer's checked-production medians and
  // geolog cells the shared-container check medians, both from the
  // a40-retime receipts; repulse100_uniform's produce cell gains
  // mark e (campaign wall, synthesis genus not yet ported).
  "sweep-times-table": String.raw`
\texttt{persist\_2d} \tdemph{(uniform twin)} & ---$^{\,\text{a}}$ & $254.5$ & ---$^{\,\text{a}}$ & ---$^{\,\text{a}}$ & ---$^{\,\text{a}}$ \\
\texttt{faulty\_marbles} & $23$ & $44.5$ & ---$^{\,\text{b}}$ & $0.016$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_unreliable} & $29$ & $52.3$ & ---$^{\,\text{b}}$ & $0.015$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_regions} & $29$ & $49.9$ & ---$^{\,\text{b}}$ & $0.016$ & ---$^{\,\text{c}}$ \\
\texttt{cliff\_crossing} & $1105$ & $153.6$ & $2.58$ & $1.01$ & $31.9$ \\
\texttt{repulse100} & $207$ & $10.2$ & $0.15$ & $0.10$ & $4.60$ \\
\texttt{repulse100\_uniform} & $205$ & $35.5$ & $4.44^{\,\text{e}}$ & $0.87$ & $13.6$ \\
\altrow\tdemph{discrete-uniform reading} & $207$ &  & $0.23$ & $0.11$ & $4.57$ \\
\texttt{repulse100\_2d} & $2267$ & $63.5$ & $6.38$ & $3.03$ & $71.8$ \\
\texttt{faulty\_concave} & $60$ & $21.3$ & $0.053$ & $0.089$ & $1.09^{\,\text{d}}$ \\
\texttt{faulty\_varying} & $60$ & $26.8$ & $0.054$ & $0.09$ & $1.13^{\,\text{d}}$ \\
\altrow\tdemph{last-step reading} & $60$ &  & $0.054$ & $0.087$ & $1.12^{\,\text{d}}$ \\
\texttt{fixed\_loop} & $22$ & $0.15$ & $0.0012$ & $0.006$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_loop} & $22$ & $0.16$ & $0.0015$ & $0.006$ & ---$^{\,\text{c}}$ \\
\altrow\tdemph{as printed ($i_0{=}0$)} & $22$ &  & $0.0015$ & $0.006$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_uniform} & $30$ & $8.08$ & $0.17$ & $0.031$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_rare} & $22$ & $0.27$ & $0.0016$ & $0.006$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_easy1} & $84$ & $0.31$ & $0.0064$ & $0.027$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_ndecr} & $60$ & $0.33$ & $0.0044$ & $0.017$ & ---$^{\,\text{c}}$ \\
\texttt{faulty\_walk} & $16$ & $0.32$ & $0.0011$ & $0.009$ & ---$^{\,\text{c}}$ \\
\altrow\tdemph{deterministic 12-step reading} & $28$ &  & $0.0015$ & $0.014$ & ---$^{\,\text{c}}$ \\
`,
  "gym-table": String.raw`
FrozenLake 4x4 (slippery) & U3 & $\ge 0.7441902$ & \cellcolor{exacttint}optimal, exactly & $15\,\mu$s \\
FrozenLake 8x8 (slippery) & U3 & $\ge 0.9132201$ & \cellcolor{exacttint}optimal, exactly & $100\,\mu$s \\
FrozenLake 4x4 (succ.\ $3/4$) & U3 & $\ge 0.8731132$ & \cellcolor{exacttint}optimal, exactly & $15\,\mu$s \\
FrozenLake 4x4 (not slippery) & U3 & $= 1$ & \cellcolor{exacttint}optimal, exactly & $13\,\mu$s \\
CliffWalking & U2 & $\ge -13$ & \cellcolor{exacttint}optimal, exactly & $24\,\mu$s \\
CliffWalking (slippery) & U3 & $\ge -63.0133733$ & \cellcolor{exacttint}optimal, exactly & $35\,\mu$s \\
Taxi & U2 & $\ge 7.93$ & \cellcolor{exacttint}optimal, exactly & $0.50$\,ms \\
Taxi (rainy) & U3 & $\ge 3.9545745165$ & within $3{\times}10^{-14}$ & $0.70$\,ms \\
Taxi (fickle passenger) & U3 & $\ge 8.38$ & \cellcolor{exacttint}optimal, exactly & $1.1$\,ms \\
Taxi (rainy + fickle) & U3 & $\ge 4.5756463469$ & within $3{\times}10^{-14}$ & $1.5$\,ms \\
Blackjack (S\&B rules) & U3 & $\ge -0.0431131055$ & \cellcolor{exacttint}optimal, exactly & $0.16$\,ms \\
`,
  "ll-table": String.raw`
LunarLander-v3 & $\mathbb{E}[R] \ge$, $\alpha = 1/100$ & \cellcolor{winA}$236.2$ & $200$ \\
LunarLanderContinuous-v3 & $\mathbb{E}[R] \ge$, $\alpha = 1/100$ & \cellcolor{winA}$281.1$ & $200$ \\
`,
  "nmc-table": String.raw`
this toolkit & \cellcolor{exacttint}$219$ & $206$ & \cellcolor{exacttint}$209$ & \cellcolor{exacttint}$634$ & checkable certificate \\
neural, NeurIPS'25 (MILP) & $192$ & $190$ & $172$ & $554$ & solver verdict \\
nuXmv (BDD) & $180$ & $206$ & $154$ & $540$ & tool verdict \\
rIC3 1.5.1 & $110$ & $196$ & $129$ & $435$ & tool verdict \\
ABC & $78$ & $206$ & $96$ & $380$ & tool verdict \\
neural, NeurIPS'24 (re-run) & $172$ & n/a & n/a & $172$ & solver verdict \\
virtual best of all baselines & $207$ & $206$ & $204$ & $617$ & portfolio of verdicts \\
`,
  // pinned 2026-07-23 from the certified-checker replay receipts
  // (supermartingales/out/sec2/certified-checker/); re-pinned
  // 2026-07-27 (column redesign: accepted-as-fraction + divergence
  // count replace the three numerically identical count columns):
  // counts are certified content; the wall column is timing
  // (TIMING_COLS below).
  "certified-checker-table": String.raw`
\texttt{cliff\_crossing} & \cellcolor{exacttint}$1105/1105$ & $0$ & $19$ / $29$ / $36$ / $53$\,ms \\
\texttt{faulty\_concave} & \cellcolor{exacttint}$38/38$ & $0$ & $22$ / $30$ / $34$ / $37$\,ms \\
\texttt{faulty\_varying} & \cellcolor{exacttint}$38/38$ & $0$ & $21$ / $30$ / $34$ / $36$\,ms \\
\texttt{faulty\_varying\_\_last\_step} & \cellcolor{exacttint}$38/38$ & $0$ & $21$ / $30$ / $35$ / $38$\,ms \\
\texttt{repulse100} & \cellcolor{exacttint}$207/207$ & $0$ & $14$ / $23$ / $26$ / $36$\,ms \\
\texttt{repulse100\_2d} & \cellcolor{exacttint}$2267/2267$ & $0$ & $18$ / $31$ / $37$ / $117$\,ms \\
\texttt{repulse100\_uniform} & \cellcolor{exacttint}$205/205$ & $0$ & $18$ / $64$ / $98$ / $117$\,ms \\
\texttt{repulse100\_uniform\_\_discrete} & \cellcolor{exacttint}$207/207$ & $0$ & $15$ / $22$ / $26$ / $31$\,ms \\
all families & \cellcolor{exacttint}$4105/4105$ & $0$ & $14$ / $30$ / $37$ / $117$\,ms \\
`,
};

/** The data rows of a tabular: lines ending in \\ that are neither
 * the header (first such line) nor rules. */
function dataRows(tabularText: string): string[] {
  const rows: string[] = [];
  for (const raw of tabularText.split("\n")) {
    const line = raw.trim();
    if (line.endsWith("\\\\") && !line.startsWith("%")) {
      rows.push(line);
    }
  }
  return rows.slice(1); // drop the header row
}

function normCells(row: string): string[] {
  // split on '&' column separators only (not on the escaped \& of
  // e.g. "S\&B"), then strip generation comments and whitespace
  const cells = row.replace(/\\+$/, "").split(/(?<!\\)&/);
  return cells
    .map((c) => c.replace(/(?<!\\)%.*$/, ""))
    .map((c) => c.replace(/\s+/g, " ").trim());
}

/** Python's repr() for the ASCII strings this script prints in verify
 * diagnostics: single-quoted (double- when the string contains a single
 * quote and no double quote), backslashes and quotes escaped. */
function pyRepr(s: string): string {
  const q = s.includes("'") && !s.includes('"') ? '"' : "'";
  let out = "";
  for (const ch of s) {
    if (ch === "\\") out += "\\\\";
    else if (ch === q) out += "\\" + q;
    else if (ch === "\n") out += "\\n";
    else if (ch === "\r") out += "\\r";
    else if (ch === "\t") out += "\\t";
    else out += ch;
  }
  return q + out + q;
}

// Columns whose cells are wall-clock measurements: these are re-timed on
// every receipt regeneration and drift with machine load (the receipts'
// own same-box-benching caveat), so a divergence from the pinned
// reference is reported but does not fail the build.  Certified content
// (bounds, counts, claims) must match exactly.
const TIMING_COLS: Record<string, Set<number>> = {
  "gym-table": new Set([4]),
  "certified-checker-table": new Set([3]),
  // produce / geolog / qtslab walls re-time at each receipt regeneration
  "sweep-times-table": new Set([3, 4, 5]),
};

/** Diff generated data rows against the frozen hand-typed reference.
 * Returns [content_mismatches, timing_drifts, receipt_gaps]. */
function verify(emitted: Record<string, string>): [number, number, number] {
  let bad = 0;
  let drift = 0;
  let gaps = 0;
  for (const [name, ref] of Object.entries(REFERENCE)) {
    const genRows = dataRows(emitted[name]);
    const refRows = dataRows(
      "\\begin{tabular}{}\n\\toprule\nh \\\\\n" + ref.trim() + "\n",
    );
    const tag = `verify[${name}]`;
    if (genRows.length !== refRows.length) {
      console.log(
        `${tag}: ROW COUNT ${genRows.length} vs reference ${refRows.length}`,
      );
      bad += 1;
      continue;
    }
    type Entry = [string, number, string, string];
    const rowsBad: Entry[] = [];
    const rowsDrift: Entry[] = [];
    const rowsGap: Entry[] = [];
    for (let r = 0; r < genRows.length; r += 1) {
      const gc = normCells(genRows[r]);
      const rc = normCells(refRows[r]);
      for (let i = 0; i < Math.min(gc.length, rc.length); i += 1) {
        const got = gc[i];
        const want = rc[i];
        if (got === want) continue;
        const entry: Entry = [rc[0], i, want, got];
        if (got.startsWith("---")) {
          rowsGap.push(entry); // honest hole: no receipt
        } else if (TIMING_COLS[name]?.has(i)) {
          rowsDrift.push(entry); // re-benched wall-clock
        } else {
          rowsBad.push(entry); // certified content moved
        }
      }
    }
    if (rowsBad.length === 0 && rowsDrift.length === 0 && rowsGap.length === 0) {
      console.log(`${tag}: MATCH (${genRows.length} rows)`);
    }
    for (const [kind, entries] of [
      ["CONTENT MISMATCH", rowsBad],
      ["timing drift", rowsDrift],
      ["receipt gap", rowsGap],
    ] as [string, Entry[]][]) {
      for (const [row, i, want, got] of entries) {
        console.log(
          `${tag}: ${kind} at row ${pyRepr(row)} col ${i}: ` +
            `reference ${pyRepr(want)} vs generated ${pyRepr(got)}`,
        );
      }
    }
    bad += rowsBad.length;
    drift += rowsDrift.length;
    gaps += rowsGap.length;
  }
  return [bad, drift, gaps];
}

function main(): number {
  const noVerify = process.argv.includes("--no-verify");
  fs.mkdirSync(OUT, { recursive: true });

  const emitted: Record<string, string> = {
    "sweep-table": buildSweepTable(),
    "sweep-times-table": buildSweepTimesTable(),
    "gym-table": buildGymTable(),
    "ll-table": buildLlTable(),
    "nmc-table": buildNmcTable(),
    "atari-table": buildAtariTable(),
    "certified-checker-table": buildCertifiedCheckerTable(),
  };
  for (const [name, text] of Object.entries(emitted)) {
    fs.writeFileSync(path.join(OUT, `${name}.tex`), BANNER + text);
    console.log(`wrote chapter/generated/${name}.tex`);
  }

  if (noVerify) return 0;
  const [bad, drift, gaps] = verify(emitted);
  if (drift || gaps) {
    console.log(
      `verify: ${drift} re-benched timing cell(s), ${gaps} ` +
        "receipt-gap cell(s) — reported above, not failures.",
    );
  }
  if (bad) {
    console.log(
      `verify: ${bad} CONTENT mismatch(es) — a certified number ` +
        "moved; if the receipts were legitimately regenerated, " +
        "update REFERENCE alongside.",
    );
    return 2;
  }
  return 0;
}

// exitCode rather than process.exit(): lets stdout drain before the
// process ends (process.exit can truncate piped output on some platforms).
process.exitCode = main();
