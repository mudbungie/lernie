//! **The rule's doors** (bl-4e55, bl-d74b): the name opens the switcher, each
//! selector its dropup — each wearing the ops it spends — and Ctrl+O is the
//! model's dropup by key.

use super::super::{Door, NO_MODEL, name, runs};
use crate::paint_probe::frame::Window;
use crate::reply::transcript::{Entry, EntryKind, Usage};
use crate::test_support::window::{dropped, seated, selecting};
use crate::ui::{Model, Selector};
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

/// **The model slot**: the worker's model where its row is in hand, else what
/// answered, else the door's own word — a dropup door every time.
#[test]
fn the_model_slot_is_the_worker_s_else_what_answered() {
    let model_run = |model: &Model| runs(model).0.swap_remove(1);
    let said = model_run(&answered());
    assert_eq!(said.text, "model-a");
    assert_eq!(said.door, Some(Door::Drop(Selector::Model)));
    assert_eq!(model_run(&seated()).text, NO_MODEL);
    let mut tuned = answered();
    tuned.roles = selecting().roles;
    assert_eq!(model_run(&tuned).text, "house-model-1");
    assert!(
        matches!(name(&seated()).door, Some(Door::Row(row)) if row.word == crate::ui::switcher::WORD)
    );
}

/// **Effort and priority stand only where the worker's provider row takes
/// them** — or where the providers read has not said.
#[test]
fn effort_and_priority_follow_the_provider_row() {
    let words =
        |model: &Model| -> Vec<String> { runs(model).0.into_iter().map(|r| r.text).collect() };
    let mut model = selecting();
    model.providers = None;
    assert!(words(&model).contains(&"effort medium".to_owned()));
    model.providers = Some(vec![crate::reply::providers::ProviderRow {
        effort: false,
        priority: false,
        ..crate::test_support::window::provider("housevendor")
    }]);
    let said = words(&model);
    assert!(
        !said
            .iter()
            .any(|w| w.starts_with("effort") || w.starts_with("priority")),
        "{said:?}"
    );
}

/// **Every selector is a tagged link on the glass**, wearing the read its
/// dropup posts, and the name is the switcher's. The workspace's is the
/// bar's foot (`crate::ui::bar`'s suite).
#[test]
fn the_selectors_are_tagged_links() {
    let harness = crate::snapshot::seat(selecting(), 1400.0, 900.0);
    for (label, op) in [
        ("housevendor", crate::verbs::PROVIDERS.word),
        ("house-model-1", crate::verbs::MODELS.word),
        ("effort medium", crate::verbs::PROVIDERS.word),
    ] {
        let node = harness
            .query_all_by_label(label)
            .find(|node| format!("{:?}", node.role()) == "Link")
            .unwrap_or_else(|| panic!("{label:?} is a link"));
        let token = format!("{}{op}", crate::ui::act::PREFIX);
        assert_eq!(node.author_id(), Some(token.as_str()), "{label}");
    }
    assert!(
        harness
            .query_all_by_label("port the paint probe")
            .any(|node| format!("{:?}", node.role()) == "Link"),
        "the rule's name is a link"
    );
}

/// **Click the rule's run reading `text`** on a whole window: a settling
/// frame first, then the lowest run reading it — the card at the head can
/// say the same words, and the rule is under the transcript.
pub(super) fn click_rule(window: &Window, model: &mut Model, text: &str) {
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, model);
    window.frame(Vec::new(), &mut body);
    let at = crate::paint_probe::seen_of(&window.frame(Vec::new(), &mut body))
        .into_iter()
        .filter(|run| run.text == text)
        .map(|run| run.shown.center())
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| panic!("the rule says {text:?}"));
    crate::paint_probe::frame::click(window, at, &mut body);
}

/// **A click on the model opens its dropup and posts its read; a second
/// closes it; a click on the name opens the switcher.**
#[test]
fn a_click_on_each_door_opens_what_it_names() {
    let window = Window::new();
    let mut model = selecting();
    click_rule(&window, &mut model, &dropped("house-model-1"));
    let open = model.dropup.clone().expect("the model's dropup stands");
    assert_eq!(open.selector, Selector::Model);
    let models = crate::verbs::models("home".to_owned(), "housevendor".to_owned());
    assert_eq!(
        model.outbox.last().map(|p| p.envelope.clone()),
        Some(models)
    );
    let reloading = format!("house-model-1 {}", crate::ui::theme::glyph::RELOAD);
    click_rule(&window, &mut model, &reloading);
    assert!(model.dropup.is_none(), "a second click closes it");
    let mut model = seated();
    click_rule(&Window::new(), &mut model, "port the paint probe");
    assert!(model.switching, "the name opens the switcher");
}

