//! **The files half of the records pane**: where the work lands, what the
//! worktree holds, and the bounded preview of one file in it.
//!
//! Split from [`super`] beside [`super::steps`] and for its reason (bl-d1ae):
//! the pane file is the pane, and this is one section of it with the two
//! sentences it paints computed beside the paint.
//!
//! **The worktree's absence and an empty listing are two claims**, and the
//! wire keeps them two on purpose — so this half paints two sentences where a
//! tidier one would have painted one.

use super::{EMPTY_WORKTREE, FILES_HEAD, NO_WORKTREE, NOT_ANSWERED_FILES, TRUNCATED};
use crate::reply::files::{Files, Preview};
use crate::ui::theme;

/// The files half: the section it opens, where the work lands, the listing,
/// and the preview.
pub(super) fn half(ui: &mut egui::Ui, files: Option<&Files>) {
    theme::paint::section(ui, FILES_HEAD);
    let Some(answer) = files else {
        ui.label(NOT_ANSWERED_FILES);
        return;
    };
    // **Where the work lands rides on the line under the section word**, and
    // the walked entries share one wrapped line with each other: seven halves
    // ride under this pane's one scroll and a control laid out past the frame
    // is unreachable (`crate::snapshot::clipped`).
    ui.horizontal_wrapped(|ui| {
        if let Some(dir) = &answer.working_dir {
            ui.colored_label(theme::INK_WEAK, format!("working in {dir}"));
        }
        match &answer.listing {
            None => {
                ui.label(NO_WORKTREE);
            }
            Some(listing) if listing.rows.is_empty() => {
                ui.label(EMPTY_WORKTREE);
            }
            Some(listing) => {
                for row in &listing.rows {
                    ui.label(entry(row));
                }
                if listing.truncated {
                    ui.colored_label(theme::accent(theme::State::Annotation), TRUNCATED);
                }
            }
        }
    });
    if let Some(preview) = &answer.preview {
        ui.label(egui::RichText::new(previewed(preview)).monospace());
    }
}

/// One walked entry as a line: a directory wears its slash, a file its size.
pub fn entry(row: &crate::reply::files::FileRow) -> String {
    if row.dir {
        format!("{}/", row.path)
    } else {
        format!("{}  {} B", row.path, row.size)
    }
}

/// A bounded preview as text — the engine's three classes, and the rung-3
/// word painted as itself. Shared with the drill-in's two capture logs
/// ([`super::drill`]), which are the same bounded reading.
pub fn previewed(preview: &Preview) -> String {
    match preview {
        Preview::Text(text) => text.clone(),
        Preview::Truncated { text, size } => format!("{text}\n… {size} bytes in all"),
        Preview::Binary { size } => format!("binary — {size} bytes"),
        Preview::Unknown(word) => format!("a {word:?} preview, which this seat cannot show"),
    }
}
