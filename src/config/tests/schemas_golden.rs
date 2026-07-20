//! Golden test for the generated JSON Schemas — replaces the former
//! `gen-schemas` binary (ARCH §2.2, §4.1). [`crate::config::schemas::write_to`]
//! is the generator; this test pins its output byte-for-byte against the
//! checked-in `schemas/` directory. `UPDATE_SCHEMAS=1` rewrites `schemas/`
//! in place instead of comparing against a throwaway dir — the insta-style
//! update flow `make schemas` drives.

use crate::config::schemas;
use std::path::PathBuf;
use tempfile::TempDir;

/// The checked-in `schemas/` directory at the crate root.
fn schemas_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

#[test]
fn generated_schemas_are_byte_identical_to_checked_in() {
    let checked_in = schemas_dir();
    // Update mode regenerates the checked-in dir in place (`make schemas`);
    // the default run generates into a throwaway dir and then asserts each
    // file matches. `write_to` runs identically either way — only its
    // target differs — so the generator is exercised on every test run.
    let update = std::env::var_os("UPDATE_SCHEMAS").is_some();
    let tmp = TempDir::new().expect("scratch dir");
    // Both candidates are built unconditionally (so neither is left an
    // unexecuted branch in a normal run), then selected by index: update
    // writes the checked-in dir in place (`make schemas`), the default run
    // writes a throwaway dir and compares against the checked-in files.
    let candidates = [tmp.path().to_path_buf(), checked_in.clone()];
    let out_dir = &candidates[usize::from(update)];
    let written = schemas::write_to(out_dir).expect("generate schemas");
    for path in &written {
        let name = path.file_name().expect("schema file name");
        let generated = std::fs::read(path).expect("read generated schema");
        let expected = std::fs::read(checked_in.join(name)).unwrap_or_else(|e| {
            panic!("checked-in schema {name:?} missing: {e}; run `make schemas`")
        });
        assert_eq!(
            generated, expected,
            "schema {name:?} is stale; run `make schemas` to regenerate it",
        );
    }
}
