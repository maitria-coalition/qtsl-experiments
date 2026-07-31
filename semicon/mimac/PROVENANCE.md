# PROVENANCE — MIMAC 7 recovery

*Hunt conducted 2026-07-19. Mission: recover the seventh MIMAC
dataset cited in the literature as "MIMAC 1–7," of which a prior survey
could only confirm six public datasets.*

## Answer, upfront

**MIMAC 7 is real, distinct, and its bytes are not lost.** It ships inside the
same `MIMAC_datasets.zip` archive the campaign's scoping census already found at the
p2schedgen mirror — one directory level deeper than a top-level listing shows
(`factory_datasets/set7/`). The census's "six" undercount has a precise,
now-documented cause: the testbed's own catalogue text (`readme.txt` /
`datasets.doc`) says "the testbed currently contains 6 factory-level
datasets" and enumerates Set1–Set6 only — but that text is dated **1997-08-07**,
while Set7's files inside the same archive are dated **1997-08-12/15**, i.e.
Set7 was added eight days *after* the catalogue description was last written,
and the catalogue was never revised to mention it — not in 1997, and not in a
2005 re-save of the identical text as `datasets.doc` (Word metadata: created
and saved 2005-07-11, content byte-identical to the 1997 readme). Anyone who
reads the prose catalogue (which is most citing papers, and the census) will
report six; anyone who actually unzips the archive and lists directories
finds seven.

## Identity of MIMAC 7 (a.k.a. "Fab7")

