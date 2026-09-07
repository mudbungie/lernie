//! The turn in flight: the two halves of the prose, and the tool window under
//! them.

use super::{Stream, Window, streaming};

/// One call, as much of it as has been said.
fn call(exit_code: Option<i32>) -> Window {
    Window {
        tool_use: "toolu_01".to_owned(),
        tool: Some("box2_Bash".to_owned()),
        input: Some("{\"command\":\"uptime\"}".to_owned()),
        exit_code,
    }
}

/// **What is running, where, and what it came back with** (REMOTE §5.5). The
/// window is painted in the committed transcript's own shape — a row as the
/// call is dispatched, a row when its capture lands — so one call reads the
/// same while it happens and afterwards.
#[test]
fn the_window_paints_a_row_as_a_call_opens_and_another_when_it_lands() {
    let mut fold = Stream {
        text: Some("running it".to_owned()),
        tools: vec![call(None)],
        ..Stream::default()
    };
    let running = streaming(&fold);
    assert_eq!(running.len(), 2, "{running:?}");
    assert!(running[1].who.contains("box2_Bash"), "{:?}", running[1]);
    assert!(running[1].who.contains("toolu_01"), "{:?}", running[1]);
    assert_eq!(running[1].said, "{\"command\":\"uptime\"}");

    fold.tools = vec![call(Some(0))];
    let landed = streaming(&fold);
    assert_eq!(landed.len(), 3, "{landed:?}");
    assert_eq!(landed[2].said, "exit 0");
    assert!(landed[2].who.contains("returned"), "{:?}", landed[2]);

    fold.tools = vec![call(Some(2))];
    let failed = streaming(&fold);
    assert!(failed[2].who.contains("failed"), "{:?}", failed[2]);
    assert_eq!(failed[2].said, "exit 2");
}

/// **A call this fold never saw open is named by its id**, which is what the
/// frame carried — and a fold with no window at all paints no window rows,
/// where a placeholder would claim something ran.
#[test]
fn a_call_with_no_opening_half_is_named_by_its_invocation() {
    let bare = Stream {
        tools: vec![Window {
            tool_use: "toolu_09".to_owned(),
            tool: None,
            input: None,
            exit_code: Some(1),
        }],
        ..Stream::default()
    };
    let painted = streaming(&bare);
    assert_eq!(painted.len(), 2, "{painted:?}");
    assert!(painted[0].who.contains("toolu_09"), "{:?}", painted[0]);
    assert_eq!(painted[0].said, "");
    assert_eq!(streaming(&Stream::default()), Vec::new());
}
