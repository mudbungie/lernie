//! The brazen pin has one home — the `brazen = "=<pin>"` dependency in
//! `Cargo.toml` — and [`crate::prompt::brazen_pin`] derives from it
//! (§4.4 "Version skew is guarded"; PRINCIPLES: single source of truth).

use crate::prompt::{brazen_pin, parse_brazen_pin};

#[test]
fn pin_derives_from_the_embedded_manifest() {
    let pin = brazen_pin();
    assert!(pin.chars().next().is_some_and(|c| c.is_ascii_digit()));
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
