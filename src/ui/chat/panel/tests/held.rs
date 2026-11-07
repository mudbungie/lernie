//! **The dial state on the glass** (bl-4cac): a channel that will not dial
//! says why at the status bar's right end (DESIGN §4.48 item 4), and one not
//! heard from yet says that in the switcher. No line counts servers.

use crate::paint_probe::{Seen, frame::Window};
use crate::test_support::window::seated;
use crate::ui::{Held, Model, theme};

/// Collapse every run of whitespace, so a wrapped sentence reads back whole.
fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Two settled frames of the whole window at this size, read off the glass.
fn glass(model: &mut Model, w: f32, h: f32) -> Vec<Seen> {
    let window = Window::sized(w, h);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, model))
}

/// Every run in `ink`, read as one sentence.
fn said_in(seen: &[Seen], ink: egui::Color32) -> String {
    words(
        &seen
            .iter()
            .filter(|run| run.ink == ink)
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// **First run, nothing provisioned**: the status bar says the sentence
/// `lernie entries` prints, whole at the desk and at the phone — wrapped
/// above its line where it does not fit beside it — and never
/// `0 workspaces, 0 conversations`.
#[test]
fn an_unprovisioned_seat_says_why_and_claims_no_server() {
    let scratch = crate::test_support::Scratch::new();
    let roster = crate::seat::channels(scratch.path());
    let Some(Held::Unheld(why)) = roster.first().map(|chunk| chunk.held.clone()) else {
        panic!("an empty data root is unheld: {roster:?}");
    };
    for (w, h) in [(1400.0, 900.0), (400.0, 800.0)] {
        let mut model = Model {
            roster: roster.clone(),
            ..Model::default()
        };
        let seen = glass(&mut model, w, h);
        let error = theme::accent(theme::State::Error);
        let read = said_in(&seen, error);
        assert!(read.contains(&words(&why)), "at {w}: {read:?}");
        for run in seen.iter().filter(|run| run.ink == error) {
            assert_eq!(run.shown, run.laid, "at {w}: {:?} is cut", run.text);
        }
        let all = said_in(&seen, theme::INK_WEAK);
        assert!(!all.contains("0 workspaces"), "at {w}: {all:?}");
        assert!(!seen.iter().any(|run| run.text == "1 server"), "at {w}");
    }
}

/// **A server that went down**: the status bar carries the wire's sentence
/// in red, and its stale asking is not a green count.
#[test]
fn a_dead_server_is_red_on_the_status_bar_and_its_count_is_not_green() {
    let mut model = Model {
        conversation: None,
        aim: None,
        ..seated()
    };
    model.roster[0].workspaces[0].attention = 2;
    model.unreachable(
        &model.roster[0].channel.clone(),
        "connection refused".to_owned(),
    );
    let seen = glass(&mut model, 1400.0, 900.0);
    let read = said_in(&seen, theme::accent(theme::State::Error));
    assert!(
        read.contains("(this box's own server) — cannot reach this server: connection refused"),
        "{read:?}"
    );
    assert!(!seen.iter().any(|run| run.text.contains("waiting")));
}

/// **A server not heard from yet** says so in the switcher in weak ink, and
/// nothing calls it broken.
#[test]
fn an_unheard_server_says_it_is_not_answered_yet() {
    let mut model = Model {
        conversation: None,
        aim: None,
        ..seated()
    };
    model.roster[0].held = Held::Unheard;
    model.roster[0].workspaces.clear();
    let quiet = glass(&mut model, 1400.0, 900.0);
    let error = theme::accent(theme::State::Error);
    assert!(!quiet.iter().any(|run| run.ink == error), "{quiet:?}");
    model.switching = true;
    let seen = glass(&mut model, 1400.0, 900.0);
    let weak = said_in(&seen, theme::INK_WEAK);
    assert!(weak.contains(crate::ui::roster::NOT_ANSWERED), "{weak:?}");
    assert!(!seen.iter().any(|run| run.text.contains("of 1 server")));
}
