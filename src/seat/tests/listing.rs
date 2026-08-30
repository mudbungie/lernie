//! What this box holds, said without dialling any of it.

use super::{entry, listing};
use crate::channel::entries::{WIRE, WORKSPACE};
use crate::channel::material::ADDRESS;
use crate::cli::Stream;
use crate::test_support::{Scratch, mint};

/// **The listing dials nothing.** It names this box's own engine first, then
/// one row per entry in leaf order, each carrying its address or the reason it
/// has none.
#[test]
fn the_listing_names_the_own_engine_then_every_entry_in_order() {
    let scratch = Scratch::new();
    mint::provisioned(&scratch.dir(WIRE), "engine.example:9000");
    mint::provisioned(&scratch.path().join(entry("zulu")), "zulu.example:9000");
    let renamed = scratch.path().join(entry("alpha"));
    mint::provisioned(&renamed, "alpha.example:9000");
    std::fs::write(renamed.join(WORKSPACE), "personal").expect("the workspace file");
    std::fs::create_dir_all(scratch.path().join(entry("hollow"))).expect("mkdir");

    let verdict = listing(scratch.path());
    assert_eq!(verdict.code, 0);
    assert_eq!(verdict.stream, Stream::Out);
    let lines: Vec<&str> = verdict.text.lines().collect();
    assert_eq!(lines[0], "(this box's own engine)");
    assert_eq!(lines[1].trim(), "engine.example:9000");
    assert_eq!(lines[2], r#"alpha (named "personal" on its host)"#);
    assert_eq!(lines[3].trim(), "alpha.example:9000");
    assert_eq!(lines[4], "hollow");
    assert!(lines[5].contains("is an empty entry"), "{}", lines[5]);
    assert_eq!(lines[6], "zulu");
}

/// A box holding nothing lists honestly and SUCCEEDS: zero channels is a fact
/// about provisioning, and the operator asking is the operator who would fix
/// it.
#[test]
fn a_box_holding_nothing_lists_nothing_and_succeeds() {
    let scratch = Scratch::new();
    let verdict = listing(scratch.path());
    assert_eq!(verdict.code, 0);
    assert!(
        verdict.text.contains("nothing provisioned at"),
        "{}",
        verdict.text
    );
}

/// A flat root that half exists says so in the listing too, in the material's
/// own words.
#[test]
fn a_half_provisioned_own_engine_says_so_in_the_listing() {
    let scratch = Scratch::new();
    let dir = scratch.dir(WIRE);
    mint::provisioned(&dir, "engine.example:9000");
    std::fs::remove_file(dir.join(ADDRESS)).expect("rm");
    let verdict = listing(scratch.path());
    assert_eq!(verdict.code, 0);
    assert!(
        verdict.text.contains("half-provisioned"),
        "{}",
        verdict.text
    );
}
