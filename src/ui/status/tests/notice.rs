//! **A notice longer than the rule** (bl-7e3e): it stands whole above the
//! rule at every width, and the rule keeps its one line and its name.

use super::super::READY;
use crate::paint_probe::frame::Window;
use crate::test_support::window::seated;
use crate::ui::{Model, Notice, theme};

/// The longest sentence a notice says, with its remedy at the end.
fn long() -> Notice {
    Notice::InDoubt {
        op: "message".to_owned(),
        why: "the connection closed before the reply".to_owned(),
    }
}

/// Collapse every run of whitespace, so a wrapped sentence reads back as the
/// one it was.
fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// **What two settled frames of the whole window put on the glass** at this
/// size: the bottom panels are laid where they stand on the frame after the
/// one that measured them.
fn glass(model: &mut Model, w: f32, h: f32) -> Vec<crate::paint_probe::Seen> {
    let window = Window::sized(w, h);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, model))
}

/// **At the desk and at the phone the whole sentence reaches the glass**,
/// uncut and inside the window, its remedy included — and the rule under it
/// still says where you are: the slot its calm word, the name at its end.
#[test]
fn a_long_notice_stands_whole_above_the_rule_at_every_width() {
    for (w, h) in [(1400.0, 900.0), (400.0, 800.0)] {
        let mut model = seated();
        model.notice = Some(long());
        let seen = glass(&mut model, w, h);
        let ink = theme::accent(theme::State::Error);
        let said: Vec<_> = seen.iter().filter(|run| run.ink == ink).collect();
        let read = words(&said.iter().map(|run| run.text.as_str()).collect::<String>());
        assert_eq!(read, words(&long().line()), "at {w}: the whole sentence");
        for run in &said {
            assert_eq!(run.shown, run.laid, "at {w}: {:?} is cut", run.text);
            assert!(run.laid.max.x <= w, "at {w}: {:?} runs off", run.text);
        }
        // The lowest run reading the name — cut at the phone, where the bar
        // takes its glyph column: the card at the head says it too.
        let rule = seen
            .iter()
            .filter(|run| run.text.starts_with("port t"))
            .max_by(|a, b| a.laid.min.y.total_cmp(&b.laid.min.y))
            .unwrap_or_else(|| panic!("at {w}: the rule's name is on the glass"));
        let lowest = said.iter().map(|run| run.laid.max.y).fold(0.0, f32::max);
        assert!(
            lowest <= rule.laid.min.y,
            "at {w}: the notice is above the rule"
        );
        // At the phone the diagnostics have dropped (item 11): the selectors
        // still say what you are talking to.
        assert!(
            seen.iter().any(|run| run.text.starts_with("pro")),
            "at {w}: the provider"
        );
        if w > 1000.0 {
            assert!(
                seen.iter().any(|run| run.text.contains("quiescent")),
                "at {w}: the slot says the calm state"
            );
            assert!(
                seen.iter().any(|run| run.text == super::super::NO_MODEL),
                "at {w}: the notice costs no fact"
            );
        }
    }
}

/// **A notice that fits beside every fact stays in the slot**, on the rule's
/// one line, and the slot says nothing calm beside it.
#[test]
fn a_short_notice_stays_in_the_slot() {
    let mut model = Model {
        notice: Some(Notice::Said("the act landed".to_owned())),
        ..Model::default()
    };
    let seen = glass(&mut model, 1400.0, 900.0);
    let note = seen
        .iter()
        .find(|run| run.text == "the act landed")
        .expect("the notice is on the glass");
    let name = seen
        .iter()
        .find(|run| run.text == super::super::ENGINES)
        .expect("the rule's name is on the glass");
    assert!((note.laid.center().y - name.laid.center().y).abs() < 2.0);
    assert!(!seen.iter().any(|run| run.text == READY));
}
