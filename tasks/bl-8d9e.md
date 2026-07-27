+++
title = "gate: docs"
created = 1785124190
updated = 1785125415
claimant = "Gimbal"
parent = "bl-8fb1"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-8fb1"
on = "claim"
+++
PASS. Three docs carried claims that the change falsified; all three were corrected in the same delivery.

1. `Makefile`, the `test-install` comment block — previously: "Cost, accepted deliberately: ~45s warm, and it re-installs `bz` at the `brazen` pin (§4.4) onto the cargo bin — so a locally installed `bz` newer than the pin is rolled back to what this tree links." Now states that the test writes no machine-global state and names the seam (`CARGO_INSTALL_ROOT` -> a per-worktree root under `target/`, `tests/install.rs::bz_install_root`).

2. `README.md` test-determinism section — the trap paragraph no longer blames `make test-install` for rewriting `~/.cargo/bin/bz` (only `make install` does), and a new bullet sits beside the two existing consequences: "No test writes the global `bz`."

3. `README.md` pre-commit/`make check` description — previously "It costs ~45s warm and rolls a locally installed `bz` back to the `brazen` pin this tree links"; now records that the machine-global binary is left alone and why.

`tests/install.rs` documents the reasoning at the seam itself: the module header notes the redirect, and `bz_install_root`'s doc comment states the hazard (a machine-global singleton plus the §4.4 load-time version guard), cargo's root precedence (`--root` > `CARGO_INSTALL_ROOT` > `install.root` > `CARGO_HOME`), and why the root is persistent rather than a `TempDir`.

Checked and correctly left alone: `docs/ARCHITECTURE.md` §4.4 ("`make install` installs the pinned binary (`cargo install brazen --version =<pin>`)") — still true, the user-facing recipe is unchanged; `README.md` build-targets table rows for `make install` / `make install-bz`; `.github/workflows/ci.yml`, which keys its `bz` cache on `make brazen-pin` and never names a version.