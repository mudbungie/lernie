//! The three answers a directory can give.

use super::{ADDRESS, ANCHORS, CHAIN, KEY, Whose, read_dir};
use crate::test_support::Scratch;
use std::path::Path;

/// Write every file a provisioned channel holds, with `address` as stated.
fn provision(dir: &Path, address: &str) {
    for file in [ANCHORS, CHAIN, KEY] {
        std::fs::write(dir.join(file), b"pem bytes stand in here").expect("write");
    }
    std::fs::write(dir.join(ADDRESS), address).expect("write");
}

/// An empty directory holds no channel, and that is an answer. Removing the
/// material deletes config, not code.
#[test]
fn nothing_provisioned_is_an_answer_and_not_an_error() {
    let tmp = Scratch::new();
    assert_eq!(read_dir(tmp.path(), Whose::Elsewhere), Ok(None));
}

/// A directory that does not exist reads exactly as an empty one: absent and
/// empty are one fact about what this box holds.
#[test]
fn a_directory_that_is_not_there_reads_as_nothing_provisioned() {
    let tmp = Scratch::new();
    assert_eq!(
        read_dir(&tmp.path().join("never-made"), Whose::Elsewhere),
        Ok(None)
    );
}

/// Everything present is the material, with the address trimmed.
#[test]
fn a_provisioned_directory_answers_its_material() {
    let tmp = Scratch::new();
    provision(tmp.path(), "  server.example:9000\n");
    let held = read_dir(tmp.path(), Whose::Elsewhere)
        .expect("readable")
        .expect("provisioned");
    assert_eq!(held.addresses, ["server.example:9000"]);
    assert_eq!(held.anchors, tmp.path().join(ANCHORS));
    assert_eq!(held.chain, tmp.path().join(CHAIN));
    assert_eq!(held.key, tmp.path().join(KEY));
}

/// Half a trust store refuses, and names EVERY gap at once — a remedy that
/// reveals one missing file per run is a remedy run four times.
#[test]
fn a_half_provisioned_directory_names_every_gap_at_once() {
    let tmp = Scratch::new();
    provision(tmp.path(), "server.example:9000");
    std::fs::remove_file(tmp.path().join(KEY)).expect("rm");
    std::fs::remove_file(tmp.path().join(ADDRESS)).expect("rm");
    let refusal = read_dir(tmp.path(), Whose::Elsewhere).expect_err("refused");
    assert!(refusal.contains(KEY), "{refusal}");
    assert!(refusal.contains(ADDRESS), "{refusal}");
    assert!(refusal.contains("half-provisioned"), "{refusal}");
}

/// The remedy names both roads: the offer this box can claim for itself, and
/// the leaf an operator mints on the host and carries here.
#[test]
fn every_refusal_names_registering_and_the_road_by_hand() {
    let tmp = Scratch::new();
    provision(tmp.path(), "server.example:9000");
    std::fs::remove_file(tmp.path().join(ANCHORS)).expect("rm");
    let refusal = read_dir(tmp.path(), Whose::Elsewhere).expect_err("refused");
    assert!(refusal.contains("`lernie register <url>`"), "{refusal}");
    assert!(refusal.contains("By hand instead"), "{refusal}");
    // **The assignment goes before the verb** (bl-5cbe). `WIRE_LEAF` is an
    // environment variable, not an argument `yog wire-certs` parses, so the
    // spelling this line printed for a year did not work when it was typed —
    // and this is the first line a new seat prints, to somebody who by
    // construction has not provisioned a channel before.
    assert!(
        refusal.contains("`WIRE_LEAF=<name> yog wire-certs`"),
        "{refusal}"
    );
    // **And it names what to carry.** Four files, by name.
    for named in [ANCHORS, CHAIN, KEY, ADDRESS] {
        assert!(refusal.contains(named), "{named}: {refusal}");
    }
}

/// An address file that is empty, whitespace, or unreadable is one refusal:
/// they are one fact about what this box can be told to dial.
#[test]
fn an_address_that_says_nothing_refuses_however_it_says_it() {
    for stated in ["", "   \n"] {
        let tmp = Scratch::new();
        provision(tmp.path(), stated);
        let refusal = read_dir(tmp.path(), Whose::Elsewhere).expect_err("refused");
        assert!(refusal.contains("names no address"), "{refusal}");
    }
    // Unreadable takes the same branch: a directory where the file goes.
    let tmp = Scratch::new();
    provision(tmp.path(), "server.example:9000");
    std::fs::remove_file(tmp.path().join(ADDRESS)).expect("rm");
    std::fs::create_dir(tmp.path().join(ADDRESS)).expect("mkdir");
    let refusal = read_dir(tmp.path(), Whose::Elsewhere).expect_err("refused");
    assert!(refusal.contains("half-provisioned"), "{refusal}");
}

/// **The address file is a list, in the order stated** (REMOTE §15.3): one
/// endpoint per line, a v6 host bracketed, blank lines nothing.
#[test]
fn an_address_file_of_several_lines_is_dialled_in_the_order_it_states() {
    let tmp = Scratch::new();
    provision(
        tmp.path(),
        "engine.example:7737\n\n  192.0.2.7:7737\n[2001:db8::7]:7737\n",
    );
    let held = read_dir(tmp.path(), Whose::Elsewhere)
        .expect("readable")
        .expect("provisioned");
    assert_eq!(
        held.addresses,
        [
            "engine.example:7737",
            "192.0.2.7:7737",
            "[2001:db8::7]:7737"
        ]
    );
    assert_eq!(
        held.address(),
        "engine.example:7737, 192.0.2.7:7737, [2001:db8::7]:7737"
    );
}

/// **An entry an older server filed still works** (REMOTE §15.4): the two
/// files the retired rendezvous left behind are neither read nor required,
/// whatever they hold.
#[test]
fn the_retired_rendezvous_files_are_ignored_where_an_entry_still_has_them() {
    let tmp = Scratch::new();
    provision(tmp.path(), "server.example:9000");
    let bare = read_dir(tmp.path(), Whose::Elsewhere);
    std::fs::write(tmp.path().join("rendezvous.pub"), "not a key").expect("write");
    std::fs::write(tmp.path().join("pairing.salt"), "").expect("write");
    assert_eq!(read_dir(tmp.path(), Whose::Elsewhere), bare);
    assert!(bare.expect("readable").is_some());
}
