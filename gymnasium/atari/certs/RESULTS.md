# AtariCircuit certified rows — U1 exact replay

First certified rows on the frontier circuit env: exact bit-level replay of
a fixed open-loop policy over the ALE init set (all 31 no-op offsets),
certified by a checker whose only dynamics authority is the AIG transition
relation — followed by the Montezuma's Revenge rows, which extend the
species to per-offset script families, declared single-offset claims, and
pixel-witnessed replay. Env + scale row: `../README.md`.

## Claim species

For the declared policy script $S$ (open-loop; the ALE init-randomness
model, sticky actions off), init set $K = \{0,\dots,30\}$, horizon $T$:
replaying $k$ no-ops then $S$ through the circuit from the declared initial
state yields return $R_k$ exactly (rewards over the $T$ script frames only,
matching ALE episode semantics), and $\min_k R_k \ge B$. The console is
deterministic, so this is an exact computation, not a bound estimate — the
certificate is the claim tuple itself, and the checker's replay IS the
verification. Two variations appear in the later rows: a `script-family`
claim declares one script per offset (lane $k$ replays $k$ no-ops then its
own script, and the bound is over the family), and a claim may declare a
smaller init set — the declared quantifier is part of the claim and is
displayed with the row.

## The rows

| env | policy | $T$ | init set | certified returns $R_k$ | $\min_k R_k = B$ | check wall-clock | cross-check |
|---|---|---|---|---|---|---|---|
| AtariCircuit-Pong | NOOP script | 1000 | $k \in \{0..30\}$ | $-6$ for every $k$ | $-6$ | 1405 s (31-lane bit-parallel, 1 core) | 31/31 traces bit-exact |
| AtariCircuit-Freeway | all-UP script | 1000 | $k \in \{0..30\}$ | $\in \{2,3,4\}$ by $k$ (per-lane vector in claim) | **2** | 1410 s | 31/31 traces bit-exact |
| AtariCircuit-MontezumasRevenge, pixel-witnessed | key script | 706 | $k = 0$ (declared) | **100** (the key) | **100** | 2974 s (1 lane, 61.5M clocks) | 1/1 traces bit-exact **and 1/1 frame witnesses bit-exact: 706 frames × 160×239 = 26,997,440 pixels** (29,568,000 decode writes) |
| AtariCircuit-MontezumasRevenge, per-offset family | `script-family`: 31 circuit-searched key scripts, one per offset | 828 | $k \in \{0..30\}$, each lane its own script | **100 for every $k$** | **100** | 3943 s (31-lane bit-parallel, 1 core) | 31/31 traces bit-exact (26,133 frames, 3,345,024 RAM bytes + rewards) |
| AtariCircuit-MontezumasRevenge, key+door | circuit-searched key+door script | 1548 | $k = 0$ (declared) | **400** | **400** | 6416 s (1 lane, 128.5M clocks) | 1/1 traces bit-exact (1,548 frames, 198,144 RAM bytes + rewards) |

Comparison points: no certified return claim at any class exists for an
Atari environment to our knowledge (every published Atari number is an
empirical rollout mean); these rows are firsts at their claim class, on
transition relations of 1,491–1,501 flip-flops ($2^{1491}$–$2^{1501}$
states — orders of
magnitude beyond every design in the neural-model-checking Figure 6 suite,
see `../README.md`). The honest scope statement: U1 is the *weakest* rung
of the certified ladder (fixed policy, exact replay, no quantification over
adversaries or learned certificates) — it is the rung that exists today,
and the checker lane it forced into existence is the substrate for the
liveness (A2a) and learned-certificate rungs.

### Finding: the init offset genuinely bites (Freeway)

