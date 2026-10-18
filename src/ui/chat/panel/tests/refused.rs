//! **The roster read refused** (bl-693d): a fresh server answers a fresh seat
//! *registered in no workspace*, which is an answer — so the card and the
//! switcher say it under the server, the bar does not, and the roster answer
//! that follows a founding takes it off the glass with no Esc.

use serde_json::json;

use crate::paint_probe::{Seen, frame::Window};
use crate::reply::Read;
use crate::test_support::window::seated;
use crate::ui::{Held, Model, roster, theme};

/// What the server says to a client registered nowhere, in short.
const NOWHERE: &str = "this certificate is registered in no workspace on this server";

/// Collapse every run of whitespace, so a wrapped sentence reads back whole.
fn words(seen: &[Seen], ink: egui::Color32) -> String {
    seen.iter()
        .filter(|run| run.ink == ink)
        .flat_map(|run| run.text.split_whitespace())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Two settled frames of the whole window, read off the glass.
fn glass(model: &mut Model) -> Vec<Seen> {
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, model))
}

/// A seat whose one server has not answered yet, its roster refused.
fn refused() -> Model {
    let mut model = Model {
        conversation: None,
        aim: None,
        ..seated()
    };
    model.roster[0].held = Held::Unheard;
    model.roster[0].workspaces.clear();
    let channel = model.roster[0].channel.clone();
    model.receipt(&channel, "workspaces", Read::Refusal(NOWHERE.to_owned()));
    model
}

/// **On the card, in the bar's words, and not in the bar**: the server is
/// not 'waiting to hear' — it answered.
#[test]
fn a_refused_roster_is_said_on_the_card_and_not_in_the_bar() {
    let mut model = refused();
    assert_eq!(model.notice, None, "the bar is not the channel's");
    let seen = glass(&mut model);
    let red = words(&seen, theme::accent(theme::State::Error));
    let expected = format!("(this box's own server) — the server refused: {NOWHERE}");
    assert!(red.contains(&expected), "{red:?}");
    assert!(
        !seen
            .iter()
            .any(|run| run.text.contains(roster::NOT_ANSWERED))
    );
}

/// **Under the server's row in the switcher, with the founding beside it**:
/// `+ new workspace` is the remedy the refusal names.
#[test]
fn the_switcher_says_the_refusal_and_still_offers_a_founding() {
    let mut model = refused();
    model.switching = true;
    let seen = glass(&mut model);
    let red = words(&seen, theme::accent(theme::State::Error));
    assert!(
        red.contains(&format!("the server refused: {NOWHERE}")),
        "{red:?}"
    );
    assert!(seen.iter().any(|run| run.text == roster::server::FOUND));
    assert!(
        !seen
            .iter()
            .any(|run| run.text.contains(roster::NOT_ANSWERED))
    );
}

/// **The next roster answer spends it**: once a founding registers the
/// client, the refusal is off the glass on the beat the workspaces land.
#[test]
fn the_roster_answer_after_a_founding_takes_the_refusal_down() {
    let mut model = refused();
    let channel = model.roster[0].channel.clone();
    let listing = json!({"ok": true, "kind": "workspaces", "rows": [
        {"workspace": "lab", "kind": "named", "attention": 0, "agents": 1, "running": false}
    ]});
    model.receipt(&channel, "workspaces", crate::reply::read(&listing));
    assert_eq!(model.roster[0].held, Held::Heard);
    let seen = glass(&mut model);
    assert!(!seen.iter().any(|run| run.text.contains("refused")));
}

/// **A channel this seat holds no section for** still has its refusal said:
/// the bar is where a fact with no home goes.
#[test]
fn a_refusal_for_no_section_goes_to_the_bar() {
    let mut model = Model::default();
    let stray = crate::ui::Channel {
        name: "stray".to_owned(),
        ..crate::ui::Channel::default()
    };
    model.receipt(&stray, "workspaces", Read::Refusal(NOWHERE.to_owned()));
    assert_eq!(
        model.notice,
        Some(crate::ui::Notice::Refused(NOWHERE.to_owned()))
    );
}
