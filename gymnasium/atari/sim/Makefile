# Verilator build for the AtariCircuit sim driver.
# Runs on any host with oss-cad-suite verilator. No SDL, headless.
#
# Two binaries: `atari_circuit_sim` (plain 4K cart: pong, freeway) and
# `atari_circuit_sim_e0` (E0 banked 8K cart: montezuma) — separate builds
# so the plain path stays byte-identical in behavior.
TOP = sim_top
EXEC = atari_circuit_sim
EXEC_E0 = atari_circuit_sim_e0
SRC_DIR = ../vendor/tiny-atari-2600/src

VFLAGS = -O3 --x-assign fast --x-initial fast --noassert -DATARI_SIM_DEBUG
WFLAGS = -Wno-CASEINCOMPLETE -Wno-CASEX -Wno-CASEOVERLAP -Wno-WIDTHTRUNC -Wno-WIDTHEXPAND -Wno-PINMISSING

SOURCES = $(TOP).v $(SRC_DIR)/atari2600.v $(SRC_DIR)/6502.v $(SRC_DIR)/ALU.v \
          $(SRC_DIR)/TIA.v $(SRC_DIR)/PIA.v

all: $(EXEC) $(EXEC_E0)

$(EXEC): sim_main.cpp $(SOURCES)
	verilator $(VFLAGS) $(WFLAGS) -I$(SRC_DIR) --top-module $(TOP) -cc $(SOURCES) \
	  --exe sim_main.cpp -o $(EXEC) --build -j 8
	cp obj_dir/$(EXEC) .

$(EXEC_E0): sim_main.cpp $(SOURCES) mapper_e0.v
	verilator $(VFLAGS) $(WFLAGS) -DMAPPER_E0 -I$(SRC_DIR) --top-module $(TOP) \
	  -Mdir obj_dir_e0 -cc $(SOURCES) mapper_e0.v \
	  --exe sim_main.cpp -o $(EXEC_E0) --build -j 8
	cp obj_dir_e0/$(EXEC_E0) .

clean:
	rm -rf obj_dir obj_dir_e0 $(EXEC) $(EXEC_E0)

.PHONY: all clean
