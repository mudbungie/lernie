//! What the records pane says, in every state each half can be in — and the
//! sentences computed beside the paint, read as values.

use super::{
    CLOSE, EMPTY_WORKTREE, HEADING, NO_STEPS, NO_WORKTREE, NOT_ANSWERED_FILES, NOT_ANSWERED_STEPS,
    OPEN, TRUNCATED, auth, entry, headline, orphaned, previewed, provenance, render, wounded,
};
use crate::paint_probe::frame::Window;
use crate::reply::files::{FileRow, Files, Listing, Preview};
use crate::reply::steps::Steps;
use crate::test_support::window::{click, pane, recorded, seated, seen, step};
use crate::ui::{Model, theme};

/// A closed pane paints nothing and says so, which is what lets the shell put
/// the conversation back where it was.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **The answered pane paints the whole story**: the subject, both halves'
/// rows, the orphan banner, the wound, the sign-in, the cut listing and where
/// the work lands.
#[test]
fn the_answered_pane_paints_both_halves_whole() {
    let mut model = recorded();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "on 20260830T051200Z-a1b2",
        "001  complete — 99 tokens",
        "2026-08-30T05:12Z to 2026-08-30T05:14Z  at abcdef1",
        "002  failed — 99 tokens, 2 attempts",
        "an orphaned mail tail — driver died",
        "wound: refused — no bytes",
        "a sign-in is wanted on housevendor",
        "src/",
        "src/a.rs  12 B",
        TRUNCATED,
        "working in /home/u/elsewhere",
        CLOSE,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The empty states are different sentences.** A conversation nobody has
/// been answered about is not one whose loop did nothing, and a torn-down
/// worktree is not an empty one.
#[test]
fn every_empty_state_is_its_own_sentence() {
    let unanswered = Model {
        listing: Some(crate::ui::Listing::Records),
        ..seated()
    };
    let quiet = Model {
        records: crate::ui::Records {
            steps: Some(Steps {
                rows: Vec::new(),
                orphan: crate::reply::steps::NONE.to_owned(),
                orphan_reason: None,
            }),
            files: Some(Files {
                listing: None,
                preview: None,
                working_dir: None,
            }),
            ..unanswered.records.clone()
        },
        ..unanswered.clone()
    };
    let bare = Model {
        records: crate::ui::Records {
            files: Some(Files {
                listing: Some(Listing {
                    rows: Vec::new(),
                    truncated: false,
                }),
                preview: None,
                working_dir: None,
            }),
            ..quiet.records.clone()
        },
        ..quiet.clone()
    };
    // **The cases are boxed rather than laid out in an array**: a `Model` is
    // wide enough that five of them on the stack trip
    // `clippy::large_stack_arrays`, and the sentence each case asserts is what
    // this test is about, not where the value lives.
    let cases: Vec<(Box<Model>, &str)> = vec![
        (Box::new(unanswered.clone()), NOT_ANSWERED_STEPS),
        (Box::new(unanswered), NOT_ANSWERED_FILES),
        (Box::new(quiet.clone()), NO_STEPS),
        (Box::new(quiet), NO_WORKTREE),
        (Box::new(bare), EMPTY_WORKTREE),
    ];
    for (mut model, expected) in cases {
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(
            painted.lines().any(|line| line == expected),
            "{expected:?}:\n{painted}"
        );
    }
}

/// A preview rides under the listing when the answer carried one — every
/// class, the rung-3 word included, in the sentence [`previewed`] computes.
#[test]
fn a_preview_paints_in_each_of_its_four_classes() {
    assert_eq!(previewed(&Preview::Text("body".to_owned())), "body");
    assert_eq!(
        previewed(&Preview::Truncated {
            text: "head".to_owned(),
            size: 999
        }),
        "head\n… 999 bytes in all"
    );
    assert_eq!(previewed(&Preview::Binary { size: 4 }), "binary — 4 bytes");
    let said = previewed(&Preview::Unknown("hologram".to_owned()));
    assert!(said.contains("hologram"), "{said}");
    let mut model = recorded();
    if let Some(files) = model.records.files.as_mut() {
        files.preview = Some(Preview::Text("fn main() {}".to_owned()));
    }
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains("fn main() {}"), "{painted}");
}

