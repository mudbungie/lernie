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

/// **The typed enumeration the window stamps its rows with**, and it dials
/// nothing: what a box holds is a fact about its own files, so a seat paints
/// its channels before any engine is up.
///
/// Each entry carries the name its workspace bears on its host, which is what
/// decides how a gesture aimed at one of its rows must be addressed
/// (`crate::ui::Channel::address`). The flat root carries none, because it
/// rewrites nothing.
#[test]
fn the_channels_are_the_own_engine_then_every_entry_in_order() {
    let scratch = Scratch::new();
    for (leaf, named) in [("zed", None), ("home", Some("personal"))] {
        let dir = scratch.path().join(entry(leaf));
        std::fs::create_dir_all(&dir).expect("mkdir");
        if let Some(named) = named {
            std::fs::write(dir.join(WORKSPACE), named).expect("the workspace file");
        }
    }
    let held = crate::seat::channels(scratch.path());
    assert_eq!(
        held.iter().map(|c| c.channel.clone()).collect::<Vec<_>>(),
        vec![
            crate::ui::Channel {
                name: crate::seat::OWN.to_owned(),
                named_there: None,
                dials: None,
            },
            crate::ui::Channel {
                name: "home".to_owned(),
                named_there: Some("personal".to_owned()),
                dials: None,
            },
            crate::ui::Channel {
                name: "zed".to_owned(),
                named_there: Some("zed".to_owned()),
                dials: None,
            },
        ]
    );
    // **Each arrives carrying why it has no walls yet** (bl-08b6): these three
    // are directories with no material in them, so the window's own first paint
    // says so rather than standing a section header over a blank.
    for chunk in &held {
        let crate::ui::Held::Unheld(why) = &chunk.held else {
            panic!("{} arrived claiming to have been heard", chunk.channel.name);
        };
        assert!(
            why.contains("provisioned") || why.contains("empty entry"),
            "{why}"
        );
    }
}

/// A box holding nothing still holds its own engine — the relationship it has
/// without naming it — so the roster is never empty of the one channel every
/// box has.
#[test]
fn a_box_with_no_entry_still_holds_its_own_engine() {
    let scratch = Scratch::new();
    let held = crate::seat::channels(scratch.path());
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].channel.named_there, None);
}