Under all-UP, the certified per-$k$ returns span $\{2,3,4\}$: offsets 6–7
catch a *fourth* crossing, while nine offsets (14–15, 18–24) shift the
chicken against the traffic phase and only two crossings land inside the
window. A single ALE rollout of this policy reports 2, 3, or 4 depending
on nothing but the init lottery — a $\pm 1$-point spread on a 3-point
score. The min over the init set (the quantity ALE's init randomness
actually threatens) is **2**, strictly below the modal return — a concrete
demonstration that "return of a rollout" and "return guaranteed over the
init set" are different claims, and only the second is what U1 certifies. Pong under NOOP is
init-invariant by contrast ($-6$ uniformly: the input stream is identical,
and no scoring event crosses the window boundary within 30 frames of
shift).

### Montezuma's Revenge: first certified sparse-reward scores

The Montezuma rows certify, to our knowledge, the first nonzero return
ever claimed at this class on the canonical hard-exploration
environment — the key pickup (return exactly 100), and then the key
plus a door of the first room (return exactly 400, over 1,548 frames
and 128.5 million netlist clocks).

The init-offset finding returns here amplified from a spread to a
collapse. The committed instrument row `montezuma-key-T706-k31/`
certifies the full per-offset vector for the fixed $k{=}0$ key script:
**return 100 at $k=0$ and exactly 0 at every $k \in \{1..30\}$** — a
fixed open-loop script survives no shift of the init offset at all.
The per-offset family row answers in the strong direction: 31 scripts,
each searched directly on the circuit for its own offset, certify the
key at every offset. Search on the reference emulator does not
substitute for search on the circuit: an emulator-optimized key+door
script that scores 400 under ale-py replays to exactly 100 on the
circuit — the key lands at the identical frame, then the character
clips a skull on the door leg's final pixel-perfect crossing that the
emulator's slightly different sprite timing lets it thread (a named
divergence class in `../README.md`'s posture-2 ledger). The certified
400 is a circuit-native trajectory, 14 frames shorter than the
emulator-optimized one. In every case the searcher is producer-side
and untrusted; the certificate is the exact replay.

The pixel-witnessed row extends the artefact-pair to the screen: the
claim pins per-frame frame witnesses alongside the RAM traces, and the
checker resolves the AIG's `video`/`xpos`/`ypos` output latches,
applies the declared registered-video decode rule every tick,
assembles the 160×239 frame buffers, and compares all 26,997,440
pixels of the key trajectory bit-exact. This is the checker faculty a
future pixels-to-action policy row needs — the checker seeing the same
pixels the policy would see, validated before any policy consumes it.

## Trust base (what an ACCEPT asks you to believe)

1. **The AIG** `out/{pong,freeway,montezuma}.aig` (SHA-256-pinned inside
   each claim)
   is the env's transition relation — posture 1: this file *defines*
   AtariCircuit; no emulator is in the trust story.
