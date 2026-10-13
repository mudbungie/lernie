//! The pairing and the head, as values.

use super::{Kind, Row, head, met};

/// **A call's head is its name and its input's first value**, one line,
/// elided where it was cut — and a bare name where there is no input.
#[test]
fn a_head_names_the_tool_and_the_head_of_its_input() {
    assert_eq!(head("bash", r#"{"command":"ls -la"}"#), r#"bash("ls -la")"#);
    assert_eq!(head("read", "src/ui.rs"), "read(src/ui.rs)");
    assert_eq!(head("n", "[1,2]"), "n([1,2])");
    assert_eq!(head("n", "{}"), "n()");
    assert_eq!(head("toolu_09", ""), "toolu_09");
    assert_eq!(head("p", "one\ntwo"), "p(one…)");
    let long = "x".repeat(200);
    assert_eq!(head("w", &long), format!("w({}…)", "x".repeat(60)));
}

/// **A result meets its call; one with no call stands alone**, and a call
/// with nothing back yet stays a call.
#[test]
fn a_result_meets_its_call_and_the_unmet_stand_as_they_are() {
    let rows = met(vec![
        Row::call("bash", "", "a"),
        Row::call("read", "", "b"),
        Row::result("a", "done".to_owned(), true, None),
        Row::result("z", "orphan".to_owned(), false, None),
    ]);
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert_eq!(rows[0].kind, Kind::Answered);
    assert_eq!(rows[0].said, "done");
    assert!(rows[0].failed);
    assert_eq!(rows[1].kind, Kind::Call);
    assert_eq!(rows[2].kind, Kind::Result);
    assert_eq!(rows[2].who, "returned");
}
