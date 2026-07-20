//! Test-only helpers shared across the crate's in-crate integration
//! tests — the ones migrated in from `tests/` when the library surface
//! was narrowed to [`crate::cmd`] (§3.4). Not part of the public surface:
//! `#[cfg(test)]` and non-`pub` at the crate root, so the parity checker
//! (which counts only externally-public items) never sees it.

use std::path::PathBuf;

/// Resolve the cargo-built `lernie` binary from the running test binary.
///
/// `env!("CARGO_BIN_EXE_lernie")` is set only for `tests/` integration
/// targets, not for lib unit tests, so an in-crate test that must spawn
/// the real binary derives it from `current_exe()`: the test executable
/// (`<target>/<profile>/deps/<test>-<hash>`) and the `lernie` binary
/// (`<target>/<profile>/lernie`) are siblings — walk up from the test
/// binary and take the first ancestor directory holding a `lernie` file.
pub fn lernie_binary() -> PathBuf {
    let test_exe = std::env::current_exe().expect("current_exe for the test binary");
    for dir in test_exe.ancestors() {
        let candidate = dir.join("lernie");
        if candidate.is_file() {
            return candidate;
        }
    }
    panic!("built `lernie` binary not found above {test_exe:?}; run `cargo build` first");
}
