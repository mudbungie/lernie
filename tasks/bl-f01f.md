+++
title = "Run tests/install.rs in the close gate (outside tarpaulin)"
created = 1784955529
updated = 1784955580
claimant = "Lodestar"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Harrow audit finding 7. tests/install.rs:47 is cfg_attr(tarpaulin, ignore) and make check = fmt-check lint coverage — tarpaulin only — so the install contract (first thing every user touches; carries the include_dir! embedded-asset seam) never runs in the pre-commit/close gate. Fix: add 'cargo test --test install' as its own make check step outside tarpaulin. Update README's check/pre-commit description to match.

## Measured cost (Lodestar, 2026-07-24)

`cargo test --test install` standalone: 1m03s wall from a cold debug build; the test body itself 45s. It shells out to `make install`, i.e. `cargo build --workspace --release` plus TWO `cargo install brazen --version =<pin>` invocations (the idempotency re-run). Warm, the release build is incremental and the second cargo install short-circuits ("already installed").

Global side effect, accepted: the test installs `bz` at the Cargo.toml `brazen` pin onto the user's cargo bin. A locally installed newer `bz` is rolled back to the pinned version on every gate run (observed: bz 0.0.4 -> 0.0.3). This is arguably the correct end state — the load-time version guard (ARCH §4.4) rejects a `bz` that is not the pin — but it does mutate the machine outside the repo, and concurrent `make check` runs serialize on cargo's install lock.

`tests/install.rs` is the ONLY cfg_attr(tarpaulin, ...) site in tests/, src/, crates/ — no siblings to fold in.