From `set7/set7.cf` (the dataset's own comments file, dated 1997-08-12):

- **Dataset name in the file itself: "Fab7."** Revision 0.
- **Source**: "This testbed model represents a fictitious semiconductor wafer
  fab. The dataset was obtain[ed] from a paper by Lar[e]nce M. Wein" — i.e.
  Lawrence M. Wein (Sloan/Stanford GSB operations researcher; several
  well-known late-1980s/1990s papers on wafer-fab scheduling policy). This
  is the one MIMAC set that is NOT drawn from real anonymized fab floor data
  the way Set1–Set6 are — it is a fictitious/synthetic model adapted from
  Wein's published testbed, folded into the MIMAC format for uniformity.
- **Characteristics** (from set7.cf, cross-checked against the independent
  published table below): 1 product ("R&D_wafer"), 1 process flow, 24
  equipment groups, no operators modelled, no rework modelled, ~408 wafer
  starts/month, avg. cycle time 726.64 hr in the sample run (FIFO dispatch,
  constant inter-release time, Factory Explorer simulator, 3.5-year runs ×5
  replications).

**Independent confirmation** — Zhou & Rose, "A Bottleneck Detection and
Dynamic Dispatching Strategy for Semiconductor Wafer Fabrication Facilities,"
*Proceedings of the 2009 Winter Simulation Conference*, pp. 1646–1653
(`refs/zhou-rose-2009-wsc-bottleneck.pdf`, fetched from
`https://www.informs-sim.org/wsc09papers/155.pdf`), Table 1 "Basic
characteristics of MIMAC models," citing (Fowler and Robinson 1995) directly:

| MIMAC | Products | Tool Groups | Tools | Operator Groups | Operators | Process Flows | Max. Steps |
|---|---|---|---|---|---|---|---|
| 1 | 2 | 83 | 265 | 32 | 90 | 2 | 245 |
| 2 | 7 | 97 | 274 | 97 | 274 | 7 | 354 |
| 3 | 11 | 73 | 354 | 0 | 0 | 11 | 547 |
| 4 | 7 | 35 | 69 | 0 | 0 | 7 | 92 |
| 5 | 21 | 85 | 176 | 4 | 24 | 21 | 266 |
| 6 | 9 | 104 | 228 | 7 | 97 | 9 | 355 |
| **7** | **1** | **24** | **38** | **0** | **0** | **1** | **172** |

Row 7's zero operator groups/operators and single product match the recovered
`set7.cf` exactly (no operators modelled, 1 product, 24 equipment groups).
This is independent corroboration from a paper that clearly worked from the
original Fowler & Robinson 1995 SEMATECH report table, not from the p2schedgen
mirror — two unrelated sources agree.

**The "MIMAC 1–7 = 6 datasets + 1 Engineering Model" hypothesis, refuted.**
The census flagged this as a live hypothesis worth checking. It is false:
`MIMACI_Engineering.zip` (also mirrored at p2schedgen, contents in
`full-archive/`) is a wholly separate artefact — an engineering-lot variant of
the *Set1* model specifically (its internal files are `engineering_route_1.rte`,
`engineering_route_2.rte`, `route_1.rte`, `route_2.rte`, dated 2015 as a later
re-packaging), not a numbered member of the Set1–Set7 sequence. Set7/Fab7 is
the genuine seventh member; the Engineering Model is an eighth, unnumbered
extra.

## Sources and capture details

1. **Primary recovery — live fetch, not an archive replay.**
   `https://p2schedgen.fernuni-hagen.de/fileadmin/ess/uploads_p2schedgen/Simulation/MIMAC_datasets.zip`
   and the sibling
   `https://p2schedgen.fernuni-hagen.de/fileadmin/ess/uploads_p2schedgen/Simulation/MIMACI_Engineering.zip`
   — fetched directly (not via Wayback) 2026-07-19, from the live page
   `https://p2schedgen.fernuni-hagen.de/index.php/downloads/simulation`
   (Chair of Enterprise-wide Software Systems / Lars Mönch's group,
   FernUniversität Hagen — the same modern mirror the prior survey
   already cited for "6" datasets; census evidently inspected the top-level
   listing / page prose rather than unzipping to `factory_datasets/set7/`).
   No login, registration, or paywall encountered — the file serves directly
   over plain HTTPS with no gate.
2. **Original SEMATECH/ASU site, via Internet Archive** (confirms the dead
   `www.eas.asu.edu/~masmlab` site's own framing, and dates the catalogue-text
   staleness): the site's `ftp.htm`/`ftp_readme.htm` frame pages, captured
   repeatedly 1997–2007
   (e.g. `http://web.archive.org/web/20070118015429/http://www.eas.asu.edu:80/~masmlab/ftp_readme.htm`),
   state the same "testbed currently contains 6 factory-level datasets" text
   and point to `ftp://ftp.eas.asu.edu/pub/centers/masmlab/` for the actual
   testbed — **that FTP URL was never itself captured by the Wayback Machine**
   (confirmed: `http://web.archive.org/web/20070118015429/ftp://ftp.eas.asu.edu/pub/centers/masmlab/`
   returns IA's "has not archived that URL" 404; a CDX domain query for
   `ftp.eas.asu.edu` returns zero captures). The HTTP mirror of the ASU
   masmlab site (`~masmlab/...`) was captured extensively (566 distinct URLs
   across 1997–2007, CDX-enumerated during this hunt) but none of those captured
   paths is a dataset directory listing — the site used frames pointing at
   the FTP server for the actual data, and FTP browsing was outside what IA's
   crawler of the era captured. **So the original ASU HTTP/FTP site's own
   copy of Set7 is confirmed unrecoverable via Internet Archive** — the
   p2schedgen mirror is the operative recovery path, and it worked.
3. **Citation-forensics confirmation**: Zhou & Rose 2009 (above), independently
   sourced from Fowler & Robinson 1995 directly, confirming the 7-row table.

## What would have moved the "MIMAC I Engineering Model = #7" hypothesis to true

Nothing did — it's refuted by direct inspection of both archives' internal
file structure (Set7/Fab7 has its own 5-datafile-plus-comments MIMAC format,
matching Set1–Set6 exactly; the Engineering Model has an entirely different,
newer, Factory Explorer-native file format tied specifically to Set1's
routes). Had the Engineering Model's internal files instead referenced "Fab7"
or matched Set7's product/equipment-group counts, that would have supported
the merge hypothesis; they don't.

## Checksums (sha256, this fetch)

```
c38e71ec786bb90ae9939aa0efa9571dcbba1a53360247e03161ee21de1bd658  set7/set7.cf
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  set7/set7.os   (0 bytes — no operators modeled)
302603eb696dc78d01aeae54463284bd531fc6485d4ea725ac4b7694b5d92dce  set7/set7.pr
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  set7/set7.rw   (0 bytes — no rework modeled)
7561f9f9b5fd145977c49c238a37067593935b9db6905adfabe6f43f1e8ec68e  set7/set7.ts
71f2f07b3567b4fc2b116f25692e96fec2a48682e72a8672473e96a161909aab  set7/set7.vr
249770243fcabaa60f9ef464b17983996af53037950a2e599e1cfec0556d14db  set7/Set7.XLS
39900c422f0df059492bd596db8ba9aca1744700b883cda9a28732fef4737a50  set7/set7.zip  (nested duplicate copy of the 7 files above, as shipped by the lab)
41b9617748e3831f249331701ee28f51448e8945c7445d089a216ba4e8191883  full-archive/MIMAC_datasets.zip   (Set1–Set7 + minifab + format docs, 2.4MB, as fetched live 2026-07-19)
c1bfa60842a49ac540ccb4468f5938be9c2d24853b314e19c785ecc09b24c8ff  full-archive/MIMACI_Engineering.zip  (separate artifact, NOT part of 1–7 numbering)
d3fa6175e362b9143c272c78f3ef6460d8fd5e9dd29fa3d893313097ea29a67a  refs/zhou-rose-2009-wsc-bottleneck.pdf
1df9fbf61a4276f03b1feea949a48de6aa29d9942e4f5182700690d77a42141d  readme.txt      (the stale 1997-08-07 catalog text, kept for the record)
0ca5e8e261b5d2f0888bfee880ab48e907179f0f5d20d5384a172da4c2c8f499  testbed.pdf     (Feigin/Fowler/Robinson/Leachman 1994 format spec)
51cfd5e82ad3db2bf0d4904affa002e9de837881c0e16ada2992bb0565386d65  testbed.tex     (same, TeX source)
```

Internal filesystem timestamps preserved through the zip (evidence for the
"added after the readme" dating claim above): `set1`–`set6` files carry
1997-05-12 timestamps; `set7`'s five data files + `.cf` carry 1997-08-12
through 1997-08-15 timestamps, eight days after `readme.txt`'s stated last
revision of 1997-08-07. `MIMAC_datasets.zip` and `MIMACI_Engineering.zip`
themselves were last repackaged 2015-05-07 / 2015-09-02 by the p2schedgen
maintainers (per the outer zip directory-entry timestamps), preserving the
original inner-file dates unchanged.

## Contact information (listed for reference only)

The recovered `readme.txt` requests (not requires) that testbed users email
John Fowler (john.fowler@asu.edu, ASU) to be notified of updates; this is a
courtesy request from 1997, not a license gate — the file states "the
datasets are available to anyone." No outward contact was made in the
course of this work.
