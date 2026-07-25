+++
title = "Run tests/install.rs in the close gate (outside tarpaulin)"
created = 1784955529
updated = 1784955545
claimant = "Lodestar"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Harrow audit finding 7. tests/install.rs:47 is cfg_attr(tarpaulin, ignore) and make check = fmt-check lint coverage — tarpaulin only — so the install contract (first thing every user touches; carries the include_dir! embedded-asset seam) never runs in the pre-commit/close gate. Fix: add 'cargo test --test install' as its own make check step outside tarpaulin. Update README's check/pre-commit description to match.