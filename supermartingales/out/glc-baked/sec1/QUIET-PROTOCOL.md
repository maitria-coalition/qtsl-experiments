# Section 1 quiet-conditions retiming protocol

This file is the preregistered protocol for the `gb10-quiet` timing
family. It is committed **before** any quiet campaign was run (the git
history of this branch is the ordering witness), so that quiet-conditions
cells are produced *by protocol* rather than by rerunning a
load-uncontrolled campaign until the numbers look quiet.

## Motivation

The `gb10-retime/` receipts were honestly produced on a shared box under
substantial concurrent load (their `PROVENANCE.json` records a 1-minute
load average of ~20 at campaign start), and the same-morning replication
evidence shows medians moving a few percent under such load — worst
observed: `faulty_regions` +8% relative to a lower-load conduct of the
same protocol. Selection ("rerun until the numbers come back down") would
be an invalid fix: it conditions the published value on the outcome. The
valid fix is a *declared admission gate* checked before the campaign
starts, plus recorded load samples that let a reader audit the conditions
the receipt claims.

## Protocol (all parameters declared a priori)

1. **Load gate.** The runner is invoked with `--max-load-1min 4.0`. At
   process start it reads the host 1-minute load average; if it exceeds
   4.0 (20% of this 20-core box), the campaign **refuses** (distinct exit
   code, no receipt written). The gate value, the start/end load
   averages, and the maximum 1-minute load observed at any sample point
   are recorded in the receipt's `load_context`, and every row records
   the 1-minute load average immediately before and after its bracket.
   The receipt validator refuses a receipt whose declared gate is above
   its recorded starting load — a receipt cannot claim a gate it did not
   pass.
2. **CPU affinity.** The campaign is pinned with `taskset -c 5-9,15,16`:
   the Cortex-X925 (3.9 GHz) cores of the `gb10` machine that are
   available to batch work (two further X925 cores are reserved for
   resident audio services and one for the interactive tier; the
   Cortex-A725 cores are excluded so big/little heterogeneity cannot
   enter the sample). The receipt records the effective
   `Cpus_allowed_list` read back from the process itself.
3. **Scheduler niceness.** Niceness 19, set *absolutely* (`renice 19` on
   the launching shell rather than a relative `nice -n 19`, so a
   priority-boosted parent shell cannot leak into the campaign) —
   matching the incumbent `gb10-retime` campaign's declared posture, so
   the quiet family differs from the incumbent family in load conditions
   and affinity only. The receipt records the effective nice value read
   back from the process. (Protocol amendment, pre-campaign: the first
   smoke of the read-back instrument showed a relative `nice -n 19` from
   a boosted shell landing at effective nice 11 — which also means a
   hand-written "nice -n 19" in earlier provenance was a command-line
   fact, not a verified niceness; the read-back field is the repair.)
4. **Page cache.** Warm-by-protocol: no cache dropping (shared box), no
   cold-cache claim. The roster's single flagged warmup repeat is the
   only warmup mechanism, exactly as in the incumbent campaigns.
5. **Roster and identity.** Unchanged from `gb10-retime`: 1 flagged
   warmup + 10 measured repeats per benchmark, serial, whole
   checked-production wall per row, and every output must be
   byte-identical to the trusted incumbent containers.
6. **Receipts.** Version 3 = version 2 plus the load context above; the
   version-2 accounting identity is unchanged. Receipts land in
   `gb10-quiet-retime/` under the `gb10-quiet` machine tag (see
   `MACHINES.md`).

## Running it

Build the runner from the `mtk` repository at any revision that emits
version-3 receipts (the revision actually used is pinned by the receipt
and its derived `PROVENANCE.json` once a campaign publishes):

```
cargo build --release -p mtk-strategies --bin supermart-sec1
```

then, from this repository's root:

```
make sec1-quiet-retime SUPERMART_SEC1=/path/to/supermart-sec1
```

The target stages out of tree, so a refusal (exit 3, appended to
`build/sec1-quiet-refusals.log`) leaves the committed tree untouched; a
success installs `campaign.json`, its `produce-host.json` twin, the
retained containers, and a `PROVENANCE.json` derived from the receipt's
own fields into `gb10-quiet-retime/`, then re-validates all committed
timing receipts.

## Anti-selection commitment

The **first** campaign that passes the load gate is the one published,
whatever its numbers. A rerun is licensed only by a byte-identity or
validation failure (properties independent of timing), never by the
timing values themselves. Gate refusals are recorded (invocation log
beside the campaign target), not discarded.

## Preregistered expectations

- E1: 33/33 rows byte-identical to the incumbents (timing-independent).
- E2: quiet medians land within ~3% of the lower-load replications of
  this same protocol observed on 2026-08-02 (approximately:
  `faulty_marbles` ≈ 0.042 s, `faulty_unreliable` ≈ 0.063 s,
  `faulty_regions` ≈ 0.071 s) — in particular, `faulty_regions` is
  expected to come in *below* the 0.0764 s median measured at load ~20.
- E3: because the timing table selects the minimum median across machine
  configs, quiet cells can only lower rendered values or leave them
  unchanged; any cell they win is tagged `gb10-quiet/host`.

If E2 fails under a passing gate, that is a finding about the protocol
or the machine, and it is published as such; it is not grounds for a
rerun (see the anti-selection commitment).
