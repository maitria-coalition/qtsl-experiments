// recheck.ts — re-check the committed circuit-manifest certificate suite
// against the reference checker, then run the layer-targeted mutant battery
// (ported from the retired mutants.py; see that file's docstring lineage in
// git history and the route doc's "Mutant battery").
//
//   (a) accept sweep: every fixtures/*.cbor ACCEPTs, in one batched checker
//       invocation; the fixture count and the "N certificates / M tasks"
//       convention are verified against SWEEP-RECEIPTS.md's summary line
//       (parsed, never hardcoded).
//   (b) mutant battery: positive controls P1-P3 plus mutants M1-M16,
//       generated in-memory from pristine committed fixtures and written to
//       a scratch dir (never the repo tree); each mutant must REJECT with
//       the targeted check named in the REJECT line (reject-at-attacked-
//       layer, not just reject).
//
// Trust posture: this driver is UNTRUSTED orchestration — it decodes
// fixtures, applies mutations, and tabulates what the reference checker
// said. The checker is the instrument. Self-gate on the codec: before any
// fixture is mutated, reencode(decode(fixture)) must be byte-identical to
// the committed bytes; a fixture failing that gate is never mutated and the
// gap is reported (a partial battery with named gaps beats a fake-green one).
//
// Reference-checker CLI contract (stable across qtsl revisions; the pinned
// rev implements it in python, a future rev may reimplement it in another
// language behind the SAME argv/output contract):
//   python3 $QTSL_CIRCUITS_CHECKER/ck_manifest.py <fixture.cbor>...
//   per file, one line on stdout:
//     "ACCEPT <instance>: <details>"    or
//     "REJECT <path>: <check>: <detail>"   (<check> = C0..C6)
//   exit 0 iff all files accept.
// env QTSL_CIRCUITS_CHECKER = directory containing ck_manifest.py
// (default <repo>/build/qtsl/checkers/circuits).
// Exit: 0 iff sweep + battery are both fully green.
//
// Run: node --experimental-strip-types circuits/recheck.ts

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const REPO = path.resolve(HERE, "..");
const FIXTURES = path.join(HERE, "fixtures");
const CK_DIR = process.env.QTSL_CIRCUITS_CHECKER ||
  path.join(REPO, "build", "qtsl", "checkers", "circuits");
const CK = path.join(CK_DIR, "ck_manifest.py");

// ---------------------------------------------------------------- CBOR codec
// Minimal codec covering exactly the types present in these fixtures:
// unsigned/negative integers (within Number.MAX_SAFE_INTEGER), UTF-8 text
// strings, definite-length arrays and string-keyed maps, and booleans.
// Anything else (floats, byte strings, tags, indefinite lengths, bignums,
// null) throws — the self-gate below turns such a throw into a named gap
// rather than silent corruption. Encoding is minimal-length definite-form,
// matching the encoder that produced the committed fixtures.

type CborValue = number | boolean | string | CborValue[] |
  { [key: string]: CborValue };

class CborError extends Error {}

