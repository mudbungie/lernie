//! What the drill-in paints, and what its control does — including the two
//! record classes that have no bytes to show.

use super::{
    CLOSE, ERRORED, NO_BYTES, NO_EVENTS, NO_TOOLS, OPEN, bytes, control, framing, headline, records,
};
use crate::paint_probe::frame::Window;
use crate::reply::step::{Doc, Step, ToolCall};
use crate::test_support::window::{click, drilled, pane, recorded, seated};
use crate::ui::Model;

/// **The control on a row asks about that row**, and the answer paints under
/// it — driven through the real glass so the click lands on the seat.
#[test]
fn the_control_asks_for_the_row_it_hangs_on_and_then_shows_it() {
    let window = Window::new();
    let mut model = recorded();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.outbox.len(),
        1,
        "one row's control fired one read: {:?}",
        model.outbox
    );
    assert_eq!(model.outbox[0].envelope["seq"], "001");
    assert!(!model.outbox[0].act, "a drill-in only looks");
}

/// **The answered drill-in paints every class of record**, each with the
/// engine's own framing where it wrote one.
#[test]
fn the_answered_drill_in_paints_the_records_the_stream_and_the_tool_call() {
    let mut model = Model {
        records: crate::ui::Records {
            drilled: Some(drilled("001")),
            ..recorded().records
        },
        ..recorded()
    };
    let text = pane(|ui| {
        control(ui, &mut model, "001");
        records(ui, &model, "001");
    });
    for word in [
        CLOSE,
        "{\"commit\":\"abcdef1\"}",
        NO_BYTES,
        crate::reply::step::UNPARSED,
        "not json",
        "a \"sideways\" record, which this seat cannot show",
        "toolu_1 — ended in error",
        "the adapter's last words",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **Closing it is a second act on the same control**, and it takes nothing
/// across the wire: the records are already here.
#[test]
fn the_control_puts_the_records_away_without_asking_anything() {
    let mut model = Model {
        records: crate::ui::Records {
            drilled: Some(drilled("001")),
            ..recorded().records
        },
        ..recorded()
    };
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.records.drilled.is_none());
    assert!(model.outbox.is_empty(), "putting them away asks nothing");
}

/// **A step whose stream landed nothing and which called no tool says both**,
/// which are two different claims from a record with no bytes.
#[test]
fn a_step_that_did_nothing_says_so_in_two_sentences() {
    let quiet = Step {
        response: Vec::new(),
        tools: Vec::new(),
        stderr: None,
        ..drilled("001")
    };
    let model = Model {
        records: crate::ui::Records {
            drilled: Some(quiet),
            ..recorded().records
        },
        ..recorded()
    };
    let text = pane(|ui| records(ui, &model, "001"));
    assert!(text.contains(NO_EVENTS), "{text}");
    assert!(text.contains(NO_TOOLS), "{text}");
}

/// The lines are pure functions of the record, so the suite reads the sentence
/// rather than the layout.
#[test]
fn the_lines_are_values_and_two_classes_have_no_bytes() {
    assert_eq!(
        headline(&ToolCall {
            tool_id: "toolu_2".to_owned(),
            is_error: false,
            input: Doc::Absent,
            output: Doc::Absent,
        }),
        "toolu_2"
    );
    assert_eq!(
        headline(&drilled("001").tools[0]),
        format!("toolu_1 — {ERRORED}")
    );
    assert_eq!(framing(&Doc::Absent), None);
    assert_eq!(
        framing(&Doc::Json {
            raw: "{}".to_owned()
        }),
        None
    );
    assert_eq!(bytes(&Doc::Absent), "");
    assert_eq!(bytes(&Doc::Unknown("sideways".to_owned())), "");
}

/// A pane with nothing drilled into paints only the control, on every row.
#[test]
fn an_unasked_row_paints_only_its_control() {
    let mut model = seated();
    let text = pane(|ui| {
        control(ui, &mut model, "001");
        records(ui, &model, "001");
    });
    assert!(text.contains(OPEN), "{text}");
    assert!(!text.contains(NO_BYTES), "{text}");
}
