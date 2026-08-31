//! **What a SECTION says**: its header, the address it dials, and what it has
//! instead of walls when it has none.
//!
//! Split from [`super`] at the line cap on the seam the pane itself has: a
//! section is one channel and a row is one workspace on it. The two fail for
//! different reasons — a section fails about a relationship, a row about a
//! name.

use super::super::{NO_WALLS, NOT_ANSWERED, header, render};
use crate::test_support::window::{own, pane, wall};
use crate::ui::{Channel, Chunk, Model};

/// **Every empty section says which emptiness it is** (bl-08b6): nothing has
/// answered down it yet, this box cannot dial it and knows why off its own
/// files, or the engine answered and holds no workspace.
///
/// The pane used to have one sentence, for an empty ROSTER — and an empty
/// roster is unreachable, because every box holds its own engine's slot
/// whether or not anything is provisioned in it. So the box the sentence was
/// written for got a section header over a blank.
#[test]
fn every_empty_section_says_which_emptiness_it_is() {
    for (held, expected) in [
        (crate::ui::Held::Unheard, NOT_ANSWERED.to_owned()),
        (
            crate::ui::Held::Unheld("nothing provisioned at /home/u/wire".to_owned()),
            "nothing provisioned at /home/u/wire".to_owned(),
        ),
        (crate::ui::Held::Heard, NO_WALLS.to_owned()),
    ] {
        let mut model = Model {
            roster: vec![Chunk { held, ..own() }],
            ..Model::default()
        };
        model.roster[0].walls.clear();
        let painted = pane(|ui| render(ui, &mut model));
        assert!(painted.contains(&expected), "{expected:?}:\n{painted}");
    }
}

/// The header names the local spelling, and the host's beside it where the two
/// differ — a local rename is the remedy for a name collision, and an operator
/// has to be able to see one.
#[test]
fn a_renamed_channel_says_both_names_and_an_unrenamed_one_says_one() {
    let entry = |there: &str| Chunk {
        channel: Channel {
            name: "home".to_owned(),
            named_there: Some(there.to_owned()),
            dials: None,
        },
        ..Chunk::default()
    };
    assert_eq!(
        header(&entry("personal")),
        "home (named \"personal\" on its host)"
    );
    assert_eq!(header(&entry("home")), "home");
    assert_eq!(header(&own()), "(this box's own engine)");
}

/// The engine's currency notes ride with the rows they are about.
#[test]
fn a_channel_says_how_current_its_answer_is_when_the_engine_did() {
    let mut model = Model {
        roster: vec![Chunk {
            stale: Some("derivation 4m behind".to_owned()),
            growth: Some("one grew 3 steps".to_owned()),
            ..own()
        }],
        ..Model::default()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("derivation 4m behind"), "{painted}");
    assert!(painted.contains("one grew 3 steps"), "{painted}");
}

/// **Two dead channels are both heard, each under its own name** (bl-e620).
/// The bar holds one sentence and the last writer wins, so a seat with two
/// unreachable channels was told about one of them permanently and could not
/// discover the other from the glass at all.
#[test]
fn every_unreachable_channel_says_so_on_its_own_section() {
    let mut model = Model {
        roster: vec![
            Chunk {
                channel: Channel {
                    name: "alpha".to_owned(),
                    named_there: Some("alpha".to_owned()),
                    dials: None,
                },
                ..Chunk::default()
            },
            Chunk {
                channel: Channel {
                    name: "beta".to_owned(),
                    named_there: Some("beta".to_owned()),
                    dials: None,
                },
                ..Chunk::default()
            },
        ],
        ..Model::default()
    };
    model.unreachable(
        &model.roster[0].channel.clone(),
        "connect: refused".to_owned(),
    );
    model.unreachable(
        &model.roster[1].channel.clone(),
        "the handshake did not verify".to_owned(),
    );
    let painted = pane(|ui| render(ui, &mut model));
    for expected in [
        "alpha",
        "connect: refused",
        "beta",
        "the handshake did not verify",
    ] {
        assert!(painted.contains(expected), "{expected:?}:\n{painted}");
    }
    assert_eq!(model.notice, None, "the shell-wide bar carries neither");
}

/// **A channel that answers again clears its own sentence**, because an answer
/// spends whatever the section was standing on — so there is nothing to
/// dismiss and nothing that can go stale.
#[test]
fn an_answer_clears_the_channel_s_own_sentence() {
    let mut model = Model {
        roster: vec![own()],
        ..Model::default()
    };
    model.unreachable(&own().channel, "connect: refused".to_owned());
    assert!(pane(|ui| render(ui, &mut model)).contains("connect: refused"));
    model.absorb(
        &own().channel,
        crate::reply::Read::Answer(crate::reply::Reply::Workspaces(
            crate::reply::roster::Workspaces {
                rows: vec![wall("home")],
                stale: None,
                growth: None,
            },
        )),
    );
    let painted = pane(|ui| render(ui, &mut model));
    assert!(!painted.contains("connect: refused"), "{painted}");
    assert!(painted.contains("home"), "{painted}");
}

/// **The rows a dead channel last answered stand under its sentence.** They are
/// the last thing it did say, and they are worth keeping while it is down —
/// REMOTE §8.2's *"that channel's workspaces painted unreachable"*.
#[test]
fn a_dead_channel_keeps_the_walls_it_last_answered_under_its_sentence() {
    let mut model = Model {
        roster: vec![own()],
        ..Model::default()
    };
    model.unreachable(&own().channel, "connect: refused".to_owned());
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("connect: refused"), "{painted}");
    assert!(painted.contains("home"), "{painted}");
}

/// **The section header carries the address it dials** (bl-77df), which is the
/// one fact that explains a duplicate: an entry whose `address` file holds what
/// this box's own engine listens on paints every workspace of that engine
/// twice, under two headers, and the window used to say nothing on either.
/// `lernie entries` has always printed it.
#[test]
fn a_section_header_names_the_address_it_dials() {
    let scratch = crate::test_support::Scratch::new();
    crate::test_support::mint::provisioned(
        &scratch.dir(crate::channel::entries::WIRE),
        "127.0.0.1:7737",
    );
    crate::test_support::mint::provisioned(
        &scratch.path().join(crate::test_support::wire::entry("lab")),
        "127.0.0.1:7737",
    );
    let mut model = Model {
        roster: crate::seat::channels(scratch.path()),
        ..Model::default()
    };
    let painted = pane(|ui| render(ui, &mut model));
    let carrying: Vec<&str> = painted
        .lines()
        .filter(|line| line.contains("127.0.0.1:7737"))
        .collect();
    assert_eq!(
        carrying,
        vec![
            "(this box's own engine) — 127.0.0.1:7737",
            "lab — 127.0.0.1:7737"
        ],
        "both headers name the one listener they share:\n{painted}"
    );
}

/// A channel this box cannot dial has no address to name, so its header does
/// not invent one — the sentence under it is what it has to say.
#[test]
fn a_channel_with_nothing_to_dial_names_no_address() {
    let mut model = Model {
        roster: vec![own()],
        ..Model::default()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(
        painted
            .lines()
            .any(|line| line == "(this box's own engine)"),
        "{painted}"
    );
}
