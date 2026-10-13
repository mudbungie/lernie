//! A dropup between frames: what it lists, what it asks, and what a pick
//! fires.

use super::{Pick, Reload, Selector};
use crate::reply::{Read, Reply};
use crate::test_support::window::{own, selecting};
use crate::ui::{Aim, Model};

/// The envelope the last gesture composed.
fn last(model: &Model) -> Option<serde_json::Value> {
    model.outbox.last().map(|posted| posted.envelope.clone())
}

/// **Each list is what this seat holds, its cursor on what is in force.**
#[test]
fn each_list_is_the_held_rows_with_the_cursor_on_what_is_in_force() {
    let mut model = selecting();
    let texts =
        |model: &Model| -> Vec<String> { model.choices().into_iter().map(|c| c.text).collect() };
    assert!(texts(&model).is_empty(), "nothing open, nothing listed");
    model.drop_up(Selector::Workspace);
    assert_eq!(texts(&model), vec!["home"]);
    assert_eq!(last(&model), Some(crate::verbs::workspaces()));
    model.drop_up(Selector::Model);
    assert_eq!(texts(&model), vec!["house-model-1", "house-model-2"]);
    model.drop_up(Selector::Effort);
    assert_eq!(texts(&model), vec!["low", "medium", "high", "off"]);
    assert_eq!(model.dropup.as_ref().map(|open| open.cursor), Some(1));
    model.drop_up(Selector::Priority);
    assert_eq!(texts(&model), vec!["on", "off"]);
    assert_eq!(model.dropup.as_ref().map(|open| open.cursor), Some(1));
    model.walk_dropup(-5);
    assert_eq!(model.dropup.as_ref().map(|open| open.cursor), Some(0));
    model.walk_dropup(5);
    assert_eq!(model.dropup.as_ref().map(|open| open.cursor), Some(1));
    model.roster.push(crate::ui::Chunk {
        channel: crate::ui::Channel {
            name: "annex".to_owned(),
            named_there: None,
            dials: None,
        },
        ..own()
    });
    model.drop_up(Selector::Workspace);
    assert_eq!(texts(&model)[1], "home · annex");
}

/// **The model the worker is on is listed whether or not its provider's
/// listing has come back**, and a provider with none in hand lists nothing.
#[test]
fn the_worker_s_model_is_listed_before_the_listing() {
    let mut model = selecting();
    model.offers.clear();
    model.drop_up(Selector::Model);
    let rows = model.choices();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].current);
    model.close_dropup();
    model.roles = None;
    model.drop_up(Selector::Model);
    assert!(model.choices().is_empty());
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        None,
        "nothing to ask"
    );
}

/// **Nothing aimed asks only the roster**: a wall's read has no wall.
#[test]
fn nothing_aimed_asks_nothing_of_a_wall() {
    let mut model = Model::default();
    model.drop_up(Selector::Provider);
    assert_eq!(model.dropup.as_ref().and_then(|open| open.reload), None);
    assert!(model.outbox.is_empty());
    model.drop_up(Selector::Workspace);
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        Some(Reload::Asked)
    );
}

/// **The answer lands only on the list that asked it**, and a model listing
/// is filed under the row asked.
#[test]
fn the_answer_lands_on_the_list_that_asked() {
    let mut model = selecting();
    let channel = own().channel;
    model.absorb(&channel, Read::Answer(Reply::Roles(Vec::new())));
    model.roles = selecting().roles;
    model.drop_up(Selector::Model);
    model.absorb(&channel, Read::Answer(Reply::Providers(Vec::new())));
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        Some(Reload::Asked)
    );
    let fresh = vec!["house-model-9".to_owned()];
    model.absorb(&channel, Read::Answer(Reply::Models(fresh.clone())));
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        Some(Reload::Landed)
    );
    assert_eq!(model.offered_by("housevendor"), fresh);
    model.drop_up(Selector::Workspace);
    model.absorb(&channel, Read::Answer(Reply::Models(Vec::new())));
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        Some(Reload::Asked)
    );
}

