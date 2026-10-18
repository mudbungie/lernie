//! **The dial state on the glass** (bl-4cac): a channel that will not dial
//! says why on the seat's card, and one not heard from yet says that. The
//! rule counts no servers (§4.42, item 11): the dead one is the card's.

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

/// **First run, nothing provisioned**: the card says the sentence
/// `lernie entries` prints, whole at the desk and at the phone, instead of
/// `0 workspaces, 0 conversations` — and the rule does not claim a server.
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

/// **A server that went down**: the card carries the wire's sentence, the
/// rule counts it unreached in red, and its stale asking is not a green
/// count.
#[test]
fn a_dead_server_is_red_on_the_card_and_the_rule_and_its_count_is_not_green() {
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

/// **A server not heard from yet** says so on the card in weak ink, and
/// the rule counts it unreached without calling it broken.
#[test]
fn an_unheard_server_says_it_is_not_answered_yet() {
    let mut model = Model {
        conversation: None,
        aim: None,
        ..seated()
    };
    model.roster[0].held = Held::Unheard;
    model.roster[0].workspaces.clear();
    let seen = glass(&mut model, 1400.0, 900.0);
    let weak = said_in(&seen, theme::INK_WEAK);
    let expected = format!(
        "(this box's own server) — {}",
        crate::ui::roster::NOT_ANSWERED
    );
    assert!(weak.contains(&expected), "{weak:?}");
    assert!(!seen.iter().any(|run| run.text.contains("of 1 server")));
}
