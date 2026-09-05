//! The three acts that spend no words: what each composes, and the arming that
//! is the operator's until they delete something with it.

use super::{ARM, DELETE, FLAG, RESTORE, RETARGET, REVOKE, STOP, WHY, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::{Aim, Model};
use serde_json::json;

/// The aim and the conversation the [`seated`] fixture holds, spelled once.
fn subject(model: &Model) -> (Aim, String) {
    (
        model.aim.clone().expect("the fixture is aimed at a wall"),
        model.conversation.clone().expect("and has one selected"),
    )
}

/// Paint the row on its own and click the seat reading `label`.
fn press(model: &mut Model, label: &str) {
    let (aim, agent) = subject(model);
    let window = Window::new();
    click(&window, label, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model, &aim, &agent));
    });
}

/// **All four are on the row**, and each box says what it is for rather than
/// standing there unlabelled.
#[test]
fn the_row_offers_the_four_acts_and_says_what_each_box_is_for() {
    let mut model = seated();
    let (aim, agent) = subject(&model);
    let painted = pane(|ui| render(ui, &mut model, &aim, &agent));
    for word in [STOP, RETARGET, FLAG, DELETE, ARM, WHY] {
        assert!(
            painted.lines().any(|line| line == word),
            "{word:?}:\n{painted}"
        );
    }
}

/// **Each composes the gesture the command line would have**, built by the same
/// verb row `lernie stop` spends — and nothing is sent from a frame.
#[test]
fn each_act_composes_the_envelope_its_verb_row_builds() {
    for (label, expected) in [
        (
            STOP,
            json!({"op": "stop", "workspace": "home", "agent": "20260830T051200Z-a1b2"}),
        ),
        (
            RETARGET,
            json!({"op": "retarget", "workspace": "home", "agent": "20260830T051200Z-a1b2"}),
        ),
        (
            DELETE,
            json!({"op": "delete-agent", "workspace": "home",
                   "agent": "20260830T051200Z-a1b2", "typed": ""}),
        ),
    ] {
        let mut model = seated();
        press(&mut model, label);
        assert_eq!(
            model.outbox,
            vec![crate::ui::Posted::act(expected)],
            "{label}"
        );
    }
}

/// **An empty arming is the bare form and not a refusal**, which is the wire's
/// own grammar: the one conversation goes and its descendants do not. Typing
/// the name is what admits them, and the seat carries the string verbatim
/// rather than deciding anything about it.
#[test]
fn the_typed_name_rides_the_deletion_verbatim_and_stays_the_operators() {
    let mut model = seated();
    model.typed = "port the paint probe".to_owned();
    press(&mut model, DELETE);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "delete-agent", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "typed": "port the paint probe"})
        )]
    );
    assert_eq!(
        model.typed, "port the paint probe",
        "a delete the engine refuses while the conversation is live must not \
         charge the operator a retype"
    );
}

/// **The address is the aim's, not the row's name** — the same rule the deposit
/// one row up follows, asserted here because these three build their own
/// envelopes rather than going through its body.
#[test]
fn the_acts_carry_the_address_the_channel_resolves() {
    let mut model = Model {
        aim: Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "elsewhere".to_owned(),
        }),
        ..seated()
    };
    press(&mut model, STOP);
    assert_eq!(model.outbox[0].envelope["workspace"], json!("elsewhere"));
}

/// **The raise carries the operator's words and SPENDS them** (bl-f0ef), which
/// is where it parts from the arming above: a flag that fired is said, exactly
/// as a deposit is, and the next flag on this conversation is a different
/// sentence about a different moment.
#[test]
fn the_raise_carries_the_reason_and_spends_the_box() {
    let mut model = seated();
    model.reason = "it is rewriting an unrelated crate".to_owned();
    press(&mut model, FLAG);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "flag", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2",
                    "reason": "it is rewriting an unrelated crate"})
        )]
    );
    assert!(model.reason.is_empty(), "a flag that fired is said");
}

/// **The reason is the wire's own requirement, so the control is disabled and
/// not absent**: the parameter is missing, the subject is not, and a control
/// that vanished would say the conversation could not be flagged at all.
#[test]
fn a_raise_with_no_words_is_offered_and_fires_nothing() {
    let mut model = seated();
    model.reason = "   ".to_owned();
    let (aim, agent) = subject(&model);
    let window = Window::new();
    let painted = pane(|ui| render(ui, &mut model, &aim, &agent));
    assert!(painted.lines().any(|line| line == FLAG), "{painted}");
    click(&window, FLAG, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aim, &agent));
    });
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **Nothing fires from a frame nobody clicked.** The row paints three buttons
/// and two boxes every frame the composer paints, and a frame that composed a
/// deletion for having been drawn is the one failure this row cannot have.
#[test]
fn painting_the_row_composes_nothing() {
    let mut model = seated();
    let (aim, agent) = subject(&model);
    let _ = pane(|ui| render(ui, &mut model, &aim, &agent));
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **Both floor controls are always on the glass, and each fires its own
/// assertion** (bl-bce2). Neither can be refused — a floor is a row appended
/// to the engine's trail and the receipt is re-derived from it — so there is
/// no rank to read and nothing this seat could get wrong by offering both.
#[test]
fn the_floor_pair_are_both_offered_and_each_asserts_its_own_direction() {
    let mut model = seated();
    let (aim, agent) = subject(&model);
    let painted = pane(|ui| render(ui, &mut model, &aim, &agent));
    for word in [REVOKE, RESTORE] {
        assert!(
            painted.lines().any(|line| line == word),
            "{word:?}:\n{painted}"
        );
    }
    for (word, op) in [(REVOKE, "revoke"), (RESTORE, "restore")] {
        let mut model = seated();
        press(&mut model, word);
        assert_eq!(
            model.outbox,
            vec![crate::ui::Posted::act(json!({
                "op": op, "workspace": "home", "agent": "20260830T051200Z-a1b2"
            }))],
            "{word}"
        );
    }
}
