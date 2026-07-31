# lean-route — the Lean reflection route, measured

The toolkit's third checking path loads the same sealed certificate
container every other checker consumes into Lean and has it accepted as
a theorem — compiled (a native executable built by the Lean compiler
from the route's definitions) and in the kernel itself (a generated
receipt module whose `decide` evaluates the whole pipeline inside the
trusted evaluator). The route is expected to be far too slow at machine
scale; this directory measures that expectation into a scaling law and
an **admission gate**: which certificate species are even worth a
Lean-reflection timing under a fixed time budget.

## Contents

- `fixtures/` — the size ladder: fifteen production sealed containers
  (`*.geolog`, 603 B – 126 KB, seven certificate families), committed
  byte-for-byte, each beside its object-row listing (`*.rows.txt`) so
  the pinned TypeScript reference lane can replay it (`make
  lean-route-recheck`). As everywhere in this repository, producers are
  not part of the trust story: the certificates stand on their own.
  `MANIFEST.tsv` carries per-rung structural counts (rows, proof
  nodes, species) and the generation-time checker verdicts.
- `out/results.json` — the timing receipts: one row per certificate ×
  route × pass, with outcome vocabulary `accept | refuse | timeout |
  oom | error | not_attempted`, per-stage wall clocks, peak RSS, and
  the measuring machine's identity embedded in every row.
- `species.json` — the campaign species census the admission table is
  computed over (byte ranges of each species' committed certificates,
  fragment status).
- `ADMISSION.md` — **generated** (`make lean-route`): the fitted law
  and the per-species verdicts, in the form campaign runners consume.
- `out/glc/` — the same fifteen certificates re-sealed via mtk as
  `.glc` containers on the current staged theory chain, checked from
  the bytes by the geolog fast-path checker (`make lean-route-glc`):
  full applicative obligation enumeration + a 14-mutant battery;
  scope split and reach boundaries in `GLC-CERT.md`.
- `out/qtslab/` — the same fifteen certificates as the certified
  checker consumes them: the committed fixtures with their three
  prerequisites re-marked content-resolved (`QTSLAB-MANIFEST.json`
  discloses the container-table-only transform per container;
  `tools/mint-qtslab-vintage.py` re-derives the mints byte-for-byte
  from the fixtures, and `make lean-route-mints` verifies that).
- `out/glc-times.json`, `out/qtslab-times.json` — per-certificate
  timing receipts for the two production routes (the three-route
  figure's blue and green series): one row per certificate × pass,
  end-to-end wall clock of one process pinned to one core, host and
  binary identity embedded. Measurements, not derivations — replaced,
  never blended (`tools/time-routes.py` re-measures).
- `tools/run-ladder.sh`, `tools/gen-receipt.ts` — the Lean-route
  measurement harness (below).

## Re-deriving vs. re-measuring

`make lean-route` re-derives everything derivable: replays every rung
through the pinned reference checker, recomputes the fit, the tables,
the figure data, and the admission verdicts from `out/results.json`,
and typesets the chapter section. Timing receipts themselves are
measurements, not derivations; they carry their machine identity and
are replaced, never blended, by a re-measurement.

To re-measure: build the toolkit repository's `lean/checkers/reflection`
package (`lake build geolog_reflect` in that directory) on a dedicated,
otherwise-idle machine; warm the build; then

    tools/run-ladder.sh fixtures/ <path-to-lean/checkers/reflection> out-new/ 600

with `REPS` (compiled-route passes, default 3) and `KERNEL_MAX_BYTES`
(largest rung to attempt in the kernel, default unlimited) as
environment knobs. The harness applies the budget to each certificate
on each route; a run that exceeds it is recorded as `timeout` and
rendered as an em-dash in every derived table. Kernel attempts want a
large-memory machine and still mostly end at the budget or the OOM
killer — that is the finding, not a harness defect. Merge the emitted
`receipts.jsonl` into `out/results.json` (`jq -s`) and re-run
`make lean-route`; the generator refuses receipts that span more than
one machine.
