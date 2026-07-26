.PHONY: all build release test test-install coverage lint fmt fmt-check check smoke schemas new-workspace eval install-hooks install install-verify uninstall ci clean

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

# Binaries that resolve via `PATH`: the harness CLI and the eval runner.
# The desktop frontend lives in its own repo (yog) and
# installs from there.
PATH_BINARIES     := lernie agent-eval
# The provider adapter is brazen's `bz` (ARCH §4.4) — one binary for
# every provider, installed from crates.io at the exact version the
# lernie crate links (the load-time version guard, §4.4). The pin's one
# home is the `brazen = "=<pin>"` dependency in Cargo.toml; this derives
# from it (the code-side guard derives from the same line).
BRAZEN_PIN        := $(shell sed -n 's/^brazen = "=\([^"]*\)"$$/\1/p' Cargo.toml)
# The harness-root skeleton (config-lifetime templates + machine-populated
# pools and trees, ARCH §2.2) is founded by `lernie prime`, invoked below —
# the single source of truth for what a ready installation carries. The
# Makefile no longer enumerates the subdirs or re-copies the pools.

all: check

build:
	cargo build --workspace

release:
	cargo build --workspace --release

test:
	cargo test --workspace

# The install contract end-to-end (tests/install.rs). It shells out to
# `make install` — a release build plus `cargo install brazen` — which
# contends with tarpaulin's `target/` lock, so the test carries
# `cfg_attr(tarpaulin, ignore)` and `make coverage` never runs it. This
# target is where it runs instead: uninstrumented, and part of `check`
# below, so the pre-commit/close gate exercises the first thing every
# user touches — including the `include_dir!` embedded-asset seam
# (src/install.rs, src/template/mod.rs) as a real release binary sees it.
# It is the tree's only tarpaulin-ignored test; a future sibling belongs
# on this line, not in a new target.
#
# Cost, accepted deliberately: ~45s warm, and it re-installs `bz` at the
# `brazen` pin (§4.4) onto the cargo bin — so a locally installed `bz`
# newer than the pin is rolled back to what this tree links.
test-install:
	cargo test --test install

TARPAULIN_PIN := 0.35.2

coverage:
	@have=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" != "$(TARPAULIN_PIN)" ]; then \
	  echo "tarpaulin $(TARPAULIN_PIN) required (have: $${have:-none}); see tarpaulin.toml" >&2; \
	  echo "  cargo install cargo-tarpaulin --version $(TARPAULIN_PIN) --locked" >&2; \
	  exit 1; \
	fi
	cargo tarpaulin --workspace --fail-under 100 --skip-clean --engine llvm --out Stdout --exclude-files 'src/bin/*' --exclude-files 'src/bin/lernie/*' --exclude-files 'src/e2e/*' --exclude-files 'crates/*/src/main.rs'

# Regenerate schemas/ from the crate's schema types. The generator is the
# `config::schemas` module; `make schemas` drives it through the in-crate
# golden test's update flow (UPDATE_SCHEMAS=1 rewrites schemas/ in place
# instead of asserting byte-identity). The same test, run without the env
# var, is the CI guard that schemas/ never drifts from the source.
schemas:
	UPDATE_SCHEMAS=1 cargo test --quiet --lib schemas_golden

new-workspace:
	@test -n "$(DEST)" || { echo "usage: make new-workspace DEST=<path>"; exit 1; }
	@cargo run --quiet --bin lernie -- new "$(DEST)"

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

check: fmt-check lint coverage test-install

# `make smoke` — the live-wire smoke test (README "First-run smoke test").
# The FIRST real model call the project makes: `lernie new` + one live
# `lernie prompt` against the SHIPPED defaults (worker role, anthropic /
# claude-sonnet-5) through the real `bz` data plane, with the verdict read
# from observable state (exit 0, a committed transcript on the agent ref, a
# step record with no wire error and real assistant text). Deliberately NOT
# in `check` or the close gate: it needs a configured `bz` credential for
# the anthropic provider (`bz --login --provider anthropic`, or set
# ANTHROPIC_API_KEY / BRAZEN_API_KEY) and spends money. It is the only check
# that proves the authored default model id resolves on the wire — `make
# check` mocks the wire and structurally cannot.
#
# Override the target with BOTH SMOKE_PROVIDER and SMOKE_MODEL (both-or-
# neither; unset => the shipped default above, credential note included):
#   make smoke SMOKE_PROVIDER=local SMOKE_MODEL=<a-pulled-ollama-model>
#   make smoke SMOKE_PROVIDER=codex SMOKE_MODEL=gpt-5.4
# Local ollama needs no credential, only a served model; the credential
# note applies to the anthropic default alone.
smoke:
	@cargo build --quiet --bin lernie
	@LERNIE_BIN="$(CURDIR)/target/debug/lernie" \
	 SMOKE_PROVIDER="$(SMOKE_PROVIDER)" SMOKE_MODEL="$(SMOKE_MODEL)" \
	 bash scripts/smoke.sh

install-hooks:
	git config core.hooksPath .githooks
	@echo "hooks: core.hooksPath -> .githooks"

# `make install` lays down the harness root skeleton on first run and is
# idempotent on subsequent runs. Binaries are re-installed unconditionally
# (a fresh build is the point); config files are guarded with `test -e` so
# rotated credentials and hand-edited entries survive a re-install.
install: release
	@mkdir -p "$(INSTALL_BIN)"
	@for bin in $(PATH_BINARIES); do \
		install -m 0755 "target/release/$$bin" "$(INSTALL_BIN)/$$bin"; \
		echo "installed $(INSTALL_BIN)/$$bin"; \
	done
	@test -n "$(BRAZEN_PIN)" || { echo 'could not derive the brazen pin from Cargo.toml (expected a `brazen = "=<version>"` line)' >&2; exit 1; }
	@echo "installing the provider adapter: cargo install brazen --version =$(BRAZEN_PIN)"
	@cargo install brazen --version "=$(BRAZEN_PIN)" --locked
	@# Found the harness root via the freshly-installed binary (ARCH §2.2):
	@# `lernie prime` seeds the default models.yaml, the tool/skill pools,
	@# and the workflows/ + workspaces/ dirs, seed-if-absent throughout —
	@# hand-edited config survives, and the seeding lives in one place (the
	@# verb), not duplicated here. The env below mirrors this Makefile's
	@# root resolution; the binary applies the identical policy (§2.2).
	@LERNIE_HOME='$(LERNIE_HOME)' XDG_CONFIG_HOME='$(XDG_CONFIG_HOME)' XDG_DATA_HOME='$(XDG_DATA_HOME)' "$(INSTALL_BIN)/lernie" prime
	@echo "primed harness root: $(LERNIE_CONFIG_HOME) (config) + $(LERNIE_DATA_HOME) (data)"
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
