# qtsl-experiments — full reproduction, from `cargo build --release` to
# `lualatex`. Each top-level target rebuilds one section of the
# Experiments chapter: checkers are built from the qtsl/kernels
# repositories (Cargo git references, pinned by Cargo.lock), the stored
# certificates and their mutant batteries are re-verified through the
# independent checkers, the tables are re-derived from the receipt
# files, and the section is typeset standalone.
#
#   make gymnasium        # the Gymnasium campaign (+ per-family subtargets)
#   make circuits         # hardware model checking (219/219 liveness sweep + mutants)
#   make semicon          # the semiconductor fluid ladder
#   make supermartingales # the published-table re-certification campaign
#   make chapter          # everything, plus the full-chapter PDF
#
# Orchestration is TypeScript (node >= 22, no packages); the checkers
# are Rust binaries plus the pinned qtsl checkout's reference lane.
# Third-party comparison trees are never copied into this repository:
# `make fetch-pins` materializes them from their upstreams at pinned
# commits (tools/fetch-pins.sh — house modifications, if any, are
# visible patch files applied after checkout).
#
# Public clones need no auth; private mirrors ride your own git
# credentials both for Cargo (CARGO_NET_GIT_FETCH_WITH_CLI, exported
# below) and for the pinned qtsl checkout; see README.md ("Cloning and
# building").

QTSL_REPO ?= https://github.com/maitria-coalition/qtsl-dev
# The checker's baked roster carries only the current revision-event
# pair (no legacy generations), so every baked-container corpus
# re-emits at each revision event and this pin advances with them.
# All the baked lanes this pin serves (gymnasium, spde spectrum,
# lean-route, semicon minifab, supermartingales, petri guard) are
# sealed at revision event 5.1 (qc closure 6862edc8…); the former
# separate gymnasium and guard pins each converged here when their
# corpora re-emitted at this event, exactly as their comments
# predicted.
QTSL_REV  ?= 43cd608130c2c805a5fbd5e416303daf025eb7dc

export CARGO_NET_GIT_FETCH_WITH_CLI := true

BUILD    := build
SECDIR   := $(BUILD)/sections
FLCHECK  := $(CURDIR)/target/release/flcheck
FLUIDCHECK := $(CURDIR)/target/release/fluidcheck
QTSL_CO  := $(BUILD)/qtsl
NODE     := node --experimental-strip-types

.PHONY: all chapter checkers tables fetch-pins clean \
        gymnasium gymnasium-text gymnasium-classic gymnasium-box2d gymnasium-glc \
        gymnasium-atari gymnasium-atari-replay gymnasium-mujoco circuits circuits-recheck \
        semicon semicon-glc supermartingales variadic

all: chapter

# ---------------------------------------------------------------- checkers

checkers:
	cargo build --release

# The pinned checkout re-verifies its pin: the stamp is keyed on
# $(QTSL_REV), so bumping the pin (or retargeting $(QTSL_REPO))
# re-fetches and re-checkouts in place. A bare directory-existence
# gate here would keep serving a stale tree after the pin moved.
QTSL_PIN := $(QTSL_CO)/.git/pin-$(QTSL_REV)

$(QTSL_PIN):
	@if [ ! -d $(QTSL_CO)/.git ]; then \
	  git clone --no-checkout $(QTSL_REPO) $(QTSL_CO); \
	fi
	git -C $(QTSL_CO) remote set-url origin $(QTSL_REPO)
	@if ! git -C $(QTSL_CO) cat-file -e $(QTSL_REV)^{commit} 2>/dev/null; then \
	  git -C $(QTSL_CO) fetch origin; \
	fi
	git -C $(QTSL_CO) checkout $(QTSL_REV)
	rm -f $(QTSL_CO)/.git/pin-*
	touch $@

# ------------------------------------------------------------- pinned trees

fetch-pins:
	tools/fetch-pins.sh

# ---------------------------------------------------------------- gymnasium

gymnasium-text: checkers
	FLCHECK=$(FLCHECK) $(NODE) gymnasium/run_checks.ts
	FLCHECK=$(FLCHECK) $(NODE) gymnasium/run_tt_checks.ts

gymnasium-box2d: checkers
	$(FLCHECK) gymnasium/out/ll/ll_u4_min_a100.cbor
	$(FLCHECK) gymnasium/out/ll/ll_u4_fixture_n20k.cbor
	$(FLCHECK) gymnasium/out/llc/llc_u4_min_a100.cbor
	$(FLCHECK) gymnasium/out/llc/llc_u4_fixture_n20k.cbor

