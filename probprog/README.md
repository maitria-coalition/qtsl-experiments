# probprog — guaranteed posterior bounds for probabilistic programs

Certified brackets on posterior distributions of discrete probabilistic
programs, compared like-for-like against the two published
guaranteed-bounds tools:

- **GuBPI** (Beutner–Ong–Zaiser, PLDI 2022; MIT) — universal PPL,
  interval-trace splitting, validated-float intervals.
- **Diabolo** (Zaiser–Murawski–Ong, POPL 2025; MIT) — discrete
  programs, residual-mass and geometric-bound semantics, exact
  rationals inside the tool.

Both tools compute sound bounds; neither emits a replayable witness or
has an independent checker. This family runs their own shared
comparison programs (the geometric counter, the biased random walk,
Mossel's die paradox — the three-case table the Diabolo paper uses to
compare against GuBPI) through an untrusted producer into certificates
whose every number is re-derived by an independent checker from the
stored chain rows.

## Architecture

- `producer/` — the untrusted producer (`probprog-produce`): encodes
  each source program as a finite-truncation layered chain, computes
  the exact T-step distribution by integer-scaled dynamic programming,
  and emits per-class posterior brackets in the correlated-ratio form.
  Truncation is sound by construction: states without stored rows are
  treated as absorbing by the evaluation semantics, so an incomplete
  row set can only widen brackets, never falsify them.
- The certificate species (`DtmcAbsorb`: finite-horizon absorption
  brackets for stored-row chains) is checked by the geolog fast-path
  checker (`glc-check`, built from the pinned qtsl checkout), which
  recomputes every mass from the stored rows — derived quantities are
  never stored, so forging an intermediate is unrepresentable.
  **[argued]** The species is staged for the next theory revision
  event; until that revision is sealed, container verdicts in this
  family are reported against the staged closure and labeled as such.
- Translation validity: each source program is vendored verbatim
  (MIT), and its published semantics is simulated directly against the
  chain encoding's exact masses (10^6 paths, 4-sigma binomial bands),
  plus exact closed-form containment where ground truth exists (all
  three programs have closed forms).

## Claims

**[argued]** Per case, the bracket width is at most each baseline's at
matched query and unrolling, and the end-to-end house wall (producer +
certificate check, same box) is competitive with Diabolo's
residual-mass semantics and orders of magnitude under GuBPI's — with
the third axis (replayable certificate + independent checker + mutant
battery) added rather than traded. Numbers, hardware, and the
measurement protocol land in this directory as receipts; every claim
above flips to **[shipped]** only with those receipts beside it, and
cases where a baseline wins an axis are reported as rows.
