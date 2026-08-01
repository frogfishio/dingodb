# Residiuum top-level packaging helpers.
#
#   make dist          build release product bins + release briefing → ./dist/
#   make dist-briefing briefing only (uses existing target/ if present)
#   make clean-dist    remove ./dist/
#   make help
#
# Overrides:
#   DIST_BRIEFING_PROFILE=snapshot|formal|pre-release
#   CARGO_TARGET_DIR=target
#   DIST_DIR=dist

.PHONY: help dist dist-bins dist-briefing dist-stage clean-dist

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
DIST_DIR ?= $(ROOT)/dist
CARGO_TARGET_DIR ?= $(ROOT)/target
export CARGO_TARGET_DIR

# Host triple (e.g. aarch64-apple-darwin, x86_64-unknown-linux-gnu)
TARGET_TRIPLE := $(shell rustc -vV 2>/dev/null | sed -n 's/^host: //p')
ifeq ($(TARGET_TRIPLE),)
  TARGET_TRIPLE := unknown-host
endif

# Product CLIs/tools (not test harness binaries).
DIST_BINS := \
	residiuum \
	residiuum-perf \
	residiuum-sda \
	residiuum-authority \
	residiuum-testrig

DIST_BIN_PACKAGES := \
	-p residiuum-cli \
	-p residiuum-perf \
	-p residiuum-sda-cli \
	-p residiuum-authority \
	-p residiuum-testrig

DIST_BRIEFING_PROFILE ?= snapshot
# make dist always produces HTML even if a gate fails; set to 0 to require green.
DIST_BRIEFING_ALLOW_FAIL ?= 1

RELEASE_DIR := $(CARGO_TARGET_DIR)/release
DIST_BIN_DIR := $(DIST_DIR)/bin/$(TARGET_TRIPLE)
DIST_BRIEFING_DIR := $(DIST_DIR)/briefing
DIST_LIB_DIR := $(DIST_DIR)/lib/$(TARGET_TRIPLE)

help:
	@echo "Residiuum Makefile"
	@echo ""
	@echo "  make dist             Release product bins + release-briefing → $(DIST_DIR)/"
	@echo "  make dist-bins        cargo build --release (product packages only)"
	@echo "  make dist-briefing    run scripts/release-briefing.sh"
	@echo "  make clean-dist       rm -rf $(DIST_DIR)"
	@echo ""
	@echo "Layout after make dist:"
	@echo "  $(DIST_DIR)/bin/$(TARGET_TRIPLE)/   product binaries"
	@echo "  $(DIST_DIR)/lib/$(TARGET_TRIPLE)/   selected release .rlib (optional snapshot)"
	@echo "  $(DIST_DIR)/briefing/LATEST.html    release briefing"
	@echo "  $(DIST_DIR)/MANIFEST.txt"
	@echo ""
	@echo "DIST_BRIEFING_PROFILE=$(DIST_BRIEFING_PROFILE)  (snapshot|formal|pre-release)"
	@echo "DIST_BRIEFING_ALLOW_FAIL=$(DIST_BRIEFING_ALLOW_FAIL)  (1=package even if gates fail)"

dist: dist-bins dist-briefing dist-stage
	@echo ""
	@echo "dist OK → $(DIST_DIR)"
	@echo "  bins:     $(DIST_BIN_DIR)/"
	@echo "  briefing: $(DIST_BRIEFING_DIR)/LATEST.html"
	@sed -n '1,20p' "$(DIST_DIR)/MANIFEST.txt"

dist-bins:
	@echo "== cargo build --release (product bins) =="
	cargo build --release $(DIST_BIN_PACKAGES)
	@echo "== cargo build --release (workspace libs via product graph) =="
	@# Product packages already build their dependent libs into $(RELEASE_DIR).
	@# Full workspace release is available via: cargo build --release --workspace
	@true

dist-briefing:
	@echo "== release-briefing (profile=$(DIST_BRIEFING_PROFILE)) =="
	@args="--profile $(DIST_BRIEFING_PROFILE)"; \
	if [ "$(DIST_BRIEFING_ALLOW_FAIL)" = "1" ]; then args="$$args --allow-fail"; fi; \
	bash "$(ROOT)/scripts/release-briefing.sh" $$args

