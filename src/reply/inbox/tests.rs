//! The undelivered mail: a whole deposit, a forgiving one, and the strictness
//! that names a field.

use serde_json::json;

use super::row;

/// **A result message states all four frontmatter facts** — the two only a
/// result carries among them.
#[test]
fn a_result_message_carries_its_sender_its_stamp_and_how_its_agent_ended() {
    let read = row(&json!({
        "name": "user-001.md",
        "raw": "---\nfrom: user\n---\nhi",
        "deposit": {"from": "user", "deposited_at": "t0", "epitaph": "sideways",
                    "terminal_ref": "refs/x", "body": "hi"}
    }))
    .expect("a whole row reads");
    assert_eq!(read.name, "user-001.md");
    assert_eq!(read.raw, "---\nfrom: user\n---\nhi");
    assert_eq!(read.deposit.from.as_deref(), Some("user"));
    assert_eq!(read.deposit.deposited_at.as_deref(), Some("t0"));
    assert_eq!(read.deposit.terminal_ref.as_deref(), Some("refs/x"));
    // Rung 3: an epitaph this build has no word for is the engine's own
    // forward-compatible passthrough, and it stays that word here.
    assert_eq!(read.deposit.epitaph.as_deref(), Some("sideways"));
    assert_eq!(read.deposit.body, "hi");
}

/// **A hand-edited deposit states nothing but a body, and that is a reading.**
/// The parse is forgiving upstream, so every field is absent and the body is
/// whatever was in the file — never an error.
#[test]
fn a_deposit_whose_frontmatter_did_not_parse_states_only_a_body() {
    let read = row(&json!({
        "name": "raw.md", "raw": "bare", "deposit": {"body": ""}
    }))
    .expect("a forgiving row reads");
    assert_eq!(read.raw, "bare");
    assert!(read.deposit.from.is_none());
    assert!(read.deposit.deposited_at.is_none());
    assert!(read.deposit.epitaph.is_none());
    assert!(read.deposit.terminal_ref.is_none());
    assert_eq!(read.deposit.body, "", "a result whose agent never spoke");
}

/// Rung 1: a missing or mistyped field refuses, and names itself.
#[test]
fn a_malformed_row_refuses_and_names_the_field() {
    let why = row(&json!({"name": "x", "raw": "y"})).expect_err("a row with no deposit refuses");
    assert!(why.contains("deposit"), "{why}");
    let why = row(&json!({"name": "x", "raw": "y", "deposit": {}}))
        .expect_err("a deposit with no body refuses");
    assert!(why.contains("body"), "{why}");
    let why = row(&json!("a string")).expect_err("a row that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");
}
