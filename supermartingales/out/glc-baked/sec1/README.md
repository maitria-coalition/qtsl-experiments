# Section 1 checked-production incumbents

`incumbents/<benchmark>/<benchmark>.glc` is the single authoritative shared
proof-hypernet container for each Section 1 benchmark. `MANIFEST.json` binds
its canonical bytes and exact checked claim count. The files were synthesized
from the committed 81-listing vintage by MTK at the pinned revision recorded in
`incumbents/PROVENANCE.json`, through the real conductor and QTSL provider.

The `supermart-sec1 retime` runner uses these files as mandatory byte-for-byte
comparators. It refuses missing, renamed, unchecked, undercounted, or merely
semantically similar outputs. Timing receipts belong only in machine-specific
`<machine>-retime/` directories; this incumbent lane introduces no timing value
or timing-absence declaration.