function cborDecode(buf: Buffer): CborValue {
  let pos = 0;

  function head(): [number, number] {
    const b = buf[pos++];
    const major = b >> 5;
    const ai = b & 0x1f;
    if (ai < 24) return [major, ai];
    if (ai === 24) return [major, buf[pos++]];
    if (ai === 25) { const v = buf.readUInt16BE(pos); pos += 2; return [major, v]; }
    if (ai === 26) { const v = buf.readUInt32BE(pos); pos += 4; return [major, v]; }
    if (ai === 27) {
      const v = buf.readBigUInt64BE(pos); pos += 8;
      if (v > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw new CborError(`u64 ${v} exceeds Number.MAX_SAFE_INTEGER`);
      }
      return [major, Number(v)];
    }
    throw new CborError(`unsupported additional-info ${ai} (indefinite length?)`);
  }

  function item(): CborValue {
    const [major, arg] = head();
    switch (major) {
      case 0: return arg;
      case 1: return -1 - arg;
      case 3: {
        const s = buf.subarray(pos, pos + arg).toString("utf8");
        pos += arg;
        return s;
      }
      case 4: {
        const a: CborValue[] = [];
        for (let i = 0; i < arg; i++) a.push(item());
        return a;
      }
      case 5: {
        const m: { [key: string]: CborValue } = {};
        for (let i = 0; i < arg; i++) {
          const k = item();
          if (typeof k !== "string") throw new CborError(`non-string map key ${k}`);
          m[k] = item();
        }
        return m;
      }
      case 7: {
        if (arg === 20) return false;
        if (arg === 21) return true;
        throw new CborError(`unsupported simple/float value ${arg}`);
      }
      default:
        throw new CborError(`unsupported major type ${major} (bytes/tag)`);
    }
  }

  const v = item();
  if (pos !== buf.length) throw new CborError(`trailing bytes at ${pos}`);
  return v;
}

function cborEncode(v: CborValue): Buffer {
  const parts: Buffer[] = [];

  function head(major: number, arg: number): void {
    if (!Number.isSafeInteger(arg) || arg < 0) {
      throw new CborError(`bad length/argument ${arg}`);
    }
    if (arg < 24) {
      parts.push(Buffer.from([(major << 5) | arg]));
    } else if (arg <= 0xff) {
      parts.push(Buffer.from([(major << 5) | 24, arg]));
    } else if (arg <= 0xffff) {
      const b = Buffer.allocUnsafe(3);
      b[0] = (major << 5) | 25; b.writeUInt16BE(arg, 1);
      parts.push(b);
    } else if (arg <= 0xffffffff) {
      const b = Buffer.allocUnsafe(5);
      b[0] = (major << 5) | 26; b.writeUInt32BE(arg, 1);
      parts.push(b);
    } else {
      const b = Buffer.allocUnsafe(9);
      b[0] = (major << 5) | 27; b.writeBigUInt64BE(BigInt(arg), 1);
      parts.push(b);
    }
  }

  function item(x: CborValue): void {
    if (typeof x === "number") {
      if (!Number.isSafeInteger(x)) throw new CborError(`non-integer number ${x}`);
      if (x >= 0) head(0, x);
      else head(1, -1 - x);
    } else if (typeof x === "boolean") {
      parts.push(Buffer.from([x ? 0xf5 : 0xf4]));
    } else if (typeof x === "string") {
      const b = Buffer.from(x, "utf8");
      head(3, b.length);
      parts.push(b);
    } else if (Array.isArray(x)) {
      head(4, x.length);
      for (const e of x) item(e);
    } else if (x !== null && typeof x === "object") {
      const keys = Object.keys(x);
      head(5, keys.length);
      for (const k of keys) { item(k); item(x[k]); }
    } else {
      throw new CborError(`unsupported value ${String(x)}`);
    }
  }

  item(v);
  return Buffer.concat(parts);
}

// ----------------------------------------------- producer-side poly helpers
// Mirrors of species.py: sparse polynomials {exponent-tuple: Fraction} over
// a fixed wire order, encoded as sorted [[expts...], [num, den]] lists.

type Frac = [bigint, bigint]; // reduced, den > 0

function frac(n: number | bigint, d: number | bigint = 1n): Frac {
  let num = BigInt(n), den = BigInt(d);
  if (den === 0n) throw new Error("zero denominator");
  if (den < 0n) { num = -num; den = -den; }
  const g = (function gcd(a: bigint, b: bigint): bigint {
    a = a < 0n ? -a : a;
    while (b !== 0n) [a, b] = [b, a % b];
    return a;
  })(num, den);
  return g === 0n ? [0n, 1n] : [num / g, den / g];
}

type Poly = Array<{ expts: number[]; coef: Frac }>;

function pconst(c: number, nvars: number): Poly {
  const f = frac(c);
  return f[0] === 0n ? [] : [{ expts: new Array(nvars).fill(0), coef: f }];
}

