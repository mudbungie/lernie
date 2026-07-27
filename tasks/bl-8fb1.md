+++
title = "tests/install.rs mutates the machine-global bz, breaking concurrent agents' gates"
created = 1784957523
updated = 1785124119
claimant = "Gimbal"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
`tests/install.rs::make_install_lays_down_skeleton_idempotently` shells out to `make install`, whose recipe runs `cargo install brazen --version =$(BRAZEN_PIN)` — a write to the machine-global `~/.cargo/bin/bz`, derived from THAT worktree's Cargo.toml pin.

Under agent fan-out this is a cross-worktree side channel. During bl-8c92 (bumping the pin 0.0.3 -> 0.0.4) a sibling agent's plain `cargo test` run reinstalled bz 0.0.3 from its own still-0.0.3 worktree, mid-flight, and five of this ball's e2e tests failed with `bz version "0.0.3" does not match the linked brazen crate "0.0.4"`. The workaround was to hold a copy of the 0.0.4 binary and restore it on a 10s poll for the duration of the gate — not something a gate should need.

The test is `#[cfg_attr(tarpaulin, ignore)]`, so `make check` does not trigger it; `make test` / bare `cargo test --workspace` does.

The hazard is structural, not incidental: a per-machine singleton (`~/.cargo/bin/bz`) is written by a test, while the version guard (src/prompt/resolve.rs, ARCH §4.4) makes every e2e test depend on that singleton matching the local pin. Two worktrees at different pins cannot both pass their gate on one box.

Candidate directions (undecided):
- point `make install`'s `cargo install` at a test-local `--root` when invoked from the test, so it never touches the user's cargo bin (the test asserts the harness-owned layout only — it does not assert anything about where bz landed);
- or drop the `cargo install brazen` step from what the test exercises.

Filed from bl-8c92; not fixed there (that ball was the pin bump).