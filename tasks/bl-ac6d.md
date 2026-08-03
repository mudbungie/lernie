+++
title = "release 0.0.5 carrying the fork-dialog prune (bl-5a36) to crates.io"
created = 1785732134
updated = 1785732397
claimant = "release-driver"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-7a63"
on = "close"
+++
SHIPPED 2026-08-02. lernie 0.0.5 published to crates.io, carrying bl-5a36 (fork-dialog prune: children no longer inherit the fork point's messages/**, summary/**, skills/**; compactor and --from roots exempt).

Mechanism: the repo's release-plz flow, exactly as 0.0.4. bl-5a36 (feeb127) pushed to origin/main (merge 27bcc7d, after reconciling the unpulled PR #4 merge); prep ball bl-7a63 landed 'make promote-changelog VERSION=0.0.5' on main (b682a73); release PR #5 (chore: release v0.0.5, branch release-plz-2026-08-02T23-09-47Z, Cargo.toml/lock bump only) merged at 2026-08-02T23:19:58Z (3aeda2c); the push-triggered Release-plz run 30772032953 went green on all jobs (CI, release-plz release, binaries) — publish ran in CI via crates.io trusted publishing, nothing local.

Verification: crates.io API reports max_version 0.0.5 (updated 2026-08-02T23:26:24Z); https://static.crates.io/crates/lernie/lernie-0.0.5.crate downloaded and contains src/prompt/dispatch/step_commit/inherited.rs (+ its tests) and CHANGELOG.md with the [0.0.5] section carrying the bl-5a36 bullet. Tag v0.0.5 on 3aeda2c; GitHub Release cut with linux-gnu binary attached. yog's pin bump is a separate ball, untouched here.