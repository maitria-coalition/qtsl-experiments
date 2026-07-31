# certified-checker replay receipts

Receipts for the replay (2026-07-23) of this campaign's stored
certificate corpus — the 4,105 certificate files behind eight of the
seventeen published-table rows, each pinned by hash in
`supermartingales/MANIFEST.jsonl` — through the current build of the
certified checker, a standalone aarch64 binary emitted from the
verified-compiler pipeline inside HOL4.

Result: **4,105 of 4,105 accepted**, in exact agreement with the
fast-path twin run at seal time (`seal-twin-verdicts.tsv`) and with the
reference-checker lane that gates the stored listings (the
`supermartingales` make target). Pooled wall-clock per certificate:
min 14 ms, median 30 ms, $p_{90}$ 37 ms, max 117 ms.

Files:

- `census.tsv` — one row per certificate: family, stem, exit code,
  first verdict line (the checker prints its accepted sequent), wall
  seconds. The chapter's certified-checker table is derived from this
  file by `tables/gen_tables.ts`; nothing in that table is hand-typed.
- `seal-twin-verdicts.tsv` — one row per certificate: stem, the
  fast-path twin's verdict when the input container was sealed, and an
  auxiliary count recorded by the sealing run (not consumed by the
  table generator).
- `provenance.json` — machine, toolchain, artifact hashes, protocol,
  and the corpus-identity chain.

Scope note, stated plainly: these are measured verification results
about committed and pinned artifacts. They do not upgrade the certified
checker's overall standing, which is as the book's front matter states
it. The certified build itself and the container-sealing tool have not
yet migrated into the public trees; until they do, re-derivation from
these receipts covers the table, and the binary replay itself is a
statement of what exists, not yet a link (the reproduction section's
standing rule).