gymnasium-atari: $(SECDIR)
	$(MAKE) tables
	$(MAKE) $(SECDIR)/atari.pdf
	@echo "gymnasium-atari: certified exact-replay rows (Pong, Freeway, Montezuma's Revenge)."
	@echo "The committed claim certificates and receipts are the shipped"
	@echo "artifacts (gymnasium/atari/certs/); the pinned trace/frame/netlist"
	@echo "witnesses derive from a game ROM image this repository cannot"
	@echo "redistribute. With your own ROM, re-derive and compare them:"
	@echo "  make gymnasium-atari-replay GAME=montezuma ROM=/path/to/rom.bin"
	@echo "(see gymnasium/atari/README.md for what is compared, and the"
	@echo "synth-from-your-own-ROM path for the netlist itself)."

# Replay re-verification of the committed Atari claim bundles from a
# user-supplied ROM image (never committed): re-derives every pinned
# trace/frame witness through the Verilator sim built from the pinned
# core, independently recomputes the reward stream, and compares returns,
# bounds, and witness hashes against the claim bytes.
#   usage: make gymnasium-atari-replay GAME=<pong|freeway|montezuma> ROM=/path/to/rom.bin
gymnasium-atari-replay: fetch-pins
	@test -n "$(GAME)" && test -n "$(ROM)" || { \
	  echo "usage: make gymnasium-atari-replay GAME=<pong|freeway|montezuma> ROM=/path/to/rom.bin"; exit 2; }
	gymnasium/atari/replay-recheck.sh "$(GAME)" "$(ROM)"

gymnasium-classic:
	@echo "gymnasium-classic: the continuous-state (Pendulum) artifacts are"
	@echo "still graduating from the authoring workshop; this preview carries"
	@echo "the chapter prose but not yet the reproduction spine for them."

gymnasium-mujoco:
	@echo "gymnasium-mujoco: reserved (first row in flight; no shipped rows yet)."

gymnasium: gymnasium-text gymnasium-box2d gymnasium-atari tables $(SECDIR)
	$(MAKE) $(SECDIR)/gymnasium.pdf

# ---------------------------------------------------------------- circuits

circuits: circuits-recheck $(SECDIR)
	$(MAKE) tables
	$(MAKE) $(SECDIR)/circuits.pdf

circuits-recheck: $(QTSL_PIN) fetch-pins
	QTSL_CIRCUITS_CHECKER=$(CURDIR)/$(QTSL_CO)/checkers/circuits \
	  $(NODE) circuits/recheck.ts

# ---------------------------------------------------------------- semicon

semicon: semicon-recheck $(SECDIR)
	$(MAKE) tables
	$(MAKE) $(SECDIR)/semicon.pdf

.PHONY: semicon-recheck
semicon-recheck: checkers
	FLUIDCHECK=$(FLUIDCHECK) $(NODE) semicon/recheck.ts

# The sealed-container lane (semicon/minifab/FLUID-CERT.md, "Sealed
# containers"): the fluid-drift certificates as .glc containers through
# the geolog fast-path checker, built from the pinned qtsl checkout.
# The driver walks every GLC-MANIFEST.json under semicon/out/ and
# dispatches on the manifest's shape: the baked-vocabulary lane
# (out/minifab/glc-baked — the reproduction route; vocabulary index
# derived from the pinned theory-pair chain's own bytes, per-relation
# claim counts via --expect-claims) and the original synthetic-
# vocabulary lane (out/minifab/glc — kept checkable forever;
# --expect-modes). Requires a QTSL_REV that carries the baked-cert
# checker lane and the baked theory-pair pins (theories/pins/
# baked_v1_pins.txt names the closure revision the containers pin); at
# an older pin the built checker refuses the baked containers loudly.
.PHONY: semicon-glc
semicon-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) semicon/glc-recheck.ts

# The sealed-container lane (spde/SPDE-CERT.md): the SPDE moment
# certificates under the family's split-container design — witness
# containers (moment core, synthetic vocabulary, checked with exact
# per-relation claim counts) and per-certificate spectrum containers
# (the noise spectrum re-homed onto the baked qc closure; amplitude
# well-formedness claims, --baked-cert), plus both mutant batteries.
# The manifest records each spectrum container's witness commit
# address (the cross-tie); the spectrum-law binding itself is the
# production gate's today, and the battery's designed-ACCEPT specimen
# documents that boundary honestly (the driver's header states it).
.PHONY: spde-glc
spde-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) spde/glc-recheck.ts

