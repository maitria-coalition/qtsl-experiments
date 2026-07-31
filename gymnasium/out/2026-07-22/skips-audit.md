# SKIPS audit — skip reasons re-verified (2026-07-22)

For each skipped make target, the stated reason was re-checked against the
actual tree state, and each un-skip trigger was checked for having already
fired.

## gymnasium-classic (Pendulum, species U1 continuous) — SKIP HOLDS

`make gymnasium-classic` prints exactly the graduating-from-the-workshop
message; no reproduction spine is in-repo (no
`gymnasium/classic/` artifacts directory exists). The published numbers
(certified worst-case return $\ge -1.192$; cover-refinement gap
$2.4\times10^{-4}$; five-mutant battery) appear only in
`chapter/sections/gymnasium.tex`. UN-SKIP TRIGGER (the authoring workshop's artifacts land
in-repo): **not yet fired** as of this audit.

The rider on the CartPole full-state successor is DISCHARGED: a successor
experiment is filed upstream ("CartPole full-state certified row (return=500 U1)
as the first genuinely large batched checking workload — $10^6$–$10^7$
polynomial cells, the provider-gpu flagship"), citing this SKIPS entry, the
chapter sentence (gymnasium.tex, "campaign's first genuinely large checking
workload"), and the prior scoping trail.

## gymnasium-mujoco — SKIP HOLDS

`make gymnasium-mujoco` prints "reserved (first row in flight; no shipped
rows yet)" verbatim; no published numbers exist in-repo to re-derive.
UN-SKIP TRIGGER (first shipped row): not fired in this repository. (An
inverted-pendulum first row is in flight upstream; when it ships here the
section becomes runnable.)

## chapter/sections/{claims,conventions,reproduction}.tex — SKIP HOLDS

All three exist and are pure prose (claim-species definitions, measurement
conventions, the reproduction-spine law); no numbers of their own. Editorial
rider from this campaign's atari leg: the atari delta ledger's netlist-
generation finding (1454-FF vs 1491-FF AIG generations; see
`gymnasium/atari/out/2026-07-22/delta-ledger.md`) may eventually obligate a
conventions sentence about netlist-generation pinning.

## `make chapter` umbrella — correctly not a leg

Confirmed union-target semantics in the Makefile; the closing single
full-chapter build after all section branches merge remains a final
integration step.

## Cross-cutting non-reruns — reconfirmed

Pinned external artefacts (`make fetch-pins`) remain re-parse-only; the
Lean/HOL4 rule-lemma double-checking cited in circuits.tex remains upstream
fact, out of this re-run's scope.
