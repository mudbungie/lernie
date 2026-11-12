//! **Register this box** — *Settings › Servers › register* (DESIGN §4.50;
//! yog's `docs/REMOTE.md` §15.1 steps 2 to 4).
//!
//! Where an operator adding a server looks: a field for the one-time
//! registration URL, the button that claims it, and the outcome under them —
//! the confirmation word, and the one sentence saying where to approve it.
//! The page opens with nothing aimed, because a box registering for the
//! first time holds no workspace to aim at.
//!
//! **The page fires nothing on the wire itself.** The click hands the URL to
//! the poster's lane (`crate::ui::model::register`), which makes the key,
//! claims the offer over HTTPS, files the entry and dials once. It carries no
//! `act:` tag for the same reason: an act tag names an op that crosses the
//! boundary (§4.16), and a registration crosses none — the wire is not
//! spoken on until the entry exists.

use crate::ui::{Claim, Model, theme};

/// What the page says above the field.
pub const ABOUT: &str = "Joins this box to a server. An operator on a seat that is already \
     connected mints an offer for it; paste the registration URL that offer shows. This box \
     makes its own key, and the key never leaves it.";
/// What the empty field shows.
pub const HINT: &str = "https://<host>/yog/register/<code>";
/// The button, and what it does.
pub const REGISTER: &str = "register this box";
pub const REGISTER_HELP: &str = "claims the offer at that URL with a key made on this box, \
     files the entry, and shows the confirmation word to approve it by — a URL works once";
/// What stands under the button while a claim is out.
pub const CLAIMING: &str = "claiming the offer…";
/// What the outcome's own line for the word begins with: the one line of it
/// painted strong, because it is the one the operator reads across to
/// another screen.
pub const WORD: &str = crate::seat::register::WORD_LINE;

/// The field's width in points — fixed, so the button keeps its place on the
/// row at the widths the window opens at (`crate::ui::unmake`'s reason).
const FIELD: f32 = 320.0;

/// Paint the page and take what is typed and clicked on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.add(egui::Label::new(egui::RichText::new(ABOUT).color(theme::INK_WEAK)).wrap());
    ui.add_space(theme::space::S);
    ui.horizontal_wrapped(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut model.registering.url)
                .desired_width(FIELD.min(ui.available_width()))
                .hint_text(HINT),
        );
        let ready = model.registering.ready();
        if theme::paint::button(ui, REGISTER, ready, REGISTER_HELP).clicked() {
            model.register();
        }
    });
    ui.add_space(theme::space::S);
    match &model.registering.phase {
        Claim::Idle => {}
        Claim::Claiming => {
            ui.colored_label(theme::INK_WEAK, CLAIMING);
        }
        Claim::Done(Err(refusal)) => {
            ui.add(egui::Label::new(egui::RichText::new(refusal).color(theme::NOTICE)).wrap());
        }
        Claim::Done(Ok(registered)) => {
            for line in registered.said().lines() {
                let mut words = egui::RichText::new(line);
                if line.starts_with(WORD) {
                    words = words.strong().color(theme::INK);
                }
                ui.add(egui::Label::new(words).wrap());
            }
        }
    }
}

#[cfg(test)]
mod tests;
