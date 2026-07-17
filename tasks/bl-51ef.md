+++
title = "SSOT: derive BRAZEN_PIN from Cargo.toml (single home for the brazen pin)"
created = 1784266667
updated = 1784266667
claimant = "Letdowns"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
The brazen version pin lives in three homes: Cargo.toml (brazen = "=0.0.2"), src/prompt/mod.rs BRAZEN_PIN const (doc comment admits 'the two homes of the number; keep them in lockstep'), and Makefile BRAZEN_PIN := 0.0.2. Violates single source of truth. Fix: Cargo.toml is the one home. mod.rs derives the pin at compile time (include_str! Cargo.toml + tested parse; no build.rs). Makefile derives via sed from Cargo.toml with a non-empty guard. Update the mod.rs doc comment and Makefile comment to state the derivation.