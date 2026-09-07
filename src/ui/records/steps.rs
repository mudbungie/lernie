//! **The steps half of the records pane**: what the conversation's loop did,
//! one row per step, and the four sentences a step can carry.
//!
//! Split from [`super`] at the design-time budget on the seam the pane's own
//! subject draws (bl-d1ae): the pane file is the pane — its heading, its
//! subject line and the order of its sections — and this is one section of it.
//! The sentences travel with the paint that says them, because a sentence
//! computed in one file and painted in another is two places to look when the
//! words on the glass are wrong.
//!
//! Every line here is a pure function of the row, so the suite reads the
//! sentence rather than the layout, and the class tokens ride verbatim
//! (`crate::reply` rung 3): a `framing` or a `wound` this build has no word
//! for paints as itself.

use super::{NO_STEPS, NOT_ANSWERED_STEPS, STEPS_HEAD, drill};
use crate::reply::steps::{StepRow, Steps};
use crate::ui::{Model, theme};

/// The steps half: the section it opens, the orphan banner, then one row per
/// step with the control that drills into it (bl-3257).
pub(super) fn half(ui: &mut egui::Ui, model: &mut Model, steps: Option<&Steps>) {
    // **The section word takes the line and its first fact takes the next**
    // (§4.32; bl-d1ae). `theme::paint::section` IS the divider — `L` of air, a
    // hairline and the word in weak ink — so nothing else stands between two
    // halves, and the fact under it rides with no further air. The pane's
    // content still has to FIT the window at the narrowest shape this layout
    // promises, and `crate::snapshot::clipped` fails the whole matrix over one
    // control laid out past the frame.
    theme::paint::section(ui, STEPS_HEAD);
    ui.horizontal_wrapped(|ui| match steps {
        None => {
            ui.label(NOT_ANSWERED_STEPS);
        }
        Some(listing) => {
            if let Some(said) = orphaned(listing) {
                ui.colored_label(theme::accent(theme::State::Annotation), said);
            }
            if listing.rows.is_empty() {
                ui.label(NO_STEPS);
            }
        }
    });
    let Some(listing) = steps else {
        return;
    };
    for row in &listing.rows {
        step(ui, model, row);
    }
}

/// One step: the headline, the provenance under it, what went wrong, and the
/// drill-in its `seq` addresses.
fn step(ui: &mut egui::Ui, model: &mut Model, row: &StepRow) {
    // **The headline, the provenance and the drill-in control share one
    // wrapped row.** This pane covers the window and its content has to fit
    // it: a control laid out past the frame is unreachable, and there are
    // seven halves under this one scroll (`crate::snapshot::clipped`).
    ui.horizontal_wrapped(|ui| {
        ui.label(headline(row));
        if let Some(weak) = provenance(row) {
            ui.colored_label(theme::INK_WEAK, weak);
        }
        drill::control(ui, model, &row.seq);
    });
    // **A step with no wound paints no row for one**, not even an empty one:
    // an empty `horizontal_wrapped` still allocates, and this pane pays for
    // every point it allocates (`crate::snapshot::clipped`).
    let said: Vec<String> = [wounded(row), auth(row)].into_iter().flatten().collect();
    if !said.is_empty() {
        ui.horizontal_wrapped(|ui| {
            for line in said {
                ui.colored_label(theme::accent(theme::State::Annotation), line);
            }
        });
    }
    drill::records(ui, model, &row.seq);
}

/// The orphan banner, or none: [`crate::reply::steps::NONE`] is the engine's
/// own *nothing is orphaned* and paints as silence rather than as a badge.
pub fn orphaned(listing: &Steps) -> Option<String> {
    if listing.orphan == crate::reply::steps::NONE {
        return None;
    }
    Some(match &listing.orphan_reason {
        Some(reason) => format!("an orphaned {} tail — {reason}", listing.orphan),
        None => format!("an orphaned {} tail", listing.orphan),
    })
}

/// **The one line a step always gets**: its address, how it ended, what it
/// cost — and the retries, where there were any.
pub fn headline(row: &StepRow) -> String {
    let said = format!("{}  {} — {} tokens", row.seq, row.framing, row.tokens.total);
    if row.attempts > 1 {
        return format!("{said}, {} attempts", row.attempts);
    }
    said
}

/// The weak line under it — when it ran and what commit read it — or none
/// where the step's record carried neither.
pub fn provenance(row: &StepRow) -> Option<String> {
    let mut parts = Vec::new();
    // A word and not an arrow: the toolkit's default font has no glyph for
    // `→` and paints a box in its place — photographed, not guessed.
    if let (Some(from), Some(to)) = (&row.started_at, &row.ended_at) {
        parts.push(format!("{from} to {to}"));
    }
    if let Some(commit) = &row.commit {
        parts.push(format!("at {commit}"));
    }
    (!parts.is_empty()).then(|| parts.join("  "))
}

/// The wound, said once: the class verbatim, and the adapter's own words
/// where it left any.
pub fn wounded(row: &StepRow) -> Option<String> {
    if row.wound == crate::reply::steps::NONE {
        return None;
    }
    Some(match &row.wound_reason {
        Some(reason) => format!("wound: {} — {reason}", row.wound),
        None => format!("wound: {}", row.wound),
    })
}

/// The sign-in affordance: offered at all, and the provider row it points at
/// when one was derivable.
///
/// **Offered on the wound's `refused` arm, not on a flag beside it.** The flag
/// (`auth_failed`) was deleted from the row at PROTOCOL 9 because it said what
/// the wound class already said, and two spellings of one fact are two things
/// that can disagree — so the affordance and [`wounded`] above now read the
/// same word, and a step cannot offer a sign-in while its badge denies there
/// was a refusal.
pub fn auth(row: &StepRow) -> Option<String> {
    if row.wound != crate::reply::steps::REFUSED {
        return None;
    }
    Some(match &row.auth_row {
        Some(provider) => format!("a sign-in is wanted on {provider}"),
        None => "a sign-in is wanted".to_owned(),
    })
}
