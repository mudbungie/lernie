//! **The accordion**: the order as a pure function, which server is open on a
//! seat nobody has told, and what an opening aims at.

use super::Servers;
use crate::test_support::window::{own, seated, workspace};
use crate::ui::{Aim, Channel, Chunk, Model};

/// The names, as a list a sort reads.
fn names(said: &[&str]) -> Vec<String> {
    said.iter().map(|name| (*name).to_owned()).collect()
}

/// One server holding the workspaces it is given.
fn server(name: &str, workspaces: &[&str]) -> Chunk {
    Chunk {
        channel: Channel {
            name: name.to_owned(),
            named_there: None,
            dials: None,
        },
        held: crate::ui::Held::Heard,
        workspaces: workspaces
            .iter()
            .map(|workspace_name| workspace(workspace_name))
            .collect(),
        ..Chunk::default()
    }
}

/// **The open one first, then most recently opened first, the name breaking a
/// tie** — the whole of the order, as a pure function over which is open, what
/// each opening ranks as, and the names.
#[test]
fn the_open_server_is_first_and_the_rest_follow_the_last_time_each_was_opened() {
    let mut servers = Servers::default();
    servers.opened.insert("beta".to_owned(), 1);
    servers.opened.insert("delta".to_owned(), 2);
    let held = names(&["alpha", "beta", "delta", "gamma"]);
    assert_eq!(
        servers.order(&held, Some("alpha")),
        names(&["alpha", "delta", "beta", "gamma"]),
        "the open one is hoisted and the never-opened pair fall to the bottom by name"
    );
    assert_eq!(
        servers.order(&held, Some("gamma")),
        names(&["gamma", "delta", "beta", "alpha"]),
    );
    assert_eq!(
        servers.order(&held, None),
        names(&["delta", "beta", "alpha", "gamma"]),
        "with nothing open the order is the openings alone"
    );
}

/// **An opening ranks one past the highest anybody holds**, so the server just
/// opened is the most recently opened by construction — and a second opening
/// of one already open moves it back to the front rather than tying.
#[test]
fn an_opening_outranks_every_opening_before_it() {
    let mut model = Model {
        roster: vec![server("alpha", &["a"]), server("beta", &["b"])],
        ..Model::default()
    };
    model.open_server("alpha");
    model.open_server("beta");
    assert_eq!(model.servers.open.as_deref(), Some("beta"));
    assert_eq!(
        model.server_rows()[0].name(),
        "beta",
        "the open one paints first"
    );
    model.open_server("alpha");
    let order: Vec<String> = model
        .server_rows()
        .iter()
        .map(|server| server.name().to_owned())
        .collect();
    assert_eq!(order, names(&["alpha", "beta"]));
}

/// **Which server is open on a seat nobody has told**: the aim's own, and with
/// no aim the first by name. A name this seat no longer holds is inert, which
/// is the place file's whole rule about a stale place.
#[test]
fn a_seat_nobody_has_told_opens_the_aims_server_and_otherwise_the_first_by_name() {
    let held = vec![server("beta", &["b"]), server("alpha", &["a"])];
    let bare = Model {
        roster: held.clone(),
        ..Model::default()
    };
    assert_eq!(bare.server_open().as_deref(), Some("alpha"));
    let aimed = Model {
        aim: Some(Aim {
            channel: "beta".to_owned(),
            address: "b".to_owned(),
        }),
        roster: held.clone(),
        ..Model::default()
    };
    assert_eq!(aimed.server_open().as_deref(), Some("beta"));
    let said = Model {
        servers: Servers {
            open: Some("beta".to_owned()),
            ..Servers::default()
        },
        ..aimed.clone()
    };
    assert_eq!(said.server_open().as_deref(), Some("beta"));
    let gone = Model {
        servers: Servers {
            open: Some("a channel this seat gave up".to_owned()),
            ..Servers::default()
        },
        ..aimed
    };
    assert_eq!(
        gone.server_open().as_deref(),
        Some("beta"),
        "a name nothing resolves is inert and the aim answers instead"
    );
    assert_eq!(Model::default().server_open(), None, "nothing to open");
}

/// **Opening aims the workspace last aimed under that server, and its first by
/// `ordered` where this seat has never aimed one** — so an open server is
/// never open over nothing.
#[test]
fn opening_aims_the_workspace_this_seat_was_last_on_under_it() {
    let mut model = Model {
        roster: vec![server("alpha", &["second", "first"])],
        ..Model::default()
    };
    model.open_server("alpha");
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("first".to_owned()),
        "its first by `ordered`, which is by name here"
    );
    model.aim_at("alpha", "second");
    model.open_server("alpha");
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("second".to_owned()),
        "and the one it was left on once there is one"
    );
}

/// **A workspace this seat can no longer address is not aimed at** by an opening,
/// and a server with no addressable workspace at all aims nothing rather than
/// aiming at a name no envelope carries.
#[test]
fn an_opening_aims_nothing_where_there_is_nothing_this_seat_can_address() {
    let unreachable = Chunk {
        channel: Channel {
            name: "elsewhere".to_owned(),
            named_there: Some("theirs".to_owned()),
            dials: None,
        },
        workspaces: vec![workspace("not-ours")],
        ..Chunk::default()
    };
    let mut model = Model {
        roster: vec![unreachable],
        servers: Servers {
            aimed: [("elsewhere".to_owned(), "gone".to_owned())]
                .into_iter()
                .collect(),
            ..Servers::default()
        },
        ..Model::default()
    };
    model.open_server("elsewhere");
    assert_eq!(model.aim, None);
    // A server no roster carries is opened and aims nothing: the place file
    // may name one this seat has since given up.
    model.open_server("a channel this seat gave up");
    assert_eq!(model.aim, None);
}

/// **Aiming records the workspace under its server**, wherever the aim came from —
/// one door, so the two surfaces cannot spell *what aiming means* differently.
#[test]
fn aiming_writes_the_workspace_down_under_its_server() {
    let mut model = seated();
    model.standing = Some(crate::ui::Stand::Server(own().channel.name.clone()));
    model.aim_at("(this box's own server)", "home");
    assert_eq!(
        model.servers.aimed.get("(this box's own server)"),
        Some(&"home".to_owned())
    );
    assert_eq!(model.standing, None, "an aim is where the cursor is");
}

/// **`+ new` aims its workspace, drops the selection and asks for the box**
/// (DESIGN §4.42 item 7) — and the server's opening ranks: a start is a use.
#[test]
fn beginning_on_a_workspace_aims_it_ranks_its_server_and_asks_for_the_box() {
    let mut model = Model {
        roster: vec![own(), server("lab", &["bench", "annex"])],
        ..seated()
    };
    model.begin_at(&Aim {
        channel: "lab".to_owned(),
        address: "annex".to_owned(),
    });
    assert_eq!(model.server_open().as_deref(), Some("lab"));
    assert_eq!(
        model.servers.opened.get("lab").copied(),
        Some(1),
        "an opening ranks, whichever control made it"
    );
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("annex".to_owned())
    );
    assert_eq!(model.conversation, None);
    assert_eq!(model.fill, Some(crate::ui::Fill::Goal));
}
