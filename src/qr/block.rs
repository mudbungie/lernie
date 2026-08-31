//! **The symbol on a terminal**, as text a camera can actually read off it.
//!
//! # Two rows of modules a line, because a cell is not square
//!
//! A terminal cell is about twice as tall as it is wide, so one module a cell
//! gives a symbol stretched two to one — which a decoder is entitled to refuse,
//! and which a camera at an angle refuses in practice. The half-block glyphs
//! carry the top and bottom halves of a cell independently, so two module rows
//! fit one line and the symbol comes out square in a grid that is not.
//!
//! # It carries its own paper
//!
//! A QR symbol is defined dark-on-light, and a terminal's own colours are
//! unknown — half of them are light-on-dark, which would emit the symbol
//! **inverted**. Some decoders take that and some do not, and the ones that do
//! not fail on a symbol that is otherwise correct.
//!
//! So the block sets its own two colours on every line and clears them at the
//! end of it. That is the only reason there is an escape sequence here at all:
//! not styling, not emphasis — the paper the standard requires and the terminal
//! cannot be asked for. Setting them **per line** rather than once around the
//! whole block is what makes it survive a scrolled or partly-repainted screen,
//! where a colour set before the first line and never restated is a colour some
//! of the lines were drawn without.
//!
//! # And the quiet zone, which is not decoration
//!
//! The standard requires four modules of light on every side. A decoder uses it
//! to find the symbol's edge; without it a symbol printed flush against other
//! text is one a scanner locks onto and then mismeasures. It is added here
//! rather than by the caller because the caller that forgets is the failure
//! this whole rendering exists to avoid.

use super::Symbol;

/// The four light modules the standard requires on every side.
const QUIET: usize = 4;

/// Black on white, and back to whatever the terminal was using.
const PAPER: &str = "\u{1b}[30;47m";
const RESET: &str = "\u{1b}[0m";

/// **The symbol as lines**, two module rows a line, quiet zone included, each
/// line carrying its own colours and a trailing newline.
pub(super) fn render(symbol: &Symbol) -> String {
    let span = symbol.side() + 2 * QUIET;
    let dark = |x: usize, y: usize| {
        let (x, y) = (x.checked_sub(QUIET), y.checked_sub(QUIET));
        matches!((x, y), (Some(x), Some(y)) if symbol.dark(x, y))
    };
    let mut out = String::new();
    for line in 0..span.div_ceil(2) {
        out.push_str(PAPER);
        out.extend((0..span).map(|x| glyph(dark(x, line * 2), dark(x, line * 2 + 1))));
        out.push_str(RESET);
        out.push('\n');
    }
    out
}

/// One cell: which halves of it are dark.
///
/// The glyphs draw in the FOREGROUND colour, so a dark half is a drawn half and
/// a light half is left as paper — which is why the full block means both and
/// the space means neither, rather than the other way round.
fn glyph(upper: bool, lower: bool) -> char {
    match (upper, lower) {
        (true, true) => '\u{2588}',
        (true, false) => '\u{2580}',
        (false, true) => '\u{2584}',
        (false, false) => ' ',
    }
}
