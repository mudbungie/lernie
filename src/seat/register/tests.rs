//! The whole act against a local door and a local wire: a key made here, a
//! leaf the far end signs over it, an entry filed, and the dial that proves.

use std::path::Path;

use super::{claim, key, register};
use crate::channel::entries;
use crate::channel::hello::PROTOCOL;
use crate::channel::material::{ADDRESS, ANCHORS, CHAIN, KEY};
use crate::test_support::door::{Door, NOT_FOUND, ok};
use crate::test_support::server::Server;
use crate::test_support::{Scratch, mint};
use rustls::pki_types::SubjectPublicKeyInfoDer;
use rustls::pki_types::pem::PemObject;
use serde_json::{Value, json};

/// The public key a claim's request carried.
fn posted(request: &str) -> String {
    let body = request.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    let body: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    body["public_key"].as_str().unwrap_or_default().to_owned()
}

/// A far end in `realm`: a door that signs a leaf named `phone` over whatever
/// key it is sent, and answers the entry for workspace `home` at `address`.
fn signing_door(realm: &Path, address: Vec<String>) -> Door {
    let dir = realm.to_path_buf();
    Door::start(realm, move |request| {
        ok(&json!({
            "workspace": "home",
            "name": "phone",
            "grade": "operator",
            "address": address,
            "ca": std::fs::read_to_string(dir.join(ANCHORS)).expect("the CA"),
            "cert": mint::signed(&dir, "phone", &posted(request)),
        }))
    })
}

/// **The whole of REMOTE §15.1 steps 2 to 4, from this box's side.** The key
/// in the entry is the one the leaf was signed over — the wire's own mTLS
/// accepts the pair — the word is the one the server would compute from what
/// it was sent, and the held refusal is said as waiting, not as a fault.
#[test]
fn a_claimed_offer_is_filed_shown_and_proved_on_the_wire() {
    let (realm, root) = (Scratch::new(), Scratch::new());
    mint::material(realm.path());
    let held = json!({"ok": false, "error": "this client is held until its offer is approved"});
    let wire = Server::start(realm.path(), PROTOCOL, vec![vec![held]]);
    let address = std::fs::read_to_string(realm.join(ADDRESS)).expect("the wire's address");
    let door = signing_door(realm.path(), vec![address.clone()]);

    let registered = claim(
        root.path(),
        &door.url("ABCD2345"),
        Door::roots(realm.path()),
    )
    .expect("claimed");

    let dir = entries::dir(root.path()).join("home");
    assert_eq!(registered.entry, dir);
    assert_eq!(registered.addresses, [address]);
    assert_eq!(registered.proved, Ok(()));
    for leaf in [ANCHORS, CHAIN, KEY, ADDRESS] {
        assert!(dir.join(leaf).is_file(), "{leaf}");
    }
    let key_file = std::fs::read_to_string(dir.join(KEY)).expect("the key");
    // Spelled in two halves: the whole armour line is what the leak gate
    // reads as a committed key.
    assert!(key_file.starts_with(&format!("-----BEGIN {} KEY-----\n", "PRIVATE")));
    // The word, recomputed the way the server does: from the key it was SENT.
    let sent = posted(&door.heard());
    let spki = SubjectPublicKeyInfoDer::from_pem_slice(sent.as_bytes()).expect("a public key");
    let ca = std::fs::read_to_string(realm.join(ANCHORS)).expect("the CA");
    assert_eq!(Ok(registered.word.clone()), key::word(spki.as_ref(), &ca));
    assert_eq!(registered.word.len(), 6);
    assert!(!sent.contains("PRIVATE"), "only the public half travels");
    // The wire heard this box: its preface, then the one request.
    assert_eq!(wire.heard().get(1), Some(&json!({"op": "workspaces"})));
    assert_eq!(
        registered.said(),
        format!(
            "Registered as phone (operator) in workspace home. The entry is filed at {}.\n\
             Confirmation word: {}\n\
             On a seat that is already connected, approve the offer whose row shows that word.\n\
             Waiting for approval: the server has heard this box and will refuse it until the \
             offer is approved.",
            dir.display(),
            registered.word
        )
    );
}

