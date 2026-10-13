//! **The rule's doors** (bl-4e55): the name opens the switcher, the model
//! opens the role tuning, each wears its menu row's tags, and Ctrl+O is the
//! model's door by key.

use super::super::{NO_MODEL, facts, name};
use crate::paint_probe::frame::Window;
use crate::reply::transcript::{Entry, EntryKind, Usage};
use crate::test_support::window::seated;
use crate::ui::Model;
use egui_kittest::kittest::Queryable;

/// A seated window whose transcript says which model answered.
fn answered() -> Model {
    let mut model = seated();
    model.transcript.entries.push(Entry {
        name: "002-model-a.json".to_owned(),
        raw: "{}".to_owned(),
        kind: EntryKind::Model {
            model_id: "model-a".to_owned(),
            blocks: Vec::new(),
            usage: Usage::new(),
        },
    });
    model
}

/// **The doors are the menu's own rows**: the name the switcher's, the model
/// the model's — and a wall with no model answered says the door's word
/// there, or its worker's model where its roles are in hand (bl-0966), with
/// the same door, while a window aimed nowhere has neither.
#[test]
fn the_name_and_the_model_are_the_menu_s_rows() {
    let door = |run: &super::super::Run| run.door.map(|row| row.word);
    assert_eq!(door(&name(&seated())), Some(crate::ui::switcher::WORD));
    assert_eq!(
        door(&name(&Model::default())),
        Some(crate::ui::switcher::WORD)
    );
    let said = facts(&answered());
    assert_eq!(said[0].text, "model-a");
    assert_eq!(door(&said[0]), Some(crate::ui::tuning::MODEL));
    let bare = facts(&Model {
        conversation: None,
        ..seated()
    });
    assert_eq!(bare[0].text, NO_MODEL);
    assert_eq!(door(&bare[0]), Some(crate::ui::tuning::MODEL));
    let tuned = facts(&Model {
        conversation: None,
        roles: Some(vec![
            crate::test_support::window::role("compactor"),
            crate::reply::roles::RoleRow {
                model: "worker-model".to_owned(),
                ..crate::test_support::window::role("worker")
            },
        ]),
        ..seated()
    });
    assert_eq!(tuned[0].text, "worker-model");
    assert_eq!(door(&tuned[0]), Some(crate::ui::tuning::MODEL));
    assert!(
        facts(&Model::default())
            .iter()
            .all(|run| run.door.is_none())
    );
}

/// **The model's door wears `act:roles` on the glass**, as `/roles` does, and
/// is a link — a run, not a button.
#[test]
fn the_model_s_door_is_a_tagged_link() {
    let harness = crate::snapshot::seat(answered(), 1400.0, 900.0);
    // The conversation's card says the model too, in a label; the rule's is
    // the one that is a link.
    let node = harness
        .query_all_by_label("model-a")
        .find(|node| format!("{:?}", node.role()) == "Link")
        .expect("the rule's model is a link");
    assert_eq!(format!("{:?}", node.role()), "Link");
    let token = format!("{}{}", crate::ui::act::PREFIX, crate::verbs::ROLES.word);
    assert_eq!(node.author_id(), Some(token.as_str()));
    assert!(
        harness
            .query_all_by_label("port the paint probe")
            .any(|node| format!("{:?}", node.role()) == "Link"),
        "the rule's name is a link"
    );
}

/// **Click the rule's run reading `text`** on a whole window: a settling
/// frame first — the bottom panels are laid where they stand on the frame
/// after the one that measured them — then the lowest run reading it, because
/// the card at the head can say the same words and the rule is under the
/// transcript.
fn click_rule(model: &mut Model, text: &str) {
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, model);
    window.frame(Vec::new(), &mut body);
    let at = crate::paint_probe::seen_of(&window.frame(Vec::new(), &mut body))
        .into_iter()
        .filter(|run| run.text == text)
        .map(|run| run.shown.center())
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| panic!("the rule says {text:?}"));
    crate::paint_probe::frame::click(&window, at, &mut body);
}

/// **A click on the model opens the tuning; a click on the name opens the
/// switcher** — the doors `/model` and `/conversations` are.
#[test]
fn a_click_on_each_door_opens_its_overlay() {
    let mut model = answered();
    click_rule(&mut model, "model-a");
    assert!(model.tuning.is_some(), "the model opens the role tuning");
    let mut model = Model {
        conversation: None,
        ..seated()
    };
    click_rule(&mut model, NO_MODEL);
    assert!(model.tuning.is_some(), "no model opens it too");
    let mut model = seated();
    click_rule(&mut model, "port the paint probe");
    assert!(model.switching, "the name opens the switcher");
}

/// **The waiting count is the queue's door** (§4.42, item 8; bl-2cb0) — on
/// a window aimed at nothing too, the seat most likely to be asking — and it
/// wears `/attention`'s tag.
#[test]
fn a_click_on_the_waiting_count_opens_the_queue_with_nothing_aimed() {
    let mut model = Model {
        aim: None,
        conversation: None,
        ..seated()
    };
    model.roster[0].walls.push(crate::reply::roster::WsRow {
        attention: 2,
        ..crate::test_support::window::wall("annex")
    });
    let count = facts(&model)
        .into_iter()
        .find(|run| run.text == "2 waiting")
        .expect("the count is a fact");
    assert_eq!(
        count.door.map(|row| row.ops),
        Some(&[crate::verbs::ATTENTION.word][..])
    );
    click_rule(&mut model, "2 waiting");
    assert!(model.showing(crate::ui::Listing::Queue), "the queue stands");
}

/// **Ctrl+O is `/model`** — from the box too — and opens nothing where no
/// wall is aimed, as the word is not offered there.
#[test]
fn ctrl_o_opens_the_role_tuning_where_a_wall_is_aimed() {
    let ctrl_o = |model: &mut Model| {
        let window = Window::new();
        window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
        window.frame(
            vec![egui::Event::Key {
                key: egui::Key::O,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }],
            |ctx| crate::ui::render(ctx, model),
        );
    };
    let mut model = seated();
    ctrl_o(&mut model);
    assert!(model.tuning.is_some());
    let mut nowhere = Model::default();
    ctrl_o(&mut nowhere);
    assert!(nowhere.tuning.is_none());
}