dist-stage:
	@echo "== stage $(DIST_DIR) =="
	rm -rf "$(DIST_DIR)"
	mkdir -p "$(DIST_BIN_DIR)" "$(DIST_LIB_DIR)" "$(DIST_BRIEFING_DIR)"
	@missing=0; \
	for b in $(DIST_BINS); do \
	  src="$(RELEASE_DIR)/$$b"; \
	  if [ ! -f "$$src" ]; then \
	    echo "error: missing release binary: $$src" >&2; \
	    missing=1; \
	    continue; \
	  fi; \
	  cp -p "$$src" "$(DIST_BIN_DIR)/$$b"; \
	  echo "  bin  $$b"; \
	done; \
	if [ "$$missing" != "0" ]; then exit 1; fi
	@# Snapshot main package rlibs if present (consumers still need cargo for real linking).
	@for lib in \
	  libresidiuum_sda.rlib \
	  libresidiuum_format.rlib \
	  libresidiuum_heap.rlib \
	  libresidiuum_store.rlib \
	  libresidiuum_sdk.rlib \
	  libresidiuum_client.rlib \
	  libresidiuum_server.rlib \
	  libresidiuum_examine.rlib \
	  libresidiuum_authority.rlib \
	  libresidiuum_cluster.rlib \
	  libresidiuum_perf.rlib \
	; do \
	  if [ -f "$(RELEASE_DIR)/$$lib" ]; then \
	    cp -p "$(RELEASE_DIR)/$$lib" "$(DIST_LIB_DIR)/$$lib"; \
	    echo "  lib  $$lib"; \
	  fi; \
	done
	@if [ -f "$(ROOT)/target/release-briefing/LATEST.html" ]; then \
	  cp -p "$(ROOT)/target/release-briefing/LATEST.html" "$(DIST_BRIEFING_DIR)/LATEST.html"; \
	  cp -p "$(ROOT)/target/release-briefing/LATEST.html" "$(DIST_DIR)/release-briefing.html"; \
	  echo "  html release-briefing.html"; \
	else \
	  echo "warning: no LATEST.html under target/release-briefing/" >&2; \
	fi
	@if [ -f "$(ROOT)/target/release-briefing/LATEST.json" ]; then \
	  cp -p "$(ROOT)/target/release-briefing/LATEST.json" "$(DIST_BRIEFING_DIR)/LATEST.json"; \
	fi
	@# Also copy latest timestamped briefing if LATEST missing but any exists
	@{ \
	  echo "Residiuum dist MANIFEST"; \
	  echo "generated_utc=$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	  echo "git_head=$$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"; \
	  echo "target_triple=$(TARGET_TRIPLE)"; \
	  echo "workspace_version=$$(sed -n 's/^version = \"\(.*\)\"/\1/p' "$(ROOT)/Cargo.toml" | head -1)"; \
	  echo "briefing_profile=$(DIST_BRIEFING_PROFILE)"; \
	  echo ""; \
	  echo "[bin/$(TARGET_TRIPLE)]"; \
	  ls -1 "$(DIST_BIN_DIR)" 2>/dev/null | sed 's/^/  /' || true; \
	  echo ""; \
	  echo "[lib/$(TARGET_TRIPLE)]"; \
	  ls -1 "$(DIST_LIB_DIR)" 2>/dev/null | sed 's/^/  /' || true; \
	  echo ""; \
	  echo "[briefing]"; \
	  ls -1 "$(DIST_BRIEFING_DIR)" 2>/dev/null | sed 's/^/  /' || true; \
	  if [ -f "$(DIST_DIR)/release-briefing.html" ]; then echo "  ../release-briefing.html"; fi; \
	  echo ""; \
	  echo "Notes:"; \
	  echo "  - Product bins only (not test harness executables)."; \
	  echo "  - .rlib copies are a convenience snapshot; prefer cargo for linking."; \
	  echo "  - Briefing not_run/fail steps are reported in HTML; see formal/HOW_TO_USE.md."; \
	  echo "  - Full CI mirror: ./scripts/quality.sh"; \
	} > "$(DIST_DIR)/MANIFEST.txt"

clean-dist:
	rm -rf "$(DIST_DIR)"
	@echo "removed $(DIST_DIR)"