/// **Ctrl+O and `/model` open the model's dropup** where a workspace is aimed, and
/// nothing where none is, as the word is not offered there — and over a
/// pane it replaces the pane, as Ctrl+X does.
#[test]
fn ctrl_o_opens_the_model_s_dropup_where_a_workspace_is_aimed() {
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
    let mut model = selecting();
    ctrl_o(&mut model);
    assert_eq!(
        model.dropup.map(|open| open.selector),
        Some(Selector::Model)
    );
    assert!(model.tuning.is_none(), "the pane is `/roles`'s now");
    // **Over a pane the chord puts it down and opens** (bl-27dd), and the
    // list is still standing on the glass the frame after.
    model.dropup = None;
    model.begin_trail();
    ctrl_o(&mut model);
    assert!(!model.covered(), "the trail is put down");
    let text = crate::test_support::window::painted(&mut model);
    assert!(text.contains(crate::ui::status::dropup::HINT), "{text}");
    model.dropup = None;
    model.begin_enrollment();
    ctrl_o(&mut model);
    assert!(
        model.dropup.is_none() && model.enroll.is_some(),
        "never over the enrollment"
    );
    let mut nowhere = Model::default();
    ctrl_o(&mut nowhere);
    assert!(nowhere.dropup.is_none());
}

/// **A notice's slash words are doors beside it** (bl-7a77): each menu row the
/// sentence names, once, in the order said, in the brand — and a word the
/// menu does not offer stays words. A click on one fires its row.
#[test]
fn a_refusal_carries_the_doors_it_names() {
    let mut model = seated();
    model.notice = Some(crate::ui::Notice::Refused(
        "sign in first: `worker` holds no credential — write its key with /config brazen, \
         /login a row, or /config again; /nowhere"
            .to_owned(),
    ));
    let doors = crate::ui::status::diagnostics::doors(&model);
    let said: Vec<&str> = doors.iter().map(|run| run.text.as_str()).collect();
    assert_eq!(said, ["/config", "/login"]);
    for run in &doors {
        assert!(matches!(run.door, Some(Door::Row(row)) if run.text.ends_with(row.word)));
        assert_eq!(run.ink, crate::ui::theme::BRAND);
    }
    let texts: Vec<String> = runs(&model).0.into_iter().map(|run| run.text).collect();
    assert!(texts.contains(&"/login".to_owned()), "{texts:?}");
    model.notice = None;
    assert!(crate::ui::status::diagnostics::doors(&model).is_empty());
    model.notice = Some(crate::ui::Notice::Refused("go /login now".to_owned()));
    click_rule(&Window::new(), &mut model, "/login");
    assert!(model.covered(), "the login pane stands");
}

/// **A refused role write carries the seat's own door** (bl-8a4b): the
/// server's words untouched, `/config` after them and on the rule — decided
/// by the op that was refused, so another op's refusal carries none. And an
/// act that never left names the start as what it was, never as its op.
#[test]
fn a_refused_role_write_names_config_and_another_refusal_does_not() {
    use crate::reply::Read;
    let mut model = seated();
    let flat = model.roster[0].channel.clone();
    let said = "`qwen2.5:7b` is not a plain model id";
    model.receipt(
        &flat,
        crate::verbs::MODEL.word,
        Read::Refusal(said.to_owned()),
    );
    let line = model.notice.clone().expect("refused").line();
    let door = crate::ui::model::CONFIG_DOOR;
    assert_eq!(line, format!("the server refused: {said} — {door}"));
    let doors = crate::ui::status::diagnostics::doors(&model);
    let words: Vec<&str> = doors.iter().map(|run| run.text.as_str()).collect();
    assert_eq!(words, ["/config"]);
    model.receipt(
        &flat,
        crate::verbs::NUDGE.word,
        Read::Refusal(said.to_owned()),
    );
    let line = model.notice.clone().expect("refused").line();
    assert_eq!(line, format!("the server refused: {said}"));
    let unsent = crate::channel::Reach::Unsent("no route".to_owned());
    let line = crate::ui::Notice::act(crate::verbs::PREPARE, &unsent).line();
    assert!(
        line.starts_with("the new conversation never reached"),
        "{line}"
    );
}

/// **Every selector tells what it changes** (DESIGN §4.46 item 2), each in
/// its own words.
#[test]
fn every_selector_has_its_own_sentence() {
    let said: std::collections::BTreeSet<&str> = [
        Selector::Workspace,
        Selector::Provider,
        Selector::Model,
        Selector::Effort,
        Selector::Priority,
    ]
    .into_iter()
    .map(super::super::selectors::help)
    .collect();
    assert_eq!(said.len(), 5, "{said:?}");
    assert!(said.iter().all(|s| !s.is_empty()));
}
