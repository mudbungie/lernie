//! **The status bar's right end** (DESIGN §4.48 items 1 and 4): the seat's
//! facts, which the empty column's seat panel used to carry — the dial
//! sentence in the error accent while the aimed server will not dial
//! (bl-4cac's reading), the waiting count as the queue's door, and the
//! version. A line partly about the seat is the window's, which is why the
//! bar runs under the whole window.

use crate::ui::{Model, theme};

/// **Why the server this window is aimed at will not dial**, named by the
/// server's own header — or, with nothing aimed, the first server this seat
/// holds that says why it is not reached (it will not dial, or it refused
/// the roster): with no aim there is no other server for the line to be
/// about, and a first run with nothing provisioned is exactly that case.
pub fn dial(model: &Model) -> Option<String> {
    let servers = model.folded();
    let (server, why) = match &model.aim {
        Some(aim) => {
            let why = super::diagnostics::down(model)?;
            let server = servers
                .into_iter()
                .find(|server| server.holds(&aim.channel))?;
            (server, why)
        }
        None => servers
            .into_iter()
            .find_map(|server| server.held().said().map(|why| (server, why)))?,
    };
    Some(format!("{} — {why}", server.header()))
}

/// **The right end's width at the bar's size**, the dial sentence aside —
/// the version and, while anything asks, the count — so the line's width
/// rule can reserve it.
pub fn width(ui: &egui::Ui, model: &Model) -> f32 {
    let gap = ui.spacing().item_spacing.x;
    let asking = super::waiting::waiting(model);
    let count = if asking > 0 {
        super::width(ui, &super::waiting::said(asking)) + gap
    } else {
        0.0
    };
    super::width(ui, &crate::cli::version()) + gap + count
}

/// **Paint the right end, laid right to left** — the version at the edge,
/// the queue's door, then the dial sentence where it stands inline — and
/// answer whether the door was clicked.
pub fn render(ui: &mut egui::Ui, model: &Model, dial: Option<&str>) -> bool {
    let version = crate::cli::version();
    ui.label(egui::RichText::new(version).small().color(theme::INK_WEAK));
    let asked = super::waiting::door(ui, model);
    if let Some(said) = dial {
        let ink = theme::accent(theme::State::Error);
        ui.label(egui::RichText::new(said).small().color(ink));
    }
    asked
}
