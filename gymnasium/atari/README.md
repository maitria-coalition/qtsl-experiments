# AtariCircuit — the Atari 2600 env as a sequential digital circuit

The Gymnasium-catalogue frontier rung:
"Atari, with the env *as a digital circuit* (2201.10456)". Ghica-Kaye-Sprunger
(LMCS 22(2:34), 2026) is the semantic contract — a synchronous netlist and its
cycle-by-cycle stream function are the same mathematical object — and this
directory instantiates it: the env's step function is one frame of clock
advances of a synthesized Atari 2600 netlist.

## The two postures (never blurred)

- **Posture 1 (primary): the circuit IS the env.** `AtariCircuit-<game>` is
  defined by the netlist artefacts in `out/` (AIG / BTOR2). Certified claims
  (the U1 exact-replay rung and successors) are about *this* object,
  self-contained; no emulator appears in the trust story.
- **Posture 2 (the bridge): measured model gap vs ALE.** The differential
  harness quantifies where this independent hardware implementation agrees
  with ale-py/Stella. Its receipts are a ledger row, not a proof.

## The scale row (why this extends the Figure-6 frontier)

| circuit | inputs | flip-flops | AND gates | state space |
|---|---|---|---|---|
| console (ROM as free inputs) | 61 | **1588** | 17952 | $2^{1588}$ |
| AtariCircuit-Pong (closed, ROM baked) | 52 | **1491** | 20821 | $2^{1491}$ |
| AtariCircuit-Freeway (closed) | 52 | **1491** | 20798 | $2^{1491}$ |
| AtariCircuit-MontezumasRevenge (closed, E0-banked 8K) | 52 | **1501** | 36698 | $2^{1501}$ |

(yosys 0.67 `write_aiger` after `async2sync` + `memory_map` + `aigmap`;
summary in `receipts-summary.txt`. The closed-game netlists carry the
video output pipeline — 37 of the 1491 flip-flops are pixel-path state;
an earlier emission without it closed at 1454, and the certified rows
have been checked bit-exact against both generations.
The 1491 includes the 1024-bit RIOT RAM. Montezuma's 8 KB cartridge is
bank-switched (Parker Brothers E0), so its mapper hardware is compiled
into the netlist alongside the console — the added state is dominated
by the mapper's bank-slice registers. Every design in neural model
checking's Figure 6 benchmark suite is a parameterised peripheral orders of
magnitude below this state width.)

## Differential receipts (2026-07-20; ale-py 0.12.0, sticky actions off, seed 42)

