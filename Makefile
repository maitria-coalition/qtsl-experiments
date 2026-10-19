# qtsl-experiments — public replay, receipt-to-table regeneration, and
# typesetting. Each top-level family target rebuilds one section of the
# Experiments chapter from the artefacts that family ships: checkers are
# built from the qtsl/kernels repositories (Cargo git references, pinned by
# Cargo.lock), stored certificates and mutant batteries are re-verified,
# tables are re-derived from receipts, and the section is typeset standalone.
# Producer re-conduction is family-scoped rather than implied by every target.
#
#   make gymnasium        # the Gymnasium campaign (+ per-family subtargets)
#   make circuits         # hardware model checking (219/219 liveness sweep + mutants)
#   make semicon          # the semiconductor fluid ladder
#   make minifab-dispatch # the Mini-Fab dispatch-synthesis certificate family
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
# The qtsl pin is served from the public release repository. Its
# history is a release lineage (fresh snapshot roots), so QTSL_REV
# advances by release revision, not by upstream development commit;
# see README.md ("Cloning and building").

QTSL_REPO ?= https://github.com/maitria-coalition/qtsl
# The checker's baked roster leads with the current revision-event
# pair (legacy generations ride an append-only roster until their
# corpora re-conduct), so every baked-container corpus re-emits at
# each revision event and this pin advances with them.
# Revision event 8: the dac closure (BernsteinLogic + Qtsl + Dtmc +
# DtmcAbsorb, revision 6f1b2c7e…) and the dcc closure (dac +
# DtmcContraction, revision 70b31e69…) joined the roster — the
# probprog container lane rides them flagless. The event-7 closures
# (qc 9952506d…, dc bc30a5f3…) are unchanged and stay rostered, so
# the older baked lanes (gymnasium, spde spectrum, lean-route,
# semicon minifab, supermartingales, petri guard) check as before.
QTSL_REV  ?= a9bdb62a6fba2beafbdf8022bc581f0cf4f4d433

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
        semicon semicon-glc minifab-dispatch minifab-floor-recheck supermartingales variadic

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

# The Mini-Fab dispatch-synthesis family (semicon/minifab-dispatch/
# README.md): the discrete Mini-Fab net's sealed certificate
# containers — the closed twelve-certificate baseline with its
# worst-case deadline, the open system (one policy hole as data), the
# synthesized supervisor with its certified 626N arrivals bound, and
# the Simplex switching-envelope composite — re-checked from bytes
# alone, integrity mutants refused, then the producer-tier batteries
# (campaign re-derivation + exact-tier/encoder mutant kills) re-run
# from the pinned mtk checkout. One verdict table for the family.
#
# These containers sealed at revision event 6 and once built their
# checker from their own newer pin in a second checkout; the sibling
# corpora's event-7 re-emission carried the shared pin past them, and
# the shared checker's append-only legacy roster keeps the event-6
# closure resolving — so the second checkout retired into $(QTSL_PIN),
# exactly as the former gymnasium and guard pins did. The family's own
# re-emission at the current event (which lets the checker's event-6
# legacy rows retire) rides the mtk producer campaign.

# The producer import for the family's battery leg: mtk, pinned the
# same way the checkers are (imported, never vendored). The suite is
# self-contained — pure exact-rational library tests, no external
# stores.
MTK_REPO ?= https://github.com/maitria-coalition/mtk
MTK_REV  ?= 89044676d0b31260cf76333fa35e9c5853254dcc
MTK_CO   := $(BUILD)/mtk
MTK_PIN  := $(MTK_CO)/.git/pin-$(MTK_REV)

$(MTK_PIN):
	@if [ ! -d $(MTK_CO)/.git ]; then \
	  git clone --no-checkout $(MTK_REPO) $(MTK_CO); \
	fi
	git -C $(MTK_CO) remote set-url origin $(MTK_REPO)
	@if ! git -C $(MTK_CO) cat-file -e $(MTK_REV)^{commit} 2>/dev/null; then \
	  git -C $(MTK_CO) fetch origin; \
	fi
	git -C $(MTK_CO) checkout $(MTK_REV)
	rm -f $(MTK_CO)/.git/pin-*
	touch $@

