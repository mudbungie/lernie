//! What an entry laid down here is, and what it refuses to overwrite.

use super::{FRESH, written};
use crate::channel::material::{self, ADDRESS, ANCHORS, CHAIN, KEY};
use crate::reply::enrolled::Enrolled;
use crate::test_support::Scratch;

/// Fabricated material, marked `notreal` throughout: the disclosure gate reads
/// every committed byte of this tree, and the key deliberately carries no
/// private-key banner.
fn minted() -> Enrolled {
    Enrolled {
        grade: "foot".to_owned(),
        name: "box-1".to_owned(),
        address: "engine.invalid:7737".to_owned(),
        ca: "-----BEGIN CERTIFICATE-----\nnotreal-ca\n-----END CERTIFICATE-----\n".to_owned(),
        cert: "-----BEGIN CERTIFICATE-----\nnotreal-leaf\n-----END CERTIFICATE-----\n".to_owned(),
        key: "-----BEGIN notreal KEY-----\nnotreal-key\n-----END notreal KEY-----\n".to_owned(),
    }
}

/// **What is written is what this crate reads back**, which is the whole point
/// of spending [`crate::channel::material`]'s own names: the entry laid down
/// here opens as a channel without a hand touching it.
#[test]
fn the_four_files_are_the_entry_this_seat_itself_reads() {
    let scratch = Scratch::new();
    let dir = scratch.path().join("carry");
    let said = written(&dir, &minted()).expect("the entry was filed");
    assert!(said.contains(&dir.display().to_string()), "{said}");

    let held = material::read_dir(&dir)
        .expect("it reads")
        .expect("it holds a channel");
    assert_eq!(held.address, "engine.invalid:7737");
    assert_eq!(held.anchors, dir.join(ANCHORS));
    assert_eq!(held.chain, dir.join(CHAIN));
    assert_eq!(held.key, dir.join(KEY));
    let material = minted();
    for (leaf, body) in [
        (ANCHORS, material.ca),
        (CHAIN, material.cert),
        (KEY, material.key),
    ] {
        assert_eq!(
            std::fs::read_to_string(dir.join(leaf)).expect(leaf),
            body,
            "{leaf}"
        );
    }
    // Exactly one trailing newline, on the one file that is not already PEM.
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect(ADDRESS),
        "engine.invalid:7737\n"
    );
}

/// **The key is created narrow**, not widened afterwards: there is no instant
/// in which it is readable by anyone else.
#[cfg(unix)]
#[test]
fn the_files_are_created_readable_by_nobody_else() {
    use std::os::unix::fs::PermissionsExt;

    let scratch = Scratch::new();
    let dir = scratch.path().join("carry");
    written(&dir, &minted()).expect("the entry was filed");
    for leaf in [ANCHORS, CHAIN, KEY, ADDRESS] {
        let mode = std::fs::metadata(dir.join(leaf))
            .expect(leaf)
            .permissions()
            .mode();
        assert_eq!(mode & 0o077, 0, "{leaf} is readable off this account");
    }
}

/// **A file already there is never written over.** Re-issuing distrusts
/// nothing, so a clobbered leaf leaves two live certificates under one identity
/// — and the refusal offers the act that costs nothing, because the enrollment
/// above it is already spent.
#[test]
fn a_directory_that_already_holds_material_refuses_and_names_the_remedy() {
    let scratch = Scratch::new();
    let dir = scratch.path().join("carry");
    written(&dir, &minted()).expect("the first one was filed");
    let refused = written(&dir, &minted()).expect_err("the second was refused");
    assert!(refused.contains(ANCHORS), "{refused}");
    assert!(refused.contains(FRESH), "{refused}");
    // And it left the first one exactly as it was.
    assert_eq!(
        std::fs::read_to_string(dir.join(ANCHORS)).expect(ANCHORS),
        minted().ca
    );
}

/// A destination that cannot be a directory at all is the other refusal, and it
/// names the path rather than the file it never reached.
#[test]
fn a_destination_that_is_not_a_directory_says_so() {
    let scratch = Scratch::new();
    let blocked = scratch.path().join("occupied");
    std::fs::write(&blocked, "notreal").expect("the blocker was written");
    let refused = written(&blocked.join("under"), &minted()).expect_err("it was refused");
    assert!(refused.contains("under"), "{refused}");
}
