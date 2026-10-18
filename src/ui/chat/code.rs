//! **Code in what was said** (bl-beef): a fenced block in a model's prose
//! reads as code — monospace, set apart on the surface, its fences not
//! painted, its lines never wrapped.
//!
//! Models answer in markdown, and the code in an answer is the part an
//! operator copies. Painted as prose it is the body font between two lines of
//! backticks, indistinguishable from the sentence around it, and a long line
//! wraps as if it were a sentence too.
//!
//! **The fence is the only block read** (the prose between fences reads its
//! two inline shapes, a code span and a heading line — [`inline`]): a line opening with three
//! backticks (an info string after them is allowed and not painted) begins a
//! block, the next such line ends it, and a fence still open — the turn is
//! streaming — runs to the end of the words. Everything else is prose. No markdown crate: the one shape the glass got
//! wrong is the one this reads.

use crate::ui::theme;

/// The code spans and heading lines inside prose.
pub mod inline;

/// The marker a fence line opens with.
const FENCE: &str = "```";

/// One part of what was said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// Words laid as prose.
    Prose(String),
    /// The lines inside one fence, without it.
    Code(String),
}

/// **Split `words` on its fences** — pure, so a truth table can stand
/// against it. Empty prose between two blocks is not a part.
pub fn parts(words: &str) -> Vec<Part> {
    let mut out = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    let mut fenced = false;
    for line in words.lines() {
        if line.trim_start().starts_with(FENCE) {
            close(&mut out, &mut held, fenced);
            fenced = !fenced;
        } else {
            held.push(line);
        }
    }
    close(&mut out, &mut held, fenced);
    out
}

/// End the part `held` is, if it holds anything worth a part.
fn close(out: &mut Vec<Part>, held: &mut Vec<&str>, fenced: bool) {
    let text = held.join("\n");
    held.clear();
    if fenced {
        out.push(Part::Code(text));
    } else if !text.trim().is_empty() {
        out.push(Part::Prose(text.trim_matches('\n').to_owned()));
    }
}

/// **Paint what was said**, prose as prose and each block as code, in `ink`
/// — the body's, or the pending ink for a message the server has not yet
/// acknowledged (DESIGN §4.45 item 4). `at` is the row's place in the
/// transcript, which salts each block's scroll so two blocks never share one.
pub fn paint(ui: &mut egui::Ui, at: usize, words: &str, ink: egui::Color32) {
    for (n, part) in parts(words).into_iter().enumerate() {
        match part {
            Part::Prose(text) => {
                let job = inline::job(&text, ui.available_width(), ink);
                ui.add(egui::Label::new(job).wrap());
            }
            Part::Code(text) => block(ui, (at, n), &text, ink),
        }
    }
}

/// **One block**: monospace on the surface, scrolled sideways inside the
/// column rather than wrapped or laid off the glass.
fn block(ui: &mut egui::Ui, salt: (usize, usize), text: &str, ink: egui::Color32) {
    egui::Frame::none()
        .fill(theme::SURFACE)
        .rounding(theme::RADIUS)
        .inner_margin(theme::space::S)
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt(("code", salt))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(text).monospace().color(ink)).extend(),
                    );
                });
        });
}

#[cfg(test)]
mod tests;
