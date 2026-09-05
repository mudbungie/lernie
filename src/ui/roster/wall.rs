//! **One wall's row**, and the seven per-wall controls that hang off the aimed
//! one.
//!
//! Split from [`super`] at the design-time budget on the seam that module's own
//! doc draws twice over: it is *the channels and their sections*, `acts` is
//! *the ops whose subject is every channel*, and this is *one workspace* — the
//! line it wears, whether this seat can address it at all, and what an operator
//! can do to it once it is aimed at. The first changes when a section grows a
//! sentence; this one when a wall-level op lands a control.

use crate::reply::roster::WsRow;
use crate::ui::{Chunk, Model};

/// **What a row this seat cannot address says instead of being hidden.**
///
/// Dropping it would hide a workspace the operator has; addressing it by the
/// entry's leaf would aim a gesture at a different wall. So it is painted, and
/// painted as what it is.
///
/// The wording states the FACT rather than a verdict (bl-77df). It used to read
/// *"this seat holds no name for it"*, which lands beside a perfectly correct
/// provisioning and reads as an error about the row above it. What is actually
/// true is structural: an entry directory names one workspace, the channel
/// enumerates every workspace that client is registered in, and the extras have
/// no entry of their own — so no envelope this seat can write reaches them.
pub const NO_NAME_HERE: &str = "no entry here names it, so nothing typed here can address it";

/// **The word on the control that floats this wall to the front of the strip**,
/// offered on an aimed row that is not pinned.
pub const PIN: &str = "pin to the front";
/// **And the one that takes it back out**, offered where it is pinned. Two
/// words rather than one that toggles, because the two ops are assertions: the
/// control names the act it fires (`crate::verbs::workspace`).
pub const UNPIN: &str = "unpin";

