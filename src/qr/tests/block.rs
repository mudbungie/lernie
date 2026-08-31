//! **The terminal rendering**, asserted as a reversible view of the matrix.
//!
//! A block of half-height glyphs is not a picture to eyeball — it is the matrix
//! encoded two module rows at a time, and the honest test of an encoding is
//! that it decodes. So the block is read back to modules and compared against
//! the symbol it came from, which catches an inverted polarity, a dropped row,
//! a missing quiet zone and a wrong glyph in one assertion.

use super::super::Symbol;

/// Everything the paper escapes wrap, stripped back to the cells.
fn cells(line: &str) -> Vec<char> {
    line.trim_start_matches("\u{1b}[30;47m")
        .trim_end_matches("\u{1b}[0m")
        .chars()
        .collect()
}

/// The block read back as modules: `(x, y)` dark or not, in the block's own
/// coordinates — quiet zone included, so (0, 0) is four modules outside the
/// symbol.
fn as_modules(block: &str) -> Vec<Vec<bool>> {
    block
        .lines()
        .flat_map(|line| {
            let row = cells(line);
            let half = |want: [char; 2]| {
                row.iter()
                    .map(|glyph| want.contains(glyph))
                    .collect::<Vec<bool>>()
            };
            [
                half(['\u{2588}', '\u{2580}']),
                half(['\u{2588}', '\u{2584}']),
            ]
        })
        .collect()
}

/// **The block is the matrix, and nothing else is.** Read back, every module of
/// the symbol sits where the symbol has it, and every module outside it is
/// light — which is the quiet zone, asserted by the same walk rather than by a
/// margin count.
#[test]
fn the_block_reads_back_as_the_symbol_inside_its_quiet_zone() {
    let symbol = Symbol::encode(b"a block of bytes").expect("it fits");
    let read = as_modules(&symbol.block());
    let quiet = 4;
    let span = symbol.side() + 2 * quiet;
    assert!(
        read.len() >= span,
        "the block is at least the symbol's span"
    );
    for (y, row) in read.iter().enumerate() {
        assert_eq!(row.len(), span, "row {y} is the full span");
        for (x, &module) in row.iter().enumerate() {
            let inside = (quiet..quiet + symbol.side()).contains(&x)
                && (quiet..quiet + symbol.side()).contains(&y);
            let want = inside && symbol.dark(x - quiet, y - quiet);
            assert_eq!(module, want, "at ({x}, {y})");
        }
    }
}

/// **An odd span leaves a half-cell**, and it is drawn as paper rather than
/// dropped. Every version's span is odd — the side is odd and the quiet zone
/// adds eight — so this is the ordinary case rather than an edge one, and a
/// renderer that truncated it would eat the last quiet row on every symbol.
#[test]
fn the_span_s_odd_last_row_is_drawn_as_paper() {
    let symbol = Symbol::encode(b"odd").expect("it fits");
    let span = symbol.side() + 8;
    assert_eq!(span % 2, 1, "the span is odd at every version");
    let read = as_modules(&symbol.block());
    assert_eq!(read.len(), span + 1, "the half cell is a whole row");
    assert!(
        read.last().is_some_and(|row| row.iter().all(|&m| !m)),
        "and it is light"
    );
}

/// **Every line carries its own paper.** A colour set once before the first
/// line is a colour the terminal loses on a scroll or a partial repaint, and a
/// symbol drawn without it is a symbol drawn in whatever the terminal was
/// using — which for half of them is the inverse of what the standard requires.
#[test]
fn every_line_sets_and_clears_its_own_colours() {
    let block = Symbol::encode(b"paper").expect("it fits").block();
    assert!(
        block
            .lines()
            .all(|line| line.starts_with("\u{1b}[30;47m") && line.ends_with("\u{1b}[0m")),
        "a line was drawn without paper"
    );
    assert!(block.ends_with('\n'), "and the last line is terminated");
}

/// All four glyphs occur, which is the only thing that says the two half-height
/// ones are reachable at all — a renderer that emitted the full block and the
/// space alone would pass every assertion above on a symbol whose rows happened
/// to pair up.
#[test]
fn all_four_cell_glyphs_are_drawn() {
    let block = Symbol::encode(b"glyphs, all of them")
        .expect("it fits")
        .block();
    for glyph in ['\u{2588}', '\u{2580}', '\u{2584}', ' '] {
        assert!(block.contains(glyph), "no {glyph:?} anywhere in the block");
    }
}
