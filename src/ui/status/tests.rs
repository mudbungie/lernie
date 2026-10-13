//! The status rule: its state slot, its runs, its name, and what fits.

use super::diagnostics::{PLAN, count, diagnostics};
use super::{ENGINES, LEAD, READY, SEP, elide, fitting, name, runs, state, width};
use crate::reply::convs::AgentState;
use crate::test_support::window::{painted, seated, selecting, wall};
use crate::ui::{Model, Notice, theme};

/// The texts of a model's runs between the leads.
fn texts(model: &Model) -> Vec<String> {
    runs(model).0.into_iter().map(|run| run.text).collect()
}

/// **The state slot is the selected conversation's word and age, in its ink**,
/// `ready` where nothing is selected, and the notice while there is one.
#[test]
fn the_state_slot_follows_the_selection_and_the_notice() {
    let quiet = state(&Model::default());
    assert_eq!(quiet.text, READY);
    assert_eq!(quiet.ink, theme::INK_WEAK);
    let talking = state(&seated());
    assert!(talking.text.contains("quiescent"), "{}", talking.text);
    assert!(talking.text.contains("42s"), "{}", talking.text);
    assert_eq!(talking.ink, theme::state_ink(&AgentState::Quiescent));
    let mut heard = seated();
    heard.notice = Some(Notice::Refused("no".to_owned()));
    let said = state(&heard);
    assert!(said.text.contains("refused"), "{}", said.text);
    assert_eq!(said.ink, theme::accent(theme::State::Error));
    heard.notice = Some(Notice::Said("the act landed".to_owned()));
    let noted = state(&heard);
    assert!(noted.text.contains("the act landed"), "{}", noted.text);
    assert_eq!(noted.ink, theme::accent(theme::State::Annotation));
}

/// **The rule's runs, left to right**: the five selectors, then the state —
/// and nothing seat-wide, however many conversations are asking or engines
/// are reached.
#[test]
fn the_runs_are_the_selectors_then_the_conversation() {
    let mut model = selecting();
    model.roster[0].walls.push(crate::reply::roster::WsRow {
        attention: 2,
        ..wall("annex")
    });
    assert_eq!(
        texts(&model),
        vec![
            "home",
            "housevendor",
            "house-model-1",
            "effort medium",
            "priority off",
            "quiescent · 42s",
        ]
    );
    assert_eq!(runs(&model).1, 3, "three never drop");
    let none = texts(&Model::default());
    assert_eq!(none, vec![super::selectors::NO_WORKSPACE, READY]);
    let glass = painted(&mut model);
    for gone in ["2 waiting", "1 engine"] {
        assert!(!glass.contains(gone), "{gone:?} is the seat's:\n{glass}");
    }
}

/// **The right end**: the conversation, else the wall, else the window.
#[test]
fn the_name_is_the_conversation_the_wall_or_the_window() {
    assert_eq!(name(&seated()).text, "port the paint probe");
    let mut starting = seated();
    starting.conversation = None;
    assert_eq!(name(&starting).text, "home");
    assert_eq!(name(&Model::default()).text, ENGINES);
}

/// **The diagnostics are the selected conversation's row**: tokens in and
/// out, the cache where it moved, the money where a rate priced it, and the
/// context — and none of it for a row that answered another conversation.
#[test]
fn the_diagnostics_are_the_conversation_s_own_figures() {
    let mut own = crate::test_support::window::panes::records::own_row();
    own.spend.tokens = crate::reply::steps::Spend {
        input: 12_345,
        output: 678,
        cache_read: 0,
        cache_write: 0,
        total: 13_023,
    };
    own.context = Some(crate::reply::agent::Fullness {
        model: "m".to_owned(),
        prompt_tokens: 340,
        window: 1000,
        percent: 34,
    });
    let mut model = seated();
    model.records.agent = Some(own.clone());
    let said: Vec<String> = diagnostics(&model).into_iter().map(|r| r.text).collect();
    assert!(said.contains(&"12.3k in · 678 out".to_owned()), "{said:?}");
    assert!(said.contains(&"context 34%".to_owned()), "{said:?}");
    assert!(!said.iter().any(|s| s.starts_with("cache")), "{said:?}");
    let priced = own.spend.cost.as_ref().map(crate::reply::spend::Cost::said);
    assert_eq!(
        priced.is_some(),
        said.iter().any(|s| Some(s) == priced.as_ref())
    );
    own.spend.tokens.cache_read = 2_000_000;
    own.spend.cost = Some(crate::reply::spend::Cost {
        usd: "$0.42".to_owned(),
        unpriced_tokens: 0,
    });
    model.records.agent = Some(own.clone());
    let said: Vec<String> = diagnostics(&model).into_iter().map(|r| r.text).collect();
    assert!(
        said.contains(&"cache 2.0M read · 0 write".to_owned()),
        "{said:?}"
    );
    assert!(said.contains(&"$0.42".to_owned()), "{said:?}");
    model.conversation = None;
    assert_eq!(diagnostics(&model).len(), 1, "a start has only its state");
}

