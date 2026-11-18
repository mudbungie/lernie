//! Removing an entry: two clicks to ask, and what the return does.

use crate::test_support::window::{own, seated, workspace};
use crate::ui::{Aim, Channel, Chunk, Model};

/// An entry's section, heard, holding one workspace.
fn entry(name: &str) -> Chunk {
    Chunk {
        channel: Channel {
            name: name.to_owned(),
            named_there: Some(name.to_owned()),
            dials: Some("engine.example:7737".to_owned()),
        },
        held: crate::ui::Held::Heard,
        workspaces: vec![workspace(name)],
        ..Chunk::default()
    }
}

fn held() -> Model {
    Model {
        roster: vec![own(), entry("lab"), entry("zoo")],
        ..seated()
    }
}

/// **Nothing armed asks nothing; the second click asks once**, and the way
/// out disarms.
#[test]
fn a_removal_is_asked_by_the_second_click_and_not_the_first() {
    let mut model = held();
    assert!(!model.forget(), "nothing armed");
    model.arm_forget("lab");
    model.arm_forget("zoo");
    assert_eq!(model.servers.forgetting.armed.as_deref(), Some("zoo"));
    model.disarm_forget();
    assert!(!model.forget(), "disarmed");
    model.arm_forget("lab");
    assert!(model.forget());
    assert_eq!(model.servers.forgetting.asked, ["lab"]);
    assert_eq!(model.servers.forgetting.armed, None);
}

/// **An entry gone from the disk is gone from the roster**, and the rest
/// stand — this box's own server included, whatever name is given.
#[test]
fn a_removed_entry_leaves_the_roster_and_nothing_else_does() {
    let mut model = held();
    model.forgotten("lab", Ok(()));
    let names: Vec<&str> = model
        .roster
        .iter()
        .map(|chunk| chunk.channel.name.as_str())
        .collect();
    assert_eq!(names, ["(this box's own server)", "zoo"]);
    assert!(model.aim.is_some(), "an aim elsewhere stands");
    model.forgotten("(this box's own server)", Ok(()));
    assert_eq!(model.roster.len(), 2, "the own server is not an entry");
}

/// **Nothing stays aimed down a channel this box no longer holds.**
#[test]
fn removing_the_aimed_entry_puts_the_aim_and_the_selection_down() {
    let mut model = held();
    model.aim = Some(Aim {
        channel: "lab".to_owned(),
        address: "lab".to_owned(),
    });
    model.forgotten("lab", Ok(()));
    assert_eq!(model.aim, None);
    assert_eq!(model.conversation, None);
}

/// **A refusal leaves every row standing and is kept to be said**; the next
/// removal that works clears it.
#[test]
fn a_refused_removal_changes_nothing_and_says_why() {
    let mut model = held();
    model.forgotten("lab", Err("was not removed".to_owned()));
    assert_eq!(model.roster.len(), 3);
    assert_eq!(
        model.servers.forgetting.refused.as_deref(),
        Some("was not removed")
    );
    model.forgotten("lab", Ok(()));
    assert_eq!(model.servers.forgetting.refused, None);
}
