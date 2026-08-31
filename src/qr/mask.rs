//! **The eight masks, and the four rules that choose between them.**
//!
//! A mask is a fixed pattern xored over every module that is not part of the
//! symbol's own furniture. It changes nothing a decoder reads — which mask was
//! applied is written into the format block, and the decoder xors it back off —
//! so **any** of the eight produces a valid symbol.
//!
//! That is exactly why the choice is made properly here rather than pinned. The
//! masks exist to keep a symbol *findable*: an unlucky payload can lay down a
//! field of large single-colour blocks, or a run that looks like the finder
//! pattern a camera locks onto, and a scanner then fails on a symbol that is
//! byte-for-byte correct. Pinning one mask is legal, and it is how that ships.
//!
//! So all eight are laid down, scored by the standard's four penalties, and the
//! lowest total wins — ties to the lower mask number, which is a rule only so
//! that one payload always produces one symbol.

/// **Is (`x`, `y`) inverted by `mask`?** The eight conditions, in the
/// standard's own order — the number in the format block is an index into this
/// list and nothing else.
pub(super) fn inverts(mask: u8, x: usize, y: usize) -> bool {
    match mask {
        0 => (y + x).is_multiple_of(2),
        1 => y.is_multiple_of(2),
        2 => x.is_multiple_of(3),
        3 => (y + x).is_multiple_of(3),
        4 => (y / 2 + x / 3).is_multiple_of(2),
        5 => ((y * x) % 2 + (y * x) % 3) == 0,
        6 => ((y * x) % 2 + (y * x) % 3).is_multiple_of(2),
        _ => ((y + x) % 2 + (y * x) % 3).is_multiple_of(2),
    }
}

/// **What one candidate symbol costs.** Lower is better; the four rules are
/// independent and their totals add.
///
/// Rules 1 and 3 read lines, so the symbol is turned into its rows and its
/// columns once and both rules walk that — the standard says "row or column"
/// for each, and a column is a row of the transpose.
pub(super) fn penalty(dark: &[bool], side: usize) -> usize {
    let at = |x: usize, y: usize| dark.get(y * side + x).copied().unwrap_or(false);
    let lines: Vec<Vec<bool>> = (0..side)
        .map(|y| (0..side).map(|x| at(x, y)).collect())
        .chain((0..side).map(|x| (0..side).map(|y| at(x, y)).collect()))
        .collect();
    let one: usize = lines.iter().map(|line| runs(line)).sum();
    let three: usize = lines.iter().map(|line| finders(line)).sum();
    one + three + squares(dark, side) + balance(dark)
}

/// **Rule 1** — a run of five or more same-coloured modules costs three, and
/// one more for every module past the fifth.
fn runs(line: &[bool]) -> usize {
    let mut total = 0;
    let mut run = 0_usize;
    let mut colour = None;
    for &module in line {
        run = if colour == Some(module) { run + 1 } else { 1 };
        colour = Some(module);
        if run == 5 {
            total += 3;
        } else if run > 5 {
            total += 1;
        }
    }
    total
}

/// **Rule 3** — the finder pattern's own 1:1:3:1:1 signature with four light
/// modules on one side of it. Forty each, because a scanner that mistakes one
/// of these for a finder does not read the symbol at all.
///
/// # Two readings, and this is the one the standard's own figures spell
///
/// The standard gives rule 3 as two eleven-module patterns —
/// `1011101 0000` and `0000 1011101` — and counts every occurrence of either.
/// A core with four light modules on **both** sides is therefore two
/// occurrences and costs eighty, and a core flush against the symbol's edge is
/// none, because there are not eleven modules there to match. Some
/// implementations read the prose instead ("preceded or followed by") and score
/// such a core once, treating the quiet zone as the light run.
///
/// Both produce valid symbols — rule 3 only ever changes *which* mask wins, and
/// all eight decode — so this is a choice rather than a correctness question.
/// It is settled by the two independent implementations the fixtures were
/// derived from disagreeing about it, and only one of them agreeing with this
/// encoder about everything the standard **does** determine (see [`tests`] on
/// the pad codeword). Following the patterns as printed is the reading that
/// leaves an oracle standing.
///
/// [`tests`]: super::tests
fn finders(line: &[bool]) -> usize {
    const CORE: [bool; 7] = [true, false, true, true, true, false, true];
    const QUIET: [bool; 4] = [false; 4];
    line.windows(11)
        .filter(|window| {
            let is = |range: std::ops::Range<usize>, want: &[bool]| {
                window.get(range).is_some_and(|part| part == want)
            };
            is(0..7, &CORE) && is(7..11, &QUIET) || is(4..11, &CORE) && is(0..4, &QUIET)
        })
        .count()
        * 40
}

/// **Rule 2** — every two-by-two block of one colour costs three. Overlapping
/// blocks each count, which is what makes a large field expensive rather than
/// merely noticed.
fn squares(dark: &[bool], side: usize) -> usize {
    let at = |x: usize, y: usize| dark.get(y * side + x).copied();
    let uniform = |x: usize, y: usize| {
        let first = at(x, y);
        first.is_some()
            && [at(x + 1, y), at(x, y + 1), at(x + 1, y + 1)]
                .iter()
                .all(|&other| other == first)
    };
    let last = side.saturating_sub(1);
    (0..last)
        .flat_map(|y| (0..last).map(move |x| (x, y)))
        .filter(|&(x, y)| uniform(x, y))
        .count()
        * 3
}

/// **Rule 4** — ten for every five percent the dark proportion strays from
/// half.
///
/// The deviation is taken on the **exact** proportion rather than on a
/// percentage rounded first, which is not a nicety: a symbol at 40.1% dark is
/// one five-percent step off half, and a percentage floored to 40 before the
/// division is two. Rounding twice is how a scoring rule drifts from its own
/// oracle at every boundary.
///
/// The `max(1)` is the divisor's own floor rather than a case: a symbol with no
/// modules has no proportion to be wrong about, and writing that as a branch
/// would be a line no test can reach honestly.
fn balance(dark: &[bool]) -> usize {
    let total = dark.len().max(1);
    let deviation = (dark.iter().filter(|&&module| module).count() * 100).abs_diff(total * 50);
    deviation / (5 * total) * 10
}
