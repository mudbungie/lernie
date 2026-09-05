//! The config family as data: the row, and the typed door whose destination is
//! a nested object.

use super::{Where, config, lineages};
use crate::envelope;
use serde_json::json;

/// **The listing is a row like any other**, and its door builds the envelope
/// the command line would have.
#[test]
fn the_lineages_row_builds_the_envelope_its_word_names() {
    assert_eq!(
        lineages("home".to_owned()),
        json!({"op": "lineages", "workspace": "home"})
    );
    assert_eq!(super::LINEAGES.usage(), "lernie lineages <workspace>");
}

/// **Each destination is the `target` object the wire spells**, and the read
/// is the gesture with no `text` at all — upstream's own discriminator, so a
/// read this seat composes can never be mistaken for a write.
#[test]
fn every_destination_is_the_target_the_wire_spells_and_carries_no_text() {
    for (at, target) in [
        (
            Where::Brazen {
                workspace: "home".to_owned(),
            },
            json!({"file": "brazen", "workspace": "home"}),
        ),
        (Where::LitanyModels, json!({"file": "litany-models"})),
        (
            Where::LitanyWorkflow {
                name: "review".to_owned(),
            },
            json!({"file": "litany-workflow", "name": "review"}),
        ),
        (Where::Cadence, json!({"file": "cadence"})),
        (
            Where::Branch {
                workspace: "home".to_owned(),
                lineage: "default".to_owned(),
                path: "providers.yaml".to_owned(),
            },
            json!({"file": "branch", "workspace": "home", "lineage": "default",
                   "path": "providers.yaml", "origin": "advance"}),
        ),
    ] {
        let built = config(&at);
        assert_eq!(built[envelope::OP], json!("config"), "{}", at.label());
        assert_eq!(built[envelope::TARGET], target, "{}", at.label());
        assert!(
            built.get("text").is_none(),
            "a read carries no text: {built}"
        );
    }
}

/// **Two destinations name a workspace and three name an engine**, which is
/// what decides whether §8.2 routes the gesture by an address or the pane
/// addresses a channel outright.
#[test]
fn only_the_two_workspace_destinations_are_routed_by_an_address() {
    let branch = Where::Branch {
        workspace: "home".to_owned(),
        lineage: "default".to_owned(),
        path: "providers.yaml".to_owned(),
    };
    for at in [
        Where::Brazen {
            workspace: "home".to_owned(),
        },
        branch.clone(),
    ] {
        assert!(at.addresses_a_workspace(), "{}", at.label());
        assert_eq!(
            envelope::workspace(&config(&at)).as_deref(),
            Some("home"),
            "and the seat reads that address off the nested target"
        );
    }
    for at in [
        Where::LitanyModels,
        Where::Cadence,
        Where::LitanyWorkflow {
            name: "review".to_owned(),
        },
    ] {
        assert!(!at.addresses_a_workspace(), "{}", at.label());
        assert_eq!(envelope::workspace(&config(&at)), None);
    }
    assert_eq!(branch.label(), "default: providers.yaml");
    assert_eq!(Where::LitanyModels.label(), "litany models");
    assert_eq!(
        Where::LitanyWorkflow {
            name: "review".to_owned()
        }
        .label(),
        "workflow review"
    );
    assert_eq!(Where::Cadence.label(), "cadence");
    assert_eq!(
        Where::Brazen {
            workspace: "home".to_owned()
        }
        .label(),
        "brazen"
    );
}
