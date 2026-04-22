.PHONY: all build test coverage lint fmt fmt-check check schemas install-hooks ci clean

all: check

build:
	cargo build

test:
	cargo test

coverage:
	cargo tarpaulin --fail-under 100 --skip-clean --engine llvm --out Stdout --exclude-files 'src/bin/*'

schemas:
	cargo run --quiet --bin gen-schemas -- schemas

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

ci: check

clean:
	cargo clean
