.PHONY: all build release test coverage lint fmt fmt-check check schemas new-workspace ui eval install-hooks install install-verify uninstall ci clean

# Install location for `make install`. Defaults to the XDG-ish user-local
# convention; override for system-wide installs or packaging:
#   make install INSTALL_PREFIX=/usr/local
INSTALL_PREFIX ?= $(HOME)/.local
INSTALL_BIN    := $(INSTALL_PREFIX)/bin

# Harness root (ARCH §2.2): installation-global state, split by XDG
# lifetime into a config root (hand-edited declarations) and a data root
# (machine-populated pools). LERNIE_HOME, if set, collapses BOTH to that
# one directory — at install time and at runtime alike. This mirrors the
# resolver policy in src/harness_root.rs exactly.
XDG_CONFIG_HOME    ?= $(HOME)/.config
XDG_DATA_HOME      ?= $(HOME)/.local/share
ifdef LERNIE_HOME
LERNIE_CONFIG_HOME := $(LERNIE_HOME)
LERNIE_DATA_HOME   := $(LERNIE_HOME)
else
LERNIE_CONFIG_HOME := $(XDG_CONFIG_HOME)/lernie
LERNIE_DATA_HOME   := $(XDG_DATA_HOME)/lernie
endif

# Binaries that resolve via `PATH`: the harness CLI and the UI shell.
PATH_BINARIES     := lernie lernie-ui-egui agent-eval
# The provider adapter is brazen's `bz` (ARCH §4.4) — one binary for
# every provider, installed from crates.io at the exact version the
# lernie crate links (the load-time version guard, §4.4). Keep this pin
# in lockstep with the `brazen = "=<pin>"` dependency in Cargo.toml.
BRAZEN_PIN        := 0.0.2
# Subdirectories laid down on first install, by lifetime (ARCH §2.2):
# config-lifetime templates under the config root, machine-populated
# pools and the workspaces tree under the data root. (The v0.3 frozen
# `agents/` profile pool is retired: the fork is the freeze, §2.2 —
# configs live as config commits inside each workspace.)
CONFIG_DIRS       := workflows
DATA_DIRS         := tools skills workspaces

all: check

build:
	cargo build --workspace

release:
	cargo build --workspace --release

test:
	cargo test --workspace

TARPAULIN_PIN := 0.35.2

coverage:
	@have=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" != "$(TARPAULIN_PIN)" ]; then \
	  echo "tarpaulin $(TARPAULIN_PIN) required (have: $${have:-none}); see tarpaulin.toml" >&2; \
	  echo "  cargo install cargo-tarpaulin --version $(TARPAULIN_PIN) --locked" >&2; \
	  exit 1; \
	fi
	cargo tarpaulin --workspace --fail-under 100 --skip-clean --engine llvm --out Stdout --exclude-files 'src/bin/*' --exclude-files 'src/bin/lernie/*' --exclude-files 'crates/*/src/main.rs'

schemas:
	cargo run --quiet --bin gen-schemas -- schemas

new-workspace:
	@test -n "$(DEST)" || { echo "usage: make new-workspace DEST=<path>"; exit 1; }
	@cargo run --quiet --bin lernie -- new "$(DEST)"

ui:
	@test -n "$(REPO)" || { echo "usage: make ui REPO=<path>"; exit 1; }
	@cargo run --quiet --bin lernie-ui-egui -- --repo "$(REPO)"

# Run the evaluation runner (ARCH §9.3): experiment × suite × N.
#   make eval CONFIG=baseline SUITE=tests/suite RUNS=5 [AGENT=<cmd>]
eval:
	@test -n "$(CONFIG)" || { echo "usage: make eval CONFIG=<experiment> SUITE=<dir> RUNS=<n> [AGENT=<cmd>]"; exit 1; }
	@cargo run --quiet -p agent-eval -- --config "$(CONFIG)" --suite "$(SUITE)" --runs "$(RUNS)" $(if $(AGENT),--agent "$(AGENT)",)

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check: fmt-check lint coverage

