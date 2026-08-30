//! What a box holds, and what a directory that is not quite an entry says.

use super::{ENTRIES, Entry, WIRE, WORKSPACE, dir, flat, read_dir};
use crate::channel::material::{ADDRESS, ANCHORS, CHAIN, KEY};
use crate::test_support::{Scratch, mint};
use std::path::Path;

/// Write every file a provisioned entry holds.
fn provision(dir: &Path, address: &str) {
    std::fs::create_dir_all(dir).expect("mkdir");
    for file in [ANCHORS, CHAIN, KEY] {
        std::fs::write(dir.join(file), b"pem bytes stand in here").expect("write");
    }
    std::fs::write(dir.join(ADDRESS), address).expect("write");
}

/// The two paths are the shape REMOTE §8.2 fixes, and the entries directory is
/// one level under the flat root rather than beside it.
#[test]
fn the_paths_are_the_wire_s_own_and_the_entries_sit_under_the_flat_root() {
    let root = Path::new("/data/lernie");
    assert_eq!(flat(root), root.join(WIRE));
    assert_eq!(dir(root), root.join(WIRE).join(ENTRIES));
}

/// **Absent, unreadable and empty are one fact**: this box holds no workspace
/// elsewhere, which is the shape every box had before §8.2 existed.
#[test]
fn a_directory_that_will_not_read_is_zero_entries_and_not_a_refusal() {
    let scratch = Scratch::new();
    assert_eq!(read_dir(&scratch.join("never-made")), Vec::new());
    assert_eq!(read_dir(&scratch.dir("empty")), Vec::new());
}

/// A provisioned entry answers its leaf, its host-side name and its material.
/// With no `workspace` file the leaf IS the name.
#[test]
fn a_provisioned_entry_answers_its_leaf_as_its_own_host_name() {
    let scratch = Scratch::new();
    let entries = scratch.dir("workspaces");
    provision(&entries.join("home"), "engine.example:9000");
    let held = read_dir(&entries);
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].leaf, "home");
    assert_eq!(held[0].workspace, "home");
    assert_eq!(
        held[0].channel.as_ref().expect("provisioned").address,
        "engine.example:9000"
    );
}

/// **The fifth file is the host's name for it.** Absent, blank and whitespace
/// are one branch — the entry states no host-side name, and the leaf is then
/// the name.
#[test]
fn the_workspace_file_names_it_on_its_host_and_says_nothing_when_blank() {
    let scratch = Scratch::new();
    let entries = scratch.dir("workspaces");
    for (leaf, stated, expected) in [
        ("renamed", "personal\n", "personal"),
        ("blank", "  \n", "blank"),
    ] {
        let path = entries.join(leaf);
        provision(&path, "engine.example:9000");
        std::fs::write(path.join(WORKSPACE), stated).expect("write");
        let held = read_dir(&entries);
        let found = held.iter().find(|e| e.leaf == leaf).expect("listed");
        assert_eq!(found.workspace, expected);
    }
}

/// Entries are sorted by leaf, and a stray FILE beside them names no intent —
/// it is not an entry with a problem.
#[test]
fn entries_are_sorted_and_a_stray_file_is_not_one() {
    let scratch = Scratch::new();
    let entries = scratch.dir("workspaces");
    for leaf in ["zulu", "alpha"] {
        provision(&entries.join(leaf), "engine.example:9000");
    }
    std::fs::write(entries.join("README"), b"not an entry").expect("write");
    let leaves: Vec<String> = read_dir(&entries).into_iter().map(|e| e.leaf).collect();
    assert_eq!(leaves, ["alpha", "zulu"]);
}

/// **A refusal is one entry's, never the set's.** An empty directory somebody
/// made names an intent with no material behind it, and it says so beside a
/// neighbour that is fine.
#[test]
fn a_refusing_entry_stands_beside_the_ones_that_are_fine() {
    let scratch = Scratch::new();
    let entries = scratch.dir("workspaces");
    provision(&entries.join("good"), "engine.example:9000");
    std::fs::create_dir_all(entries.join("hollow")).expect("mkdir");
    let half = entries.join("half");
    provision(&half, "engine.example:9000");
    std::fs::remove_file(half.join(KEY)).expect("rm");

    let held = read_dir(&entries);
    let said = |leaf: &str| -> String {
        held.iter()
            .find(|e| e.leaf == leaf)
            .expect("listed")
            .channel
            .clone()
            .expect_err("refused")
    };
    assert!(
        said("hollow").contains("is an empty entry"),
        "{}",
        said("hollow")
    );
    assert!(
        said("half").contains("half-provisioned"),
        "{}",
        said("half")
    );
    assert!(
        held.iter()
            .find(|e| e.leaf == "good")
            .expect("listed")
            .channel
            .is_ok(),
        "one entry's refusal cost its neighbour"
    );
}

/// **An entry that cannot read its material cannot be opened**, and the
/// sentence is the entry's own rather than a transport error.
#[test]
fn opening_a_refusing_entry_answers_the_entry_s_own_sentence() {
    let held = Entry {
        leaf: "hollow".to_owned(),
        workspace: "hollow".to_owned(),
        channel: Err("that entry has nothing in it".to_owned()),
    };
    assert_eq!(
        held.open().expect_err("refused"),
        "that entry has nothing in it"
    );
}

/// A provisioned entry opens onto the address it names, dialling nothing.
#[test]
fn a_provisioned_entry_opens_onto_its_own_address() {
    let scratch = Scratch::new();
    let entries = scratch.dir("workspaces");
    let path = entries.join("home");
    let held = mint::provisioned(&path, "engine.example:9000");
    assert_eq!(held.address, "engine.example:9000");
    let entry = read_dir(&entries).into_iter().next().expect("listed");
    assert_eq!(
        entry.open().expect("opened").address(),
        "engine.example:9000"
    );
}
