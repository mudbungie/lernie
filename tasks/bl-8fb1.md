+++
title = "tests/install.rs mutates the machine-global bz, breaking concurrent agents' gates"
created = 1784957523
updated = 1785125239
claimant = "Gimbal"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
RESOLVED. `tests/install.rs` no longer writes the machine-global `~/.cargo/bin/bz`.

**The seam.** `run_install` (the test's own `make install` invocation) sets `CARGO_INSTALL_ROOT` to `<worktree>/target/install-test-cargo-root`. That is cargo's own documented root override — precedence is `--root` flag, then `CARGO_INSTALL_ROOT`, then `install.root` config, then `CARGO_HOME` — so `install-bz`'s `cargo install brazen --version "=$(BRAZEN_PIN)" --locked` lands the pinned `bz` in the per-worktree root instead of on the user's cargo bin. The recipe is untouched: no `--root`, no test-only branch, no new make variable. `make install` run by a user is byte-identical.

The isolation rides on the TEST, not on a make target, so it holds under every runner — `cargo test --test install`, bare `cargo test --workspace`, `make test`, `make test-install` — which is what the report demanded (the reported breakage came from a plain `cargo test` run, which no Makefile-side fix would have covered).

The root is persistent per worktree rather than a `TempDir` so cargo's "already installed" short-circuit keeps re-runs free; `target/` is gitignored.

**Proof.** Forced the install path by shadowing `bz` on `PATH` with a shim reporting `0.0.1` (so `install-bz`'s pin guard mismatches and the real `cargo install` runs), then ran `cargo test --test install`: passed in 145s, brazen 0.0.4 compiled and installed into `target/install-test-cargo-root/bin/bz` (`.crates.toml` lists `brazen 0.0.4`), while `~/.cargo/bin/bz` kept its exact sha256 (`b6506c4d9ff6...`) and mtime (2026-07-25 01:53:24). Unforced (machine `bz` already at the pin) the guard short-circuits and nothing is installed at all — the install test step took 0.39s in the gate.

Docs updated to match: the Makefile `test-install` comment block (which previously documented the rollback as deliberately accepted), the README test-determinism section, and the README pre-commit/`make check` description.

Gate: `make check` exit=0 — fmt-check, clippy, 100.00% coverage (4678/4678 lines), `test-install` ok. Two earlier gate attempts died on load-induced infrastructure flakes under fleet load ~90 (tarpaulin `ECHILD: No child processes`, then a SIGTERM at Error 143), neither an assertion failure; the clean run is the record.