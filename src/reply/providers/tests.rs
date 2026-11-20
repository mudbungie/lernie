//! The provider table: the four required fields, the block whose absence is the
//! reading, and the offering that is a list of bare strings.

use super::{CATALOG, KIND, MODELS, Place, ProviderRow, offered, row};
use crate::reply::{Read, Reply, read};
use serde_json::{Value, json};

/// A row that can be signed in to, as the corpus carries one.
fn open() -> serde_json::Value {
    json!({
        "name": "housevendor",
        "fact": "credential present",
        "blocked": null,
        "effort": true,
        "priority": true,
    })
}

#[test]
fn a_row_carries_its_name_its_credential_fact_and_what_it_takes() {
    let read = row(&open()).expect("a provider row");
    assert_eq!(
        read,
        ProviderRow {
            name: "housevendor".to_owned(),
            fact: "credential present".to_owned(),
            blocked: None,
            effort: true,
            priority: true,
            on: None,
        }
    );
    assert!(read.signable());
    assert_eq!(read.takes(), Some("takes effort and priority".to_owned()));
}

/// **A block is a sentence, and its absence is the whole of *this row is
/// signable*.** The seat never composes either — the server knows which auth
/// model a row declares and this end does not.
#[test]
fn a_blocked_row_carries_the_servers_own_reason_and_is_not_signable() {
    let mut frame = open();
    frame["blocked"] = json!("no login flow");
    let read = row(&frame).expect("a blocked row");
    assert_eq!(read.blocked, Some("no login flow".to_owned()));
    assert!(!read.signable());
    let mut absent = open();
    absent.as_object_mut().expect("the row").remove("blocked");
    assert!(row(&absent).expect("an absent block").signable());
}

/// **The two capabilities are said in one line or in none**, because a row
/// that takes both is one fact rather than two badges to read together.
#[test]
fn what_a_row_takes_is_one_line_and_a_row_that_takes_neither_says_nothing() {
    let mut effort_only = open();
    effort_only["priority"] = json!(false);
    assert_eq!(
        row(&effort_only).expect("a row").takes(),
        Some("takes effort".to_owned())
    );
    let mut priority_only = open();
    priority_only["effort"] = json!(false);
    assert_eq!(
        row(&priority_only).expect("a row").takes(),
        Some("takes priority".to_owned())
    );
    let mut neither = open();
    neither["effort"] = json!(false);
    neither["priority"] = json!(false);
    assert_eq!(row(&neither).expect("a row").takes(), None);
}

