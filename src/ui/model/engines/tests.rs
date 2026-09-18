//! **The accordion**: the order as a pure function, which engine is open on a
//! seat nobody has told, and what an opening aims at.

use super::Engines;
use crate::test_support::window::{own, seated, wall};
use crate::ui::{Aim, Channel, Chunk, Model};

/// The names, as a list a sort reads.
fn names(said: &[&str]) -> Vec<String> {
    said.iter().map(|name| (*name).to_owned()).collect()
}

/// One engine holding the walls it is given.
fn engine(name: &str, walls: &[&str]) -> Chunk {
    Chunk {
        channel: Channel {
            name: name.to_owned(),
            named_there: None,
            dials: None,
        },
        held: crate::ui::Held::Heard,
        walls: walls.iter().map(|wall_name| wall(wall_name)).collect(),
        ..Chunk::default()
    }
}

/// **The open one first, then most recently opened first, the name breaking a
/// tie** — the whole of the order, as a pure function over which is open, what
/// each opening ranks as, and the names.
#[test]
fn the_open_engine_is_first_and_the_rest_follow_the_last_time_each_was_opened() {
    let mut engines = Engines::default();
    engines.opened.insert("beta".to_owned(), 1);
    engines.opened.insert("delta".to_owned(), 2);
    let held = names(&["alpha", "beta", "delta", "gamma"]);
    assert_eq!(
        engines.order(&held, Some("alpha")),
        names(&["alpha", "delta", "beta", "gamma"]),
        "the open one is hoisted and the never-opened pair fall to the bottom by name"
    );
    assert_eq!(
        engines.order(&held, Some("gamma")),
        names(&["gamma", "delta", "beta", "alpha"]),
    );
    assert_eq!(
        engines.order(&held, None),
        names(&["delta", "beta", "alpha", "gamma"]),
        "with nothing open the order is the openings alone"
    );
}

/// **An opening ranks one past the highest anybody holds**, so the engine just
/// opened is the most recently opened by construction — and a second opening
/// of one already open moves it back to the front rather than tying.
#[test]
fn an_opening_outranks_every_opening_before_it() {
    let mut model = Model {
        roster: vec![engine("alpha", &["a"]), engine("beta", &["b"])],
        ..Model::default()
    };
    model.open_engine("alpha");
    model.open_engine("beta");
    assert_eq!(model.engines.open.as_deref(), Some("beta"));
    assert_eq!(
        model.engine_rows()[0].channel.name,
        "beta",
        "the open one paints first"
    );
    model.open_engine("alpha");
    let order: Vec<String> = model
        .engine_rows()
        .iter()
        .map(|chunk| chunk.channel.name.clone())
        .collect();
    assert_eq!(order, names(&["alpha", "beta"]));
}

/// **Which engine is open on a seat nobody has told**: the aim's own, and with
/// no aim the first by name. A name this seat no longer holds is inert, which
/// is the place file's whole rule about a stale place.
#[test]
fn a_seat_nobody_has_told_opens_the_aims_engine_and_otherwise_the_first_by_name() {
    let held = vec![engine("beta", &["b"]), engine("alpha", &["a"])];
    let bare = Model {
        roster: held.clone(),
        ..Model::default()
    };
    assert_eq!(bare.engine_open().as_deref(), Some("alpha"));
    let aimed = Model {
        aim: Some(Aim {
            channel: "beta".to_owned(),
            address: "b".to_owned(),
        }),
        roster: held.clone(),
        ..Model::default()
    };
    assert_eq!(aimed.engine_open().as_deref(), Some("beta"));
    let said = Model {
        engines: Engines {
            open: Some("beta".to_owned()),
            ..Engines::default()
        },
        ..aimed.clone()
    };
    assert_eq!(said.engine_open().as_deref(), Some("beta"));
    let gone = Model {
        engines: Engines {
            open: Some("a channel this seat gave up".to_owned()),
            ..Engines::default()
        },
        ..aimed
    };
    assert_eq!(
        gone.engine_open().as_deref(),
        Some("beta"),
        "a name nothing resolves is inert and the aim answers instead"
    );
    assert_eq!(Model::default().engine_open(), None, "nothing to open");
}

/// **Opening aims the wall last aimed under that engine, and its first by
/// `ordered` where this seat has never aimed one** — so an open engine is
/// never open over nothing.
#[test]
fn opening_aims_the_wall_this_seat_was_last_on_under_it() {
    let mut model = Model {
        roster: vec![engine("alpha", &["second", "first"])],
        ..Model::default()
    };
    model.open_engine("alpha");
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("first".to_owned()),
        "its first by `ordered`, which is by name here"
    );
    model.aim_at("alpha", "second");
    model.open_engine("alpha");
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("second".to_owned()),
        "and the one it was left on once there is one"
    );
}

/// **A wall this seat can no longer address is not aimed at** by an opening,
/// and an engine with no addressable wall at all aims nothing rather than
/// aiming at a name no envelope carries.
#[test]
fn an_opening_aims_nothing_where_there_is_nothing_this_seat_can_address() {
    let unreachable = Chunk {
        channel: Channel {
            name: "elsewhere".to_owned(),
            named_there: Some("theirs".to_owned()),
            dials: None,
        },
        walls: vec![wall("not-ours")],
        ..Chunk::default()
    };
    let mut model = Model {
        roster: vec![unreachable],
        engines: Engines {
            aimed: [("elsewhere".to_owned(), "gone".to_owned())]
                .into_iter()
                .collect(),
            ..Engines::default()
        },
        ..Model::default()
    };
    model.open_engine("elsewhere");
    assert_eq!(model.aim, None);
    // An engine no roster carries is opened and aims nothing: the place file
    // may name one this seat has since given up.
    model.open_engine("a channel this seat gave up");
    assert_eq!(model.aim, None);
}

/// **Aiming records the wall under its engine**, wherever the aim came from —
/// one door, so the two surfaces cannot spell *what aiming means* differently.
#[test]
fn aiming_writes_the_wall_down_under_its_engine() {
    let mut model = seated();
    model.standing = Some(own().channel.name.clone());
    model.aim_at("(this box's own engine)", "home");
    assert_eq!(
        model.engines.aimed.get("(this box's own engine)"),
        Some(&"home".to_owned())
    );
    assert_eq!(model.standing, None, "an aim is where the cursor is");
}
