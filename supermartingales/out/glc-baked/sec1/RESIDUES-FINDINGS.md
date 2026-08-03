# Section 1 residues: gb10 cold-anchor retake (v2 runner) + receipt path-field normalization

Working notes for the final two items of the 2026-08-02 timing-interpretation
follow-up program. Findings append here as they arrive; this file is the
leg's durable record.

## Scope

1. **gb10 cold-anchor retake at v2+**: the published gb10 cold-anchor cells
   (median 1.24 s, fresh-process single-repeat) were produced with the v1
   runner, whose progress monitor joins by sleeping out a full 1 s tick —
   a fixed inflation that dominates cold single-row cells. The independent-
   machine campaign's apparent cross-machine cold inversion (w2145 0.743 s
   vs gb10 1.238 s) was runner vintage, not architecture. Retake: the same
   anchor benchmark and protocol (fresh process per repeat, x10), v2+ runner
   built from landed mtk main (pinned sha in receipts), quiet-gate conduct
   (loadavg <= 4.0, two gate attempts max; refusal receipts are an honest
   partial). Receipts land beside the existing cold-anchor family with the
   vintage note; FINDINGS updated so the cross-machine cold comparison is
   same-vintage.

2. **Receipt register cleanliness**: the landed gb10-checker-compare
   receipts carry private build-tree paths in the binary-path field family
   that the independent-machine campaign normalized to bare binary names.
   Normalize those path/provenance fields to the neutral form — measurement
   fields byte-untouched — and sweep the rest of supermartingales/out/ for
   the same field family (home-directory prefixes, build-worktree segments).

## Findings

### Residue 2 — receipt register cleanliness: DONE

- The four `gb10-checker-compare/` receipts (`cold-anchor.json`,
  `retain-vs-noretain.json`, `external-shared.json`,
  `external-perrow-bridge.json`) carried absolute build-tree paths in the
  `path` field under `checker_binary`/`runner_binary`. All six occurrences
  are normalized to the bare binary name (`glc-check`, `supermart-sec1`) —
  the byte-exact string replacement touched nothing else (diff witness: six
  `path` lines, zero measurement bytes). The `sha256` beside each path — the
  identity that matters — is untouched. This matches the
  `w2145-checker-compare/` convention, whose README documents the same
  one-field-family transformation.
- Whole-tree sweep (`grep -rla` over `supermartingales/out/` for
  home-directory prefixes, build-worktree segments, and local data-dir
  names): **nothing found beyond those four files.** No other receipt family
  in the tree carries the leak class.
- Recurrence prevention at the source: `supermartingales/checker-compare.py`
  now emits bare binary names directly, and its `--glc-check` argument lost
  its absolute home-directory default (now required, with a clear refusal) —
  the committed driver no longer contains any private path.

### Residue 1 — gb10 cold-anchor retake at v2+: DONE (attempt 2 passed the gate, 2026-08-03; receipt `gb10-checker-compare/cold-anchor-v2.json`)

What the retake needs, all landed on this branch:

- **Runner**: `supermart-sec1` built at mtk `b414ba5baab4ba4f894cc924d3a305ddb862c43f`
  (the version-3-receipt merge: monotonic process wall, channel-wakeup
  monitor join — the 1 s sleep-join that inflated the version-1 cold cells
  is gone at this vintage; plus the `--max-load-1min` quiet gate). Build
  recipe: `cargo build --release -p mtk-strategies --bin supermart-sec1`.
  The binary built for the attempts had sha256
  `f8c992d5ae7a5df2890589f377bd6a720f7297fd2c01379333808ed8fd33fc67`.
- **Driver**: the `cold` cell of `supermartingales/checker-compare.py` now
  passes the quiet gate through to every fresh runner process (each of the
  10 spawns gates itself; a refusal is exit 3 with no receipt) and records
  each repeat's 1-minute load average from the runner's own version-3
  `load_context`.
- **One-verb re-fire**: `make sec1-cold-anchor-v2
  SUPERMART_SEC1=/path/to/supermart-sec1 GLC_CHECK=/path/to/glc-check
  QTSL_REV=<qtsl rev the checker was built from>` — quiet conduct per
  `QUIET-PROTOCOL.md` (gate 4.0, `taskset -c 5-9,15,16`, absolute
  `renice 19`), staging out of tree, refusals appended to
  `build/sec1-cold-anchor-v2-refusals.log`, success installs
  `gb10-checker-compare/cold-anchor-v2.json` beside the version-1 cell.
  The receipt directory stays outside the tables scanner's
  `^<machine>-retime$` pattern: no rendered cell can move.