/// **Each pick fires its door**: a wall aims, a provider opens its models, a
/// model binds the worker, effort and priority tune it.
#[test]
fn each_pick_fires_its_door() {
    let mut model = selecting();
    model.drop_up(Selector::Provider);
    model.pick(1);
    let open = model
        .dropup
        .clone()
        .expect("the model list on the picked row");
    assert_eq!(
        (open.selector, open.provider.as_deref()),
        (Selector::Model, Some("othervendor"))
    );
    let asked = crate::verbs::models("home".to_owned(), "othervendor".to_owned());
    assert_eq!(last(&model), Some(asked));
    model
        .offers
        .insert("othervendor".to_owned(), vec!["other-1".to_owned()]);
    model.pick(0);
    let bound = crate::verbs::model(
        "home".to_owned(),
        "worker".to_owned(),
        "othervendor".to_owned(),
        "other-1".to_owned(),
    );
    assert_eq!(last(&model), Some(bound));
    model.drop_up(Selector::Priority);
    model.pick(0);
    assert_eq!(
        last(&model),
        Some(crate::verbs::priority(
            "home".to_owned(),
            "worker".to_owned(),
            true
        ))
    );
    model.drop_up(Selector::Effort);
    model.pick(3);
    assert_eq!(
        last(&model),
        Some(crate::verbs::effort(
            "home".to_owned(),
            "worker".to_owned(),
            None
        ))
    );
    model.drop_up(Selector::Workspace);
    model.pick(9);
    assert!(
        model.dropup.is_some(),
        "a row that is not there picks nothing"
    );
    model.dropup = Some(super::Dropup {
        selector: Selector::Model,
        provider: None,
        cursor: 0,
        reload: None,
    });
    model
        .offers
        .insert(String::new(), vec!["orphan".to_owned()]);
    let before = model.outbox.len();
    model.pick(0);
    assert_eq!(
        model.outbox.len(),
        before,
        "a model on no provider binds nothing"
    );
    model.conversation = None;
    model.drop_up(Selector::Workspace);
    model.pick(0);
    assert_eq!(
        model.aim,
        Some(Aim {
            channel: own().channel.name,
            address: "home".to_owned()
        })
    );
    assert!(matches!(
        Pick::Wall(model.aim.clone().unwrap_or_else(|| unreachable!())),
        Pick::Wall(_)
    ));
}

/// **A refusal of the read the list posted puts its reload down**; a refusal
/// of another op, or an answer, does not.
#[test]
fn a_refusal_of_its_read_puts_the_reload_down() {
    let mut model = selecting();
    let channel = own().channel;
    model.drop_up(Selector::Model);
    let no = || Read::Refusal("no credential".to_owned());
    model.receipt(&channel, crate::verbs::PROVIDERS.word, no());
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        Some(Reload::Asked)
    );
    model.receipt(
        &channel,
        crate::verbs::MODELS.word,
        Read::Answer(Reply::Models(Vec::new())),
    );
    assert_eq!(
        model.dropup.as_ref().and_then(|open| open.reload),
        Some(Reload::Landed)
    );
    model.receipt(&channel, crate::verbs::MODELS.word, no());
    assert_eq!(model.dropup.as_ref().and_then(|open| open.reload), None);
    model.drop_up(Selector::Workspace);
    model.receipt(
        &channel,
        crate::verbs::WORKSPACES.word,
        Read::Unreadable("x".to_owned()),
    );
    assert_eq!(model.dropup.as_ref().and_then(|open| open.reload), None);
    model.drop_up(Selector::Effort);
    model.receipt(&channel, crate::verbs::PROVIDERS.word, no());
    assert_eq!(model.dropup.as_ref().and_then(|open| open.reload), None);
    model.close_dropup();
    model.receipt(&channel, crate::verbs::WORKSPACES.word, no());
    assert!(model.notice.is_some());
}