# ------------------------------------------------------- supermartingales

supermartingales: $(QTSL_PIN) $(SECDIR)
	QTSL_CHECKERS=$(CURDIR)/$(QTSL_CO)/checkers/supermartingales \
	  $(NODE) supermartingales/recheck.ts
	$(MAKE) tables
	$(MAKE) $(SECDIR)/supermartingales.pdf

# The sealed-container lane (supermartingales/GLC-CERT.md): the same
# certificates as .glc containers through the geolog fast-path checker,
# built from the pinned qtsl checkout — both committed lanes: the
# synthetic-chain containers (out/glc/, kept checkable forever) and the
# baked-closure re-emission (out/glc-baked/: the full corpus plus the
# committed falsifier listings sealed as containers, per-relation claim
# counts via --expect-claims). Requires a QTSL_REV that carries the BL
# reproduction vocabulary AND the reserved predicate-hole wire
# classification the baked re-emission uses; at an older pin the baked
# lane's cone claims refuse loudly rather than passing vacuously.
# A fully-green run writes its timing receipt out of tree (build/);
# GLC_PUBLISH_TIMINGS=1 refreshes the committed receipt instead
# (supermartingales/out/glc/GLC-TIMINGS.json, read by the tables build).
.PHONY: supermartingales-glc
supermartingales-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) supermartingales/glc-recheck.ts

# The sealed-container lane (lean-route/GLC-CERT.md): the ladder's
# fifteen certificates as .glc containers through the geolog fast-path
# checker — the full APPLICATIVE obligation roster (cone / Farkas /
# syzygy / conclusion-form / enclosure / promise wires) enumerated
# from the sealed bytes, across both committed lanes (staged chain at
# out/glc/, baked closure at out/glc-baked/). Requires a QTSL_REV that
# carries the widened BL reproduction vocabulary and the reserved
# predicate-hole wire classification; the driver's roster gate refuses
# a checker built at an older pin (no vacuous passes on the
# zero-cone-node rungs).
.PHONY: lean-route-glc
lean-route-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) lean-route/glc-recheck.ts

# The sealed-container lane (gymnasium/GLC-CERT.md): the hitting-lane
# certificates as .glc containers through the geolog fast-path
# checker — every deadline-table rule node discharged by the
# exact-rational deadline-table walk from the sealed bytes alone; the
# reward-weighted and episode-process families drop in as sibling
# manifests with zero driver edits (the driver discovers every
# GLC-MANIFEST.json under gymnasium/out/ and dispatches on each
# manifest's mode flag). Requires a checker pin that carries the gym
# container mode (the reprovocab gym slice + glc-check --gym); at an
# older pin the built checker refuses these containers loudly, and
# the pin advances past that checker's landing — the same seam every
# sealed-container lane here rides. The baked hitting lane
# (out/<family>/glc-baked/, mode --baked-cert; gymnasium/GLC-CERT.md
# "the baked lane") additionally requires a pin whose baked roster
# carries the revision event the corpus is sealed at (currently
# event 5.1) — QTSL_REV carries it since the sibling baked lanes'
# re-emissions converged the pins.
.PHONY: gymnasium-glc
gymnasium-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) gymnasium/glc-recheck.ts

# ------------------------------------------------------------------ petri

# Petri P-invariants over MCC 2026 nets, as sealed containers checked
# from the bytes alone. The model corpus (and everything derived from
# its structure, the containers included) is not redistributed in
# this repository pending license clarification with the contest
# organizers — petri/GLC-MANIFEST.expected.json pins sha256s and
# exact obligation counts, and the driver verifies a MATERIALIZED
# copy (PETRI_GLC_DIR, default petri/out/glc) bit-for-bit before
# checking; absent a copy it says so and skips honestly. The
# re-derivation path: petri/README.md.
.PHONY: petri-glc
petri-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) petri/glc-recheck.ts

# Guarded specifications over the same MCC corpus: one sealed
# container per net certifying S ⊨ Guard(R, φ) — specification, guard
# region, region-satisfaction warrant, and route-discipline claim
# rows, discharged in exact rational arithmetic. Same disposition as
# petri-glc (MCC-derived containers not redistributed; the manifest
# pins sha256s + exact per-relation claim counts; the driver verifies
# a MATERIALIZED copy — GUARD_GLC_DIR, default petri/guard/out — and
# skips honestly without one). Builds its checker from the shared
# pin, QTSL_REV: the corpus is sealed at revision event 5.1 like its
# sibling baked lanes (the lane's original event-4 pin converged here
# at the re-emission, retiring the second checkout). Re-derivation
# path: petri/guard/README.md.
.PHONY: petri-guard-glc
petri-guard-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) petri/guard/glc-recheck.ts

