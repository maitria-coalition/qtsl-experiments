# Section 1 checked production, independent machine (w2145)

This directory is the independent-machine / cross-architecture replication of
the Section 1 checked-production campaign: the same three benchmarks, the same
one-warmup/ten-repeat serial host-lane protocol, the version-2 receipt (process
wall + attribution), run on `w2145` (see `MACHINES.md`) — an x86_64 box, where
the published cells' machine (`gb10`) is aarch64.

`PROVENANCE.json` is the entry point: revisions, binary digests, the exact
invocation, and the environment block (co-tenant load, the scheduler
calibration probe, and the stability replication). `produce-host.json` is the
version-2 receipt; `produce-host-run2.json` is a second full campaign run as a
same-box stability datum (medians agree with run 1 within ~1–2%). `FINDINGS.md`
states the cross-machine picture and the publication rule.

These receipts are deliberately NOT in a `<machine>-retime/` directory: the
best-of scan in `tables/gen_tables.ts` must not consume them, because the
rendered table cells remain the gb10 receipts (this directory is evidence
about machine-dependence, not a candidate for the cells). The receipt was
verified against the landed validator by transient staging (accepted; a
corrupted-sha mutant refused; rendered tables byte-identical with and without
the staged copy). Every one of the 66 measured-or-warmup rows across both runs
emitted bytes identical to the committed incumbents — the certificates are
architecture-independent; only the walls are the machine's.
