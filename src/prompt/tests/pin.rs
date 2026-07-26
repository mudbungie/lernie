//! The brazen pin has one home — the `brazen = "=<pin>"` dependency in
//! `Cargo.toml` — and [`crate::prompt::brazen_pin`] derives from it
//! (§4.4 "Version skew is guarded"; PRINCIPLES: single source of truth).

use crate::prompt::{brazen_pin, parse_brazen_pin};

#[test]
fn pin_derives_from_the_embedded_manifest() {
    let pin = brazen_pin();
    assert!(pin.chars().next().is_some_and(|c| c.is_ascii_digit()));
}

/// The pin has one home and TWO readers, and they must agree: the crate's
/// [`brazen_pin`] (which the load-time version guard compares `bz
/// --version` against, §4.4) and the Makefile's `BRAZEN_PIN` (which names
/// the pin-keyed `bz` the test targets put first on `PATH`, and the
/// version `make install` lays down). Were the two spellings to drift,
/// every e2e test would fail the guard against a `bz` the Makefile itself
/// installed — the exact confusion this pins shut.
#[test]
fn the_makefile_derives_the_same_pin() {
    let out = std::process::Command::new("make")
        // Drop the outer make's flags: this runs under `make coverage`,
        // and an inherited jobserver would warn on a recursive make.
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .args(["--no-print-directory", "-C", env!("CARGO_MANIFEST_DIR")])
        .arg("brazen-pin")
        .output()
        .expect("spawn `make brazen-pin`");
    assert!(
        out.status.success(),
        "make brazen-pin: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), brazen_pin());
}

#[test]
fn parse_reads_the_inline_source_spelling() {
    let manifest = "[dependencies]\nbrazen = \"=0.0.9\"\nserde = \"1\"\n";
    assert_eq!(parse_brazen_pin(manifest), Some("0.0.9"));
}

#[test]
fn parse_reads_the_normalized_published_spelling() {
    let manifest = "[dependencies.serde]\nversion = \"1\"\n\n\
                    [dependencies.brazen]\nversion = \"=0.0.9\"\n";
    assert_eq!(parse_brazen_pin(manifest), Some("0.0.9"));
}

#[test]
fn parse_declines_a_manifest_without_an_exact_pin() {
    // A non-exact requirement is no pin at all — the version guard
    // would be comparing against a range, not a number.
    assert_eq!(parse_brazen_pin("brazen = \"0.0.9\""), None);
    // A `version =` line outside the brazen table is another
    // dependency's, not ours.
    assert_eq!(
        parse_brazen_pin("[dependencies.serde]\nversion = \"=1.0.0\"\n"),
        None
    );
    assert_eq!(parse_brazen_pin(""), None);
}