2. **The reward decode** named by the claim (`pong`: $\Delta(\mathrm{ram}[14]-\mathrm{ram}[13]$);
   `freeway`: clamped $\Delta$ of BCD $\mathrm{ram}[103]$; `montezuma`:
   unclamped $\Delta$ of $\mathrm{BCD}(\mathrm{ram}[19])\cdot 10^4 +
   \mathrm{BCD}(\mathrm{ram}[20])\cdot 10^2 + \mathrm{BCD}(\mathrm{ram}[21])$,
   mirroring ale-py 0.12.0's own decode), a few lines in the
   checker, mirroring ALE's own RAM decodes — the montezuma decode
   additionally cross-checked against ALE's reward stream on the key
   trajectory during differential validation (streams identical,
   totals 100/100).
3. **The observation map**: the 1024 latches named `console.ram[B][b]` in
   the AIG's own symbol table are the RIOT RAM; the `vsync` output latch
   delimits frames; the input symbol map (joy/switches/paddles) is faithful.
   For the pixel-witnessed row, additionally: the `video[6:0]`,
   `xpos[7:0]`, `ypos[8:0]` output latches are the TIA's registered pixel
   stream and beam counters, and the checker's declared registered-video
   decode rule (~30 lines) correctly maps the per-tick stream to screen
   state — the same *kind* of trust item as the reward decode,
   cross-witnessed by the independent C++ implementation on the sim side
   of the artefact-pair.
4. **The claim data** (boot protocol, fixed inputs incl. the pot-132 paddle
   calibration, script, init set, zero-init declaration) — part of the
   claim, not trust: they define *which* env variant the row is about.
5. **The checker binary** (`replaycheck`, ~600 lines of Rust, no
   arithmetic beyond u64 bit-ops and i64 sums; proptest battery:
   bit-parallel step vs independent scalar reference on synthetic AIGs).
   It is part of the replay harness that rides the producer migration
   (see `../README.md`) and is not in this export; its verdicts, the
   claim certificates, and every pinned witness it checked are the
   committed record here.

NOT in the trust base: the Verilator build, sim_main.cpp, the python
producer (migrated layer), ale-py, and the pinned upstream Verilog
sources — these live on the producer side of the artefact-pair. So does
`../harness/simreplay`, the committed comparator that re-derives every
pinned witness from a user-supplied ROM through the sim
(`make gymnasium-atari-replay` at the repo root; see `../README.md`) —
it establishes that the committed claims are about the ROM you hold,
not the AIG-replay verdict itself.

## The AIG-vs-Verilator artefact-pair (the named follow-up to D-A1)

The claim bundles carry the Verilator sim's `.atrc` traces as pinned
witnesses; `replaycheck check` replays the AIG *and* compares every frame's
128 RAM bytes + reward bit-exactly against them. Two independent
implementations of "the console" — Verilator's compiled C++ semantics of
the Verilog vs yosys's synthesized AIG under this crate's evaluator —
agreeing byte-for-byte on every sampled frame of every lane:

- Pong row: 31 lanes x (k+1000) frames = 31,465 frames, 4,027,520 RAM bytes
  + the full reward stream — zero divergence (87,303,218 netlist clocks).
- Freeway row: 31,465 frames, 4,027,520 RAM bytes — zero divergence
  (87,386,010 clocks).
- Smoke + mutant-base bundles (T=3/T=60, both ROMs): likewise exact.
- Montezuma rows: per-offset family — 26,133 frames, 3,345,024 RAM bytes
  (73.6M clocks); key+door — 1,548 frames, 198,144 RAM bytes (128.5M
  clocks); instrument row — 22,351 frames, 2,860,928 RAM bytes; the
  pixel-witnessed row adds 26,997,440 pixel comparisons across its 706
  frames — all zero divergence.

The zero-latch-init declaration was validated by exactly this pair: the
AIGER file declares all 1491 latches uninitialized; the checker initializes
them to 0; Verilator ran with `--x-initial fast` (NOT guaranteed zero) plus
the 2000-clk reset hold + 68-frame boot — bit-exactness from frame 0 shows
the boot protocol scrubs all init-sensitive state. A divergence here was
the campaign's declared STOP condition; it did not occur.

## Mutant ceremony (11 mutants + 2 controls; receipts: `mutants/mutants_receipt.json`)

| mutant | tamper | observed refusal + witness |
|---|---|---|
| m01 | trace byte flipped, claim hash stale | `trace-hash` (sha mismatch named) |
| m02 | trace RAM byte flipped, hash recomputed | `trace-ram`: lane 1 frame 30 ram[77] trace=0xff replay=0x00 |
| m03 | trace reward flipped, hash recomputed | `trace-reward`: lane 2 frame 20 trace=1 replay=0 |
| m04 | trace shifted by one frame (length preserved) | `trace-ram`: lane 0 frame 0 ram[8] 0x45 vs 0x44 |
| m05 | trace truncated by one record | `trace-length`: 62 records, claim declares 63 |
| m06 | wrong init: lane $k{=}1$ given $k{=}2$'s trajectory | `trace-ram`: lane 1 frame 1 ram[4] 0x0f vs 0x0e |
| m07 | one claimed return perturbed $+1$ | `return-mismatch`: lane 2 computed 0, claim says 1 |
| m08 | claimed bound perturbed to $\min{+}1$ | `bound`: min 0 < claimed 1 |
| m09 | netlist gate flip (AND 17843, feeds `console.ram[14][0]` next-state; the index is derived at ceremony time from the pinned AIG's own latch table + symbol map, so it follows the netlist across re-synthesis) | `trace-ram`: lane 0 frame 0 ram[7] 0x07 vs 0x02 |
| m10 | AIG file byte flipped | `aig-hash` |
| m11 | reward decode swapped (pong decode on freeway row) | `trace-reward`: lane 0 frame 0 trace=0 replay=1 |
| controls | untampered short bundles (pong + freeway) | ACCEPT + ACCEPT (~167 s each) |

Ceremony finding, recorded: the first m11 direction (freeway decode
asserted over the *pong* short row) ACCEPTED — in frames 0..63 pong's
ram[13], ram[14], and ram[103] are all constant, so both decodes emit the
identical all-zero reward stream and the claim stays true under the swap.
The mutant was redesigned in the discriminating direction (freeway's
ram[14] varies 7..12 inside the window). Lesson for future batteries: a
decode-swap mutant must be placed where the two decodes *disagree on the
data*, and the ceremony's control-plus-mutant structure is what caught the
undiscriminating locus.

## Receipts index

- `pong-noop-T1000/` — claim.cbor, producer + checker receipts. The 31 trace
  witnesses (.atrc.gz) are sha256-pinned inside the claim but not committed —
  like the netlists, they embed ROM-derived contents (see the ROM note in the
  top-level README); `make gymnasium-atari-replay` re-derives and compares
  them from your own ROM (`../harness/simreplay`; see `../README.md`)
- `freeway-up-T1000/` — same shape
- `montezuma-key-family-T828/`, `montezuma-door-T1548/`,
  `montezuma-key-T706-pixels/`, `montezuma-key-T706-k31/` — the
  Montezuma rows, same shape (trace and frame witnesses sha256-pinned
  inside each claim, not committed, for the same ROM-contents reason;
  the k31 instrument bundle carries claim + checker receipt)
- `mutants/` — base bundles (T=60, k 0..3), ceremony receipt; the
  tampered variants are deterministic re-derivations by the mutant
  tooling (replay harness, migrating layer — never committed). m10's
  tampered AIG is likewise sha256-pinned in its receipt, not committed:
  it is the game netlist with one byte flipped (`aig[len/2] ^= 0x01`),
  and a netlist embeds ROM contents whether or not one bit is bent
- Lane: the `replaycheck` checker (`check`/`info`/`bench` verbs) and the
  python claim producer — both in the replay harness noted under the
  trust base above
- Wall-clocks: single core, one GB10 (aarch64), release build, nice'd,
  with 2–10 sibling checker processes sharing the box; replay throughput
  0.062 Mclk/s (~16 us/netlist-clock at 20821 ANDs, all 31 lanes riding
  one u64 word — i.e. ~0.52 us per lane-clock)

## Named next steps

1. **A2a (TIA-counter GF liveness row)**: netlist-level readiness was
   established by D-A1 (`out/console.aig`/`.btor2`); waits on the D-B1
   ranking-condition miter pipeline per the scoping sequencing.
2. **Reactive policy lane**: `policy.kind = "lookup"` (action = table over
   declared RAM bytes) — the checker evaluates the policy itself, keeping
   the trust base closed; enables win-shaped Pong rows once the
   paddle-motion transduction work (D-A1 successor) lands.
3. **Checker throughput**: levelize + SVE2/NEON gather, or per-level dense
   transposition; the serial gate-dependency chain is the current limit.
4. **More ROMs** ride the existing lane unchanged; the E0 bank-switch
   mapper behind the Montezuma rows was the first mapper beyond the 4K
   address window (Breakout is 2K — no mapper needed; other banked
   carts want their own mappers).