.PHONY: minifab-dispatch
minifab-dispatch: $(QTSL_PIN) $(MTK_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  MTK_MANIFEST=$(CURDIR)/$(MTK_CO)/Cargo.toml \
	  $(NODE) semicon/minifab-dispatch/recheck.ts

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

# ------------------------------------------------------------- probprog
# Guaranteed posterior bounds for probabilistic programs: the shared
# three-program comparison against GuBPI and Diabolo (probprog/README.md).
# `probprog` re-derives everything derivable without a quiet box: the
# producer's exact-mass and closed-form-containment tests, the 1e6-path
# Monte-Carlo translation differentials, and the bracket printout at the
# comparison configurations. Wall-clock measurement is deliberately a
# separate target (probprog-measure) — it requires a load-quiet box and
# the baselines built same-box, and receipts are committed, never
# regenerated silently. The container lane (probprog-glc) follows the
# family pattern: the certificate species sealed at revision event 8
# (dac + dcc closures), the pin advanced past it, and container
# verdicts ride the baked roster flagless.

.PHONY: probprog probprog-measure probprog-glc
probprog:
	cargo test --release -p probprog-producer
	./target/release/probprog-produce eval geo-counter 100 --classes 8
	./target/release/probprog-produce eval asym-rw 14 --classes 8
	./target/release/probprog-produce eval die-paradox 6 --classes 8
	./target/release/probprog-produce eval burglar-alarm 4
	./target/release/probprog-produce eval beauquier-3 51
	./target/release/probprog-produce eval coupon-collector5 80 --classes 1
	./target/release/probprog-produce eval israeli-jalfon4 31 --classes 1
	./target/release/probprog-produce eval herman3 31 --classes 2
	./target/release/probprog-produce eval 3d-asym-rw 100 --classes 1
	$(NODE) probprog/gen-table.ts

# Discrete-sweep reproduction, one verb: re-verify every committed
# probprog certificate through both lanes (containers via the baked
# event-8 roster; the remaining EV rows are the 3-D pattern face,
# dev-oracle pending that face's own design leg and revision event),
# re-derive the producer-side oracles and differentials, then rebuild
# the comparison table from the committed receipts and require it to
# re-derive byte-identically.
# Measurement receipts (comparison.json walls/bounds) are committed by
# the measurement protocol on a load-quiet box, never regenerated
# here — this target proves the committed numbers' derivation chain,
# not their wall clocks.
.PHONY: probprog-discrete
probprog-discrete:
	cargo test --release -p probprog-producer
	$(MAKE) probprog-ev
	$(MAKE) probprog-glc
	$(NODE) probprog/gen-table.ts
	git diff --exit-code -- probprog/out/TABLE.md

# Container recheck, baked lane: the DtmcAbsorb + DtmcContraction
# species sealed at revision event 8 and the QTSL_REV pin advanced
# past it, so this target rides the standard $(QTSL_PIN) shape — the
# committed containers' prerequisite tables name the baked dac/dcc
# revisions and the checker resolves them via its roster, no flags
# (manifest `closure` field carries the revisions).
probprog-glc: $(QTSL_PIN)
	cd $(QTSL_CO)/crates/qtsl-geolog-provider && \
	  cargo build --release --bin glc-check
	GLC_CHECK=$(CURDIR)/$(QTSL_CO)/target/release/glc-check \
	  $(NODE) probprog/glc-recheck.ts

# EV-bracket recheck, dev-oracle lane — 3-D pattern face only. The
# finite-face EV rows (coupon-collector5, israeli-jalfon4, herman3)
# folded into their containers when the generator-form seam sealed at
# revision event 8 (the dcc closure); glc-check owns those verdicts
# through probprog-glc above. The 3-D walk's rows are the PATTERN
# face (finitely presented infinite chain), descoped from event 8 to
# its own design leg: until that face's revision event, the checker
# is probprog-evcheck from this workspace and verdicts are labeled
# accordingly (manifest `lane` field).
.PHONY: probprog-ev
probprog-ev:
	cargo build --release -p probprog-producer
	EVCHECK=$(CURDIR)/target/release/probprog-evcheck \
	  $(NODE) probprog/ev-recheck.ts

# ------------------------------------------------------- supermartingales

supermartingales: $(QTSL_PIN) $(SECDIR)
	QTSL_CHECKERS=$(CURDIR)/$(QTSL_CO)/checkers/supermartingales \
	  $(NODE) supermartingales/recheck.ts
	$(MAKE) tables
	$(MAKE) $(SECDIR)/supermartingales.pdf

# ------------------------------------------- sec1 quiet-conditions retime
# The preregistered quiet-load campaign (QUIET-PROTOCOL.md beside the
# receipts): the runner itself refuses (exit 3, no receipt) when the
# 1-minute load average exceeds the gate at campaign start, so quiet
# cells are produced by protocol, never by rerunning until the numbers
# look quiet. Requires SUPERMART_SEC1 = a supermart-sec1 binary built at
# the mtk revision the receipts will pin (build recipe in the retime
# directory README). Success installs receipts + derived PROVENANCE into
# the gb10-quiet-retime directory and re-validates; refusals and
# failures append to build/sec1-quiet-refusals.log and leave the
# committed tree untouched (the campaign stages out of tree first).
QUIET_GATE ?= 4.0
QUIET_CPUS ?= 5-9,15,16
QUIET_DIR   := supermartingales/out/glc-baked/sec1/gb10-quiet-retime
QUIET_STAGE := $(BUILD)/sec1-quiet-stage
.PHONY: sec1-quiet-retime
sec1-quiet-retime:
	@test -n "$(SUPERMART_SEC1)" || { \
	  echo "sec1-quiet-retime: set SUPERMART_SEC1=/path/to/supermart-sec1 (see $(QUIET_DIR)/../QUIET-PROTOCOL.md)"; \
	  exit 2; }
	@mkdir -p $(QUIET_STAGE)/containers
	bash -c 'renice 19 -p $$$$ >/dev/null; \
	  exec taskset -c $(QUIET_CPUS) "$(SUPERMART_SEC1)" retime \
	  --experiments $(CURDIR) --machine gb10-quiet --lane host \
	  --warmups 1 --repeats 10 --max-load-1min $(QUIET_GATE) \
	  --out $(QUIET_STAGE)/campaign.json \
	  --compare $(CURDIR)/supermartingales/out/glc-baked/sec1/incumbents \
	  --retain-containers $(QUIET_STAGE)/containers --pretty' \
	  || { code=$$?; \
	       echo "$$(date -u +%Y-%m-%dT%H:%M:%SZ) exit=$$code gate=$(QUIET_GATE) loadavg='$$(cat /proc/loadavg)'" \
	         >> $(BUILD)/sec1-quiet-refusals.log; \
	       exit $$code; }
	$(NODE) supermartingales/quiet-provenance.ts \
	  $(QUIET_STAGE)/campaign.json $(QUIET_STAGE)/PROVENANCE.json
	mkdir -p $(QUIET_DIR)
	cp $(QUIET_STAGE)/campaign.json $(QUIET_DIR)/campaign.json
	cp $(QUIET_STAGE)/campaign.json $(QUIET_DIR)/produce-host.json
	cp $(QUIET_STAGE)/PROVENANCE.json $(QUIET_DIR)/PROVENANCE.json
	rm -rf $(QUIET_DIR)/containers
	cp -r $(QUIET_STAGE)/containers $(QUIET_DIR)/containers
	$(NODE) tables/gen_tables.ts --validate-retimes-only
	@echo "sec1-quiet-retime: receipts installed in $(QUIET_DIR); run 'make timing-tables' next"

# The cold-process-per-repeat calibration anchor, retaken at the
# version-2+ runner vintage under the quiet-conditions conduct (gate,
# affinity, niceness — QUIET-PROTOCOL.md): the version-1 gb10 cold
# anchor's process walls carry the old runner's 1 s monitor sleep-join,
# so cold anchors are comparable only within one runner vintage. The
# receipt installs BESIDE the version-1 cell (cold-anchor-v2.json in the
# checker-compare family — outside the tables scanner's pattern, so no
# rendered cell can move). Requires SUPERMART_SEC1 built from mtk at a
# version-3-receipt revision, and GLC_CHECK = a built glc-check binary.
COLDV2_DIR   := supermartingales/out/glc-baked/sec1/gb10-checker-compare
COLDV2_STAGE := $(BUILD)/sec1-cold-anchor-v2-stage
.PHONY: sec1-cold-anchor-v2
sec1-cold-anchor-v2:
	@test -n "$(SUPERMART_SEC1)" -a -n "$(GLC_CHECK)" || { \
	  echo "sec1-cold-anchor-v2: set SUPERMART_SEC1=/path/to/supermart-sec1 GLC_CHECK=/path/to/glc-check"; \
	  exit 2; }
	@mkdir -p $(COLDV2_STAGE)
	bash -c 'renice 19 -p $$$$ >/dev/null; \
	  exec taskset -c $(QUIET_CPUS) python3 supermartingales/checker-compare.py cold \
	  --machine gb10-quiet --runner "$(SUPERMART_SEC1)" --glc-check "$(GLC_CHECK)" \
	  $(if $(QTSL_REV),--qtsl-rev $(QTSL_REV),) \
	  --max-load-1min $(QUIET_GATE) --out-dir $(COLDV2_STAGE)' \
	  || { code=$$?; \
	       echo "$$(date -u +%Y-%m-%dT%H:%M:%SZ) exit=$$code gate=$(QUIET_GATE) loadavg='$$(cat /proc/loadavg)'" \
	         >> $(BUILD)/sec1-cold-anchor-v2-refusals.log; \
	       exit $$code; }
	cp $(COLDV2_STAGE)/cold-anchor.json $(COLDV2_DIR)/cold-anchor-v2.json
	@echo "sec1-cold-anchor-v2: receipt installed at $(COLDV2_DIR)/cold-anchor-v2.json"

# The order/interleave methods experiment (preregistration + findings in
# the gb10-order-interleave directory): four arms under the quiet gate,
# affinity, and niceness, varying roster structure only — three roster
# rotations and one round-robin interleaved conduct. A methods
# experiment, never a timing family: the receipts directory does not
# match the tables scanner's ^<machine>-retime$ pattern, so no rendered
# timing cell can move from these receipts. Each arm re-checks the gate;
# completed arms stand on a mid-sequence refusal (see PREREG.md).
OI_DIR   := supermartingales/out/glc-baked/sec1/gb10-order-interleave
OI_STAGE := $(BUILD)/sec1-order-interleave-stage
.PHONY: sec1-order-interleave
sec1-order-interleave:
	@test -n "$(SUPERMART_SEC1)" || { \
	  echo "sec1-order-interleave: set SUPERMART_SEC1=/path/to/supermart-sec1 (see $(OI_DIR)/PREREG.md)"; \
	  exit 2; }
	@mkdir -p $(OI_STAGE)
	bash supermartingales/order-interleave-arms.sh \
	  "$(SUPERMART_SEC1)" "$(CURDIR)" "$(OI_STAGE)" "$(QUIET_GATE)" "$(QUIET_CPUS)" \
	  || { code=$$?; \
	       echo "$$(date -u +%Y-%m-%dT%H:%M:%SZ) exit=$$code gate=$(QUIET_GATE) loadavg='$$(cat /proc/loadavg)'" \
	         >> $(BUILD)/sec1-order-interleave-refusals.log; \
	       exit $$code; }
	@echo "sec1-order-interleave: receipts installed in $(OI_DIR); analyze with supermartingales/order-interleave-analyze.py"

# The sealed-container lane (supermartingales/GLC-CERT.md): the same
# certificates as .glc containers through the geolog fast-path checker,
# built from the pinned qtsl checkout — the committed lanes: the
# synthetic-chain containers (out/glc/, kept checkable forever), the
# baked-closure re-emission (out/glc-baked/: the full corpus plus the
# committed falsifier listings sealed as containers, per-relation claim
# counts via --expect-claims), and the persist_2d re-conduct lane
# (out/glc-baked/persist2d/: both twins as shared-subtree forest
# containers + designed mutants — GLC-CERT.md "the persist_2d
# re-conduct lane"). Requires a QTSL_REV that carries the BL
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
# event 7) — QTSL_REV carries it since the sibling baked lanes'
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
# pin, QTSL_REV: the corpus is sealed at revision event 7 like its
# sibling baked lanes (the lane's original event-4 pin converged onto
# the shared pin at the event-5.1 re-emission, retiring the second
# checkout). Re-derivation path: petri/guard/README.md.
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

.PHONY: lean-route lean-route-recheck timing-receipt-validator-test timing-tables
lean-route-recheck: $(QTSL_PIN)
	QC=$(CURDIR)/$(QTSL_CO)/checkers/supermartingales; \
	$(NODE) $$QC/check_rows.ts --stack $$QC/theories/BLstack.rows.txt \
	  --cert $$QC/theories/BLCert.rows.txt lean-route/fixtures/*.rows.txt

lean-route: lean-route-recheck lean-route-mints $(SECDIR)
	$(MAKE) $(SECDIR)/lean-route.pdf

# ---------------------------------------------------------------- chapter

timing-receipt-validator-test:
	$(NODE) tables/gen_tables.ts --test-retime-validator
	$(NODE) tables/gen_tables.ts --validate-retimes-only

# Narrow timing-table path: validates timing receipt schemas and mutant
# refusals, then regenerates only tables whose values come from committed
# timing receipts. It neither runs campaign producers nor fetches unrelated
# benchmark pins.
timing-tables: timing-receipt-validator-test
	$(NODE) tables/gen_tables.ts --timing-tables-only

# Full table generation retains the broader pin-fetching path needed by the
# non-timing tables.
tables: fetch-pins timing-receipt-validator-test
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

# Mini-Fab optimum (semicon/minifab-floor/README.md): model-R z3 search,
# the original z3 instrument, CNF+LRAT/DRAT certificates (cake_lpr /
# lrat-check / drat-trim), the Lean floor + axiom audit + mutants, the
# decoded-container attainment replay, and the queue-order-free cross-check.
# MAX_N bounds the model-R legs, NOFIFO_MAX_N the cross-check lane.
minifab-floor-recheck:
	bash semicon/minifab-floor/recheck.sh