Conduct record (2026-08-02, ~11:45–13:10 BST; committed refusal log:
`gb10-checker-compare/cold-anchor-v2-refusals.log`):

- Gate attempt 1 (10:57:52Z): launched inside an observed quiet window
  (1-min load 3.82); by spawn the 1-minute load average read 4.28 and the
  runner refused per protocol — exit 3, no receipt written. The gate
  working as declared, recorded here as the protocol requires (refusals
  are recorded, not discarded).
- ~65 further minutes of 10-second polling never met the launch margin
  again: sustained daytime parallel-build load (1-minute floors 6–12,
  spikes to 22) from concurrent builds, CI gate suites, and a
  book build. The second licensed gate attempt was deliberately not spent
  into a known-refusing load — a guaranteed refusal adds nothing the
  attempt-1 receipt and these load samples don't already record.

Expected numbers, preregistered here for the retake (from the runner-vintage
diagnosis, not from any gb10 v2 cold measurement — none exists yet): the
version-1 gb10 cold anchor's 1.238 s median carries up to ~1 s of monitor
sleep-join; the v2 process wall should land near the w2145 v2 anchor's
shape (0.743 s there, at a 3.0–3.5× cold-intercept machine ratio in the
other direction), i.e. roughly 0.2–0.5 s around the unchanged ~44 ms row
wall. If it lands elsewhere under a passing gate, that is a finding, not
grounds for a rerun.

Comparison hygiene while the retake was pending (now discharged below):
the cross-machine cold comparison (w2145 0.743 s vs gb10 **v1** 1.238 s)
was cross-VINTAGE and was not to be ratioed — the vintage cautions in
`gb10-checker-compare/FINDINGS.md` and `w2145-checker-compare/README.md`
governed; the v1 cell remains history-only under the same rule.

Conduct record, gate attempt 2 (2026-08-03, the retake; the second and
last licensed attempt):

- The quiet window was waited out, not selected: the mtk post-merge full
  suite (00:45:58Z–~03:22Z, previous-cycle duration 9614 s) held the
  1-minute load at 5–18 all night; the launch poll armed only after its
  unit went inactive. The concurrent qtsl post-merge cycle was in its
  barn-routed (remote) Lean phase at launch — locally quiet by
  construction. To close the observation→spawn gap that refused attempt 1
  (3.82 observed → 4.28 at spawn), the poll and the launch ran in one
  shell: 20-second load polling with the make invocation exec'd in the
  same breath the threshold (≤3.2) was met.
- Attempt 2 (03:29:59Z): launched at 1-minute load 3.16; all 10 fresh
  spawns self-gated at 3.16 and none refused. Campaign wall ~2 s
  end-to-end; receipt installed.

Verdict against the preregistered expectation (likelihood-shaped):

- **Vintage diagnosis: confirmed and quantified.** v2 median process wall
  0.1836 s (spread 0.1785–0.1867, ±2%) vs v1's 1.238 s (spread
  1.11–4.91 s under ambient load): the ~1.05 s delta is the monitor
  sleep-join the v3 runner removed, and the two-orders-of-magnitude
  spread collapse is the quiet conduct doing what the protocol claims.
- **Row wall unmoved, as expected:** median 0.0424 s vs the anticipated
  ~44 ms (v1: 0.0439 s) — the retake moved only the process envelope.
  10/10 containers byte-identical.
- **One miss, recorded as a finding, not grounds for a rerun:** the
  preregistered band was 0.2–0.5 s; the observed 0.184 s sits just below
  it. The gb10 v2 cold intercept (~0.14 s of setup around the row) is
  smaller than the w2145-shape extrapolation predicted — the cross-machine
  cold-intercept ratio lands at ~4.0–4.2× (w2145 0.743 s vs gb10
  0.184 s), slightly above the anticipated 3.0–3.5× bracket and in line
  with the machines' phase-by-phase warm scale (3.0–4.9×,
  `w2145-independent/FINDINGS.md`).
- The same-vintage cross-machine statement replacing the retired
  inversion: **gb10 is ~4× faster cold than w2145 at v2** — the v1
  "inversion" was runner vintage plus load, in the direction and roughly
  the magnitude the diagnosis predicted.
