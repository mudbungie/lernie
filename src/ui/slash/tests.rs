//! The slash verbs: what is offered, what a line parses to, and what firing
//! one does. The menu over the box is `menu`.

/// The menu over the box: what it shows, and what the keys and a click do.
mod menu;
/// One overlay at a time: a word fired from under one puts it down.
mod uncover;

use super::{DELETE, PLAN, REFRESH, fire, matching, parsed, rows, wanting};
use crate::reply::agent::Offer;
use crate::test_support::window::{pinned, seated};
use crate::ui::Model;
use serde_json::json;

/// The words of every row offered on `model`.
fn words(model: &Model) -> Vec<&'static str> {
    rows(model).into_iter().map(|row| row.word).collect()
}

/// **A slash line is the word and the rest**; anything else is no line.
#[test]
fn a_line_parses_to_its_word_and_the_rest() {
    assert_eq!(parsed("/flag it loops"), Some(("flag", "it loops")));
    assert_eq!(parsed("/stop"), Some(("stop", "")));
    assert_eq!(parsed("/"), Some(("", "")));
    assert_eq!(parsed("ship it"), None);
    assert_eq!(parsed(""), None);
}

/// **A row's line is its word and its summary**, and a wire verb's summary is
/// the verb table's own.
#[test]
fn a_row_says_what_lernie_help_says() {
    let model = seated();
    let stop = rows(&model)
        .into_iter()
        .find(|row| row.word == crate::verbs::STOP.word)
        .expect("stop is offered on a seated window");
    assert_eq!(stop.summary, crate::verbs::STOP.summary);
    assert_eq!(
        stop.line(),
        format!("/stop  {}", crate::verbs::STOP.summary)
    );
    assert_eq!(stop.ops, &[crate::verbs::STOP.word]);
}

/// **What is offered follows what is on the glass**: the window's own verbs
/// always, the workspace's once aimed, the conversation's once selected, `plan`
/// only in the start's mode, and the pin by the workspace's rank.
#[test]
fn what_is_offered_follows_the_aim_and_the_selection() {
    let bare = words(&Model::default());
    assert!(bare.contains(&REFRESH) && bare.contains(&crate::ui::queue::WORD));
    assert!(!bare.contains(&PLAN) && !bare.contains(&crate::ui::records::WORD));
    assert!(!bare.contains(&crate::verbs::PIN.word));

    let mut starting = seated();
    starting.conversation = None;
    let start = words(&starting);
    assert!(start.contains(&PLAN) && start.contains(&crate::verbs::PIN.word));
    assert!(!start.contains(&crate::ui::records::WORD));

    let talking = words(&seated());
    for word in [
        crate::ui::records::WORD,
        crate::verbs::INTERRUPT.word,
        crate::verbs::STOP.word,
        crate::verbs::NUDGE.word,
        crate::verbs::FLAG.word,
        DELETE,
        crate::ui::enroll::WORD,
        crate::ui::unmake::WORD,
    ] {
        assert!(talking.contains(&word), "{word}: {talking:?}");
    }
    assert!(!talking.contains(&PLAN));
    assert!(words(&pinned()).contains(&crate::verbs::UNPIN.word));
}

/// **Only what the server offers on the turn is listed** — `stop` and
/// `interrupt` while it runs, `nudge` while it rests — and everything until
/// the server has answered.
#[test]
fn the_turn_s_verbs_follow_the_server_s_offers() {
    let mut model = seated();
    let mut row = crate::test_support::window::panes::records::own_row();
    row.offers = vec![Offer::Nudge];
    model.records.agent = Some(row);
    let offered = words(&model);
    assert!(offered.contains(&crate::verbs::NUDGE.word));
    assert!(!offered.contains(&crate::verbs::STOP.word));
    assert!(!offered.contains(&crate::verbs::INTERRUPT.word));
}

/// **The typed word narrows the rows** to those it starts, and a bare slash
/// is all of them.
#[test]
fn the_word_narrows_the_menu() {
    let model = seated();
    assert_eq!(matching(&model, "/").len(), rows(&model).len());
    let re: Vec<&str> = matching(&model, "/re")
        .into_iter()
        .map(|row| row.word)
        .collect();
    assert_eq!(
        re,
        vec!["records", "revoke", "restore", "retarget", "refresh"]
    );
    assert!(matching(&model, "/zzz").is_empty());
    assert!(matching(&model, "no slash").is_empty());
}

/// **Firing composes what the control it replaced composed**, through the
/// same verb rows, and a verb whose parameter is missing fires nothing.
#[test]
fn firing_composes_the_envelope_the_strip_composed() {
    let mut model = seated();
    assert!(fire(&mut model, crate::verbs::FLAG.word, "it loops"));
    assert!(fire(&mut model, crate::verbs::STOP.word, ""));
    assert!(fire(&mut model, crate::verbs::INTERRUPT.word, "no, this"));
    assert!(fire(&mut model, DELETE, "port the paint probe"));
    assert!(fire(&mut model, crate::verbs::NUDGE.word, ""));
    assert!(fire(&mut model, crate::verbs::REVOKE.word, ""));
    assert!(fire(&mut model, crate::verbs::RESTORE.word, ""));
    assert!(fire(&mut model, crate::verbs::RETARGET.word, ""));
    let ops: Vec<_> = model
        .outbox
        .iter()
        .map(|posted| posted.envelope["op"].clone())
        .collect();
    assert_eq!(
        ops,
        vec![
            json!("flag"),
            json!("stop"),
            json!("interrupt"),
            json!("delete-agent"),
            json!("nudge"),
            json!("revoke"),
            json!("restore"),
            json!("retarget")
        ]
    );
    assert_eq!(model.outbox[0].envelope["reason"], json!("it loops"));
    assert_eq!(model.outbox[2].envelope["content"], json!("no, this"));
    assert_eq!(
        model.outbox[3].envelope["typed"],
        json!("port the paint probe")
    );
    let before = model.outbox.len();
    assert!(
        !fire(&mut model, crate::verbs::FLAG.word, ""),
        "a flag needs its reason"
    );
    assert!(
        !fire(&mut model, crate::verbs::INTERRUPT.word, ""),
        "a cut needs its words"
    );
    assert!(!fire(&mut model, "no-such-verb", ""));
    assert_eq!(model.outbox.len(), before);
    // **`/stop children` takes the subtree**, as the row's summary says
    // (bl-0740); the bare word stops the root alone.
    assert!(fire(&mut model, crate::verbs::STOP.word, "children"));
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["children"].clone()),
        Some(json!(true))
    );
    assert_eq!(model.outbox[1].envelope.get("children"), None);
}

