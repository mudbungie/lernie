//! **When a turn runs** (DESIGN §4.45 item 5; bl-2101): read off the
//! transcript's shape, not off the open response — so the working stands
//! open while a tool executes, with the model's own prose whole in it until
//! the final prose stands.

use super::super::{RESPONSE, rows};
use super::idiom::{entry, glass, prose, result, running, with, working};
use super::line::head;
use crate::paint_probe::seen_of;
use crate::reply::transcript::{Entry, EntryKind};
use crate::test_support::window::said;
use crate::ui::Model;
use crate::ui::chat::Kind;
use crate::ui::theme::{Bubble, Speaker, glyph};

/// **Whether the last turn runs, as its rows say it**: every row of it live
/// and every model prose in it intermediate, or no row live. One query feeds
/// both bits, so a turn half live is a broken projection and fails here.
fn runs(entries: Vec<Entry>) -> bool {
    let shown = rows(&with(entries).transcript, None);
    let from = shown
        .iter()
        .rposition(|row| row.bubble == Bubble::Operator)
        .map_or(0, |i| i + 1);
    let turn = shown.get(from..).unwrap_or(&[]);
    let live = turn.iter().filter(|row| row.live).count();
    assert!(live == 0 || live == turn.len(), "{shown:?}");
    if live > 0 {
        assert!(
            turn.iter()
                .filter(|row| row.kind == Kind::Said && row.weight == Speaker::Model)
                .all(|row| row.bubble == Bubble::Intermediate),
            "{shown:?}"
        );
    }
    live > 0
}

/// **Three shapes run, and the rest are at rest**: a model entry that called
/// a tool and has no answer yet, a result the model has yet to answer, and
/// the open response — where a final prose, a wound, a compaction or a
/// deposit at the end is a turn that settled.
#[test]
fn the_turn_runs_until_its_final_prose_stands_or_it_is_wounded() {
    let turn = || vec![said("op", "port it"), prose("first I will read"), working()];
    assert!(runs(turn()), "a call unanswered");
    let mut landed = turn();
    landed.push(result("done", false));
    assert!(runs(landed.clone()), "a result unanswered");
    landed.push(running("so"));
    assert!(runs(landed.clone()), "the response open");
    landed.pop();
    landed.push(prose("the seam is real"));
    assert!(!runs(landed.clone()), "the final prose stands");
    for end in [
        entry(EntryKind::Wounded {
            wound: "refused".to_owned(),
            reason: None,
            auth_row: None,
        }),
        entry(EntryKind::Compacted {
            first: 1,
            last: 2,
            summary: String::new(),
        }),
        said("judge-one", "seconded"),
    ] {
        let mut ended = turn();
        ended.push(result("done", true));
        ended.push(end.clone());
        assert!(!runs(ended), "{:?}", end.kind);
    }
    assert!(!runs(Vec::new()), "nothing is not running");
}

/// What one settled frame put on the glass, as whole runs.
fn seen(model: &mut Model) -> Vec<String> {
    seen_of(&glass(model))
        .into_iter()
        .map(|run| run.text)
        .collect()
}

/// **While a tool executes the working stands open**, the call whole, the
/// finished reasoning compact — and the prose before them whole, *printed
/// directly*; and when its result lands with no response open yet, the same,
/// with the call compact the moment it returned. No `response` elbow either
/// way, since no answer stands.
#[test]
fn a_tool_in_flight_keeps_the_working_open_and_the_prose_before_it_whole() {
    let mut calling = with(vec![
        said("op", "port it"),
        prose("first I will read"),
        working(),
    ]);
    let flying = seen(&mut calling);
    assert!(flying.contains(&head(true, [1, 1, 1, 0])), "{flying:?}");
    assert!(
        flying.iter().any(|t| t == "first I will read"),
        "{flying:?}"
    );
    assert!(flying.contains(&glyph::CALL.to_owned()), "{flying:?}");
    let thought = format!(
        "{} model-a (thinking) {} weighing two seams",
        glyph::NOTE,
        glyph::NOTE
    );
    assert!(flying.contains(&thought), "{flying:?}");
    assert!(!flying.iter().any(|t| t == RESPONSE), "{flying:?}");

    let mut landed = with(vec![
        said("op", "port it"),
        prose("first I will read"),
        working(),
        result("done", false),
    ]);
    let waiting = seen(&mut landed);
    assert!(waiting.contains(&head(true, [1, 1, 1, 0])), "{waiting:?}");
    assert!(
        waiting.iter().any(|t| t == "first I will read"),
        "{waiting:?}"
    );
    let returned = format!("{} read(src/ui.rs)", glyph::DONE);
    assert!(waiting.contains(&returned), "{waiting:?}");
    assert!(!waiting.contains(&glyph::CALL.to_owned()), "{waiting:?}");
    assert!(!waiting.iter().any(|t| t == RESPONSE), "{waiting:?}");
}
