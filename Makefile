.PHONY: all build release test coverage lint fmt fmt-check check schemas new-conversation install-hooks install uninstall ci clean

# Install location for `make install`. Defaults to the XDG-ish user-local
# convention; override for system-wide installs or packaging:
#   make install INSTALL_PREFIX=/usr/local
INSTALL_PREFIX ?= $(HOME)/.local
INSTALL_BIN    := $(INSTALL_PREFIX)/bin

BINARIES := lernie lernie-provider-anthropic lernie-ui-egui

all: check

build:
	cargo build --workspace

release:
	cargo build --workspace --release

test:
	cargo test --workspace

coverage:
	cargo tarpaulin --workspace --fail-under 100 --skip-clean --engine llvm --out Stdout --exclude-files 'src/bin/*' --exclude-files 'crates/*/src/main.rs'

schemas:
	cargo run --quiet --bin gen-schemas -- schemas

new-conversation:
	@test -n "$(DEST)" || { echo "usage: make new-conversation DEST=<path>"; exit 1; }
	@cargo run --quiet --bin lernie -- new "$(DEST)"

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

install: release
	@mkdir -p "$(INSTALL_BIN)"
	@for bin in $(BINARIES); do \
		install -m 0755 "target/release/$$bin" "$(INSTALL_BIN)/$$bin"; \
		echo "installed $(INSTALL_BIN)/$$bin"; \
	done
	@echo
	@echo "make sure $(INSTALL_BIN) is on your PATH."
	@echo "drop external provider adapters into $(INSTALL_BIN) (or any PATH dir)"
	@echo "with the name 'lernie-provider-<name>' — see ARCH §4.4."

uninstall:
	@for bin in $(BINARIES); do \
		rm -f "$(INSTALL_BIN)/$$bin" && echo "removed $(INSTALL_BIN)/$$bin"; \
	done

ci: check

clean:
	cargo clean
