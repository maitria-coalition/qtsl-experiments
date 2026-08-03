// gen-table.ts — regenerate the three-way comparison table from
// committed receipts. Tables are derived artifacts: this script reads
// probprog/out/comparison.json (bounds + walls + provenance, committed
// by the measurement protocol) and writes probprog/out/TABLE.md. It
// never computes a bound itself and refuses rows missing provenance.
//
// Receipt schema (comparison.json):
// {
//   "box": { "name": str, "arch": str, "cpu": str, "cpus": int },
//   "cases": [ {
//     "case": str,                     // geo | asym-rw | die-paradox
//     "query": str,                    // e.g. "posterior masses, matched bins"
//     "rows": [ {
//       "tool": str,                   // gubpi | diabolo-residual | house
//       "config": str,                 // exact flags / horizon
//       "bound_width": str,            // decimal rendering of the width metric
//       "width_metric": str,           // what "width" means for this row
//       "wall_median_s": number,       // end-to-end process wall, median
//       "runs": int,
//       "internal_time_s": number|null,// the tool's own reported time, context
//       "check_wall_s": number|null,   // house rows: certificate check wall
//       "checker": str|null,           // house rows: checker + closure label
//       "provenance": str              // receipts path / raw output file
//     } ]
//   } ]
// }

import * as fs from "node:fs";
import * as path from "node:path";

const HERE = path.dirname(path.resolve(process.argv[1]));
const IN = path.join(HERE, "out", "comparison.json");
const OUT = path.join(HERE, "out", "TABLE.md");

if (!fs.existsSync(IN)) {
  console.error(`gen-table: ${IN} missing — run the measurement protocol first`);
  process.exit(2);
}
const doc = JSON.parse(fs.readFileSync(IN, "utf8"));

let md = `# Guaranteed posterior bounds: three-way comparison\n\n`;
md += `Box: ${doc.box.name} (${doc.box.arch}, ${doc.box.cpus} cores, ${doc.box.cpu}). `;
md += `All rows same box; walls are end-to-end process medians; tools' own `;
md += `internally reported times are context, never the comparator.\n\n`;

for (const c of doc.cases) {
  md += `## ${c.case} — ${c.query}\n\n`;
  md += `| tool | config | bound width | wall median (s) | internal (s) | check (s) | checker |\n`;
  md += `|---|---|---|---|---|---|---|\n`;
  for (const r of c.rows) {
    if (!r.provenance) {
      console.error(`gen-table: row without provenance in ${c.case}/${r.tool}`);
      process.exit(1);
    }
    md += `| ${r.tool} | ${r.config} | ${r.bound_width} (${r.width_metric}) | ` +
      `${r.wall_median_s.toFixed(4)} | ` +
      `${r.internal_time_s == null ? "—" : r.internal_time_s} | ` +
      `${r.check_wall_s == null ? "—" : r.check_wall_s.toFixed(4)} | ` +
      `${r.checker ?? "—"} |\n`;
  }
  md += `\n`;
}

fs.mkdirSync(path.dirname(OUT), { recursive: true });
fs.writeFileSync(OUT, md);
console.log(`gen-table: wrote ${OUT} (${doc.cases.length} cases)`);