function pvar(i: number, nvars: number): Poly {
  const e = new Array(nvars).fill(0);
  e[i] = 1;
  return [{ expts: e, coef: frac(1) }];
}

function padd(...ps: Poly[]): Poly {
  const out = new Map<string, { expts: number[]; coef: Frac }>();
  for (const p of ps) {
    for (const { expts, coef } of p) {
      const k = expts.join(",");
      const cur = out.get(k);
      if (!cur) {
        out.set(k, { expts: [...expts], coef });
      } else {
        const [an, ad] = cur.coef, [bn, bd] = coef;
        const s = frac(an * bd + bn * ad, ad * bd);
        if (s[0] === 0n) out.delete(k);
        else cur.coef = s;
      }
    }
  }
  return [...out.values()];
}

function pscale(c: number, p: Poly): Poly {
  const f = frac(c);
  if (f[0] === 0n) return [];
  return p.map(({ expts, coef }) => ({
    expts: [...expts],
    coef: frac(f[0] * coef[0], f[1] * coef[1]),
  }));
}

function encFrac(x: number): number[] {
  const [n, d] = frac(x);
  return [Number(n), Number(d)];
}

function encPoly(p: Poly): CborValue[] {
  const rows = p.map(({ expts, coef }) =>
    [[...expts], [Number(coef[0]), Number(coef[1])]] as CborValue);
  rows.sort((a, b) => {
    const ea = (a as number[][])[0], eb = (b as number[][])[0];
    for (let i = 0; i < Math.min(ea.length, eb.length); i++) {
      if (ea[i] !== eb[i]) return ea[i] - eb[i];
    }
    return ea.length - eb.length;
  });
  return rows;
}

type Cell = { [key: string]: CborValue };

function cell(box: number[][], gamma: boolean,
              step?: Poly[], imageBox?: number[][]): Cell {
  if (!gamma && (step === undefined || imageBox === undefined)) {
    throw new Error("non-gamma cell needs step and image_box");
  }
  const d: Cell = { box: box.map((iv) => [...iv]), gamma };
  if (step !== undefined) d.step = step.map(encPoly);
  if (imageBox !== undefined) d.image_box = imageBox.map((iv) => [...iv]);
  return d;
}

function piece(box: number[][], poly: Poly): Cell {
  return { box: box.map((iv) => [...iv]), poly: encPoly(poly) };
}

const deepEq = (a: CborValue, b: CborValue): boolean =>
  JSON.stringify(a) === JSON.stringify(b);

// ------------------------------------------------------------ checker calls

