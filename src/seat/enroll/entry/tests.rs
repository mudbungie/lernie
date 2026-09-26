//! What an entry laid down here is, and what it refuses to overwrite.

use super::{FRESH, written};
use crate::channel::material::{self, ADDRESS, ANCHORS, CHAIN, KEY};
use crate::channel::rendezvous::{self, PUBLIC, SALT};
use crate::reply::enrolled::{Enrolled, Handoff};
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
        rendezvous: None,
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

    let held = material::read_dir(&dir, material::Whose::Elsewhere)
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

/// [`minted`] from an engine holding rendezvous material (REMOTE §8.4,
/// edition 20).
fn roving() -> Enrolled {
    Enrolled {
        rendezvous: Some(Handoff {
            public: "ab".repeat(32),
            salt: "cd".repeat(32),
        }),
        ..minted()
    }
}

/// The file names under `dir`, sorted.
fn names(dir: &std::path::Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .expect("the entry")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

/// **Six files with the pair, four without** — and the two extra are exactly
/// what this seat's own rendezvous reader opens, so the entry roves without a
/// hand touching it (bl-5378).
#[test]
fn the_pair_lands_as_the_two_files_the_rendezvous_reader_reads() {
    let scratch = Scratch::new();
    let plain = scratch.path().join("plain");
    written(&plain, &minted()).expect("filed");
    assert_eq!(names(&plain), [ADDRESS, ANCHORS, KEY, CHAIN]);
    assert_eq!(rendezvous::read_dir(&plain), Ok(None));

    let dir = scratch.path().join("roving");
    let said = written(&dir, &roving()).expect("filed");
    assert_eq!(names(&dir), [ADDRESS, ANCHORS, KEY, CHAIN, SALT, PUBLIC]);
    for named in [PUBLIC, SALT] {
        assert!(said.contains(named), "{named} unsaid: {said}");
    }
    let pairing = rendezvous::read_dir(&dir)
        .expect("it reads")
        .expect("it holds a pairing");
    assert_eq!(pairing.engine, [0xab; 32]);
    assert_eq!(pairing.salt, [0xcd; 32]);
    assert_eq!(
        std::fs::read_to_string(dir.join(PUBLIC)).expect(PUBLIC),
        format!("{}\n", "ab".repeat(32))
    );
}

/// **The modes the engine mints**: the public key is readable, the salt is
/// not — it is shared with the engine and is material.
#[cfg(unix)]
#[test]
fn the_public_key_is_open_and_the_salt_is_narrow() {
    use std::os::unix::fs::PermissionsExt;

    let scratch = Scratch::new();
    let dir = scratch.path().join("roving");
    written(&dir, &roving()).expect("filed");
    let mode = |leaf: &str| {
        std::fs::metadata(dir.join(leaf))
            .expect(leaf)
            .permissions()
            .mode()
    };
    assert_eq!(
        mode(SALT) & 0o077,
        0,
        "the salt is readable off this account"
    );
    assert_eq!(
        mode(PUBLIC) & 0o022,
        0,
        "the public key is writable by others"
    );
    assert_eq!(mode(PUBLIC) & 0o400, 0o400, "the public key is unreadable");
}
