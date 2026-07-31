#!/bin/bash
# netlist emission on a remote build host (oss-cad-suite yosys).
# Usage: run-synth.sh <workdir>   (expects vendor sources + synth_top.v +
# roms/{pong,freeway,montezuma}.hex staged beside it; writes receipts/ +
# out/. Games whose .hex is not staged are skipped — synthesize just the
# ROMs you have. ROM images are never committed; convert your own dump
# with one hex byte per line, e.g. `xxd -p -c1 game.bin > roms/game.hex`
# (2K ROMs: double the image first — the 4K mirror is baked into the hex).)
set -euo pipefail
WORK="${1:-.}"
cd "$WORK"
SRC=vendor/tiny-atari-2600/src
mkdir -p out receipts

# ---- 1. console-only (ROM-invariant "console silicon"): AIG + BTOR2 + stat ----
yosys -p "
read_verilog $SRC/atari2600.v $SRC/6502.v $SRC/ALU.v $SRC/TIA.v $SRC/PIA.v
hierarchy -top atari2600
proc; async2sync; flatten
memory_collect; memory_map
opt -full
stat
techmap; opt -full
dfflegalize -cell \$_DFF_P_ 01
stat
aigmap
opt_clean
stat
write_aiger -symbols out/console.aig
" > receipts/synth-console.log 2>&1
echo "console AIG: $(head -1 out/console.aig)"

yosys -p "
read_verilog $SRC/atari2600.v $SRC/6502.v $SRC/ALU.v $SRC/TIA.v $SRC/PIA.v
hierarchy -top atari2600
proc; async2sync; flatten
memory_collect
opt
dffunmap
write_btor out/console.btor2
" > receipts/synth-console-btor.log 2>&1
echo "console BTOR2: $(wc -l < out/console.btor2) lines"

# ---- 2. per-game closed circuits ----
# pong/freeway: -DVIDEO_OUT ON since 2026-07-20 —
# all three carts' AIGs expose the registered video[6:0] pixel stream
# (one-clk lag behind xpos/ypos; see synth_top.v). The certs/ rows were
# re-certified against the VIDEO_OUT pins in that same emission; re-emission
# must stay bit-identical (two-run byte-stability discipline).
for game in pong freeway; do
  if [ ! -f "roms/$game.hex" ]; then
    echo "$game: skipped (roms/$game.hex not staged)"
    continue
  fi
  yosys -p "
read_verilog -DVIDEO_OUT -DROM_HEX=\"roms/$game.hex\" synth/synth_top.v $SRC/atari2600.v $SRC/6502.v $SRC/ALU.v $SRC/TIA.v $SRC/PIA.v
hierarchy -top synth_top
proc; async2sync; flatten
memory_collect; memory_map
opt -full
stat
techmap; opt -full
dfflegalize -cell \$_DFF_P_ 01
stat
aigmap
opt_clean
stat
write_aiger -symbols out/$game.aig
" > receipts/synth-$game.log 2>&1
  echo "$game AIG: $(head -1 out/$game.aig)"

  yosys -p "
read_verilog -DVIDEO_OUT -DROM_HEX=\"roms/$game.hex\" synth/synth_top.v $SRC/atari2600.v $SRC/6502.v $SRC/ALU.v $SRC/TIA.v $SRC/PIA.v
hierarchy -top synth_top
proc; async2sync; flatten
memory_collect
opt
dffunmap
write_btor out/$game.btor2
" > receipts/synth-$game-btor.log 2>&1
  echo "$game BTOR2: $(wc -l < out/$game.btor2) lines"
done

# ---- 2b. montezuma: E0 banked cart (sim/mapper_e0.v) + video outputs ----
if [ -f roms/montezuma.hex ]; then
E0_DEFS='-DMAPPER_E0 -DVIDEO_OUT -DROM_HEX="roms/montezuma.hex"'
E0_SRCS="synth/synth_top.v sim/mapper_e0.v $SRC/atari2600.v $SRC/6502.v $SRC/ALU.v $SRC/TIA.v $SRC/PIA.v"
yosys -p "
read_verilog $E0_DEFS $E0_SRCS
hierarchy -top synth_top
proc; async2sync; flatten
memory_collect; memory_map
opt -full
stat
techmap; opt -full
dfflegalize -cell \$_DFF_P_ 01
stat
aigmap
opt_clean
stat
write_aiger -symbols out/montezuma.aig
" > receipts/synth-montezuma.log 2>&1
echo "montezuma AIG: $(head -1 out/montezuma.aig)"

yosys -p "
read_verilog $E0_DEFS $E0_SRCS
hierarchy -top synth_top
proc; async2sync; flatten
memory_collect
opt
dffunmap
write_btor out/montezuma.btor2
" > receipts/synth-montezuma-btor.log 2>&1
echo "montezuma BTOR2: $(wc -l < out/montezuma.btor2) lines"
else
  echo "montezuma: skipped (roms/montezuma.hex not staged)"
fi

# ---- 3. receipts summary ----
{
  echo "== console (ROM as free inputs) =="
  rg -A12 "^=== atari2600 ===" receipts/synth-console.log | rg "Number of cells|_DFF_|_AND_|_NOT_|Number of wires" | head -20
  for game in pong freeway montezuma; do
    [ -f "receipts/synth-$game.log" ] || continue
    echo "== $game (closed) =="
    rg -A12 "^=== synth_top ===" receipts/synth-$game.log | rg "Number of cells|_DFF_|_AND_|_NOT_" | head -20
  done
  echo "== AIG headers (M I L O A) =="
  for f in out/*.aig; do echo "$f: $(head -1 "$f")"; done
  echo "== BTOR2 line counts =="
  for f in out/*.btor2; do echo "$f: $(wc -l < "$f") lines"; done
} > receipts/SUMMARY.txt
cat receipts/SUMMARY.txt
