//! **What the world is charged**: each engine's price table, its ceiling, the
//! spend against it, and the two acts that write them (REMOTE §9.23; DESIGN
//! §4.41; bl-9111).
//!
//! # A window-level pane, sectioned per channel
//!
//! `prices` names no workspace, so its control hangs off the roster beside
//! the trail's and the verb table's, and the pane is one section per channel
//! under the roster's own header — two engines are two tables, and a union
//! would say they were one. What the model holds and why the acts go down one
//! channel is `crate::ui::model::pricing`'s.
//!
//! # Nothing here is armed, and §4.20 is why
//!
//! A ceiling set under the world's spend PARKS every conversation over it —
//! conversations no control here names, which is §4.20's amended test for an
//! arming. It is not armed because the test's other half fails: the act is
//! undone by doing the other thing, ONCE. Raising the bound or lifting it
//! drives every conversation it parked, and the receipt says how many woke.
//! The cascade stop is armed because its undoing is a nudge per conversation;
//! this one's undoing is the control beside it, and the spend line above both
//! says what the bound is about to be measured against.

use crate::ui::{Model, PriceTable, theme};

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "prices…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "what the world is charged";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear what these engines charge";
/// What a section with no row says — every count it reports is unpriced.
pub const NO_ROWS: &str = "no row is priced, so no count carries money";
/// The act that writes the drafted row down one channel.
pub const PRICE: &str = "price it";
/// The act that deletes one row.
pub const UNPRICE: &str = "unprice";
/// The act that sets the drafted bound.
pub const SET: &str = "set ceiling";
/// The act that lifts a standing bound.
pub const LIFT: &str = "lift ceiling";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.pricing() {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_lookup();
        }
    });
    ui.separator();
    drafting(ui, model);
    ui.separator();
    let tables = model.pricing.tables.clone();
    if tables.is_empty() {
        ui.label(NOT_ANSWERED);
        return true;
    }
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for table in &tables {
                ui.separator();
                ui.label(crate::ui::roster::header(&table.channel));
                section(ui, model, table);
            }
        });
    true
}

/// The four boxes both acts are composed from.
fn drafting(ui: &mut egui::Ui, model: &mut Model) {
    let draft = &mut model.pricing;
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut draft.provider).hint_text("provider"));
        ui.add(egui::TextEdit::singleline(&mut draft.model).hint_text("model, or *"));
        ui.add(
            egui::TextEdit::singleline(&mut draft.rates)
                .hint_text("input output [cache-read [cache-write]] per Mtok"),
        );
    });
    ui.add(egui::TextEdit::singleline(&mut draft.bound).hint_text("ceiling in USD, or off"));
}

/// One channel's table: the standing line, the rows, and the acts.
fn section(ui: &mut egui::Ui, model: &mut Model, table: &PriceTable) {
    ui.label(table.prices.standing());
    if let Some(woke) = table.prices.woke() {
        ui.colored_label(theme::NOTICE, woke);
    }
    if table.prices.rows.is_empty() {
        theme::paint::empty(ui, NO_ROWS);
    }
    for row in &table.prices.rows {
        let mut fired = false;
        ui.horizontal_wrapped(|ui| {
            ui.label(row.said());
            let off = ui.button(UNPRICE);
            crate::ui::act::tag(&off, &[crate::verbs::PRICE]);
            fired = off.clicked();
        });
        if fired {
            model.post_unprice(&table.channel, &row.provider, &row.model);
        }
    }
    acts(ui, model, table);
}

/// The section's two acts, each disabled with its reason beside it.
fn acts(ui: &mut egui::Ui, model: &mut Model, table: &PriceTable) {
    let channel = &table.channel;
    let unpriceable = model.unpriceable(channel);
    let unbounded = model.unbounded();
    let (mut price, mut set, mut lift) = (false, false, false);
    ui.horizontal_wrapped(|ui| {
        let act = ui.add_enabled(unpriceable.is_none(), egui::Button::new(PRICE));
        crate::ui::act::tag(&act, &[crate::verbs::PRICE]);
        price = act.clicked();
        let act = ui.add_enabled(unbounded.is_none(), egui::Button::new(SET));
        crate::ui::act::tag(&act, &[crate::verbs::CEILING]);
        set = act.clicked();
        if table.prices.ceiling.is_some() {
            let act = ui.button(LIFT);
            crate::ui::act::tag(&act, &[crate::verbs::CEILING]);
            lift = act.clicked();
        }
    });
    // **A greyed control says a thing is not live and nothing about what
    // would make it live** (DESIGN §4.20), so each reason stands beside it.
    for why in [unpriceable, unbounded].into_iter().flatten() {
        ui.colored_label(theme::NOTICE, why);
    }
    if price {
        model.post_price(channel);
    }
    if set {
        model.post_ceiling(channel);
    }
    if lift {
        model.post_lift(channel);
    }
}

#[cfg(test)]
mod tests;
