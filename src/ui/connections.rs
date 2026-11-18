//! **The connections** — *Settings › Servers › connections* (DESIGN §4.52).
//!
//! Every channel this box holds, one block each, said in full: its name
//! here, its name on its host, the addresses it dials, where its files are,
//! what it last said, and which other connection dials the same address. The
//! roster folds two names for one server under one section; this page never
//! folds, because the question it answers is *what does this box hold*, and
//! two entries are two sets of files.
//!
//! **Removing is two clicks, and the way out is first** (§4.20's order).
//! The first click arms one entry and says what goes with it; the second
//! hands its name to the poster's lane (`crate::ui::model::forget`). The page
//! writes nothing itself and fires nothing on the wire.
//!
//! **This box's own server is listed and not removable.** It is the flat
//! root, not an entry, and the page says so rather than offering a control
//! that would unprovision the box.

use crate::channel::entries::{ENTRIES, WIRE};
use crate::ui::{Chunk, Held, Model, theme};

/// What the page says above the list.
pub const ABOUT: &str = "Every server this box reaches. Each is a set of files on this box: \
     a key, a certificate, the server's anchors and its address.";
/// The first click, before the entry's name.
pub const REMOVE: &str = "remove";
pub const REMOVE_HELP: &str = "arms removing this connection; nothing is deleted until confirmed";
/// What the armed entry says before the second click.
pub const WARNING: &str = "Deletes this box's key and certificate for this server. The server \
     still lists this box until an operator there removes it, and joining again takes a new \
     offer.";
/// The way out, and the second click.
pub const KEEP: &str = "keep it";
pub const KEEP_HELP: &str = "leaves this connection as it is";
pub const CONFIRM: &str = "remove it";
pub const CONFIRM_HELP: &str = "deletes this connection's files from this box";
/// What stands where the control would, on this box's own server.
pub const OWN: &str =
    "This box's own server: its files are the box's own, not an entry, and are not removed here.";
/// What a connection with no readable address says.
pub const NO_ADDRESS: &str = "dials nothing: this box holds no address for it";

/// The first click's word for one entry — the name is in it, so two rows'
/// controls are two things.
pub fn remove(name: &str) -> String {
    format!("{REMOVE} {name}")
}

/// **What one connection is, a line per fact.**
pub fn facts(chunk: &Chunk, roster: &[Chunk]) -> Vec<String> {
    let channel = &chunk.channel;
    let mut said = Vec::new();
    if let Some(there) = &channel.named_there {
        said.push(format!("named {there:?} on its host"));
    }
    said.push(match &channel.dials {
        Some(at) => format!("dials {at}"),
        None => NO_ADDRESS.to_owned(),
    });
    let twins: Vec<&str> = roster
        .iter()
        .map(|other| &other.channel)
        .filter(|other| other.name != channel.name && other.dials == channel.dials)
        .filter(|_| channel.dials.is_some())
        .map(|other| other.name.as_str())
        .collect();
    if !twins.is_empty() {
        said.push(format!("the same address as {}", twins.join(", ")));
    }
    said.push(match &channel.named_there {
        Some(_) => format!(
            "files: {WIRE}/{ENTRIES}/{} under this box's data root",
            channel.name
        ),
        None => format!("files: {WIRE} under this box's data root"),
    });
    said.push(match &chunk.held {
        Held::Unheard => crate::ui::roster::NOT_ANSWERED.to_owned(),
        Held::Heard => {
            let names: Vec<&str> = chunk
                .workspaces
                .iter()
                .map(|row| row.workspace.as_str())
                .collect();
            format!(
                "answered with {} workspace(s): {}",
                names.len(),
                names.join(", ")
            )
        }
        held @ (Held::Unheld(_) | Held::Refused(_)) => held.said().unwrap_or_default(),
    });
    said
}

/// Paint the page and take what is clicked on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.add(egui::Label::new(egui::RichText::new(ABOUT).color(theme::INK_WEAK)).wrap());
    if let Some(why) = &model.servers.forgetting.refused {
        ui.add(egui::Label::new(egui::RichText::new(why).color(theme::NOTICE)).wrap());
    }
    let roster = model.roster.clone();
    for chunk in &roster {
        ui.add_space(theme::space::S);
        ui.label(
            egui::RichText::new(&chunk.channel.name)
                .strong()
                .color(theme::INK),
        );
        for fact in facts(chunk, &roster) {
            ui.add(egui::Label::new(egui::RichText::new(fact).color(theme::INK_WEAK)).wrap());
        }
        if chunk.channel.named_there.is_some() {
            controls(ui, model, &chunk.channel.name);
        } else {
            ui.add(egui::Label::new(egui::RichText::new(OWN).color(theme::INK_FAINT)).wrap());
        }
    }
}

/// One entry's control: the first click, or — armed — what goes with it and
/// the two ways on.
fn controls(ui: &mut egui::Ui, model: &mut Model, name: &str) {
    if model.servers.forgetting.armed.as_deref() != Some(name) {
        if theme::paint::button(ui, &remove(name), true, REMOVE_HELP).clicked() {
            model.arm_forget(name);
        }
        return;
    }
    ui.add(egui::Label::new(egui::RichText::new(WARNING).color(theme::NOTICE)).wrap());
    ui.horizontal(|ui| {
        if theme::paint::button(ui, KEEP, true, KEEP_HELP).clicked() {
            model.disarm_forget();
        }
        if theme::paint::button(ui, CONFIRM, true, CONFIRM_HELP).clicked() {
            model.forget();
        }
    });
}

#[cfg(test)]
mod tests;