# --------------------------------------------------------------- lean-route

# The Lean-route ladder: the pinned reference lane re-verifies every
# ladder certificate (the certificates stand on their own; the Lean
# timings are committed receipts from the named dedicated machine),
# then the section's table, fit, figure data, and admission gate are
# re-derived from those receipts.  Re-MEASURING (as opposed to
# re-deriving) is a dedicated-box procedure: lean-route/README.md.
# The certified-checker mints (lean-route/out/qtslab/) are a pure
# function of the committed fixtures; this verifies the committed
# containers + manifest re-derive byte-for-byte (no drift, no
# hand-edits).  The timing receipts beside them are measurements and
# are replaced by tools/time-routes.py, never re-derived.
.PHONY: lean-route-mints
lean-route-mints:
	python3 lean-route/tools/mint-qtslab-vintage.py --verify \
	  lean-route/fixtures lean-route/out/qtslab

.PHONY: lean-route lean-route-recheck
lean-route-recheck: $(QTSL_PIN)
	QC=$(CURDIR)/$(QTSL_CO)/checkers/supermartingales; \
	$(NODE) $$QC/check_rows.ts --stack $$QC/theories/BLstack.rows.txt \
	  --cert $$QC/theories/BLCert.rows.txt lean-route/fixtures/*.rows.txt

lean-route: lean-route-recheck lean-route-mints $(SECDIR)
	$(MAKE) $(SECDIR)/lean-route.pdf

# ---------------------------------------------------------------- chapter

tables: fetch-pins
	$(NODE) tables/gen_tables.ts
	$(NODE) tables/gen_lean_route.ts

$(SECDIR):
	mkdir -p $(SECDIR)

# A section PDF: the standalone wrapper typesets one section fragment
# (twice, for stable references), then the glyph gate refuses silently
# dropped characters.
$(SECDIR)/%.pdf: chapter/sections/%.tex chapter/standalone.tex tables | $(SECDIR)
	cd chapter && for pass in 1 2; do \
	  lualatex -interaction=nonstopmode -halt-on-error \
	    -output-directory=../$(SECDIR) -jobname=$* \
	    "\def\SectionInput{sections/$*}\input{standalone.tex}" >/dev/null; \
	  done
	@if grep -q 'Missing character' $(SECDIR)/$*.log; then \
	  echo "glyph gate: dropped characters in $(SECDIR)/$*.log" >&2; exit 1; fi
	@echo "built: $(SECDIR)/$*.pdf"

chapter: gymnasium-text gymnasium-box2d circuits-recheck semicon-recheck \
         supermartingales-recheck lean-route-recheck variadic tables $(SECDIR)
	$(MAKE) $(SECDIR)/gymnasium.pdf $(SECDIR)/atari.pdf $(SECDIR)/circuits.pdf \
	        $(SECDIR)/semicon.pdf $(SECDIR)/supermartingales.pdf \
	        $(SECDIR)/lean-route.pdf
	cd chapter && for pass in 1 2; do \
	  lualatex -interaction=nonstopmode -halt-on-error \
	    -output-directory=../$(BUILD) -jobname=experiments \
	    "\input{chapter-standalone.tex}" >/dev/null; \
	  done
	@if grep -q 'Missing character' $(BUILD)/experiments.log; then \
	  echo "glyph gate: dropped characters in $(BUILD)/experiments.log" >&2; exit 1; fi
	@echo "built: $(BUILD)/experiments.pdf (the full chapter)"

# recheck-only twin (no PDF), for the chapter target's dependency spine
.PHONY: supermartingales-recheck
supermartingales-recheck: $(QTSL_PIN)
	QTSL_CHECKERS=$(CURDIR)/$(QTSL_CO)/checkers/supermartingales \
	  $(NODE) supermartingales/recheck.ts

# ---------------------------------------------------------------- variadic

# Transfer-theorem countermodel batteries (plated certificates over
# token populations): exact-rational fixtures, no dependencies; each
# script exits nonzero on any failed cell. Kernel-checked twins live
# in the qtsl repository (lean/qtsl/Qtsl/Plate/TTransfer.lean); see
# variadic/README.md for the cell-by-cell map.
variadic:
	python3 variadic/transfer_countermodels.py
	python3 variadic/normalized_head_countermodels.py

clean:
	rm -rf $(BUILD) target
