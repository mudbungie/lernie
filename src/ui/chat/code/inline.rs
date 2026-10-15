//! **The two inline shapes in prose** (bl-f5a7): a `` `code` `` span and a
//! `## heading` line — read on the prose between fences ([`super::parts`]),
//! because each painted raw: the ticks and the hashes reached the glass as
//! characters, which is markdown shown rather than read.
//!
//! **Only these two are read**, as only the fence is read a level up: a span
//! is the text between a pair of backticks on one line, painted in the
//! monospace family at `MONO` on the surface without its ticks; a heading is
//! a line opening with one to six `#` and a space, painted without its marks
//! at the `HEADING` size — `docs/STYLE.md` §1, *a heading is a size, not a
//! hue*, and the language carries no bold face to set one in. An unpaired
//! tick stays as typed, and so does a `#` with no space after it.

use crate::ui::theme;

/// One run of prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    /// Words laid as prose, line breaks included.
    Words(String),
    /// A code span, without its ticks.
    Code(String),
    /// A heading line's words, without its marks.
    Heading(String),
}

/// The mark a span opens and closes with.
const TICK: char = '`';

/// **Split prose into its runs** — pure, so a truth table stands against it.
/// Adjacent words are one run.
pub fn spans(prose: &str) -> Vec<Span> {
    let mut out = Vec::new();
    for (n, line) in prose.split('\n').enumerate() {
        if n > 0 {
            words(&mut out, "\n");
        }
        if let Some(said) = heading(line) {
            out.push(Span::Heading(said.to_owned()));
            continue;
        }
        let mut rest = line;
        while let Some((before, code, after)) = ticked(rest) {
            words(&mut out, before);
            out.push(Span::Code(code.to_owned()));
            rest = after;
        }
        words(&mut out, rest);
    }
    out
}

/// A heading line's words, where `line` is one.
fn heading(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches('#');
    let marks = line.len() - rest.len();
    let said = rest.strip_prefix(' ')?;
    (1..=6).contains(&marks).then(|| said.trim())
}

/// The text before the first closed span, the span, and the rest — where
/// `line` holds a pair of ticks with something between them.
fn ticked(line: &str) -> Option<(&str, &str, &str)> {
    let (before, inner) = line.split_once(TICK)?;
    let (code, after) = inner.split_once(TICK)?;
    (!code.is_empty()).then_some((before, code, after))
}

/// Append `said` to the last run where it is words, else start one.
fn words(out: &mut Vec<Span>, said: &str) {
    if said.is_empty() {
        return;
    }
    match out.last_mut() {
        Some(Span::Words(held)) => held.push_str(said),
        _ => out.push(Span::Words(said.to_owned())),
    }
}

/// **The runs as one job**, so the prose wraps as one paragraph and each run
/// wears its own face: words at the body, a span in the monospace family on
/// the surface, a heading at its size.
pub fn job(prose: &str, width: f32, ink: egui::Color32) -> egui::text::LayoutJob {
    let face = |size, family| egui::TextFormat {
        font_id: egui::FontId::new(size, family),
        color: ink,
        line_height: Some(theme::type_scale::LEADING),
        ..egui::TextFormat::default()
    };
    let body = face(theme::type_scale::BODY, egui::FontFamily::Proportional);
    let mut job = egui::text::LayoutJob::default();
    for span in spans(prose) {
        match span {
            Span::Words(said) => job.append(&said, 0.0, body.clone()),
            Span::Code(said) => job.append(
                &said,
                0.0,
                egui::TextFormat {
                    background: theme::SURFACE,
                    ..face(theme::type_scale::MONO, egui::FontFamily::Monospace)
                },
            ),
            Span::Heading(said) => job.append(
                &said,
                0.0,
                face(theme::type_scale::HEADING, egui::FontFamily::Proportional),
            ),
        }
    }
    job.wrap.max_width = width;
    job
}

#[cfg(test)]
mod tests;
