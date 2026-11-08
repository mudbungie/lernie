//! **The search** (DESIGN §4.48 item 7): a box at the overlay's head that
//! finds a row by its label or its help on every page, and jumps to it. The
//! baseline binds Ctrl+K to it; here a box in the overlay is enough, and no
//! global chord is spent on it.
//!
//! **What it reads is the table**: the knob rows, the pin's row and the pages
//! themselves by their labels and help. The workspace pages' rows are the
//! wire's — a role, a provider, a machine — and are what their page shows,
//! not what a search over this seat's words can promise.
//!
//! While the box holds text the hits stand where the page would; picking one
//! clears the box, turns the overlay to the hit's page and names the row for
//! that page to scroll to (`super::rows::knob`).

use crate::settings::KNOBS;
use crate::ui::{Model, Page, Seeking, theme};

/// What the box says while it is empty, and under the pointer.
pub const HINT: &str = "search settings";
pub const HELP: &str = "finds a row by its label or its help, on every page";
/// What stands where the hits would, when nothing matches.
pub const NOTHING: &str = "no row says that";
/// The word on a hit's door.
pub const OPEN: &str = "open";
/// The id the box wears.
const BOX: &str = "the settings' search box";

/// One row the search found: the words it matched on, the page it is on, and
/// the knob it is, where it is one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub label: &'static str,
    pub help: &'static str,
    pub page: Page,
    pub key: Option<&'static str>,
}

/// Paint the box, and the hits in the page's place while it holds text.
/// Answers whether the hits stand.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    theme::paint::field(
        ui,
        egui::Id::new(BOX),
        &mut model.seeking.text,
        HINT,
        None,
        HELP,
    );
    ui.add_space(theme::space::XS);
    let needle = model.seeking.text.trim().to_lowercase();
    if needle.is_empty() {
        return false;
    }
    let found = hits(&needle);
    if found.is_empty() {
        ui.colored_label(theme::INK_WEAK, NOTHING);
        return true;
    }
    let mut picked = None;
    for hit in found {
        super::rows::row(ui, hit.label, hit.help, |ui| {
            let door = theme::paint::link(ui, OPEN, theme::INK_WEAK, &super::tree::path(hit.page));
            if door.clicked() {
                picked = Some(hit.clone());
            }
        });
    }
    if let Some(hit) = picked {
        model.seeking = Seeking {
            text: String::new(),
            jump: hit.key,
        };
        model.turn_to(hit.page);
    }
    true
}

/// **Every row whose label or help says `needle`** (lower-cased): the knobs,
/// the pin, then the pages.
pub fn hits(needle: &str) -> Vec<Hit> {
    let says = |label: &str, help: &str| {
        label.to_lowercase().contains(needle) || help.to_lowercase().contains(needle)
    };
    let pin = Hit {
        label: super::rows::PINNED,
        help: super::rows::PINNED_HELP,
        page: Page::Seat(crate::settings::Page::Bar),
        key: None,
    };
    KNOBS
        .iter()
        .map(|knob| Hit {
            label: knob.label,
            help: knob.help,
            page: Page::Seat(knob.page),
            key: Some(knob.key),
        })
        .chain([pin])
        .chain(Page::ALL.into_iter().map(|page| Hit {
            label: page.label(),
            help: page.help(),
            page,
            key: None,
        }))
        .filter(|hit| says(hit.label, hit.help))
        .collect()
}