/// **A count is whole, then `k`, then `M`.**
#[test]
fn a_count_is_whole_then_thousands_then_millions() {
    assert_eq!(count(9_999), "9999");
    assert_eq!(count(10_000), "10.0k");
    assert_eq!(count(12_345_678), "12.3M");
}

/// **What runs out of width drops from the right, and the three that never
/// drop are cut instead**: none of the rest in no room, all of it in plenty,
/// and a shown count that only grows with the room.
#[test]
fn runs_drop_from_the_right_and_the_kept_are_cut() {
    let model = selecting();
    let window = crate::paint_probe::frame::Window::new();
    let (all, kept) = runs(&model);
    let mut seen = Vec::new();
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for room in (0..120).map(|n| f32::from(u16::try_from(n * 10).unwrap_or(0))) {
                seen.push(fitting(ui, &all, kept, 40.0, room));
            }
            seen.push(fitting(ui, &all, kept, 40.0, 10_000.0));
            let cut = elide(ui, "a long workspace name", 40.0);
            assert!(cut.ends_with(theme::glyph::ELIDED) && width(ui, &cut) <= 40.0);
            assert_eq!(elide(ui, "wide", 10_000.0), "wide");
        });
    });
    assert_eq!(seen[0], (kept, 0.0));
    assert_eq!(seen.last(), Some(&(all.len(), f32::INFINITY)));
    assert!(seen.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    assert!(
        seen.iter()
            .all(|(shown, cut)| *shown == kept || cut.is_infinite())
    );
}

/// **The rule reaches the glass whole** on a seated window: the lead, the
/// selectors, a separator, the state, the name.
#[test]
fn the_rule_reaches_the_glass() {
    let mut model = selecting();
    let text = painted(&mut model);
    for word in [LEAD, SEP, "home", "housevendor", "quiescent · 42s"] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    assert!(text.contains("port the paint probe"), "{text}");
}

/// The rule's doors and its dropups.
mod doors;
mod dropup;

/// **`/plan` stands on the rule while it is on** (bl-6f3f), first among the
/// diagnostics and in the brand — the operator's own arming — and not
/// otherwise.
#[test]
fn the_plan_stands_on_the_rule_while_it_is_on() {
    let mut model = seated();
    model.conversation = None;
    assert!(!diagnostics(&model).iter().any(|run| run.text == PLAN));
    model.plan = true;
    let first = diagnostics(&model).into_iter().next().expect("a fact");
    assert_eq!(first.text, PLAN);
    assert_eq!(first.ink, theme::BRAND);
    let glass = painted(&mut model);
    assert!(glass.lines().any(|line| line == PLAN), "{glass}");
}

/// **The drop order at the matrix's three widths**: at the desk every run
/// stands; narrower the diagnostics go first, then priority, then effort;
/// at the phone the workspace, provider and model still stand — cut, never
/// dropped — beside the name.
#[test]
fn the_drop_order_at_the_three_widths() {
    let mut model = selecting();
    let mut own = crate::test_support::window::panes::records::own_row();
    own.context = Some(crate::reply::agent::Fullness {
        model: "m".to_owned(),
        prompt_tokens: 1,
        window: 2,
        percent: 50,
    });
    model.records.agent = Some(own);
    let on = |width: f32, model: &Model| -> Vec<String> {
        let window = crate::paint_probe::frame::Window::sized(width, 800.0);
        let mut model = model.clone();
        let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
        window.frame(Vec::new(), &mut body);
        crate::test_support::window::seen(&window, &mut body)
            .into_iter()
            .map(|run| run.text)
            .collect()
    };
    let desk = on(1400.0, &model);
    for word in ["home", "effort medium", "priority off", "context 50%"] {
        assert!(
            desk.contains(&word.to_owned()),
            "{word:?} at the desk: {desk:?}"
        );
    }
    let narrow = on(560.0, &model);
    assert!(narrow.contains(&"effort medium".to_owned()), "{narrow:?}");
    assert!(!narrow.contains(&"context 50%".to_owned()), "{narrow:?}");
    let phone = on(400.0, &model);
    assert!(phone.contains(&"home".to_owned()), "{phone:?}");
    assert!(
        phone.iter().any(|run| run.starts_with("house")),
        "{phone:?}"
    );
    assert!(
        !phone.iter().any(|run| run.starts_with("effort")),
        "{phone:?}"
    );
}

/// A notice longer than the rule.
mod notice;

/// **An aimed engine that will not dial says so in the state slot**, in the
/// error accent, its own sentence — and a notice still stands over it.
#[test]
fn an_aimed_engine_that_will_not_dial_says_so_in_the_slot() {
    let mut model = seated();
    model.roster[0].held = crate::ui::Held::Unheld("nothing provisioned".to_owned());
    let said = state(&model);
    assert_eq!(said.text, "nothing provisioned");
    assert_eq!(said.ink, theme::accent(theme::State::Error));
    model.notice = Some(Notice::Said("landed".to_owned()));
    assert_eq!(state(&model).text, "landed");
    model.aim = None;
    model.notice = None;
    assert_ne!(
        state(&model).text,
        "nothing provisioned",
        "not aimed, not ours"
    );
}
