/** Emit the PROVENANCE.json entry point for a quiet-conditions retime
 * directory from its own version-3 receipt, so the human entry point can
 * never disagree with the receipt it fronts: every load-bearing field is
 * derived from receipt fields rather than typed twice. Refuses receipts
 * that did not declare a quiet gate — this generator exists only for the
 * gated protocol (QUIET-PROTOCOL.md beside the receipts). */
import * as fs from "node:fs";

const [receiptPath, outPath] = process.argv.slice(2);
if (receiptPath === undefined || outPath === undefined) {
  console.error("usage: quiet-provenance.ts <campaign.json> <PROVENANCE.json>");
  process.exit(2);
}

function fail(message: string): never {
  console.error(`quiet-provenance: ${message}`);
  process.exit(1);
}

const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
if (receipt.receipt_version !== 3) {
  fail(`expected a version-3 receipt, got receipt_version ${receipt.receipt_version}`);
}
const lc = receipt.load_context;
if (lc === null || typeof lc !== "object") fail("receipt has no load_context");
if (lc.gate_max_load_1min === null || lc.gate_max_load_1min === undefined) {
  fail("receipt is load-uncontrolled (no declared gate); quiet provenance refuses");
}
if (typeof receipt.binary_sha256 !== "string" || receipt.binary_sha256.length !== 64) {
  fail("receipt has no plausible binary_sha256");
}

const provenance = {
  date: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
  machine: `${receipt.machine} (see MACHINES.md; same GB10 hardware as gb10; quiet-conditions protocol per supermartingales/out/glc-baked/sec1/QUIET-PROTOCOL.md)`,
  protocol:
    `${receipt.protocol}; admitted at 1-minute load average ${lc.loadavg_start[0]} ` +
    `(max 1-minute load observed at any sample during the campaign: ${lc.loadavg_max_1min_observed}); ` +
    `pinned to CPUs ${lc.cpus_allowed_list} (${lc.cpus_allowed_count} cores); nice ${lc.nice}`,
  command:
    "make sec1-quiet-retime SUPERMART_SEC1=<supermart-sec1> — equivalently, " +
    `from a shell reniced to absolute niceness ${lc.nice} ` +
    "(renice, not a relative nice -n, so a priority-boosted parent cannot leak in): " +
    `taskset -c ${lc.cpus_allowed_list} supermart-sec1 retime ` +
    "--experiments <qtsl-experiments> --machine gb10-quiet --lane host " +
    `--warmups 1 --repeats 10 --max-load-1min ${lc.gate_max_load_1min} ` +
    "--out supermartingales/out/glc-baked/sec1/gb10-quiet-retime/campaign.json " +
    "--compare supermartingales/out/glc-baked/sec1/incumbents " +
    "--retain-containers supermartingales/out/glc-baked/sec1/gb10-quiet-retime/containers --pretty",
  binaries: {
    "supermart-sec1": { path: "supermart-sec1", sha256: receipt.binary_sha256 },
  },
  revisions: {
    mtk: receipt.mtk_revision,
    qtsl: receipt.qtsl_revision,
    "qtsl-experiments": receipt.qtsl_experiments_revision,
    closure: receipt.closure_revision,
  },
  trusted_incumbents:
    "supermartingales/out/glc-baked/sec1/incumbents (digests bound by ../MANIFEST.json)",
  standalone_geolog:
    "not required: every measured row and flagged warmup embeds a clean geolog acceptance receipt and the same benchmark container SHA",
  progress: "runner monitor emitted progress once per second",
  receipt: "campaign.json (produce-host.json is its byte-identical published twin)",
};

fs.writeFileSync(outPath, `${JSON.stringify(provenance, null, 1)}\n`);
console.log(`quiet-provenance: wrote ${outPath}`);
