# Section 1 quiet-conditions timing receipt

This directory records the `gb10-quiet/host` re-conduct of the three
Section 1 published-table families under the preregistered
quiet-conditions protocol (`../QUIET-PROTOCOL.md`, committed before the
first campaign ran). `PROVENANCE.json` is the human entry point; it is
*derived* from the receipt's own fields by
`supermartingales/quiet-provenance.ts`, so it cannot disagree with the
receipt it fronts.

The producer source is the `mtk` repository at the revision pinned in
`PROVENANCE.json`. Build the recorded executable with:

```
git clone https://github.com/maitria-coalition/mtk
cd mtk
git checkout <revisions.mtk from PROVENANCE.json>
cargo build --release -p mtk-strategies --bin supermart-sec1
```

The unsuffixed URL is the coalition's intended public release endpoint;
until the private-preview flip, this is a complete provenance recipe
rather than a claim that an unauthenticated clone is available. Conduct
the campaign with `make sec1-quiet-retime SUPERMART_SEC1=<binary>` from
the repository root (the exact expansion is the `command` field of
`PROVENANCE.json`). The runner refuses — exit 3, no receipt — when the
1-minute load average exceeds the declared gate at campaign start.

`campaign.json` is a version-3 timing receipt: the version-2 accounting
identity unchanged, plus a `load_context` block (declared gate,
start/end/max load averages, CPU affinity and niceness read back from
the process itself) and per-row 1-minute load averages. The receipt
validator enforces that the recorded starting load actually passed the
declared gate and that the max-observed load re-derives from the
receipt's own samples, so the directory's quietness claim is checkable
from the receipt alone.

Every measured output is required to match the committed incumbent
bytes and checked claim count, exactly as in `../gb10-retime/`, whose
load-uncontrolled receipts remain committed beside this family for
comparison.
