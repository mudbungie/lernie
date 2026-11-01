//! The start's hint: the workspace the Enter begins on, the plan, and — on a
//! workspace whose worker reaches no model — the provider the rule shows and
//! what it lacks, so the hint and the rule never state one fact two ways
//! (DESIGN §4.45; bl-6ef6).

use super::{aimed, unselected};
use crate::test_support::window::pane;
use crate::ui::Model;
use crate::ui::composer::start::{DOORS, goal, missing, render};

/// **The hint names the workspace the Enter begins on, and says when it begins a
/// plan** (bl-9031).
#[test]
fn the_hint_names_the_workspace_and_the_plan() {
    let mut model = unselected();
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(painted.contains("on home is for"), "{painted}");
    model.plan = true;
    let planned = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(planned.contains(&goal("home", true)), "{planned}");
}

/// **A workspace whose worker reaches no model says which provider lacks what,
/// before a goal is typed** (bl-7a77; bl-6ef6): the worker's provider row is
/// bare, or absent from the answered table — brazen's routing table, which
/// lists no unfunded preset nobody named (bl-67aa) — so the hint names the
/// provider the rule shows, what it lacks, and the doors; a credentialed row,
/// or a table not yet read, asks the goal.
#[test]
fn a_workspace_whose_worker_holds_no_credential_names_the_provider_and_the_doors() {
    let row = |fact: &str| crate::reply::providers::ProviderRow {
        name: "anthropic".to_owned(),
        fact: fact.to_owned(),
        blocked: None,
        effort: true,
        priority: true,
    };
    let bare = Some("anthropic holds no credential".to_owned());
    let unlisted = Some("anthropic is on no row of this workspace".to_owned());
    for (providers, expected) in [
        (Some(vec![row("auth api_key · no credential stored")]), bare),
        (Some(vec![row("auth api_key · credential stored")]), None),
        (Some(Vec::new()), unlisted.clone()),
        (
            Some(vec![crate::test_support::window::provider("housevendor")]),
            unlisted,
        ),
        (None, None),
    ] {
        let mut model = Model {
            roles: Some(vec![worker()]),
            providers,
            ..unselected()
        };
        assert_eq!(missing(&model), expected);
        let painted = pane(|ui| render(ui, &mut model, &aimed()));
        match &expected {
            Some(lack) => {
                let hint = format!("{lack}{}{}", crate::ui::chat::panel::PART, DOORS[0]);
                assert!(painted.contains(&hint), "{painted}");
                assert!(!painted.contains(&goal("home", false)), "{painted}");
            }
            None => assert!(painted.contains(&goal("home", false)), "{painted}"),
        }
    }
    assert_eq!(
        missing(&unselected()),
        None,
        "no roles read, nothing claimed"
    );
}

/// **The hint and the rule read one query** (bl-6ef6; DESIGN §4.45): on a
/// workspace whose table holds no row for the worker's provider, the rule's
/// provider segment names that provider and the hint names the same one —
/// never *no provider yet* under a rule that shows one.
#[test]
fn the_hint_names_the_provider_the_rule_shows() {
    let model = Model {
        roles: Some(vec![worker()]),
        providers: Some(Vec::new()),
        ..unselected()
    };
    let (runs, _) = crate::ui::status::selectors::selectors(&model);
    assert_eq!(runs[0].text, "anthropic");
    let lack = missing(&model).expect("the row is missing");
    assert!(lack.starts_with(&runs[0].text), "{lack}");
    assert!(!lack.contains("no provider"), "{lack}");
}

/// The worker role bound to a provider, as the roles read answers it.
fn worker() -> crate::reply::roles::RoleRow {
    crate::reply::roles::RoleRow {
        role: "worker".to_owned(),
        provider: "anthropic".to_owned(),
        model: "m".to_owned(),
        priority: false,
        effort: None,
    }
}
