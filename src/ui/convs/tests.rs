//! The conversation list: the empty states, the headline, the age, the indent,
//! and the click that selects.

use super::{NO_CONVERSATIONS, NO_WALL, NOT_ANSWERED, age, headline, no_channel, render};
use crate::paint_probe::frame::Window;
use crate::reply::convs::{AgentState, Tone};
use crate::test_support::window::{click, conv, pane, seated, seen};
use crate::ui::Model;

/// **Four empty states and they are four different facts** (bl-f780): nothing
/// aimed at, a wall nobody has been answered about yet, a wall that answered
/// nothing, and an aim on a channel this seat does not hold.
///
/// The third used to be said about all three of the last three, which states a
/// definite fact about a wall nobody looked at — the same thing
/// `super::UNCERTAIN` refuses to do one level down.
#[test]
fn the_empty_states_say_four_different_things() {
    let mut nowhere = Model::default();
    assert!(pane(|ui| render(ui, &mut nowhere)).contains(NO_WALL));

    let mut waiting = seated();
    waiting.convs.clear();
    waiting.answered = None;
    let painted = pane(|ui| render(ui, &mut waiting));
    assert!(painted.contains(NOT_ANSWERED), "{painted}");
    assert!(!painted.contains(NO_CONVERSATIONS), "{painted}");
    assert!(
        painted.contains("home"),
        "the wall is still named: {painted}"
    );

    let mut empty = seated();
    empty.convs.clear();
    empty.answered = empty.aim.clone();
    let painted = pane(|ui| render(ui, &mut empty));
    assert!(painted.contains(NO_CONVERSATIONS), "{painted}");

    let mut stale = seated();
    stale.convs.clear();
    stale.aim = Some(crate::ui::Aim {
        channel: "a channel this box lost".to_owned(),
        address: "home".to_owned(),
    });
    let painted = pane(|ui| render(ui, &mut stale));
    assert!(
        painted.contains(&no_channel("a channel this box lost")),
        "{painted}"
    );
    assert!(!painted.contains(NOT_ANSWERED), "{painted}");
}

/// **Aiming somewhere else retires the ANSWER with the rows.** The transient
/// face of the same defect: between the keypress and the reply the pane held
/// the new wall's name over an empty list, and said the wall held nothing.
#[test]
fn aiming_elsewhere_leaves_the_pane_waiting_rather_than_reporting_nothing() {
    let mut model = seated();
    assert_eq!(model.answered, None, "seated() has not been answered");
    model.absorb(
        &model.roster[0].channel.clone(),
        crate::reply::Read::Answer(crate::reply::Reply::Conversations(vec![conv("id", "one")])),
    );
    assert_eq!(model.answered, model.aim, "the answer is about the aim");
    model.aim_at("(this box's own engine)", "elsewhere");
    assert_eq!(model.answered, None, "and it is retired with the rows");
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains(NOT_ANSWERED), "{painted}");
}

/// A headline is a glance: the label, what it is doing, how long since it moved
/// — and the two counts only where there is something to count.
#[test]
fn a_headline_says_the_state_the_age_and_only_the_counts_that_matter() {
    let quiet = headline(&conv("id", "port the probe"));
    assert_eq!(quiet, "port the probe  [quiescent]  42s");
    let busy = headline(&crate::reply::convs::ConvRow {
        state: AgentState::Live,
        attention: 2,
        members: 4,
        age_secs: 7200,
        ..conv("id", "port the probe")
    });
    assert!(quiet.len() < busy.len());
    assert!(busy.contains("[live]"), "{busy}");
    assert!(busy.contains("2h"), "{busy}");
    assert!(busy.contains("2 waiting"), "{busy}");
    assert!(busy.contains("4 members"), "{busy}");
}

/// **An age in the future is not a thing to paint.** Two machines' clocks
/// disagreeing is a fact about a seat that dials somewhere else, so a negative
/// age clamps rather than refusing or showing a minus sign.
#[test]
fn an_age_is_compact_and_never_negative() {
    for (secs, said) in [
        (-3, "0s"),
        (0, "0s"),
        (42, "42s"),
        (59, "59s"),
        (60, "1m"),
        (3599, "59m"),
        (3600, "1h"),
        (86_399, "23h"),
        (86_400, "1d"),
        (172_800, "2d"),
    ] {
        assert_eq!(age(secs), said, "{secs}");
    }
}