/// The close control puts the pane down and nothing else.
#[test]
fn the_done_control_closes_the_pane() {
    let mut model = recorded();
    let window = Window::new();
    click(&window, CLOSE, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert!(!model.showing(crate::ui::Listing::Records));
    assert!(model.records.steps.is_some(), "the answers stay");
}

/// **The sentences, as values** — the quiet arms that paint nothing, and the
/// halves a full row does not exercise.
#[test]
fn the_quiet_rows_say_nothing_and_the_half_absences_read_right() {
    let quiet = step("001");
    assert_eq!(wounded(&quiet), None, "no wound, no badge");
    assert_eq!(auth(&quiet), None, "no sign-in wanted");
    let mut wordless = step("001");
    wordless.wound = "output_limit".to_owned();
    wordless.wound_reason = None;
    assert_eq!(wounded(&wordless), Some("wound: output_limit".to_owned()));
    let mut unrouted = step("001");
    unrouted.wound = crate::reply::steps::REFUSED.to_owned();
    assert_eq!(auth(&unrouted), Some("a sign-in is wanted".to_owned()));
    let mut half = step("001");
    half.started_at = None;
    assert_eq!(
        provenance(&half),
        Some("at abcdef1".to_owned()),
        "one timestamp is no span"
    );
    half.commit = None;
    assert_eq!(provenance(&half), None, "nothing recorded, no line");
    assert_eq!(headline(&step("001")), "001  complete — 99 tokens");
    let dirless = FileRow {
        path: "src".to_owned(),
        size: 0,
        dir: true,
    };
    assert_eq!(entry(&dirless), "src/");
    let unorphaned = Steps {
        rows: Vec::new(),
        orphan: crate::reply::steps::NONE.to_owned(),
        orphan_reason: None,
    };
    assert_eq!(orphaned(&unorphaned), None);
    let mute = Steps {
        orphan: "tool_window".to_owned(),
        ..unorphaned
    };
    assert_eq!(
        orphaned(&mute),
        Some("an orphaned tool_window tail".to_owned())
    );
}

/// **The open control leads the composer's second row**, tagged with the two
/// reads it stands up — and opening it is a model act, not a gesture.
#[test]
fn the_open_control_stands_the_pane_up_from_the_composer() {
    let mut model = seated();
    let window = Window::new();
    click(&window, OPEN, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::ui::composer::render(ui, &mut model);
        });
    });
    assert!(model.showing(crate::ui::Listing::Records), "the pane is up");
    assert!(model.outbox.is_empty(), "a look composes nothing");
}

/// **The pane's anatomy is on the glass** (`docs/STYLE.md` §5, *a covering
/// pane*): every half opens with its own word in weak ink, and the way out
/// stands at the end of the subject's own line rather than on a line of its
/// own.
///
/// It reads the runs and their ink rather than the layout, because the word
/// that divides two halves is a claim about hierarchy and hierarchy here is
/// spelled in ink: a section word in body ink would read as a fact.
#[test]
fn every_half_opens_with_its_word_in_weak_ink_and_the_close_rides_the_subject() {
    let mut model = recorded();
    let window = Window::new();
    let runs = seen(&window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    let read = |word: &str| {
        runs.iter()
            .find(|run| run.text == word)
            .unwrap_or_else(|| panic!("{word:?} is not on the glass"))
            .clone()
    };
    for word in [
        super::header::HEAD,
        super::STEPS_HEAD,
        super::FILES_HEAD,
        super::spine::GOVERNING_HEAD,
        super::spine::SPINE_HEAD,
        super::mail::HEAD,
    ] {
        assert_eq!(read(word).ink, theme::INK_WEAK, "{word:?}");
    }
    let subject = read("on 20260830T051200Z-a1b2");
    assert_eq!(
        subject.ink,
        theme::INK_WEAK,
        "the subject is one step weaker"
    );
    let close = read(CLOSE);
    assert!(
        (close.laid.min.y - subject.laid.min.y).abs() <= theme::ROW,
        "the close stands on the subject's line: {:?} against {:?}",
        close.laid,
        subject.laid
    );
}