/// One wall: selectable when this seat can address it, a plain line when it
/// cannot.
///
/// **The six per-wall controls hang off the aimed row and off no other**,
/// because
/// an enrollment mints the pair `(client, workspace)` and the workspace is
/// exactly what an aim is. Offering it on every row would be offering it before
/// the operator had said which wall — and the answer to that question is
/// already on the screen, once.
pub fn render(ui: &mut egui::Ui, model: &mut Model, chunk: &Chunk, row: &WsRow, reveal: bool) {
    let Some(address) = chunk.channel.address(row) else {
        ui.label(format!("{}  — {NO_NAME_HERE}", line(row)));
        return;
    };
    let aimed = model.aimed_at(&chunk.channel.name, Some(&address));
    let seat = ui.selectable_label(aimed, line(row));
    // **The read this gesture reaches**, not the one that painted the row
    // (yog's `docs/PARITY.md` §2: the interactable a query owes a seat is the
    // affordance that reaches the view it populates). Aiming at a wall is what
    // makes this seat read that wall's conversations.
    crate::ui::act::tag(&seat, &[crate::verbs::CONVERSATIONS.word]);
    if aimed && reveal {
        seat.scroll_to_me(None);
    }
    if seat.clicked() {
        model.aim_at(&chunk.channel.name.clone(), &address);
    }
    // **All seven per-wall controls hang off the aimed row and off no other**,
    // and all seven stand down while a pane already covers the conversation:
    // what they open would replace what is standing there, so offering them is
    // offering to lose it without saying so.
    if !aimed || model.covered() {
        return;
    }
    // **The pin is a control whose WORD and op follow the row's own rank**
    // (bl-7782), because the wire's two ops are assertions and not a toggle:
    // a pinned row is offered the unpin and an unpinned one the pin, so the
    // control always names the act it fires. Reading the rank off the row is
    // reading what is on the glass rather than holding a second opinion about
    // a list only the engine keeps.
    let pinned = row.pinned.is_some();
    let strip = ui.button(if pinned { UNPIN } else { PIN });
    crate::ui::act::tag(
        &strip,
        &[if pinned {
            crate::verbs::UNPIN.word
        } else {
            crate::verbs::PIN.word
        }],
    );
    if strip.clicked() {
        model.post_pin(!pinned);
    }
    if ui.button(crate::ui::enroll::OPEN).clicked() {
        model.begin_enrollment();
    }
    // **The read this gesture reaches**, exactly as the wall's own seat above
    // carries the conversation list's: opening the tuning pane is what makes
    // this seat read that wall's roles (`crate::state::Standing`), and the read
    // has no control of its own.
    let tune = ui.button(crate::ui::tuning::OPEN);
    crate::ui::act::tag(&tune, &[crate::verbs::ROLES.word]);
    if tune.clicked() {
        model.begin_tuning();
    }
    // **The sign-in hangs beside the tuning control**, because both are about
    // the wall's own configuration and neither destroys anything. It carries
    // the read the gesture reaches, exactly as the tuning control does:
    // opening the login pane is what makes this seat read that wall's provider
    // table (`crate::state::Standing`), and the read has no control of its own.
    let sign = ui.button(crate::ui::login::OPEN);
    crate::ui::act::tag(&sign, &[crate::verbs::PROVIDERS.word]);
    if sign.clicked() {
        model.begin_login();
    }
    // **The machines hang beside the sign-in**, because a wall's registered
    // clients are the third fact about its configuration and reading them
    // destroys nothing (bl-e53c). It carries the read the gesture reaches, as
    // the two above do: opening the pane is what makes this seat ask that wall
    // which machines it holds (`crate::state::Standing`), and the read has no
    // control of its own.
    let machines = ui.button(crate::ui::clients::OPEN);
    crate::ui::act::tag(&machines, &[crate::verbs::CLIENTS.word]);
    if machines.clicked() {
        model.begin_clients();
    }
    // **The config pane hangs beside them**, the fourth read of a wall's own
    // configuration and the last that destroys nothing (bl-5c53). It carries
    // the read the gesture reaches, as the three above do.
    let files = ui.button(crate::ui::config::OPEN);
    crate::ui::act::tag(&files, &[crate::verbs::LINEAGES.word]);
    if files.clicked() {
        model.begin_configuring();
    }
    // **The fleet hangs after the four reads and before the unmaking**
    // (bl-a43a), which is where an act that STARTS things belongs: it is the
    // one per-wall control that spawns drones and spends money, and it is not
    // the unmaking. It carries no `act:` token, because what it opens is a
    // pane and each of its five ACTS is tagged on the control inside that
    // fires one — the division `enroll a box…` keeps with `mint`. What it
    // does carry is its two READS, exactly as the tuning and login controls
    // carry theirs: opening the pane is what makes this seat ask that wall for
    // its attempts and for what its agents changed
    // (`crate::state::Standing`), and neither read has a control of its own.
    let drones = ui.button(crate::ui::fleet::OPEN);
    crate::ui::act::tag(
        &drones,
        &[crate::verbs::SCIENCE.word, crate::verbs::WORK_DIFF.word],
    );
    if drones.clicked() {
        model.begin_fleet();
    }
    // **The unmaking hangs here and LAST**, under the two controls that make
    // and change things, because that is the order a destructive act belongs in
    // wherever it is offered beside others (DESIGN §4.20). It carries no `act:`
    // token: what it opens is a pane, and the op is tagged on the control
    // inside it that actually fires one (`crate::ui::unmake`) — the same
    // division `enroll a box…` keeps with `mint`.
    if ui.button(crate::ui::unmake::OPEN).clicked() {
        model.begin_unmaking();
    }
}

/// One wall's line: what it is called, how it is classified, and its rollups.
/// The two rollups are stated only when they are non-zero — a roster of `0
/// waiting` on every row teaches nothing and costs the one that says `3`.
pub fn line(row: &WsRow) -> String {
    let mut said = vec![format!(
        "{}  ({})  {} conversations",
        row.workspace,
        row.kind.label(),
        row.agents
    )];
    if row.attention > 0 {
        said.push(format!("{} waiting", row.attention));
    }
    if row.running {
        said.push("running".to_owned());
    }
    said.join("  ")
}

#[cfg(test)]
mod tests;
