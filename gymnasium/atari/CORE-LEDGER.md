# AtariCircuit core ledger

## Candidates, live-resolved 2026-07-20 (GitHub API; supersedes the campaign's earlier scoping table)

| core | language | license | activity | verdict |
|---|---|---|---|---|
| **rejunity/tiny-atari-2600** | Verilog | **Apache-2.0** | 54 stars, pushed 2025-12 | **ADOPTED** — see rationale |
| MiSTer-devel/Atari2600_MiSTer | VHDL | GPL-3.0 | 32 stars, pushed 2022-01 | runner-up: most wear-tested, but VHDL (ghdl-yosys-plugin friction, no Verilator path) + MiSTer framework extraction cost |
| MiSTle-Dev/A2600Nano | VHDL | GPL-3.0 | 40 stars, pushed 2026-05 | same VHDL/framework costs |
| skibo/Atari2600_Basys3 | Verilog | none stated | 0 stars, pushed 2026-03 | unlicensed — weak foundation even for private use |
| lawrie/ulx3s_atari_2600 | Verilog | none stated | 3 stars, pushed 2022 | the common Verilog ancestor; adopted transitively via rejunity |
| rejunity/Atari-2600-FPGA | Verilog | none | 0 stars, 2021 | superseded by tiny-atari-2600 |

## Adoption rationale

rejunity/tiny-atari-2600 (pinned at commit `4682d2f6`, 2024-11-19;
fetched from upstream by `make fetch-pins`, never copied into this
repository):

1. **Single clock domain by design** — built for ASIC tapeout (Tiny Tapeout 09,
   SkyWater 130nm), so the whole console (6502 + TIA + PIA + RAM) ticks one
   clock with enable strobes. This is what makes yosys `write_aiger` /
   `write_btor` a clean one-pass flow and matches the 2201.10456 frame
   (sequential circuit, all feedback delay-guarded).
2. **FOSS-flow proven** — its CI runs yosys/OpenLane; synthesizability is not
   a hope, it is the project's own acceptance test.
3. **Cartridge is external** — the console top exposes
   `rom_address/rom_data` ports; our wrappers supply a plain ROM array (no
   QSPI machinery in the env).
4. **Runs real games** — River Raid, Pitfall, Star Wars screenshots in-repo;
   built on Arlet Ottens' 6502 + Lawrie Griffiths' TIA/PIA lineage with
   rejunity's TIA/PIA fixes.
5. **Apache-2.0** — friendliest license among all candidates.

## Component provenance inside the pinned tree

| file | lineage | license note |
|---|---|---|
| `src/atari2600.v`, `src/tt_um_*.v`, `src/qspi_flash.v` | rejunity original | Apache-2.0 SPDX headers |
| `src/6502.v`, `src/ALU.v` | Arlet Ottens verilog-6502, rejunity-modified | Arlet's permissive notice ("feel free to use... keep this message") retained in-file |
| `src/TIA.v`, `src/PIA.v` | lawrie/ulx3s_atari_2600, rejunity-fixed | lawrie's repo states NO license; rejunity redistributes under Apache-2.0. this repository never redistributes them — the tree is fetched from upstream at a pinned commit rather than copied here; an upstream license clarification would be a prerequisite for anyone redistributing copies of these two files. |
| `roms/pong.bin`, `roms/freeway.bin` (in `../roms/`) | ale-py 0.12.0 bundled ROMs (md5 60e0ea3c... = Video Olympics 1978; 8e0ab801... = Freeway) | redistributed by ale-py; ROM images are never committed to this repository (see the ROM note in the top-level README) |

## House patches (all marked in-file with "HOUSE PATCH")

| id | file | what | why |
|---|---|---|---|
| P0 | `atari2600.v` | `ifdef ATARI_SIM_DEBUG`: 1024-bit flattened RAM peek + RAM init write port | harness taps; synthesis (no define) unaffected |
| P1 | `PIA.v`, `atari2600.v` | two-player SWCHA: low nibble now reads `input_joystick_1` (upstream duplicated joystick 0 into both nibbles) | real-hardware fidelity; without it player B receives player A's inputs |
| P2 | `TIA.v`, `atari2600.v` | INPT1-3 get the same scanline-threshold paddle transduction as INPT0 (upstream hardwired them to never-charged) | ALE parity for paddle games; enables the pot calibration below |
| P3 | `test/conftest.py` (new file) | guard upstream cocotb HDL testbenches from pytest collection when cocotb is absent | the upstream project's own hardware tests need a cocotb+simulator stack this project doesn't run; without the guard, repo-wide pytest gates fail on ImportError |

## Paddle transduction calibration (measured 2026-07-20)

The core's model: `INPTn reads 0x80 once scanline ypos > pot_n`, which the
game's read loop turns into a paddle position of roughly `pot - 23`, clamped
to `[0x25, 0xdc]`. Measured against ALE (Stella charge model, default paddle
state): **pot = 132 reproduces ALE's default-paddle position bytes exactly**
(`ram[59] = 0x6d` in Video Olympics). ALE initializes only the left-port
paddle pair; right-port paddles idle at minimum. Env preset therefore:
`paddle0 = paddle1 = 132, paddle2 = paddle3 = 0`
(the gym-API env adapter's `ALE_DEFAULT_POTS` preset — part of the
producer layer migrating to TypeScript, see `README.md`). Dynamic paddle
*motion* mapping (ALE paddle-delta to pot trajectory) is named successor
work, not covered by this calibration.
