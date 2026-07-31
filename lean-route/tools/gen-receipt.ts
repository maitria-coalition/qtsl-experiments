#!/usr/bin/env node
/* gen-receipt.ts — emit a kernel-acceptance receipt module for one
 * sealed certificate container.
 *
 *   node --experimental-strip-types gen-receipt.ts CERT.geolog OUT.lean
 *
 * The emitted module embeds the container bytes and asks the Lean
 * kernel to evaluate the entire reflection route over them:
 * `acceptBytesSealedB bytes = true` discharged by `decide`.  Wall
 * time and peak memory of elaborating the emitted module ARE the
 * kernel-route measurement; the module itself is otherwise identical
 * in shape to the committed receipt modules of the reflection route.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { basename } from "node:path";

const [cert, out] = process.argv.slice(2);
if (!cert || !out) {
  console.error("usage: gen-receipt.ts CERT.geolog OUT.lean");
  process.exit(2);
}

const raw = readFileSync(cert);
const stem = basename(cert).replace(/\.geolog$/, "");
// UpperCamel identifier from the fixture stem.
const name = stem
  .split(/[^A-Za-z0-9]+/)
  .filter(Boolean)
  .map((w) => w[0].toUpperCase() + w.slice(1))
  .join("");

const bytes = Array.from(raw).join(", ");

writeFileSync(
  out,
  `import Reflection.Accept

/-!
# Kernel acceptance receipt: ${name}

The embedded bytes are a sealed certificate container (${raw.length} bytes,
fixture \`${basename(cert)}\`).  \`accepted\` is checked by the kernel:
\`decide\` evaluates the entire route — container decode, row scan,
certificate reconstruction, and (by root arm) elevation + validity
elaboration or the enclosure-judgment obligation — inside the trusted
evaluator, and the acceptance theorem then applies with no further
hypotheses.  Generated; do not edit by hand.
-/

namespace PPLC.Geolog.Receipt.${name}

set_option maxRecDepth 131072

def bytes${name} : PPLC.Geolog.Bytes := [${bytes}]

set_option maxHeartbeats 0 in
theorem accepted : acceptBytesSealedAnyB bytes${name} = true := by decide +kernel

theorem claim_valid (a : AcceptedAny) (h : acceptBytesSealedAny bytes${name} = some a) :
    a.claimValid :=
  acceptBytesSealedAny_sound _ _ h

end PPLC.Geolog.Receipt.${name}
`,
);
console.log(`${out}: ${raw.length} container bytes -> theorem accepted (by decide)`);