function runChecker(paths: string[]): { rc: number; lines: string[] } {
  const p = spawnSync("python3", [CK, ...paths],
    { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  if (p.error) throw p.error;
  if (p.stderr && p.status !== 0 && p.status !== 1) {
    throw new Error(`checker crashed (rc=${p.status}): ${p.stderr.slice(0, 500)}`);
  }
  return { rc: p.status ?? -1, lines: p.stdout.trim().split("\n") };
}

// --------------------------------------------------------------- battery

interface BatteryRow {
  tag: string;
  refused: boolean | null; // null = gap (codec gate failed for base fixture)
  gotCheck: string | null;
  wantCheck: string | null; // null = positive control (wants ACCEPT)
  pass: boolean;
  note: string;
}

function expectReject(scratch: string, rows: BatteryRow[],
                      m: CborValue, wantCheck: string, tag: string): void {
  const file = path.join(scratch,
    tag.replace(/[^A-Za-z0-9]+/g, "_") + ".cbor");
  fs.writeFileSync(file, cborEncode(m));
  const { rc, lines } = runChecker([file]);
  const line = lines[lines.length - 1] ?? "";
  if (rc !== 0 && line.startsWith("REJECT ")) {
    const rest = line.slice("REJECT ".length);
    const sep = rest.indexOf(": ");
    const msg = sep >= 0 ? rest.slice(sep + 2) : rest;
    const got = msg.split(":")[0];
    const pass = got === wantCheck;
    rows.push({ tag, refused: true, gotCheck: got, wantCheck, pass,
                note: msg.slice(0, 90) });
    console.log(pass
      ? `  ${tag}: REJECT at ${got} as designed (${msg.slice(0, 70)}...)`
      : `  ${tag}: FAIL — rejected at ${got}, wanted ${wantCheck} (${msg})`);
  } else {
    rows.push({ tag, refused: false, gotCheck: null, wantCheck, pass: false,
                note: line.slice(0, 90) });
    console.log(`  ${tag}: FAIL — ACCEPTED, wanted REJECT at ${wantCheck}`);
  }
}

function expectAccept(scratch: string, rows: BatteryRow[],
                      m: CborValue, tag: string): void {
  const file = path.join(scratch,
    tag.replace(/[^A-Za-z0-9]+/g, "_") + ".cbor");
  fs.writeFileSync(file, cborEncode(m));
  const { rc, lines } = runChecker([file]);
  const line = lines[lines.length - 1] ?? "";
  const pass = rc === 0 && line.startsWith("ACCEPT ");
  rows.push({ tag, refused: rc !== 0, gotCheck: null, wantCheck: null, pass,
              note: line.slice(0, 90) });
  console.log(pass ? `  ${tag}: ${line}` : `  ${tag}: FAIL — (${line})`);
}

function gap(rows: BatteryRow[], tag: string, why: string): void {
  rows.push({ tag, refused: null, gotCheck: null, wantCheck: null,
              pass: false, note: why });
  console.log(`  ${tag}: GAP — ${why}`);
}

// ------------------------------------------------------------------- main

function main(): number {
  let ok = true;

  // ---- (a) accept sweep, counts verified against SWEEP-RECEIPTS.md
  const fixtureFiles = fs.readdirSync(FIXTURES)
    .filter((f) => f.endsWith(".cbor")).sort();
  const receiptsMd = fs.readFileSync(
    path.join(HERE, "SWEEP-RECEIPTS.md"), "utf8");
  const sm = receiptsMd.match(
    /(\d+)\/(\d+) certificates ACCEPT, covering (\d+) of the suite's (\d+) pure-liveness tasks/);
  if (!sm) {
    console.log("FAIL: SWEEP-RECEIPTS.md summary line not found/parseable");
    return 1;
  }
  const [nAcc, nCerts, nTasksCovered, nTasksPaper] =
    [sm[1], sm[2], sm[3], sm[4]].map(Number);
  if (nAcc !== nCerts) {
    console.log(`FAIL: receipts claim ${nAcc}/${nCerts} — not all-accept`);
    ok = false;
  }
  if (fixtureFiles.length !== nCerts) {
    console.log(`FAIL: ${fixtureFiles.length} fixtures on disk but ` +
      `SWEEP-RECEIPTS.md says ${nCerts} certificates`);
    ok = false;
  }
  // task convention: pair-2 conjunction certificates share their pair-1
  // task, so tasks = certificates - #pair2 fixtures.
  const nPair2 = fixtureFiles.filter(
    (f) => /^gray3_\d+_p2\.cbor$/.test(f) ||
           /^seven_seg2_\d+_p2\.cbor$/.test(f)).length;
  if (fixtureFiles.length - nPair2 !== nTasksCovered) {
    console.log(`FAIL: ${fixtureFiles.length} fixtures - ${nPair2} pair-2 = ` +
      `${fixtureFiles.length - nPair2} tasks, but SWEEP-RECEIPTS.md says ` +
      `${nTasksCovered} (of ${nTasksPaper})`);
    ok = false;
  }

  const sweep = runChecker(fixtureFiles.map((f) => path.join(FIXTURES, f)));
  const accepts = sweep.lines.filter((l) => l.startsWith("ACCEPT ")).length;
  const rejects = sweep.lines.filter((l) => l.startsWith("REJECT "));
  for (const r of rejects) console.log(`  ${r}`);
  if (sweep.rc !== 0 || accepts !== fixtureFiles.length) {
    console.log(`FAIL: accept sweep rc=${sweep.rc}, ` +
      `${accepts}/${fixtureFiles.length} accepted`);
    ok = false;
  } else {
    console.log(`accept sweep: ${accepts}/${fixtureFiles.length} fixtures ` +
      `ACCEPT (batched, 1 invocation) — matches SWEEP-RECEIPTS.md ` +
      `${nAcc}/${nCerts} certificates / ${nTasksCovered} of ` +
      `${nTasksPaper} tasks`);
  }

  // ---- codec self-gate + pristine bases for the battery
  const BASES = ["delay_1", "gray_1", "vga_1", "load_store_1", "i2c_1",
                 "lcd_1", "thermocouple_1", "uart_transmit_1",
                 "seven_seg2_1_p2", "blink_1"];
  const base: Record<string, CborValue> = {};
  const gateFailed = new Set<string>();
  let gateChecked = 0;
  for (const f of fixtureFiles) {
    const name = f.replace(/\.cbor$/, "");
    const bytes = fs.readFileSync(path.join(FIXTURES, f));
    try {
      const decoded = cborDecode(bytes);
      if (!cborEncode(decoded).equals(bytes)) {
        throw new CborError("reencode not byte-identical");
      }
      gateChecked += 1;
      if (BASES.includes(name)) base[name] = decoded;
    } catch (e) {
      gateFailed.add(name);
      console.log(`codec self-gate FAILED for ${f}: ${(e as Error).message}` +
        (BASES.includes(name) ? " — its mutants become named gaps" : ""));
    }
  }
  console.log(`codec self-gate: reencode(decode(f)) byte-identical for ` +
    `${gateChecked}/${fixtureFiles.length} fixtures` +
    (gateFailed.size ? ` (FAILED: ${[...gateFailed].join(", ")})` : ""));
  if (gateFailed.size > 0) ok = false;

  // ---- (b) mutant battery (ported verbatim from the retired mutants.py)
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "qtsl-circuit-mutants-"));
  const rows: BatteryRow[] = [];
  const clone = <T extends CborValue>(x: T): T => structuredClone(x);
  type Manifest = { [key: string]: CborValue } & {
    cells: Cell[]; pieces: Cell[];
  };
  const has = (n: string) => base[n] !== undefined;
  const N = 750, W = 1023;

  try {
    // -------- delay/gray battery (route doc's designed battery, worked
    // delay_1 instance); row order mirrors the retired mutants.py exactly:
    // P1, P2, M1..M8, P3, M9..M16 --------
    const d = has("delay_1") ? (base["delay_1"] as Manifest) : null;
    const g = has("gray_1") ? (base["gray_1"] as Manifest) : null;
    const delayGap = (t: string) =>
      gap(rows, t, "delay_1 failed the codec self-gate");
    const grayGap = (t: string) =>
      gap(rows, t, "gray_1 failed the codec self-gate");

    if (d) expectAccept(scratch, rows, clone(d), "P1 golden delay_1");
    else delayGap("P1 golden delay_1");
    if (g) expectAccept(scratch, rows, clone(g), "P2 golden gray_1");
    else grayGap("P2 golden gray_1");

    // M1: R = (N-1) - cnt — image point cnt = N evaluates to -1
    if (d) {
      const m = clone(d);
      m.pieces = [piece([[0, W]],
        padd(pconst(N - 1, 1), pscale(-1, pvar(0, 1))))];
      expectReject(scratch, rows, m, "C6", "M1 perturbed ranking constant");
    } else delayGap("M1 perturbed ranking constant");

    // M2: reset cell (rst=1) mis-marked non-gamma; its honest step/image
    if (d) {
      const m = clone(d);
      m.cells[0] = cell([[0, W], [1, 1]], false,
        [pconst(0, 2)], [[0, 0]]);
      expectReject(scratch, rows, m, "C5", "M2 scope-drop (reset cell non-gamma)");
    } else delayGap("M2 scope-drop (reset cell non-gamma)");

    // M3: gamma threshold moved to N+1: the cell [N, N] x {rst=0} becomes
    // non-gamma; its TRUE step (per the SV, reset fires at cnt >= N) is 0
    if (d) {
      const m = clone(d);
      m.cells = [
        cell([[0, W], [1, 1]], true, [pconst(0, 2)]),
        cell([[N + 1, W], [0, 0]], true, [pconst(0, 2)]),
        cell([[0, N - 1], [0, 0]], false,
          [padd(pvar(0, 2), pconst(1, 2))], [[1, N]]),
        cell([[N, N], [0, 0]], false, [pconst(0, 2)], [[0, 0]]),
      ];
      expectReject(scratch, rows, m, "C5", "M3 target-decode gamma' = cnt >= N+1");
    } else delayGap("M3 target-decode gamma' = cnt >= N+1");

    // M4: eps0 = 0
    if (d) {
      const m = clone(d);
      m.eps0 = encFrac(0);
      expectReject(scratch, rows, m, "C0", "M4 eps0 = 0");
    } else delayGap("M4 eps0 = 0");

    // M5: partition gap
    if (d) {
      const m = clone(d);
      m.cells.splice(0, 1);
      expectReject(scratch, rows, m, "C2", "M5 partition gap (rst cell deleted)");
    } else delayGap("M5 partition gap (rst cell deleted)");

    // M6: partition leak (duplicate non-gamma cell)
    if (d) {
      const m = clone(d);
      m.cells.push(clone(m.cells[2]));
      expectReject(scratch, rows, m, "C2", "M6 partition overlap (duplicated cell)");
    } else delayGap("M6 partition overlap (duplicated cell)");

    // M7: gray with a single linear piece (no wrap handoff): the wrap cell
    // cnt = W -> 0 demands R(0) <= R(W) - 1, i.e. (W+1) <= 0: refused
    if (g) {
      const m = clone(g);
      const Wg = 255;
      m.pieces = [piece([[0, Wg]],
        padd(pconst(Wg + 1, 1), pscale(-1, pvar(0, 1))))];
      expectReject(scratch, rows, m, "C5", "M7 gray without the wrap piece");
    } else grayGap("M7 gray without the wrap piece");

    // M8: image box shrunk to exclude the true image endpoint cnt = N
    if (d) {
      const m = clone(d);
      m.cells[2] = cell([[0, N - 1], [0, 0]], false,
        [padd(pvar(0, 2), pconst(1, 2))], [[1, N - 1]]);
      expectReject(scratch, rows, m, "C4", "M8 image-box shrink");
    } else delayGap("M8 image-box shrink");

    // -------- FSM-family battery (the sweep-2 leg) --------
    const aff3 = (c: number, ch: number, cv: number): Poly =>
      padd(pconst(c, 3), pscale(ch, pvar(0, 3)), pscale(cv, pvar(1, 3)));

    if (has("vga_1")) {
      const v = base["vga_1"] as Manifest; // s=1: H=66, V=28, hp=50, vp=25, Wh=127, Wv=31
      expectAccept(scratch, rows, clone(v), "P3 golden vga_1");

      // M9: junk-v pieces flattened into P6/P7's formulas ((V-v)H - h < 0
      // above V); partition intact, C6 refuses on the junk-v image
      let m = clone(v);
      const [H, V, vp, Wh, Wv] = [66, 28, 25, 127, 31];
      const dropped: CborValue[] = [
        [[0, H - 1], [vp, V - 1], [0, 0]],
        [[0, H - 1], [V, Wv], [0, 0]],
        [[H, Wh], [vp, V - 1], [0, 0]],
        [[H, Wh], [V, Wv], [0, 0]],
      ];
      const keep = m.pieces.filter(
        (p) => !dropped.some((b) => deepEq(p.box, b)));
      keep.push(piece([[0, H - 1], [vp, Wv], [0, 0]], aff3(V * H, -1, -H)));
      keep.push(piece([[H, Wh], [vp, Wv], [0, 0]],
        aff3((V - 1) * H + 1, 0, -H)));
      m.pieces = keep;
      expectReject(scratch, rows, m, "C6", "M9 vga junk-v pieces flattened");

      // M10: display-interior de=1 twin mis-marked non-gamma
      m = clone(v);
      for (const c of m.cells) {
        if (deepEq(c.box, [[0, 48], [0, 24], [1, 1], [0, 0]])) {
          c.gamma = false;
          c.image_box = [[1, 49], [0, 24], [1, 1]];
        }
      }
      expectReject(scratch, rows, m, "C5", "M10 vga target cell mis-marked non-gamma");
    } else {
      for (const t of ["P3 golden vga_1", "M9 vga junk-v pieces flattened",
        "M10 vga target cell mis-marked non-gamma"]) {
        gap(rows, t, "vga_1 failed the codec self-gate");
      }
    }

    // M11: load_store with the climb ranking on both modes
    if (has("load_store_1")) {
      const m = clone(base["load_store_1"]) as Manifest; // N=750, W=1023
      m.pieces = m.pieces.map((p) =>
        (!deepEq((p.box as CborValue[])[1], [0, 0]) ||
         !deepEq((p.box as CborValue[])[0], [0, 749]))
          ? p
          : piece([[0, 749], [0, 0]],
              padd(pconst(750, 2), pscale(-1, pvar(0, 2)))));
      expectReject(scratch, rows, m, "C5", "M11 load_store climb ranking on descend mode");
    } else {
      gap(rows, "M11 load_store climb ranking on descend mode",
        "load_store_1 failed the codec self-gate");
    }

    // M12: i2c junk-scl lattice collapsed onto the consistent pieces
    if (has("i2c_1")) {
      const m = clone(base["i2c_1"]) as Manifest; // d=3: E=6, X=8, L=11, W=15
      const np: Cell[] = [];
      for (const p of m.pieces) {
        if (deepEq(p.box, [[0, 4], [1, 1], [0, 0]])) {
          np.push(piece([[0, 5], [1, 1], [0, 0]],
            padd(pconst(6, 3), pscale(-1, pvar(0, 3)))));
        } else if (deepEq(p.box, [[5, 11], [1, 1], [0, 0]])) {
          np.push(piece([[6, 11], [1, 1], [0, 0]],
            padd(pconst(18, 3), pscale(-1, pvar(0, 3)))));
        } else {
          np.push(p);
        }
      }
      m.pieces = np;
      expectReject(scratch, rows, m, "C5", "M12 i2c junk-scl pieces collapsed");
    } else {
      gap(rows, "M12 i2c junk-scl pieces collapsed",
        "i2c_1 failed the codec self-gate");
    }

    // M13: lcd state-0 completion transcribed without the block-1
    // cascade (lands (1,0) instead of the true (1,1))
    if (has("lcd_1")) {
      const m = clone(base["lcd_1"]) as Manifest; // f=1: A0=500, W=511
      for (const c of m.cells) {
        if ((deepEq(c.box, [[0, 0], [500, 500], [1, 1]]) ||
             deepEq(c.box, [[0, 0], [501, 511], [1, 1]])) && !c.gamma) {
          c.step = [encPoly(pconst(1, 3)), encPoly(pconst(0, 3))];
          c.image_box = [[1, 1], [0, 0]];
        }
      }
      expectReject(scratch, rows, m, "C5", "M13 lcd cascade misread (1,0)");
    } else {
      gap(rows, "M13 lcd cascade misread (1,0)",
        "lcd_1 failed the codec self-gate");
    }

    // M14: thermocouple junk state 3 given the target's R = 0
    if (has("thermocouple_1")) {
      const m = clone(base["thermocouple_1"]) as Manifest; // f=10, W=63
      m.pieces = m.pieces.map((p) =>
        !deepEq(p.box, [[3, 3], [0, 63]])
          ? p
          : piece([[3, 3], [0, 63]], pconst(0, 2)));
      expectReject(scratch, rows, m, "C5", "M14 thermocouple junk-state R = 0");
    } else {
      gap(rows, "M14 thermocouple junk-state R = 0",
        "thermocouple_1 failed the codec self-gate");
    }

    // M15: uart with an over-claimed descent rate
    if (has("uart_transmit_1")) {
      const m = clone(base["uart_transmit_1"]) as Manifest;
      m.eps0 = encFrac(2);
      expectReject(scratch, rows, m, "C5", "M15 uart eps0 = 2");
    } else {
      gap(rows, "M15 uart eps0 = 2",
        "uart_transmit_1 failed the codec self-gate");
    }

    // P4/M17: blink ('25 suite; the upstream battery's blink pair). M17
    // mis-transcribes the blocking read led = mode as post-toggle
    // (led' = mode'), with its honest image: at (cnt=0, mode=1) the
    // misread step is (1, 0, 0) — the run stays dark for a whole count
    // period, and the golden ranking's row at the cell (R_cell = 1)
    // against the long-count image piece demands (W+2) - 1 + 1 <= 0.
    if (has("blink_1")) {
      const b = base["blink_1"] as Manifest;
      expectAccept(scratch, rows, clone(b), "P4 golden blink_1");
      const m = clone(b);
      const inc4 = padd(pvar(0, 4), pconst(1, 4));
      const flip4 = padd(pconst(1, 4), pscale(-1, pvar(1, 4)));
      m.cells[5] = cell([[0, 0], [1, 1], [0, 0], [0, 0]], false,
        [inc4, flip4, flip4], [[1, 1], [0, 0], [0, 0]]);
      expectReject(scratch, rows, m, "C5",
        "M17 blink blocking-order misread (led post-toggle)");
    } else {
      for (const t of ["P4 golden blink_1",
        "M17 blink blocking-order misread (led post-toggle)"]) {
        gap(rows, t, "blink_1 failed the codec self-gate");
      }
    }

    // M16: seven_seg2 pair-2 polarity: a ds=0 gamma cell mis-marked
    if (has("seven_seg2_1_p2")) {
      const m = clone(base["seven_seg2_1_p2"]) as Manifest; // freq=250
      for (const c of m.cells) {
        if (deepEq(c.box, [[0, 249], [0, 0], [0, 0]])) {
          c.gamma = false;
          c.image_box = [[1, 250], [0, 0]];
        }
      }
      expectReject(scratch, rows, m, "C5", "M16 seven_seg2_p2 target polarity flip");
    } else {
      gap(rows, "M16 seven_seg2_p2 target polarity flip",
        "seven_seg2_1_p2 failed the codec self-gate");
    }
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }

  const battOk = rows.every((r) => r.pass);
  if (!battOk) ok = false;

  // machine-readable battery table (one row per control/mutant)
  console.log("\nbattery table (tag\trefused\ttargeted-check-hit):");
  for (const r of rows) {
    const hit = r.wantCheck === null
      ? (r.pass ? "accept-control-ok" : "accept-control-FAIL")
      : (r.refused === null ? "GAP"
         : r.gotCheck === r.wantCheck ? `hit:${r.gotCheck}`
         : `MISS:got=${r.gotCheck ?? "ACCEPT"},want=${r.wantCheck}`);
    console.log(`  ${r.tag}\t${r.refused}\t${hit}`);
  }
  console.log("mutant battery:", battOk ? "ALL PASS" : "FAILURES");
  return ok ? 0 : 1;
}

process.exit(main());
