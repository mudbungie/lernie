//! **The roster**: every workspace this seat can reach, grouped by the channel
//! it came down.
//!
//! The grouping is the point. A seat holds one channel per workspace it
//! participates in elsewhere plus this box's own engine (§8.2), and those are
//! separate trust relationships that share nothing — not anchors, not leaves,
//! not addresses. Painting them as one flat list would say they are one thing.
//!
//! **A row carries the channel it came from as a client-side stamp** and no
//! origin ever crosses the wire; the stamp is applied where the answer is
//! absorbed ([`crate::ui::Model::absorb`]) and read here.

use crate::reply::roster::WsRow;
use crate::ui::{Aim, Chunk, Model, theme};

/// What a box with no channel at all says. It is a fact about provisioning and
/// the operator reading it is the operator who would fix it, so it names the
/// act rather than reporting an absence.
pub const NO_CHANNEL: &str = "no channel provisioned — material arrives by hand";

/// What an unreachable row says instead of being hidden.
///
/// Dropping it would hide a workspace the operator has; addressing it by the
/// entry's leaf would aim a gesture at a different wall. So it is painted, and
/// painted as what it is.
pub const NO_NAME_HERE: &str = "this seat holds no name for it";

/// The word this pane wears, and the subject the arrows act on when it is
/// focused.
pub const HEADING: &str = "channels";

/// **Every wall this seat can aim at, in the order the pane paints them.**
///
/// It is the keyboard's cursor track and it is a **query**, derived from the
/// same rows and the same order [`render`] draws — so a key cannot walk onto a
/// row a click cannot reach, and cannot walk in an order the glass does not
/// show. A row this seat holds no name for is in neither: no envelope can
/// address it, so neither surface offers it.
pub fn aimable(model: &Model) -> Vec<Aim> {
    let mut rows = Vec::new();
    for chunk in &model.roster {
        for row in ordered(&chunk.walls) {
            if let Some(address) = chunk.channel.address(&row) {
                rows.push(Aim {
                    channel: chunk.channel.name.clone(),
                    address,
                });
            }
        }
    }
    rows
}

/// Paint the roster and take a click on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.heading(crate::ui::keys::heading(
        HEADING,
        model.focus == crate::ui::Pane::Roster,
    ));
    if model.roster.is_empty() {
        ui.label(NO_CHANNEL);
        return;
    }
    for chunk in model.roster.clone() {
        section(ui, model, &chunk);
    }
}

/// One channel's section: its header, how current its answer is, and its walls.
fn section(ui: &mut egui::Ui, model: &mut Model, chunk: &Chunk) {
    ui.separator();
    ui.label(header(chunk));
    for note in [chunk.stale.as_ref(), chunk.growth.as_ref()]
        .into_iter()
        .flatten()
    {
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), note);
    }
    for row in ordered(&chunk.walls) {
        wall(ui, model, chunk, &row);
    }
}

/// The section header: what this box calls the channel, and what its host calls
/// the workspace when the two differ — because a local rename is the remedy for
/// a name collision, and an operator has to be able to see one.
pub fn header(chunk: &Chunk) -> String {
    match &chunk.channel.named_there {
        Some(there) if *there != chunk.channel.name => {
            format!("{} (named {:?} on its host)", chunk.channel.name, there)
        }
        _ => chunk.channel.name.clone(),
    }
}

/// One wall: selectable when this seat can address it, a plain line when it
/// cannot.
fn wall(ui: &mut egui::Ui, model: &mut Model, chunk: &Chunk, row: &WsRow) {
    let Some(address) = chunk.channel.address(row) else {
        ui.label(format!("{}  — {NO_NAME_HERE}", line(row)));
        return;
    };
    let aimed = model.aimed_at(&chunk.channel.name, Some(&address));
    if ui.selectable_label(aimed, line(row)).clicked() {
        model.aim_at(&chunk.channel.name.clone(), &address);
    }
}

/// **Pinned first, in pin order**, then the rest by name.
///
/// The rank is what makes this a sort rather than a filter: a seat given only a
/// flag would have to read the pin list back to order them, which is the seat
/// joining an answer against a document only the engine holds.
pub fn ordered(walls: &[WsRow]) -> Vec<WsRow> {
    let mut rows = walls.to_vec();
    rows.sort_by(|a, b| {
        (a.pinned.unwrap_or(u64::MAX), &a.workspace)
            .cmp(&(b.pinned.unwrap_or(u64::MAX), &b.workspace))
    });
    rows
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
