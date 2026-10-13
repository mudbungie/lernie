//! **The table**: every row this seat offers right now, read off what is
//! aimed at and selected (`super`).

use super::{DELETE, PLAN, REFRESH, Row};
use crate::reply::agent::Offer;
use crate::ui::Model;

/// What a row says when the verb's own table says nothing about it.
const NO_SUMMARY: &str = "";

/// The summary a wire verb's own table row carries, so the menu says what
/// `lernie help` says and there is one text to keep true.
fn said(word: &str) -> &'static str {
    crate::verbs::find(word).map_or(NO_SUMMARY, |verb| verb.summary)
}

/// The aimed wall's own row, where the roster holds one.
fn aimed(model: &Model) -> Option<&crate::reply::roster::WsRow> {
    let aim = model.aim.as_ref()?;
    model
        .roster
        .iter()
        .find(|chunk| chunk.channel.name == aim.channel)?
        .walls
        .iter()
        .find(|wall| wall.workspace == aim.address)
}

/// Whether the engine offers `act` on the selected conversation — and every
/// act where it has not answered about it yet.
fn offered(model: &Model, act: Offer) -> bool {
    model.agent().is_none_or(|row| row.offers.contains(&act))
}

/// **Every row this seat offers right now**, in the order they are read: the
/// selected conversation's, the aimed wall's, then the window's own.
pub fn rows(model: &Model) -> Vec<Row> {
    let mut out = Vec::new();
    let addressable = model.conversation.is_some() && model.pending().is_none();
    if addressable {
        conversation_rows(model, &mut out);
    } else if model.aim.is_some() {
        out.push(Row::new(PLAN, "the next start is a plan", &[]));
    }
    if let Some(wall) = aimed(model) {
        wall_rows(wall.pinned.is_some(), &mut out);
    }
    window_rows(&mut out);
    out
}

/// The verbs on the selected conversation: its ledger, the acts on its turn
/// the engine offers, and the acts on it as an object.
fn conversation_rows(model: &Model, out: &mut Vec<Row>) {
    use crate::verbs;
    out.push(Row::new(
        crate::ui::records::WORD,
        "the conversation's own ledger",
        &[
            verbs::STEPS.word,
            verbs::FILES.word,
            verbs::RAIL.word,
            verbs::GOVERNING.word,
            verbs::AGENT.word,
            verbs::INBOX.word,
        ],
    ));
    if offered(model, Offer::Stop) {
        out.push(Row::new(
            verbs::INTERRUPT.word,
            said(verbs::INTERRUPT.word),
            &[verbs::INTERRUPT.word],
        ));
        out.push(Row::new(
            verbs::STOP.word,
            said(verbs::STOP.word),
            &[verbs::STOP.word],
        ));
    }
    if offered(model, Offer::Nudge) {
        out.push(Row::new(
            verbs::NUDGE.word,
            said(verbs::NUDGE.word),
            &[verbs::NUDGE.word],
        ));
    }
    out.push(Row::new(
        verbs::REVOKE.word,
        verbs::REVOKE.summary,
        &[verbs::REVOKE.word],
    ));
    out.push(Row::new(
        verbs::RESTORE.word,
        verbs::RESTORE.summary,
        &[verbs::RESTORE.word],
    ));
    out.push(Row::new(
        verbs::RETARGET.word,
        verbs::RETARGET.summary,
        &[verbs::RETARGET.word],
    ));
    out.push(Row::new(
        verbs::FLAG.word,
        verbs::FLAG.summary,
        &[verbs::FLAG.word],
    ));
    out.push(Row::new(
        DELETE,
        verbs::DELETE_AGENT.summary,
        &[verbs::DELETE_AGENT.word],
    ));
}

/// The verbs on the aimed wall: its rank first, then its panes, the unmaking
/// last (DESIGN §4.20).
fn wall_rows(pinned: bool, out: &mut Vec<Row>) {
    use crate::verbs;
    if pinned {
        out.push(Row::new(
            verbs::UNPIN.word,
            verbs::UNPIN.summary,
            &[verbs::UNPIN.word],
        ));
    } else {
        out.push(Row::new(
            verbs::PIN.word,
            verbs::PIN.summary,
            &[verbs::PIN.word],
        ));
    }
    out.push(Row::new(
        crate::ui::enroll::WORD,
        verbs::ENROLL.summary,
        &[],
    ));
    out.push(Row::new(
        crate::ui::tuning::WORD,
        said(verbs::ROLES.word),
        &[verbs::ROLES.word],
    ));
    out.push(Row::new(
        crate::ui::tuning::MODEL,
        "choose the model the worker runs on — Ctrl+O",
        &[verbs::MODELS.word],
    ));
    out.push(Row::new(
        crate::ui::login::WORD,
        said(verbs::PROVIDERS.word),
        &[verbs::PROVIDERS.word],
    ));
    out.push(Row::new(
        crate::ui::clients::WORD,
        said(verbs::CLIENTS.word),
        &[verbs::CLIENTS.word],
    ));
    out.push(Row::new(
        crate::ui::config::WORD,
        said(verbs::LINEAGES.word),
        &[verbs::LINEAGES.word],
    ));
    out.push(Row::new(
        crate::ui::fleet::WORD,
        "the fleet: run this wall's balls and watch them",
        &[verbs::SCIENCE.word, verbs::WORK_DIFF.word],
    ));
    out.push(Row::new(
        crate::ui::unmake::WORD,
        verbs::DELETE_WORKSPACE.summary,
        &[],
    ));
}

/// The window's own verbs: the ones whose subject is every engine this seat
/// reaches.
fn window_rows(out: &mut Vec<Row>) {
    use crate::verbs;
    out.push(Row::new(
        REFRESH,
        said(verbs::WORKSPACES.word),
        &[verbs::WORKSPACES.word],
    ));
    out.push(Row::new(
        crate::ui::queue::WORD,
        said(verbs::ATTENTION.word),
        &[verbs::ATTENTION.word],
    ));
    out.push(Row::new(
        crate::ui::commands::WORD,
        verbs::HELP.summary,
        &[verbs::HELP.word],
    ));
    out.push(Row::new(
        crate::ui::doctor::WORD,
        crate::verbs::doors::DOCTOR.summary,
        &[verbs::DOCTOR],
    ));
    out.push(Row::new(
        crate::ui::trail::WORD,
        crate::verbs::doors::OPS.summary,
        &[verbs::OPS],
    ));
    out.push(Row::new(
        crate::ui::board::WORD,
        "the balls every engine holds, and the aimed wall's",
        &[
            verbs::BALLS.word,
            verbs::BOARD.word,
            verbs::WORKSPACE_BALLS.word,
            verbs::MARKS.word,
        ],
    ));
    out.push(Row::new(
        crate::ui::prices::WORD,
        said(verbs::PRICES.word),
        &[verbs::PRICES.word, verbs::PROVIDERS.word],
    ));
    out.push(Row::new(
        crate::ui::find::WORD,
        said(verbs::SEARCH.word),
        &[],
    ));
    out.push(Row::new(
        crate::ui::switcher::WORD,
        "every engine, its walls and their conversations — Ctrl+X",
        &[],
    ));
}
