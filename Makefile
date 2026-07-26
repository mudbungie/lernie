.PHONY: all build release test test-install coverage lint fmt fmt-check check smoke schemas new-workspace eval install-hooks install install-bz brazen-pin install-verify uninstall ci clean

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

# Test determinism: the pinned adapter, not whatever `bz` is on PATH.
#
# The e2e tests exec the REAL `bz`, and the load-time version guard (§4.4)
# demands the pin EXACTLY. Resolving it from `PATH` made every test run
# depend on machine-global mutable state (`~/.cargo/bin/bz`): one agent
# installing a different brazen version failed another worktree's gate
# on five-plus e2e tests, indistinguishable at a glance from a code
# regression. So the test targets below resolve `bz` from a cache keyed on
# the pin and put that directory FIRST on `PATH` — a worktree's tests always
# run the worktree's pin, whatever the machine's `bz` happens to be. This is
# test determinism only: runtime resolution for real use (§4.4 — `adapter:`
# override, injected target, else `bz` on `PATH`) is untouched, and so is
# `make install`, which still puts the pinned `bz` on the user's `PATH`.
#
# The version is BRAZEN_PIN above — Cargo.toml's `brazen = "="` line, the
# number's one home, the same line the code-side guard reads. The directory
# is NAMED after it, so a pin bump is a cache miss and nothing else; nothing
# is ever overwritten in place, which is what keeps a shared cache safe for
# parallel worktrees.
#
# Cost: cold, one `cargo install` per pin per machine (the cache is under
# XDG_CACHE_HOME, so sibling worktrees share it and only the first pays).
# Warm, one `stat` — the recipe is a file target, so make skips it outright.
XDG_CACHE_HOME ?= $(HOME)/.cache
BZ_TEST_ROOT   := $(XDG_CACHE_HOME)/lernie/bz/$(BRAZEN_PIN)
BZ_TEST_PATH   := $(BZ_TEST_ROOT)/bin:$(PATH)

$(BZ_TEST_ROOT)/bin/bz:
	@test -n "$(BRAZEN_PIN)" || { echo 'could not derive the brazen pin from Cargo.toml (expected a `brazen = "=<version>"` line)' >&2; exit 1; }
	@echo "test adapter: installing bz $(BRAZEN_PIN) into $(BZ_TEST_ROOT)"
	@cargo install brazen --version "=$(BRAZEN_PIN)" --locked --root "$(BZ_TEST_ROOT)"

test: $(BZ_TEST_ROOT)/bin/bz
	PATH="$(BZ_TEST_PATH)" cargo test --workspace

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
# newer than the pin is rolled back to what this tree links. That is the
# install contract's own business and no longer anyone else's: the e2e
# tests read the pin-keyed cache (BZ_TEST_ROOT above), so this rollback
# cannot fail a sibling worktree's gate.
test-install:
	cargo test --test install

TARPAULIN_PIN := 0.35.2

coverage: $(BZ_TEST_ROOT)/bin/bz
	@have=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" != "$(TARPAULIN_PIN)" ]; then \
	  echo "tarpaulin $(TARPAULIN_PIN) required (have: $${have:-none}); see tarpaulin.toml" >&2; \
	  echo "  cargo install cargo-tarpaulin --version $(TARPAULIN_PIN) --locked" >&2; \
	  exit 1; \
	fi
	PATH="$(BZ_TEST_PATH)" cargo tarpaulin --workspace --fail-under 100 --skip-clean --engine llvm --out Stdout --exclude-files 'src/bin/*' --exclude-files 'src/bin/lernie/*' --exclude-files 'src/e2e/*' --exclude-files 'crates/*/src/main.rs'

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
#   make eval CONFIG=baseline SUITE=tests/suite RUNS=5 AGENT=<driver-cmd>
# AGENT is REQUIRED and has no default: the runner drives the agent under
# test through an external harness-driver program (the §9.3 agent seam),
# and no such driver ships with lernie. Write one against the contract in
# README "Run the suite" and pass it here. There is deliberately no stand-in
# default — a made-up one only moves the failure from this line to a failed
# spawn once per task.
eval:
	@test -n "$(CONFIG)" -a -n "$(AGENT)" || { \
	  echo "usage: make eval CONFIG=<experiment> SUITE=<dir> RUNS=<n> AGENT=<driver-cmd>"; \
	  echo "AGENT is required: no harness driver ships with lernie (README, \"Run the suite\")"; \
	  exit 1; }
	@cargo run --quiet -p agent-eval -- --config "$(CONFIG)" --suite "$(SUITE)" --runs "$(RUNS)" --agent "$(AGENT)"

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

# Print the brazen pin on stdout, and nothing else. Exists so a consumer
# that needs the version as a *value* reads it from the pin's one home
# (the `brazen = "="` line in Cargo.toml, via BRAZEN_PIN above) instead
# of copying the number: `.github/workflows/ci.yml` keys its `bz` cache
# on `make brazen-pin`, so bumping the dependency bumps the cache too and
# no workflow file ever names a version.
brazen-pin:
	@echo "$(BRAZEN_PIN)"

# Install the provider adapter `bz` (ARCH §4.4) at the pinned version.
# Idempotent and cheap: a no-op when the `bz` already on PATH reports the
# pin, so a warm CI cache and a re-run of `make install` both cost
# nothing. The load-time version guard demands an EXACT match, so a `bz`
# at any other version — newer included — is replaced, not kept.
#
# This is the USER-facing install, not a test prerequisite: `make test`
# and `make coverage` feed the e2e tests their own pin-keyed `bz` (see
# BZ_TEST_ROOT above) and no longer care what is on the cargo bin.
install-bz:
	@test -n "$(BRAZEN_PIN)" || { echo 'could not derive the brazen pin from Cargo.toml (expected a `brazen = "=<version>"` line)' >&2; exit 1; }
	@have=$$(bz --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" = "$(BRAZEN_PIN)" ]; then \
	  echo "provider adapter: bz $(BRAZEN_PIN) already on PATH"; \
	else \
	  echo "installing the provider adapter: cargo install brazen --version =$(BRAZEN_PIN)"; \
	  cargo install brazen --version "=$(BRAZEN_PIN)" --locked; \
	fi

# `make install` lays down the harness root skeleton on first run and is
# idempotent on subsequent runs. The binaries built from this tree are
# re-installed unconditionally (a fresh build is the point) while the
# crates.io-pinned `bz` is left alone when it already matches the pin
# (see `install-bz`); config files are guarded with `test -e` so
# rotated credentials and hand-edited entries survive a re-install.
install: release
	@mkdir -p "$(INSTALL_BIN)"
	@for bin in $(PATH_BINARIES); do \
		install -m 0755 "target/release/$$bin" "$(INSTALL_BIN)/$$bin"; \
		echo "installed $(INSTALL_BIN)/$$bin"; \
	done
	@$(MAKE) --no-print-directory install-bz
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