/// A member hangs under its root, and its preview is painted in the row's own
/// tone — which is not derivable from the state beside it.
#[test]
fn a_member_is_indented_and_its_preview_carries_the_row_s_tone() {
    let mut model = seated();
    model.convs = vec![crate::reply::convs::ConvRow {
        depth: 2,
        tone: Tone::Weak,
        preview: "the galley lies about elision".to_owned(),
        ..conv("child", "a member")
    }];
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("a member"), "{painted}");
    assert!(
        painted.contains("the galley lies about elision"),
        "{painted}"
    );
}

/// **A red row says what is wrong with it** (REMOTE §9.10). A conversation
/// whose latest model call failed used to paint a `bad` tone and no words, so
/// a wall whose provider row holds no credential was a list of red rows an
/// operator opened one by one to learn the one thing all of them said.
///
/// Three assertions, and the ordering is the one that would rot silently: the
/// clause is on the glass, it is painted in the row's own tone rather than in
/// an ink this pane decided for it, and it stands ABOVE the preview — the row
/// reads label, then why nothing more happened, then what was last said.
#[test]
fn a_failed_row_paints_its_clause_above_its_preview_and_in_its_own_tone() {
    let mut model = seated();
    model.convs = vec![crate::reply::convs::ConvRow {
        tone: Tone::Bad,
        failure: "no credential for provider row \"work\"".to_owned().into(),
        preview: "the last thing it managed to say".to_owned(),
        ..conv("id", "a refusing conversation")
    }];
    let window = Window::sized(900.0, 600.0);
    let runs = seen(&window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    let at = |needle: &str| {
        runs.iter()
            .find(|run| run.text.contains(needle))
            .unwrap_or_else(|| panic!("{needle:?} is not on the glass"))
            .clone()
    };
    let clause = at("no credential for provider row");
    assert_eq!(clause.ink, crate::ui::theme::tone_ink(&Tone::Bad));
    assert!(
        clause.laid.min.y < at("the last thing it managed to say").laid.min.y,
        "the clause stands above the preview: {:?}",
        clause.laid
    );
}

/// **Selecting a conversation drops the last one's transcript**, for the reason
/// the roster drops the last wall's list: content under a new header reads as
/// current, and a transcript is the one place that lies worst.
#[test]
fn a_click_selects_the_conversation_and_drops_the_last_one_s_transcript() {
    let mut model = seated();
    model.conversation = None;
    let window = Window::new();
    click(&window, &headline(&model.convs[0].clone()), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));
    assert!(model.transcript.entries.is_empty());
    assert_eq!(model.live, None);
}

/// **A state nothing observed wears a `?`**, inside the badge it qualifies. The
/// engine answers a reading and whether it could take one, and a badge that
/// painted only the first would state a definite fact about a conversation
/// nobody looked at.
#[test]
fn an_unobserved_state_is_marked_rather_than_painted_as_definite() {
    let unsure = headline(&crate::reply::convs::ConvRow {
        uncertain: true,
        ..conv("id", "port the probe")
    });
    assert_eq!(unsure, "port the probe  [quiescent?]  42s");
}

/// **The conversation this window just started stands in the list**, before the
/// engine has anything to answer about it: the minted name, the operator's own
/// goal beneath it, and a badge that says nothing observed any of it.
#[test]
fn a_started_conversation_stands_in_the_list_before_the_engine_can_answer() {
    let mut model = Model {
        conversation: Some("brisk-otter".to_owned()),
        start: Some(crate::ui::model::Start {
            address: "home".to_owned(),
            goal: "port the paint probe".to_owned(),
            phase: crate::ui::model::Phase::Started("brisk-otter".to_owned()),
        }),
        ..seated()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("brisk-otter  [quiescent?]"), "{painted}");
    assert!(painted.contains("port the paint probe"), "{painted}");
    assert!(
        !painted.contains("[live]"),
        "no driver this seat has seen: {painted}"
    );
}
