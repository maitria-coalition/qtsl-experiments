# The route's first measurement (predecessor format, July 2026)

The chapter section beside this directory measures the *reflection*
route on the current sealed-container format. The same question — what
does it cost to push machine-scale certificates through Lean's kernel?
— was first measured in July 2026 on the toolkit's predecessor binary
certificate format (since retired), by the other road into the kernel:
**elaboration**. An emitter translated binary certificate bytes into
generated Lean source, and `lake build` elaborated the result into
kernel-checked `.olean` objects, each module closing with a validity
theorem at the standard axiom closure. This file records that
measurement so the chapter can cite it honestly. Because the format it
measured is retired, nothing here is mechanically re-derivable from
this repository: these are development-record numbers, quoted as the
route's history, with their original measured/projected distinctions
preserved. No current number in the chapter rests on them.

## Measured rungs (July 17, 2026 — full kernel verdicts)

Three production certificates were taken end-to-end to kernel-checked
validity theorems. All three emitted-source/binary pairs were
byte-verified (the emitted header's SHA-256 matches the binary fixture).

| certificate content | proof nodes | binary bytes | generated Lean bytes | `.olean` bytes | elaboration wall |
|---|---:|---:|---:|---:|---|
| cart-pole core decrease | 197 | 19,103 | 153,175 ($\times 8.0$) | 3,871,664 | 3 m 22 s [a] |
| composite syzygy | 377 | 39,743 | 312,435 ($\times 7.9$) | 7,765,816 | 9.0 m [a] / 10.5–12.7 m [b] |
| composite small-gain | 1,081 | 109,525 | 874,249 ($\times 8.0$) | 20,918,344 | 28.0–28.5 m [b] |

[a] build-record timing, machine unrecorded. [b] sampled live on a
128-core x86_64 box (same box for both [b] rows, so [b]-to-[b] is the
controlled comparison). Peak resident memory, sampled at 45 s
granularity (lower bounds): $\geq 7.5$ GiB at 377 nodes, $\geq 13.7$
GiB at 1,081 nodes. Two structural facts made the projection below
semi-analytic rather than statistical: the emitter's per-node-kind
template costs were byte-identical across all three modules, and the
emitted module's definition counts equaled the binary structure pool's
record counts exactly (the two formats' sharing structures coincide).

## The comparator on identical content (July 21, 2026)

The certified checker build contemporaneous with the predecessor format
— machine code emitted through a verified compiler, carrying a
machine-code soundness theorem — verified the same three certificates'
bytes, serially, medians of three runs:

| certificate content | one Arm core (20-core Arm host, 121 GB) | one x86 core (rented x86_64 box) | peak memory |
|---|---:|---:|---:|
| cart-pole core decrease | 6.23 s | 7.06 s | 411 MiB |
| composite syzygy | 2.86 s | 3.00 s | 527 MiB |
| composite small-gain | 9.93 s | 10.41 s | 1.41 GiB |

Comparing like hardware with like ([b] elaboration rows against the x86
certified column): elaboration ran $\times 30$ (cart-pole, [a] row,
machine caveat as stated) to $\times 250$ (syzygy) slower than the
certified checker on identical certificate content at kilobyte scale,
with the factor growing in certificate size. The cross-machine spread
on the certified checker itself was under $\times 1.15$.

## The machine-scale projection

The same measurement campaign took a streaming census of a real
machine-scale certificate — a closed-loop neural-network advisory-region
property for an aircraft collision-avoidance network: **43,285,639
proof nodes, 1.24 GB in binary form**, semantically re-checked by a
reference checker in 2.4 hours (197 µs/node, exact rational
arithmetic, in an interpreter). Applying the byte-stable per-kind
templates to the census (projection, not measurement — the stated
assumption is per-node elaboration cost staying in its measured band
another four orders of magnitude out):

- $\approx 7.15$ GB of generated Lean source;
- $\approx 66$ million declarations in one namespace — roughly one
  hundred times the declaration count of all of mathlib;
- $\approx 176$ GB of `.olean` (honest band 150–250 GB: the measured
  $\times 24$–$25$ object-code ratio applied outside the mix it was
  measured on);
- **1.4–2.7 core-years of serial elaboration** at the measured
  1.0–2.0 s/node band; and memory as the first wall — 66 million
  declarations at even 1 KB of environment apiece is $\geq 66$ GB
  before the elaborator's working set.

## Why the present route looks different

The reflection route this directory measures abandons source
generation altogether: a fixed, format-generic decoder and checker are
defined once in Lean, and each certificate is ordinary *input bytes* —
consumed either by the Lean-compiled native executable or by the
kernel evaluator directly. That moves the measured question from
"what does gigabyte-scale generated source cost?" (answered above:
core-years) to "how does a kernel-evaluable reader scale in container
bytes?" — answered by the chapter's ladder, scaling law, and admission
gate. The two generations agree on the conclusion stated there: the
kernel is the wrong container ship for forty-million-node cargo, and
measuring exactly where the boundary sits is what keeps the claim
honest.