/// **`/delete` is armed by the conversation's own name and nothing else**
/// (bl-df70): the wire deletes a leaf on any word, an empty one included, so
/// a missing or mistyped name must compose no envelope at all — and every
/// verb that fires nothing for a missing word says why.
#[test]
fn a_missing_or_wrong_word_fires_nothing_and_says_why() {
    let mut model = seated();
    for (word, rest) in [
        (DELETE, ""),
        (DELETE, "port the paint"),
        (crate::verbs::FLAG.word, ""),
        (crate::verbs::INTERRUPT.word, ""),
    ] {
        let why = wanting(&model, word, rest).expect("says why");
        assert!(why.starts_with(&format!("/{word} ")), "{why}");
        assert!(!fire(&mut model, word, rest), "{word} {rest:?}");
    }
    assert!(model.outbox.is_empty());
    assert!(
        wanting(&model, DELETE, "").is_some_and(|why| why.ends_with("port the paint probe")),
        "the refusal states the name that arms it"
    );
    assert!(fire(&mut model, DELETE, "  port the paint probe "));
    assert_eq!(
        model.outbox[0].envelope["typed"],
        json!("port the paint probe")
    );
    assert_eq!(wanting(&model, crate::verbs::STOP.word, ""), None);
    model.conversation = Some("ghost".to_owned());
    assert!(
        wanting(&model, DELETE, "").is_some_and(|why| why.ends_with(" ghost")),
        "a conversation the list does not carry is named by its id, as the server names it"
    );
    model.conversation = None;
    assert_eq!(
        wanting(&model, DELETE, ""),
        None,
        "nothing selected, nothing to arm"
    );
}

/// **Every opener opens its pane**, and the ones with a subject spend it:
/// `find` with words searches, `pin` posts the rank, `plan` toggles,
/// `refresh` asks the roster again.
#[test]
fn every_opener_opens_what_its_word_names() {
    let mut model = seated();
    for (word, open) in [
        (
            crate::ui::queue::WORD,
            (|m: &Model| m.showing(crate::ui::Listing::Queue)) as fn(&Model) -> bool,
        ),
        (crate::ui::records::WORD, |m: &Model| {
            m.showing(crate::ui::Listing::Records)
        }),
        (crate::ui::tuning::WORD, |m: &Model| m.tuning.is_some()),
        (crate::ui::tuning::MODEL, |m: &Model| m.dropup.is_some()),
        (crate::ui::login::WORD, |m: &Model| m.login.is_some()),
        (crate::ui::clients::WORD, |m: &Model| {
            m.showing(crate::ui::Listing::Clients)
        }),
        (crate::ui::config::WORD, |m: &Model| m.configuring.is_some()),
        (crate::ui::fleet::WORD, |m: &Model| m.fleet.is_some()),
        (crate::ui::unmake::WORD, |m: &Model| m.unmaking.is_some()),
        (crate::ui::commands::WORD, Model::commanding),
        (crate::ui::doctor::WORD, Model::doctoring),
        (crate::ui::settings::WORD, |m: &Model| {
            m.lookup == Some(crate::ui::Lookup::Settings)
        }),
        (crate::ui::trail::WORD, Model::trailing),
        (crate::ui::board::WORD, Model::boarding),
        (crate::ui::prices::WORD, Model::pricing),
        (crate::ui::find::WORD, Model::finding),
        (crate::ui::enroll::WORD, |m: &Model| m.enroll.is_some()),
    ] {
        assert!(fire(&mut model, word, ""), "{word}");
        assert!(open(&model), "{word} opens its pane");
        model.escape();
        assert!(!model.covered(), "{word}: Escape puts it down");
    }
    let mut model = seated();
    assert!(fire(&mut model, crate::ui::find::WORD, "needle"));
    assert_eq!(model.needle, "needle");
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["op"].clone()),
        Some(json!("search"))
    );
    assert!(fire(&mut model, crate::verbs::PIN.word, ""));
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["op"].clone()),
        Some(json!("pin"))
    );
    let mut pinned = pinned();
    assert!(fire(&mut pinned, crate::verbs::UNPIN.word, ""));
    assert_eq!(
        pinned.outbox.last().map(|p| p.envelope["op"].clone()),
        Some(json!("unpin"))
    );
    let before = model.outbox.len();
    assert!(fire(&mut model, REFRESH, ""));
    assert!(model.outbox.len() > before, "refresh asks again");
    let mut starting = seated();
    starting.conversation = None;
    assert!(fire(&mut starting, PLAN, ""));
    assert!(starting.plan);
    assert!(fire(&mut starting, PLAN, ""));
    assert!(!starting.plan);
}