/// **Rung 1 refuses by name**, and the name is the whole remedy.
#[test]
fn every_required_field_refuses_and_says_which_one() {
    for (field, why) in [
        ("name", "non-string"),
        ("fact", "non-string"),
        ("effort", "non-boolean"),
        ("priority", "non-boolean"),
    ] {
        let mut frame = open();
        frame[field] = json!(7);
        let said = row(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let mut wrong = open();
    wrong["blocked"] = json!(7);
    let said = row(&wrong).expect_err("a block of the wrong type still refuses");
    assert!(said.contains("blocked"), "{said}");
    let said = row(&json!("not an object")).expect_err("a row that is not one");
    assert!(said.contains("not an object"), "{said}");
}

/// One model id is a bare string, and anything else refuses.
#[test]
fn an_offering_is_a_list_of_bare_strings() {
    assert_eq!(
        offered(&json!("house-model-1")).expect("a model"),
        "house-model-1"
    );
    let said = offered(&json!({"name": "house-model-1"})).expect_err("an object is not one");
    assert!(said.contains("not a string"), "{said}");
}

/// Both frames, through the real door — and a listing that refuses on one row
/// refuses whole rather than shortening.
#[test]
fn the_frames_read_as_their_listings_and_one_bad_row_fails_it_all() {
    let frame = json!({"kind": KIND, "ok": true, "rows": [open()]});
    let Read::Answer(Reply::Providers(rows)) = read(&frame) else {
        panic!("a providers frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows.len(), 1);
    let frame = json!({"kind": CATALOG, "ok": true, "rows": [open()]});
    let Read::Answer(Reply::Catalog(catalog)) = read(&frame) else {
        panic!("a catalog frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(catalog, rows, "one row shape, one parser, two kinds");
    let frame = json!({"kind": MODELS, "ok": true, "rows": ["opus", "sonnet"]});
    let Read::Answer(Reply::Models(rows)) = read(&frame) else {
        panic!("a models frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows, vec!["opus".to_owned(), "sonnet".to_owned()]);
    let mut broken = open();
    broken["name"] = json!(null);
    let frame = json!({"kind": KIND, "ok": true, "rows": [open(), broken]});
    assert!(matches!(read(&frame), Read::Unreadable(_)), "one bad row");
}

/// **A row on another machine is one more row** (yog's `docs/REMOTE.md`
/// §5.7): `<row>@<client>` reads under both kinds with its name whole, and its
/// credential fact is that machine's.
#[test]
fn a_row_named_for_another_machine_keeps_its_whole_name() {
    let mut far = open();
    far["name"] = json!("housevendor@attic");
    far["fact"] = json!("auth oauth2 · signed in");
    for kind in [KIND, CATALOG] {
        let frame = json!({"kind": kind, "ok": true, "rows": [open(), far.clone()]});
        let Read::Answer(Reply::Providers(rows) | Reply::Catalog(rows)) = read(&frame) else {
            panic!("{kind} is an answer: {:?}", read(&frame));
        };
        assert_eq!(rows[1].name, "housevendor@attic", "{kind}");
        assert!(rows[1].signable() && !rows[1].bare(), "{kind}");
    }
}

/// **A row is bare where the server's words say it needs a credential and
/// holds none** (bl-7a77): both endings, and never a stored, signed-in or
/// keyless row, nor words this build cannot read.
#[test]
fn a_row_is_bare_only_where_the_server_says_it_holds_no_credential() {
    let said = |fact: &str| ProviderRow {
        fact: fact.to_owned(),
        ..row(&open()).expect("a provider row")
    };
    for fact in [
        "auth oauth2 · not signed in",
        "auth api_key · no credential stored",
    ] {
        assert!(said(fact).bare(), "{fact}");
    }
    for fact in [
        "auth oauth2 · signed in",
        "auth api_key · credential stored",
        "auth none · no credential needed",
        "credential present",
    ] {
        assert!(!said(fact).bare(), "{fact}");
    }
}

/// **Where a row lives is the server's statement** (REMOTE §5.7, edition 30;
/// bl-d927): `client` and `row` beside the name on a machine's row, read as
/// they are spelled — a client that carries the joiner included — and a row
/// stating neither, or only one, is the server's own. Neither is ever read
/// off the name, and a value of the wrong type refuses by its field.
#[test]
fn a_row_says_which_machine_it_is_on_and_the_name_is_not_read() {
    let with = |extra: Value| {
        let mut frame = open();
        frame
            .as_object_mut()
            .expect("a row")
            .extend(extra.as_object().expect("keys").clone());
        frame["name"] = json!("housevendor@loft@annex");
        frame
    };
    let far = row(&with(
        json!({ "client": "loft@annex", "row": "housevendor" }),
    ))
    .expect("a machine's row");
    assert_eq!(
        Place::machine(far.on.as_ref()).as_deref(),
        Some("loft@annex")
    );
    assert_eq!(Place::shown(far.on.as_ref(), &far.name), "housevendor");
    for own in [
        json!({}),
        json!({ "client": "annex" }),
        json!({ "row": "x" }),
    ] {
        let own = row(&with(own)).expect("the server's own row");
        assert_eq!(Place::machine(own.on.as_ref()), None);
        assert_eq!(
            Place::shown(own.on.as_ref(), &own.name),
            "housevendor@loft@annex"
        );
    }
    for (bad, field) in [
        (json!({ "client": 7, "row": "x" }), "client"),
        (json!({ "client": "c", "row": 7 }), "row"),
    ] {
        let why = row(&with(bad)).expect_err("a mistyped half");
        assert!(why.contains(field), "{why}");
    }
}
