.PHONY: all build release test coverage lint fmt fmt-check check schemas new-conversation ui install-hooks install install-verify uninstall ci clean

# Install location for `make install`. Defaults to the XDG-ish user-local
# convention; override for system-wide installs or packaging:
#   make install INSTALL_PREFIX=/usr/local
INSTALL_PREFIX ?= $(HOME)/.local
INSTALL_BIN    := $(INSTALL_PREFIX)/bin

# Harness root (ARCH §2.2): installation-global state, defaults to
# `~/.lernie/`, overridable via LERNIE_HOME — at install time and at
# runtime alike.
LERNIE_HOME    ?= $(HOME)/.lernie

# Binaries that resolve via `PATH`: the harness CLI and the UI shell.
PATH_BINARIES     := lernie lernie-ui-egui
# Binaries the harness resolves at `<harness-root>/adapters/`
# (ARCH §4.1, §4.4) — provider adapters live alongside the harness root,
# not on PATH, so credential rotations and adapter swaps stay local to
# this install.
ADAPTER_BINARIES  := lernie-provider-anthropic
# Subdirectories of the harness root laid down on first install.
HARNESS_DIRS      := adapters workflows tools skills agents conversations

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
	cargo tarpaulin --workspace --fail-under 100 --skip-clean --engine llvm --out Stdout --exclude-files 'src/bin/*' --exclude-files 'crates/*/src/main.rs'

schemas:
	cargo run --quiet --bin gen-schemas -- schemas

new-conversation:
	@test -n "$(DEST)" || { echo "usage: make new-conversation DEST=<path>"; exit 1; }
	@cargo run --quiet --bin lernie -- new "$(DEST)"

ui:
	@test -n "$(REPO)" || { echo "usage: make ui REPO=<path>"; exit 1; }
	@cargo run --quiet --bin lernie-ui-egui -- --repo "$(REPO)"

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
	@for d in $(HARNESS_DIRS); do mkdir -p "$(LERNIE_HOME)/$$d"; done
	@for bin in $(PATH_BINARIES); do \
		install -m 0755 "target/release/$$bin" "$(INSTALL_BIN)/$$bin"; \
		echo "installed $(INSTALL_BIN)/$$bin"; \
	done
	@for bin in $(ADAPTER_BINARIES); do \
		install -m 0755 "target/release/$$bin" "$(LERNIE_HOME)/adapters/$$bin"; \
		echo "installed $(LERNIE_HOME)/adapters/$$bin"; \
	done
	@if [ ! -e "$(LERNIE_HOME)/providers.yaml" ]; then \
		install -m 0644 install/providers.yaml "$(LERNIE_HOME)/providers.yaml"; \
		echo "installed $(LERNIE_HOME)/providers.yaml"; \
	else \
		echo "kept     $(LERNIE_HOME)/providers.yaml (existing)"; \
	fi
	@if [ ! -e "$(LERNIE_HOME)/agents/default" ]; then \
		cp -R template "$(LERNIE_HOME)/agents/default"; \
		echo "installed $(LERNIE_HOME)/agents/default/"; \
	else \
		echo "kept     $(LERNIE_HOME)/agents/default/ (existing)"; \
	fi
	@$(MAKE) --no-print-directory install-verify
	@echo
	@echo "make sure $(INSTALL_BIN) is on your PATH."
	@echo "harness root: $(LERNIE_HOME) (override with LERNIE_HOME=...)"
	@echo "drop external provider adapters into $(LERNIE_HOME)/adapters/"
	@echo "with the name 'lernie-provider-<name>' — see ARCH §4.4."

# Smoke-test the freshly installed binaries: `lernie --version` proves the
# CLI loads, `lernie new` exercises the embedded template scaffold against
# a throwaway path. Failure here aborts `make install` with a non-zero
# exit, since a half-installed harness is worse than none.
install-verify:
	@tmp=$$(mktemp -d) && trap "rm -rf $$tmp" EXIT && \
		"$(INSTALL_BIN)/lernie" --version >/dev/null && \
		"$(INSTALL_BIN)/lernie" new "$$tmp/test" >/dev/null && \
		echo "verify: lernie --version + lernie new ok"

uninstall:
	@for bin in $(PATH_BINARIES); do \
		rm -f "$(INSTALL_BIN)/$$bin" && echo "removed $(INSTALL_BIN)/$$bin"; \
	done
	@for bin in $(ADAPTER_BINARIES); do \
		rm -f "$(LERNIE_HOME)/adapters/$$bin" && echo "removed $(LERNIE_HOME)/adapters/$$bin"; \
	done

ci: check

clean:
	cargo clean
