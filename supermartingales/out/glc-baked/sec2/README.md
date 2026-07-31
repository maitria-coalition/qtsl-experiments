# sec2 — baked-closure certificates, shared-subtree form

One sealed `.glc` container per benchmark. Each container carries the
benchmark's full family of Bernstein Logic proof trees as a **proof
hypernet**: structurally identical subtrees (box complexes shared by a
state's claims, common constants, the prelude) are stored once and
cited by every tree that uses them — element in-degree above one, in
the ordinary tower vocabulary. No sharing-specific vocabulary exists:
the container speaks the same closure declarations as a per-tree
emission, and the geolog checker validates it directly (all claims
discharge at the counts in `MANIFEST.json`).

Tree-shaped consumers use the extraction inverse:

    qtsl2bl --expand <benchmark>.glc --store <baked store> --out-dir D

which duplicates shared subtrees back into one container per proof
tree. The expansion's row multisets are identical to a per-listing
emission modulo local element renumbering (the sealed body's element
numbering follows emission traversal order, which a shared emission
cannot and need not reproduce); the renaming-invariant digest match
and checker-tally equality are recorded in `MANIFEST.json`.
