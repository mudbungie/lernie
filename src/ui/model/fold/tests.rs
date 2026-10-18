//! The fold: two channels dialling one address are one server, counted and
//! painted once, with every name it is held under beside it (bl-13b9).

use super::ALSO;
use crate::test_support::window::{own, painted, seated, workspace};
use crate::ui::{Channel, Chunk, Held, Model};

/// A workspace entry naming `there` on the server this box's own server is.
fn entry(name: &str, there: &str, dials: &str) -> Chunk {
    Chunk {
        channel: Channel {
            name: name.to_owned(),
            named_there: Some(there.to_owned()),
            dials: Some(dials.to_owned()),
        },
        workspaces: vec![crate::reply::roster::WsRow {
            attention: 2,
            ..workspace("home")
        }],
        ..own()
    }
}

/// A seat whose own server and one entry both dial `127.0.0.1:7747`, the
/// entry held FIRST — the lead is still the box's own, which addresses every
/// workspace by its own name.
fn doubled() -> Model {
    let mut model = seated();
    model.conversation = None;
    model.roster[0].channel.dials = Some("127.0.0.1:7747".to_owned());
    model.roster[0].workspaces[0].attention = 2;
    model
        .roster
        .insert(0, entry("annex", "home", "127.0.0.1:7747"));
    model
}

/// **One server, its names beside it, and each workspace once**; the aim goes
/// down the lead, and an unknown address folds nothing.
#[test]
fn two_channels_dialling_one_address_fold_into_one_server() {
    let model = doubled();
    let servers = model.folded();
    assert_eq!(servers.len(), 1);
    let one = &servers[0];
    assert_eq!(one.name(), "(this box's own server)");
    assert!(one.holds("annex"));
    assert_eq!(one.workspaces().len(), 1);
    assert_eq!(one.held(), Held::Heard);
    assert!(
        one.header().ends_with(&format!("{ALSO} annex")),
        "{}",
        one.header()
    );
    let aim = one.aim(&one.workspaces()[0]).expect("addressable");
    assert_eq!(aim.channel, "(this box's own server)");
    assert_eq!(model.lead_of("annex").as_deref(), Some(one.name()));
    let mut apart = doubled();
    apart.roster[1].channel.dials = None;
    apart.roster[0].channel.dials = None;
    assert_eq!(apart.folded().len(), 2, "no address, no fold");
    assert_eq!(apart.lead_of("nowhere"), None);
    assert_eq!(apart.server_of("nowhere"), "nowhere");
    assert_eq!(model.server_of("annex"), one.name());
}

/// **On the glass**: the seat card lists the server once, its names beside
/// it, and the seat-wide count says two waiting, not four.
#[test]
fn the_seat_card_and_the_waiting_count_hold_one_server_once() {
    let mut model = doubled();
    let text = painted(&mut model);
    assert!(
        text.contains("1 workspace, 2 conversations, 2 waiting"),
        "{text}"
    );
    assert_eq!(text.matches("1 workspace,").count(), 1, "{text}");
    assert!(text.contains("2 waiting"), "{text}");
    assert!(!text.contains("4 waiting"), "{text}");
}

/// **A reached name outranks a broken one**, and a fold none of whose names
/// answered says the first sentence it has.
#[test]
fn an_server_is_reached_if_any_name_answered() {
    let mut model = doubled();
    model.roster[0].held = Held::Unheld("no leaf".to_owned());
    assert_eq!(model.folded()[0].held(), Held::Heard);
    model.roster[1].held = Held::Unheard;
    assert_eq!(model.folded()[0].held(), Held::Unheld("no leaf".to_owned()));
    model.roster[0].held = Held::Unheard;
    assert_eq!(model.folded()[0].held(), Held::Unheard);
}

/// `rows` answered as `channel`'s queue.
fn answer(model: &mut Model, channel: &str, rows: Vec<crate::reply::queue::QueueRow>) {
    let channel = model
        .roster
        .iter()
        .find(|chunk| chunk.channel.name == channel)
        .map_or_else(
            || Channel {
                name: channel.to_owned(),
                named_there: None,
                dials: None,
            },
            |chunk| chunk.channel.clone(),
        );
    model.absorb(
        &channel,
        crate::reply::Read::Answer(crate::reply::Reply::Attention(rows)),
    );
}

/// **The queue folds as the card does** (bl-52d2): both names answer the
/// server's whole queue, and the pane holds one section with each row once —
/// beside the first channel that can address its workspace, so a row only the
/// lead can reach goes down the lead.
#[test]
fn the_queue_holds_one_server_s_rows_once() {
    use crate::test_support::window::waiting;
    let mut model = doubled();
    model.roster[1].workspaces.push(workspace("far"));
    let rows = vec![waiting("home", "c-1"), waiting("far", "c-2")];
    answer(&mut model, "annex", rows.clone());
    answer(&mut model, "(this box's own server)", rows);
    let sections = model.sections();
    assert_eq!(sections.len(), 1);
    assert!(sections[0].header.ends_with(&format!("{ALSO} annex")));
    let down: Vec<(&str, &str)> = sections[0]
        .rows
        .iter()
        .map(|(channel, row)| (channel.name.as_str(), row.agent.as_str()))
        .collect();
    assert_eq!(
        down,
        [("annex", "c-1"), ("(this box's own server)", "c-2")],
        "each row once, down a channel that reaches it"
    );
    answer(&mut model, "elsewhere", vec![waiting("x", "c-9")]);
    assert_eq!(
        model.sections()[1].header,
        "elsewhere",
        "an unheld channel stands alone"
    );
}

/// **On the glass**: the open queue says each asking conversation once.
#[test]
fn the_open_queue_paints_each_asking_row_once() {
    use crate::test_support::window::waiting;
    let mut model = doubled();
    let rows = vec![waiting("home", "c-1")];
    answer(&mut model, "annex", rows.clone());
    answer(&mut model, "(this box's own server)", rows);
    model.begin_queue();
    let text = painted(&mut model);
    assert_eq!(text.matches("c-1").count(), 1, "{text}");
    assert_eq!(
        text.matches(ALSO).count(),
        2,
        "the card and one section: {text}"
    );
}