Protocol: identical boot (hard reset, 60 NOOP frames, two 4-frame
console-RESET presses — mirroring ALE's `StellaEnvironment::reset()`),
identical per-frame action scripts, RAM sampled at the TIA VSYNC rising edge
(a sample-delay sweep confirmed this phase is the agreement optimum).
Circuit RAM zero-initialized (ALE's RAM init measured seed-invariant; both
games' boot code clears what they use).

| run | return (ALE / circuit) | reward events | byte-level |
|---|---|---|---|
| Pong, 1000 frames NOOP | **-6 / -6** | 6 events each, rally period exactly 140 frames on both sides; every event at +2 frames in the circuit (255 vs 257, ..., 955 vs 957) | 5 persistent divergent addrs {8, 21, 49, 50, 54}; ~15 flickering (ball-flight bytes under the 2-frame phase skew) |
| Freeway, 1000 frames all-UP | **+3 / +3** | first crossing at the SAME frame (171); later crossings +1 frame (437 vs 438, 637 vs 638) | 7 persistent {1, 2, 14, 18, 19, 121, 122}; car-lane bytes flicker |
| Freeway, 600-frame pseudorandom walk | 0 / 0 (Freeway scores nothing under weak play — expected) | — | same family |

Full tables + JSON ride the certified rows' receipts (`certs/`).

### Named divergence classes (posture-2 ledger)

1. **Paddle transduction (SOLVED for statics).** Upstream hardwired INPT1-3
   dead; house patch P2 + measured calibration (pot=132 = ALE default; see
   CORE-LEDGER.md) collapsed the entire paddle-position family — ten
   always-divergent addresses became zero. The operative protocol record is
   the full four-pot tuple in each claim's `inputs`: `paddle = [132, 132,
   0, 0]` (the sim's own default is 128; any other tuple resurrects the
   paddle divergence family — re-derived from scratch 2026-07-22). Dynamic
   paddle-motion mapping = named successor work.
2. **Constant 1-2 frame event-phase skew** (Pong +2, Freeway +1 late): a
   boot-time capture difference, suspected timer/CPU-cycle-timing class —
   Arlet's 6502 is not cycle-exact against the NMOS 6502 (WSYNC/instruction
   timing), so timer-read-derived values fork at boot and stay constant.
   Game *dynamics* (rally period 140, crossing times) match exactly modulo
   this phase.
3. **Boot-captured constants**: Pong ram[8] (0x48 vs 0x44), Freeway
   ram[1, 2, 18, 19] — constant from frame 0, seed-invariant on the ALE side,
   same suspected class as (2). Not paddle-tracked (verified by sweep).
4. **Undocumented-opcode / TIA-edge risks**: not yet exercised by these two
   ROMs; will surface per-ROM as the catalogue grows.

## Layout

- `vendor/tiny-atari-2600/` — the adopted core (Apache-2.0), fetched
  from its upstream at a pinned commit by `make fetch-pins`, never
  copied into this repository; house patches applied as the visible
  patch file `vendor/house-patches/house.patch` (P0-P3); provenance +
  license ledger: `CORE-LEDGER.md`
- `sim/` — Verilator wrapper: `sim_top.v` (console + ROM array),
  `sim_main.cpp` (batch traces, the registered-video pixel tap
  `--framedump`, and an interactive gym protocol), `mapper_e0.v` (the
  Parker Brothers E0 bank-switch mapper behind the Montezuma rows —
  wrapper-side cartridge logic; the vendored console is untouched),
  `Makefile` (two builds: plain 4K cart, and `-DMAPPER_E0` for E0
  banked 8K carts)
- `harness/atrcdiff/` — Rust trace comparator (posture 2's instrument)
- `harness/simreplay/` — Rust comparator that re-derives a claim's
  pinned witnesses from *your* ROM through the sim and compares
  returns, bounds, and witness hashes (see below); untrusted producer
  side of the artefact-pair, not the AIG replay checker
- `certs/` — certified U1 rows (claim.cbor + pinned trace witnesses +
  receipts); results + trust-base statement: `certs/RESULTS.md`
- `synth/` — yosys flow to AIG/BTOR2 (`run-synth.sh`, `synth_top.v`)

The trace-production layer around these (ALE ground-truth tracing,
deterministic action scripts, the claim-bundle and mutant producers, and
the gym-API env adapter) was python and is migrating to TypeScript on
the modelling toolkit's API along with every other producer in this
repository; it returns with that migration.
Netlists, RAM traces, and ROM hex are never committed (they embed ROM
contents — see the ROM note in the top-level README).

## Build & run

Requires a FOSS silicon toolchain (yosys + Verilator; e.g. an
oss-cad-suite release) and a legally obtained ROM image.

```
make fetch-pins                     # pinned core + house patches (repo root)
cd gymnasium/atari/sim && make      # verilator builds, plain + E0 (~8 s)
bash ../synth/run-synth.sh ..       # netlists + stats, from your ROM hex
```

`run-synth.sh` synthesizes whichever games' hex images are staged in
`roms/` and skips the rest (`simreplay --rom2hex game.bin roms/game.hex`
converts a dump; 2K images are doubled so the 4K mirror is baked in).

Sim throughput: ~43 emulated frames/s single-thread (~2.9M netlist-clock
evals/s Verilated). A frame is ~79.6k clocks.

## Replay re-verification from your own ROM

Every committed claim pins sha256 hashes of its trace (and, for the
pixel-witnessed row, frame) witnesses; the witness bytes themselves are
never committed because they embed ROM-derived contents. With your own
ROM image, one make target re-derives all of them:

```
make gymnasium-atari-replay GAME=montezuma ROM=/path/to/rom.bin   # repo root
```

This builds the sim from the pinned core, replays every lane of every
committed claim bundle for the game (`harness/simreplay`), independently
recomputes the reward stream from the RAM trace (a third implementation
of the decode, after the sim's C++ and the producer's python), and
compares per-lane returns, the min-over-lanes bound, and the sha256 of
every re-derived raw witness against the claim bytes. ACCEPT means your
ROM re-derives the certified rows bit-identically; a wrong or
differently-dumped ROM refuses at the first compare. The dumps these
rows were produced from (md5, for identification): pong
`60e0ea3cbe0913d39803477945e9e5ec`, freeway
`8e0ab801b1705a740b476b7f588c6d16`, montezuma
`3347a6dd59049b15a38394aa2dafa585`.

Toolchain sensitivity, measured: the trace/frame witnesses reproduce
bit-identically across Verilator builds and hosts (oss-cad-suite on the
production machine vs distro Verilator 5.020 elsewhere — zero
divergence). The netlist does not: `out/<game>.aig` byte-identity (and
so the claim's `aig_sha256` pin) requires the pinned yosys 0.67 —
yosys 0.33 emits the same 1,501 latches for Montezuma but a slightly
different AND count and different bytes. `simreplay --aig` compares the
netlist hash when you have synthesized one; the replay target treats it
as optional for exactly this reason.

## D-A2 readiness

- **A2b (netlist-replay checker lane): LANDED 2026-07-20.** First certified
  rows: AtariCircuit-Pong (NOOP, T=1000, all 31 init offsets, return −6
  exactly) and AtariCircuit-Freeway (all-UP, T=1000, min return 2 — the
  init offset genuinely bites; see the finding). Checker replays the AIG
  only, bit-parallel over the init set; every frame cross-checked bit-exact
  against the Verilator traces (the D-A1-named artefact-pair, zero
  divergence). Rows, trust base, mutant ceremony: `certs/RESULTS.md`.
- **Montezuma's Revenge rows: LANDED 2026-07-21.** The key (return 100)
  certified at every one of the 31 init offsets via one circuit-searched
  script per offset (`certs/montezuma-key-family-T828/`); key + door
  certified at return exactly 400 over 1,548 frames from the declared
  offset (`certs/montezuma-door-T1548/`); the honest-quantifier instrument
  row (the fixed k=0 key script scores 0 at every other offset:
  `certs/montezuma-key-T706-k31/`); and the pixel-witnessed key row
  (26,997,440 pixels compared bit-exact:
  `certs/montezuma-key-T706-pixels/`). Details: `certs/RESULTS.md`.
- **A2a (TIA-counter liveness row): READY at the netlist level** —
  `out/console.aig`/`console.btor2` contain the horizontal counter; the row
  waits on D-B1's ranking-condition miter pipeline (its stated dependency).
- Named successor work: paddle-motion transduction map; cycle-exact 6502
  core swap (would kill divergence classes 2-3) — candidate cores to be
  re-surveyed live when that work starts; more ROMs (Breakout is 2K;
  the E0 bank-switch mapper behind the Montezuma rows was the first
  mapper beyond the 4K address window — other banked carts want their
  own).