install-hooks:
	git config core.hooksPath .githooks
	@echo "hooks: core.hooksPath -> .githooks"

# `make install` lays down the harness root skeleton on first run and is
# idempotent on subsequent runs. Binaries are re-installed unconditionally
# (a fresh build is the point); config files are guarded with `test -e` so
# rotated credentials and hand-edited entries survive a re-install.
install: release
	@mkdir -p "$(INSTALL_BIN)"
	@for d in $(CONFIG_DIRS); do mkdir -p "$(LERNIE_CONFIG_HOME)/$$d"; done
	@for d in $(DATA_DIRS); do mkdir -p "$(LERNIE_DATA_HOME)/$$d"; done
	@for bin in $(PATH_BINARIES); do \
		install -m 0755 "target/release/$$bin" "$(INSTALL_BIN)/$$bin"; \
		echo "installed $(INSTALL_BIN)/$$bin"; \
	done
	@echo "installing the provider adapter: cargo install brazen --version =$(BRAZEN_PIN)"
	@cargo install brazen --version "=$(BRAZEN_PIN)" --locked
	@if [ ! -e "$(LERNIE_CONFIG_HOME)/models.yaml" ]; then \
		install -m 0644 install/models.yaml "$(LERNIE_CONFIG_HOME)/models.yaml"; \
		echo "installed $(LERNIE_CONFIG_HOME)/models.yaml"; \
	else \
		echo "kept     $(LERNIE_CONFIG_HOME)/models.yaml (existing)"; \
	fi
	@# Data-root pools (ARCH §3.3): the tool JSON schemas and skill dirs
	@# the descriptions-always producer snapshots into each new repo's
	@# `descriptions/**` at `lernie new`. Shipped artifacts are re-copied
	@# (a fresh source is the point); user-added pool entries survive.
	@install -m 0644 schemas/tools/*.json "$(LERNIE_DATA_HOME)/tools/"
	@echo "installed tool schemas -> $(LERNIE_DATA_HOME)/tools/"
	@cp -R skills/. "$(LERNIE_DATA_HOME)/skills/"
	@echo "installed skill pool   -> $(LERNIE_DATA_HOME)/skills/"
	@$(MAKE) --no-print-directory install-verify
	@echo
	@echo "make sure $(INSTALL_BIN) is on your PATH (and that 'bz' resolves there too)."
	@echo "config root: $(LERNIE_CONFIG_HOME)   data root: $(LERNIE_DATA_HOME)"
	@echo "  (LERNIE_HOME collapses both; else \$$XDG_CONFIG_HOME / \$$XDG_DATA_HOME)"
	@echo "provider endpoints/auth live in brazen's config: bz --dump-config / bz --login."
	@echo "declare model capabilities in $(LERNIE_CONFIG_HOME)/models.yaml — see ARCH §4.2/§4.4."

# Smoke-test the freshly installed binaries: `lernie --version` proves the
# CLI loads, `lernie new` exercises workspace creation (bare repo.git +
# first config commit, ARCH §2.2) against a throwaway path. Failure here
# aborts `make install` with a non-zero exit, since a half-installed
# harness is worse than none.
install-verify:
	@tmp=$$(mktemp -d) && trap "rm -rf $$tmp" EXIT && \
		"$(INSTALL_BIN)/lernie" --version >/dev/null && \
		"$(INSTALL_BIN)/lernie" new "$$tmp/test" >/dev/null && \
		echo "verify: lernie --version + lernie new ok"

uninstall:
	@for bin in $(PATH_BINARIES); do \
		rm -f "$(INSTALL_BIN)/$$bin" && echo "removed $(INSTALL_BIN)/$$bin"; \
	done
	@echo "note: 'bz' (brazen) was installed via cargo; remove with 'cargo uninstall brazen'."

ci: check

clean:
	cargo clean
