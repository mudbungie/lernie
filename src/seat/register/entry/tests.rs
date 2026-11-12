//! The door's answer read, and the entry it becomes.

use super::{Claimed, file, read};
use crate::channel::entries;
use crate::channel::material::{ADDRESS, ANCHORS, CHAIN, KEY, Whose, read_dir};
use crate::test_support::Scratch;
use serde_json::{Value, json};

fn answer() -> Value {
    json!({
        "workspace": "home",
        "name": "phone",
        "grade": "operator",
        "address": ["engine.example:7737", "192.0.2.7:7737"],
        "ca": "-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n",
        "cert": "-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n",
    })
}

fn claimed() -> Claimed {
    read(answer().to_string().as_bytes()).expect("the answer reads")
}

/// REMOTE §15.1's six fields, the address as the list it is.
#[test]
fn the_answer_reads_as_its_six_fields() {
    let held = claimed();
    assert_eq!(held.workspace, "home");
    assert_eq!(held.name, "phone");
    assert_eq!(held.grade, "operator");
    assert_eq!(held.addresses, ["engine.example:7737", "192.0.2.7:7737"]);
    assert!(held.ca.contains("BEGIN CERTIFICATE"));
    assert!(held.cert.contains("BEGIN CERTIFICATE"));
}

/// A field that is absent, empty or of the wrong type is named; a body that
/// is not JSON names the first one it would have carried.
#[test]
fn an_answer_missing_a_field_names_it() {
    for key in ["workspace", "name", "grade", "address", "ca", "cert"] {
        for wrong in [Value::Null, json!(""), json!(7), json!([])] {
            let mut answer = answer();
            answer[key] = wrong;
            let refusal = read(answer.to_string().as_bytes()).expect_err(key);
            assert!(refusal.contains(&format!("no `{key}`")), "{refusal}");
        }
    }
    let refusal = read(b"<html>").expect_err("not JSON");
    assert!(refusal.contains("no `address`"), "{refusal}");
}

/// **A workspace name that is not one plain directory name files nothing** —
/// it came off the network and is about to be a path.
#[test]
fn a_workspace_name_that_is_not_one_path_component_is_refused() {
    for name in ["..", ".", "a/b", "/abs", "a/..", "home/"] {
        let mut answer = answer();
        answer["workspace"] = json!(name);
        let refusal = read(answer.to_string().as_bytes()).expect_err(name);
        assert!(refusal.contains("cannot be a directory name"), "{refusal}");
    }
}

/// **The entry is the four files `material` reads back**, the address one
/// endpoint per line, and every file readable by this account alone.
#[test]
fn the_entry_is_filed_as_the_four_files_and_reads_back() {
    let scratch = Scratch::new();
    let dir = file(scratch.path(), &claimed(), "the private key").expect("filed");
    assert_eq!(dir, entries::dir(scratch.path()).join("home"));
    let held = read_dir(&dir, Whose::Elsewhere)
        .expect("readable")
        .expect("provisioned");
    assert_eq!(held.addresses, ["engine.example:7737", "192.0.2.7:7737"]);
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect("address"),
        "engine.example:7737\n192.0.2.7:7737\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(KEY)).expect("key"),
        "the private key\n"
    );
    #[cfg(unix)]
    for leaf in [ANCHORS, CHAIN, KEY, ADDRESS] {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.join(leaf)).expect(leaf).permissions();
        assert_eq!(mode.mode() & 0o777, 0o600, "{leaf}");
    }
}

/// **An entry that exists is never written over**, and the refusal says the
/// offer is spent and what to do.
#[test]
fn an_existing_entry_directory_is_refused_and_left_alone() {
    let scratch = Scratch::new();
    let dir = entries::dir(scratch.path()).join("home");
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(dir.join(KEY), "the key that was here").expect("write");
    let refusal = file(scratch.path(), &claimed(), "a new key").expect_err("refused");
    assert!(refusal.contains("already holds an entry"), "{refusal}");
    assert!(refusal.contains("was not overwritten"), "{refusal}");
    assert!(refusal.contains("The offer is spent"), "{refusal}");
    assert_eq!(
        std::fs::read_to_string(dir.join(KEY)).expect("key"),
        "the key that was here"
    );
    assert!(!dir.join(ANCHORS).exists());
}

/// A data root that cannot hold the entries directory, and an entry
/// directory that cannot be made for another reason, each say the path.
#[test]
fn a_root_that_cannot_hold_the_entry_says_where() {
    let scratch = Scratch::new();
    std::fs::write(scratch.join("blocked"), "a file").expect("write");
    let refusal = file(&scratch.join("blocked"), &claimed(), "k").expect_err("refused");
    assert!(refusal.contains("blocked"), "{refusal}");
    // The entries directory exists but the name is longer than a file name
    // may be, so `create_dir` fails for a reason that is not "exists".
    let long = Claimed {
        workspace: "w".repeat(300),
        ..claimed()
    };
    let refusal = file(scratch.path(), &long, "k").expect_err("refused");
    assert!(!refusal.contains("already holds"), "{refusal}");
}

/// A file that cannot be created inside the entry says which.
#[test]
fn a_file_that_cannot_be_laid_names_its_path() {
    let scratch = Scratch::new();
    let path = scratch.join("absent").join(KEY);
    let refusal = super::laid(&path, "k").expect_err("no such directory");
    assert!(refusal.contains("absent"), "{refusal}");
}
