//! **The headless paint probe**: render a widget tree off-screen and read back
//! every galley it painted.
//!
//! Ported from yog with the rule that governs it
//! (`rules/no-hand-rolled-paint-walk.yml`), and the rule travels because the
//! defect it exists to catch is not a yog defect.
//!
//! **`Galley::text()` is the string that went IN.** A label the toolkit elided
//! to `…` still reports itself whole, so every assertion made against it is
//! blind to truncation — the one defect the paint layer is the only witness
//! for. Upstream found that three times: once in the probe itself, where 1815
//! tests passed while covering no truncation at all; once in two copies of the
//! walk; and once more in a third copy that aimed every pointer test's click by
//! input text. It arrives here as a rule and not as a memory.
//!
//! **So there is ONE walk** — [`descend`] — and everything else is a projection
//! of it. A caller that needs something else adds a projection here rather than
//! a walk of its own; that is what the ast-grep rule enforces, and its `ignores`
//! list names this file alone.
//!
//! It is `cfg(test)`: nothing in a released binary reads its own paint.

/// How a frame is **produced**, as against how one is **read** here.
pub(crate) mod frame;

pub(crate) use frame::paint;

/// One painted galley: the glyphs that reached the glass, and the rect they
/// landed on in screen points.
pub(crate) type Painted = (String, egui::Rect);

/// One galley **as the operator sees it**: its glyphs, where they were laid,
/// what its clip rect let through, and the ink it reached the glass in.
///
/// [`Painted`] answers *what was laid out*, which is not the same question as
/// *what is on the glass*: egui emits shapes it then clips away, and a run laid
/// wider than its container is sliced at the container's edge — mid-glyph, with
/// no ellipsis, because the galley itself was never truncated and so never had
/// one added. Only the two rects together tell that apart from a run that fits.
#[derive(Clone, Debug)]
pub(crate) struct Seen {
    /// The laid-out glyphs — [`visible`]'s read, never `Galley::text()`.
    pub(crate) text: String,
    /// Where the run was laid, in screen points.
    pub(crate) laid: egui::Rect,
    /// The part of [`Self::laid`] the clip rect let through.
    pub(crate) shown: egui::Rect,
    /// The colour it was painted in — how a toned row is legible to a test at
    /// all, since a tone is an ink on the run and nothing about its rect.
    pub(crate) ink: egui::Color32,
}

/// **The one walk.** Glyphs, rect and ink, descending `Shape::Vec` and ignoring
/// every shape that carries no text.
fn descend(shape: &egui::Shape, out: &mut Vec<(String, egui::Rect, egui::Color32)>) {
    match shape {
        egui::Shape::Text(t) => out.push((
            visible(&t.galley),
            t.galley.rect.translate(t.pos.to_vec2()),
            ink(t),
        )),
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| descend(s, out)),
        _ => {}
    }
}

/// The colour a run reached the glass in — its layout section's, or the shape's
/// fallback where that section defers to it.
///
/// `Color32::PLACEHOLDER` is what a plain `ui.label` lays: the widget declines
/// to name a colour and egui resolves it at paint time from the fallback. Both
/// halves here, so "what colour is this run?" is one question of the frame
/// rather than a guess about which widget drew it.
fn ink(t: &egui::epaint::TextShape) -> egui::Color32 {
    match t.galley.job.sections.first().map(|s| s.format.color) {
        Some(colour) if colour != egui::Color32::PLACEHOLDER => colour,
        _ => t.fallback_color,
    }
}

/// **What a laid-out galley actually SHOWS**: its glyphs, row by row.
///
/// This is the whole reason the module exists. `Galley::text()` would report
/// the string that went in, so `contains("Login")` would pass on a button
/// rendered as a bare `…`. The glyphs are what reached the screen, which is
/// what makes a dump evidence rather than a restatement of the input.
fn visible(galley: &egui::Galley) -> String {
    let mut out = String::new();
    for row in &galley.rows {
        out.extend(row.glyphs.iter().map(|g| g.chr));
        if row.ends_with_newline {
            out.push('\n');
        }
    }
    out
}

/// Every painted galley of one shape, with its position.
pub(crate) fn collect(shape: &egui::Shape, out: &mut Vec<Painted>) {
    let mut inked = Vec::new();
    descend(shape, &mut inked);
    out.extend(inked.into_iter().map(|(text, rect, _)| (text, rect)));
}

/// The same, with the positions dropped — one line per galley.
pub(crate) fn collect_text(shape: &egui::Shape, out: &mut String) {
    let mut painted = Vec::new();
    collect(shape, &mut painted);
    for (text, _) in painted {
        out.push_str(&text);
        out.push('\n');
    }
}

/// Every galley of one finished frame, as text.
pub(crate) fn text_of(output: &egui::FullOutput) -> String {
    let mut text = String::new();
    for clipped in &output.shapes {
        collect_text(&clipped.shape, &mut text);
    }
    text
}

/// Every galley one finished frame put on the glass, each narrowed to what its
/// clip rect let through. A shape clipped away entirely is dropped: it is not
/// on screen and is not evidence about it.
pub(crate) fn seen_of(output: &egui::FullOutput) -> Vec<Seen> {
    let mut out = Vec::new();
    for clipped in &output.shapes {
        let mut here = Vec::new();
        descend(&clipped.shape, &mut here);
        out.extend(here.into_iter().filter_map(|(text, laid, ink)| {
            let shown = laid.intersect(clipped.clip_rect);
            (shown.width() > 0.5 && shown.height() > 0.5).then_some(Seen {
                text,
                laid,
                shown,
                ink,
            })
        }));
    }
    out
}

/// The centre of the first galley reading exactly `text` — the coordinate a
/// pointer test aims at.
///
/// **A click is aimed by painted glyphs, like every other assertion here.** The
/// third copy of the walk upstream found was exactly this function reading
/// `Galley::text()`, which aimed clicks confidently at seats whose painted text
/// was not what they named.
pub(crate) fn locate(output: &egui::FullOutput, text: &str) -> Option<egui::Pos2> {
    seen_of(output)
        .into_iter()
        .find(|seen| seen.text == text)
        .map(|seen| seen.shown.center())
}

#[cfg(test)]
mod tests;