/// **Every non-200 is one sentence**: the offer cannot be claimed, and
/// nothing is filed.
#[test]
fn a_refused_claim_says_the_offer_cannot_be_claimed_and_files_nothing() {
    let (realm, root) = (Scratch::new(), Scratch::new());
    mint::material(realm.path());
    let door = Door::start(realm.path(), |_| NOT_FOUND.to_vec());
    let url = door.url("ABCD2345");
    let refusal = claim(root.path(), &url, Door::roots(realm.path())).expect_err("refused");
    assert_eq!(
        refusal,
        format!(
            "this offer cannot be claimed: {} answered 404. The code is unknown, expired or \
             already used — a registration URL works once. Ask for a new offer.",
            url.trim_start_matches("https://")
                .trim_end_matches("/yog/register/ABCD2345")
        )
    );
    // The command line says the same sentence, as a failure.
    let verdict = register(root.path(), &url, Door::roots(realm.path()));
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.text, format!("lernie: {refusal}"));
    assert!(!entries::dir(root.path()).exists(), "nothing was filed");
}

/// **A wire that could not be reached is said, with the entry still filed** —
/// the registration stands and the next dial proves it.
#[test]
fn a_wire_that_cannot_be_reached_leaves_the_entry_and_says_what_is_owed() {
    let (realm, root) = (Scratch::new(), Scratch::new());
    mint::material(realm.path());
    let door = signing_door(realm.path(), vec!["127.0.0.1:1".to_owned()]);
    let verdict = register(
        root.path(),
        &door.url("ABCD2345"),
        Door::roots(realm.path()),
    );
    assert_eq!(verdict.code, 0, "the registration itself succeeded");
    assert!(
        verdict
            .text
            .contains("The server has not heard this box yet"),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("connect 127.0.0.1:1"),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("Confirmation word: "),
        "{}",
        verdict.text
    );
    assert!(entries::dir(root.path()).join("home").join(KEY).is_file());
}

/// **An entry that exists is not overwritten**, through the whole act.
#[test]
fn an_existing_entry_is_refused_through_the_whole_act() {
    let (realm, root) = (Scratch::new(), Scratch::new());
    mint::material(realm.path());
    let dir = entries::dir(root.path()).join("home");
    std::fs::create_dir_all(&dir).expect("mkdir");
    let door = signing_door(realm.path(), vec!["127.0.0.1:1".to_owned()]);
    let refusal = claim(
        root.path(),
        &door.url("ABCD2345"),
        Door::roots(realm.path()),
    )
    .expect_err("refused");
    assert!(refusal.contains("already holds an entry"), "{refusal}");
    assert_eq!(std::fs::read_dir(&dir).expect("the entry").count(), 0);
}

/// What is not a registration URL, an answer that is not the six fields, and
/// a `ca` with no certificate in it each refuse with their own sentence.
#[test]
fn every_other_fault_is_its_own_sentence() {
    let (realm, root) = (Scratch::new(), Scratch::new());
    mint::material(realm.path());
    let roots = || Door::roots(realm.path());
    let refusal = claim(root.path(), "engine.example", roots()).expect_err("no URL");
    assert!(refusal.contains("is not a registration URL"), "{refusal}");

    let door = Door::start(realm.path(), |_| ok(&json!({"workspace": "home"})));
    let refusal = claim(root.path(), &door.url("ABCD2345"), roots()).expect_err("short");
    assert!(refusal.contains("carried no `address`"), "{refusal}");

    let door = Door::start(realm.path(), |_| {
        ok(&json!({
            "workspace": "home", "name": "phone", "grade": "foot",
            "address": ["127.0.0.1:1"], "ca": "no certificate", "cert": "none",
        }))
    });
    let refusal = claim(root.path(), &door.url("ABCD2345"), roots()).expect_err("no CA");
    assert!(refusal.contains("no CA certificate"), "{refusal}");
    assert!(!entries::dir(root.path()).exists(), "nothing was filed");
}
