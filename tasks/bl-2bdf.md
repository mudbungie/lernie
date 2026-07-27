+++
title = "gate: docs"
created = 1785129517
updated = 1785129609
claimant = "Thimble"
parent = "bl-9c06"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-9c06"
on = "claim"
+++
PASS (no doc change required). Gate for bl-9c06 (delivered on main as 0d0587a).

The delivered change is entirely test-side — `src/cmd/tests/mod.rs`, `src/cmd/tests/verbs.rs`, `src/e2e/bundle_replay_cli.rs`, `src/test_support.rs`. No production code, no CLI surface, no config schema, and no on-disk contract moved, so nothing in `docs/ARCHITECTURE.md`, `docs/PRINCIPLES.md`, `docs/TAXONOMY.md`, or `README.md` describes anything that changed. Checked specifically:

- `docs/ARCHITECTURE.md` §2.2 (harness root, `LERNIE_HOME` as the single override collapsing both roots) still describes resolution exactly as `src/harness_root.rs` implements it — that file was not touched.
- The README's install/run surface is unaffected; `make check` still runs fmt-check, clippy `-D warnings`, tarpaulin coverage, and the install contract.

Documentation that *did* need updating is the in-code doc comment, and it moved with the code: the `ENV_LOCK` / `with_lernie_home` rustdoc now lives on the shared helper in `src/test_support.rs` and states the actual invariant ("the lock, not the caller's own scope, is what makes it safe"), replacing the stale `--test-threads=1` justification that `src/e2e/bundle_replay_cli.rs` had been carrying. The ball body itself was rewritten as the record of mechanism, fix shape, and all three determinism campaigns.