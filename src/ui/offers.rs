//! **The offers page** — *Settings › Servers › offers*, and the word
//! `/offers` (DESIGN §4.51; yog's `docs/REMOTE.md` §15.1 steps 1 and 5).
//!
//! How a new device joins the aimed workspace, from the seat that is already
//! in it. Top to bottom, in the order an operator uses it: a form that mints
//! an offer (a name and a grade); the minted URL as text and as a QR symbol,
//! with when its code expires; and the workspace's pending offers — name,
//! grade, state and confirmation word — each with `approve` and `decline`.
//!
//! **`approve` is dark until the row is proved.** The server would refuse it
//! otherwise, and the dark control says why before it is asked: the device
//! has not dialled yet. The operator's whole job is to read the word off the
//! new device and approve the row that shows the same one.
//!
//! It replaces the enrollment, which covered the whole window because what
//! it drew was a private key. Nothing here is one, so this is a page like
//! its neighbours.

use crate::reply::offers::OfferRow;
use crate::ui::model::Grade;
use crate::ui::{Model, theme};

/// **The word typed after the slash** that opens it (`crate::ui::slash`).
pub const WORD: &str = "offers";
/// The form's heading, its name box and its button.
pub const MINT_HEADING: &str = "offer a new device a place here";
pub const NAME_HINT: &str = "the new device's name";
pub const NAME_HELP: &str =
    "the name the server will know the new device by; one nothing was ever issued under";
pub const GRADE_HELP: &str =
    "what the new device may do: an operator asks and acts, a foot only executes";
pub const MINT: &str = "mint offer";
pub const MINT_HELP: &str =
    "asks the server for a one-time registration URL for a device of that name and grade";
/// What stands over a minted URL, and what its expiry is said with.
pub const MINTED: &str = "the new device opens this URL, or scans the symbol — it works once";
pub const UNTIL: &str = "its code expires at";
/// The list's heading and its two empty states.
pub const PENDING: &str = "pending offers";
pub const NOT_ANSWERED: &str = "waiting to hear which offers are pending";
pub const NONE_PENDING: &str = "no offer is pending in this workspace";
/// A row's two acts.
pub const APPROVE: &str = "approve";
pub const APPROVE_HELP: &str =
    "seats this device in the workspace — only when the word on its screen is this row's word";
pub const APPROVE_DARK: &str =
    "cannot be approved yet: the device has not dialled the server with its new certificate";
pub const DECLINE: &str = "decline";
pub const DECLINE_HELP: &str =
    "voids this offer; if a device already claimed it, its certificate stays refused for good";
/// What a row with no word yet says where the word will be.
pub const NO_WORD: &str = "no word yet — nothing has claimed it";

const NAME_BOX: &str = "the offer's name box";
/// The most a symbol is drawn across, in points: a phone reads it at arm's
/// length well under this, and it leaves the list on the page.
const SYMBOL: f32 = 220.0;

/// Paint the page and take what is typed and clicked on it. Answers whether
/// there was one to paint — the overlay scrolls it (`crate::ui::settings`).
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.showing(crate::ui::Listing::Offers) {
        return false;
    }
    if let Some(aim) = model.aim.clone() {
        ui.colored_label(
            theme::INK_WEAK,
            format!("on {} — {}", aim.address, aim.channel),
        );
    }
    form(ui, model);
    minted(ui, model);
    ui.add_space(theme::space::M);
    theme::paint::section(ui, PENDING);
    match model.pending.clone() {
        None => {
            ui.label(NOT_ANSWERED);
        }
        Some(rows) if rows.is_empty() => {
            ui.label(NONE_PENDING);
        }
        Some(rows) => {
            for row in &rows {
                pending(ui, model, row);
            }
        }
    }
    true
}

/// The form: a name, a grade, and the mint.
fn form(ui: &mut egui::Ui, model: &mut Model) {
    theme::paint::section(ui, MINT_HEADING);
    theme::paint::field(
        ui,
        egui::Id::new(NAME_BOX),
        &mut model.offering.name,
        NAME_HINT,
        None,
        NAME_HELP,
    );
    ui.horizontal_wrapped(|ui| {
        for grade in Grade::BOTH {
            let chosen = model.offering.grade == grade;
            if theme::paint::seat(ui, chosen, grade.word(), GRADE_HELP).clicked() {
                model.offering.grade = grade;
            }
        }
        let mint = theme::paint::button(ui, MINT, model.offering.ready(), MINT_HELP);
        crate::ui::act::tag(&mint, &[crate::verbs::OFFER.word]);
        if mint.clicked() {
            model.post_offer();
        }
    });
}

/// The offer last minted: the URL as text a pointer can select, when its
/// code expires, and the same URL as a symbol.
fn minted(ui: &mut egui::Ui, model: &Model) {
    let Some(minted) = &model.offering.minted else {
        return;
    };
    ui.add_space(theme::space::S);
    ui.colored_label(theme::INK_WEAK, MINTED);
    ui.add(egui::Label::new(egui::RichText::new(&minted.offered.url).monospace()).wrap());
    ui.colored_label(
        theme::INK_WEAK,
        format!(
            "{UNTIL} {}",
            crate::reply::offers::clock(minted.offered.expires)
        ),
    );
    if let Some(drawn) = &minted.symbol {
        let side = SYMBOL.min(ui.available_width());
        ui.allocate_ui(egui::vec2(side, side), |ui| symbol::paint(ui, drawn));
    }
}

/// One pending offer: its line, its word, and its two acts.
fn pending(ui: &mut egui::Ui, model: &mut Model, row: &OfferRow) {
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}  {}  {}", row.name, row.grade, row.state));
        match &row.word {
            Some(word) => {
                ui.label(egui::RichText::new(word).monospace().strong());
            }
            None => {
                ui.colored_label(theme::INK_WEAK, NO_WORD);
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        let (live, help) = if row.proved() {
            (true, APPROVE_HELP)
        } else {
            (false, APPROVE_DARK)
        };
        let approve = theme::paint::button(ui, APPROVE, live, help);
        crate::ui::act::tag(&approve, &[crate::verbs::APPROVE.word]);
        if approve.clicked() {
            model.post_verdict(&row.name, true);
        }
        let decline = theme::paint::button(ui, DECLINE, true, DECLINE_HELP);
        crate::ui::act::tag(&decline, &[crate::verbs::DECLINE.word]);
        if decline.clicked() {
            model.post_verdict(&row.name, false);
        }
    });
}

/// The symbol, painted to the glass.
mod symbol;

#[cfg(test)]
pub(crate) mod tests;
