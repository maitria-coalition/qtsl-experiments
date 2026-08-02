# Sweep-table timing coverage: the four empty rows

Status: IN PROGRESS (2026-08-01)

## The question

The sweep table's first four rows — `persist_2d`, `faulty_marbles`,
`faulty_unreliable`, `faulty_regions` — carry no produce/geolog wall-clock
timings, while every later row does.

## Findings

- Three recall passes converged on the missing dependency: Section-1 timing receipts are structurally supported, but the exact MTK runner refuses until one trusted benchmark-level incumbent exists for each of `faulty_marbles`, `faulty_regions`, and `faulty_unreliable`.
- Exact contract inspected at MTK `541a06f488967215a08473ed048340c28b2ed8b7`: `supermart-sec1 retime` defaults `--compare` to `supermartingales/out/glc-baked/sec1/incumbents`; every production run must emit exactly one `<bench>.glc`, match that incumbent byte-for-byte, pass the real QTSL provider, and report the synthesis claim count equal to the checker tally.
- Authoritative vintage is the committed Section-1 corpus under `supermartingales/out/sec1`, pinned by MTK's 81-entry `VINTAGE.sha256`; exact synthesis expects 23 claims for marbles and 29 each for regions and unreliable.
- The shared artifact must be emitted through MTK's `SupermartSynthStrategy` + real conductor + `emit_forest` path. Concatenating per-row containers cannot satisfy the shared-hypernet semantics or the runner's one-container contract.
- The landed timing validator reads `supermartingales/out/glc-baked/sec1/MANIFEST.json`; every receipt hash must equal its benchmark entry, and its `binary_sha256` must equal the corresponding `onesys-driver` or `glc-check` identity in `<machine>-retime/PROVENANCE.json`.
- `persist_2d` remains outside this dependency and receives no invented timing or absence change.

## Verification receipts

- MTK `541a06f488967215a08473ed048340c28b2ed8b7` synthesized the three shared containers twice through `SupermartSynthStrategy(Section1)` and `Conductor::run_direct`; both runs were byte-identical. SHA-256: marbles `8fa8b39b411bc18c03918b3b87e74480a68b0a0a460a0a8ddbea1e0205eb8266`, regions `6f6574e27f70d6c5ab937b05d14db859e2528e91f0e1f9da8003913a3d0532c5`, unreliable `d6fec4dd519ab5ab9ac5a4da7c57a4a0bcd6a2435d4683117c6a605bcf8d2789`.
- QTSL `a9b33d14587d1c08a6510ef6f0d42be08f132aca` `glc-check --baked-cert --expect-claims` accepted exact tallies 38/38, 48/48, and 48/48.
- A canonical semantic mutation battery re-lowered marbles through derived encoders, then re-sealed: dropping one claim row was caught by the exact-count gate; altering a checker-relevant rational row failed/refused; the clean control accepted 38 claims.
- Wrong-benchmark swap (regions bytes installed at the marbles incumbent path) was refused as `incumbent byte mismatch`.
- The timing receipt validator accepted its valid produce/geolog fixtures and refused all 19 structural mutants. No timing value or absence declaration was added.

## Runner executability finding

The exact requested MTK tip builds and launches, but its unmodified runner cannot complete a clean production row: it compares the checker tally (38/48/48 shared claims) to the synthesis `n_claims` field (23/29/29 proof trees), then refuses `checked claim count ... != planned synthesis count ...`. A local diagnostic correction reading `share_audit.claims` instead completed a byte-identical marbles run. Therefore the dependency is fully present here, but MTK `541a06f` itself needs that count-field repair before its runner can execute successfully without modification.
