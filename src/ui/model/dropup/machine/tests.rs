//! The machine a provider's row is on: the reading of a name, the two lists
//! it parts, and the walk machine → provider → model that ends in one write.

use super::{SERVER, of, row, split, word};
use crate::test_support::window::{provider, selecting};
use crate::ui::{Model, Pick, Selector};

/// What an open list says, row by row.
fn texts(model: &Model) -> Vec<String> {
    model.choices().into_iter().map(|c| c.text).collect()
}

/// `selecting`, its table followed by two machines' rows.
fn fleet() -> Model {
    let mut model = selecting();
    let rows = model.providers.get_or_insert_default();
    for name in ["housevendor@attic", "farvendor@attic", "housevendor@shed"] {
        rows.push(provider(name));
    }
    model
}

/// **A name is cut at its last joiner**, and one with nothing on a side of
/// it is a bare row.
#[test]
fn a_name_is_cut_at_its_last_joiner() {
    assert_eq!(split("housevendor"), ("housevendor", None));
    assert_eq!(split("housevendor@attic"), ("housevendor", Some("attic")));
    assert_eq!(split("a@b@attic"), ("a@b", Some("attic")));
    assert_eq!(split("@attic"), ("@attic", None));
    assert_eq!(split("housevendor@"), ("housevendor@", None));
    assert_eq!((row("x@y"), of("x@y")), ("x", Some("y")));
    assert_eq!((word(None), word(Some("attic"))), (SERVER, "attic"));
}

/// **The machine list is the server and each client a row names**, the
/// worker's own marked; **a machine's list is its rows alone, unqualified**.
#[test]
fn the_table_is_parted_by_machine() {
    let mut model = fleet();
    model.drop_up(Selector::Machine);
    assert_eq!(texts(&model), vec![SERVER, "attic", "shed"]);
    let rows = model.choices();
    assert!(rows[0].current && !rows[1].current, "{rows:?}");
    assert_eq!(model.dropup.as_ref().map(|open| open.cursor), Some(0));
    model.drop_up(Selector::Provider);
    assert_eq!(texts(&model), vec!["housevendor", "othervendor"]);
    model.providers = None;
    model.drop_up(Selector::Machine);
    assert_eq!(texts(&model), vec![SERVER], "before the table answers");
    assert_eq!(Model::default().machine_rows().len(), 1);
}

/// **Picking a machine opens its providers and writes nothing**; picking one
/// opens its models under the qualified name; the model is the one write, and
/// it carries the qualified name.
#[test]
fn a_machine_opens_its_providers_and_only_a_model_writes() {
    let mut model = fleet();
    model.drop_up(Selector::Machine);
    let before = model.outbox.len();
    model.pick(1);
    let open = model.dropup.clone().expect("the attic's providers");
    assert_eq!(
        (open.selector, open.machine.as_deref()),
        (Selector::Provider, Some("attic"))
    );
    assert_eq!(texts(&model), vec!["housevendor", "farvendor"]);
    assert!(model.choices().iter().all(|choice| !choice.current));
    let asked = model.outbox[before..].to_vec();
    assert_eq!(asked.len(), 1, "the providers read, and no write");
    assert_eq!(
        asked[0].envelope,
        crate::verbs::providers("home".to_owned())
    );
    model.pick(1);
    let open = model.dropup.clone().expect("the far row's models");
    assert_eq!(
        (
            open.selector,
            open.provider.as_deref(),
            open.machine.as_deref()
        ),
        (Selector::Model, Some("farvendor@attic"), Some("attic"))
    );
    model
        .offers
        .insert("farvendor@attic".to_owned(), vec!["far-1".to_owned()]);
    model.pick(0);
    assert_eq!(
        model.outbox.last().map(|posted| posted.envelope.clone()),
        Some(crate::verbs::model(
            "home".to_owned(),
            "worker".to_owned(),
            "farvendor@attic".to_owned(),
            "far-1".to_owned(),
        ))
    );
}

/// **A worker on a machine's row opens on that machine**, and its machine
/// and its row are listed whether or not the table names them.
#[test]
fn a_worker_on_a_machine_opens_on_it() {
    let mut model = selecting();
    if let Some(worker) = model.roles.iter_mut().flatten().next() {
        worker.provider = "housevendor@cellar".to_owned();
    }
    model.drop_up(Selector::Machine);
    assert_eq!(texts(&model), vec![SERVER, "cellar"]);
    assert_eq!(model.dropup.as_ref().map(|open| open.cursor), Some(1));
    model.drop_up(Selector::Provider);
    let rows = model.choices();
    assert_eq!(texts(&model), vec!["housevendor"]);
    assert!(rows[0].current);
    assert_eq!(
        rows[0].pick,
        Pick::Provider("housevendor@cellar".to_owned())
    );
}
