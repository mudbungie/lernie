//! **The grid itself**: the furniture a scanner finds the symbol by, the
//! zigzag the payload is written along, and the eight candidates one of which
//! is the answer.
//!
//! # Two planes, and the second is what makes the first safe
//!
//! [`Grid`] carries the modules and, beside them, one bit per module saying
//! whether it is **furniture** — a finder, a timing line, an alignment pattern,
//! the version block, or a slot reserved for the format block. Every later step
//! reads that plane rather than recomputing a geometry: the payload skips
//! furniture, the mask skips furniture, and the format block is written into
//! furniture that was reserved and left light. No step has to know what kind of
//! furniture it is standing on, which is the whole reason the plane exists.
//!
//! # Furniture is drawn by distance, not by picture
//!
//! A finder is a 7×7 pattern inside an 8-module light separator, and an
//! alignment pattern is a 5×5. Written as pictures that is two bitmaps and
//! their clipping at the symbol's edge. Written as a **Chebyshev distance from
//! the centre** — the larger of the two axis distances — each is one
//! expression: a finder module is dark unless it is two or four rings out, an
//! alignment module is dark unless it is exactly one. The clipping falls out of
//! the bounds check every write already does.

use super::mask;
use super::version::{Plan, format_bits};

/// The modules, and which of them are furniture.
pub(super) struct Grid {
    side: usize,
    dark: Vec<bool>,
    fixed: Vec<bool>,
}

impl Grid {
    /// **The symbol with all of its furniture and none of its payload**: the
    /// two timing lines, the three finders, the alignment patterns, the always-
    /// dark module, the version block where there is one, and the format slots
    /// reserved but not yet written.
    ///
    /// **The order is the standard's and it is not tidiness.** The timing lines
    /// are drawn edge to edge and the finders are then drawn over their ends —
    /// a finder's own 7×7 covers modules the timing line would otherwise own,
    /// and the finder is what belongs there. Alignment patterns go over the
    /// timing lines for the same reason and in the same direction. Laying the
    /// finders first and the timing after gives a symbol that is wrong in eight
    /// modules per corner and still scans as *something*, which is the worst
    /// kind of wrong there is.
    pub(super) fn furnished(plan: Plan) -> Self {
        let side = plan.side();
        let mut grid = Self {
            side,
            dark: vec![false; side * side],
            fixed: vec![false; side * side],
        };
        for at in 0..side {
            grid.furnish(6, at, at % 2 == 0);
            grid.furnish(at, 6, at % 2 == 0);
        }
        // The far centre is named once rather than written twice inside the
        // literal, and the literal then fits on the line its loop is on. An
        // element of a branchless array cannot run a different number of times
        // than its siblings, but a line holding ONLY such an element holds no
        // statement of its own for llvm-cov to attribute the count to — and
        // two of these three scored zero on one machine while scoring covered
        // on another. A `let` is a statement, so there is nothing left to
        // mis-attribute.
        let far = side.saturating_sub(4);
        for centre in [(3, 3), (far, 3), (3, far)] {
            grid.stamp(centre, 4, |ring| ring != 2 && ring != 4);
        }
        let centres = plan.alignment();
        for (i, &cy) in centres.iter().enumerate() {
            for (j, &cx) in centres.iter().enumerate() {
                let corner = |a: usize, b: usize| a == 0 && b == 0;
                let last = centres.len().saturating_sub(1);
                if !(corner(i, j) || corner(i, last - j) || corner(last - i, j)) {
                    grid.stamp((cx, cy), 2, |ring| ring != 1);
                }
            }
        }
        grid.furnish(8, side.saturating_sub(8), true);
        for at in format_slots(side).into_iter().chain(version_slots(plan)) {
            grid.furnish(at.0, at.1, false);
        }
        grid
    }

    /// **The payload, along the standard's zigzag**: two-module columns walked
    /// right to left, alternating up and down, skipping the vertical timing
    /// line and every module of furniture.
    ///
    /// A symbol's area is not a whole number of codewords, so the last few
    /// modules have no bit to take. They are written light, which is what
    /// reading past the end of the stream gives — no table of remainder bits,
    /// because that table is a table of zeroes.
    pub(super) fn inscribe(&mut self, bits: &[bool]) {
        let mut taken = 0;
        let mut right = self.side.saturating_sub(1);
        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            for step in 0..self.side {
                for x in [right, right.saturating_sub(1)] {
                    let upward = (right + 1) & 2 == 0;
                    let y = if upward { self.side - 1 - step } else { step };
                    if !self.is_fixed(x, y) {
                        self.set(x, y, bits.get(taken).copied().unwrap_or(false));
                        taken += 1;
                    }
                }
            }
            right = right.saturating_sub(2);
        }
    }

    /// **The eight candidates, and the one with the lowest penalty**: the mask
    /// number and the modules it produced.
    ///
    /// **Both headers are written after the scoring, not before it**, and that
    /// is the standard's own instruction rather than an ordering convenience:
    /// the penalties judge the *masked data*, so a format or version block
    /// scored along with it would let modules the mask never touches sway which
    /// mask is chosen — and their content is a function of that choice, which
    /// makes scoring them circular besides. Ties go to the lower mask number,
    /// so that one payload always produces one symbol.
    pub(super) fn resolve(self, plan: Plan) -> (u8, Vec<bool>) {
        let scored = (0..8_u8)
            .map(|number| (number, self.masked(number)))
            .min_by_key(|(number, dark)| (mask::penalty(dark, self.side), *number));
        let (number, mut dark) = scored.unwrap_or((0, self.dark.clone()));
        self.stamp_bits(
            &mut dark,
            &format_slots(self.side),
            format_bits(u32::from(number)),
            15,
        );
        if let Some(bits) = plan.version_bits() {
            self.stamp_bits(&mut dark, &version_slots(plan), bits, 18);
        }
        (number, dark)
    }

    /// One masked symbol: the mask xored over everything that is not furniture.
    fn masked(&self, number: u8) -> Vec<bool> {
        let mut dark = self.dark.clone();
        for (at, module) in dark.iter_mut().enumerate() {
            let (x, y) = (at % self.side, at / self.side);
            if self.fixed.get(at) != Some(&true) && mask::inverts(number, x, y) {
                *module = !*module;
            }
        }
        dark
    }

    /// Write an error-corrected header into its slots, least significant bit
    /// first. The slot list repeats, one copy after another, so `span` is where
    /// it starts saying the same thing again.
    fn stamp_bits(&self, dark: &mut [bool], slots: &[(usize, usize)], bits: u32, span: usize) {
        for (index, &(x, y)) in slots.iter().enumerate() {
            if let Some(module) = dark.get_mut(y * self.side + x) {
                *module = bits >> (index % span) & 1 == 1;
            }
        }
    }

    /// One concentric-ring stamp centred on `centre`, `reach` rings out, dark
    /// where `on` says the ring is.
    fn stamp(&mut self, centre: (usize, usize), reach: i64, on: impl Fn(i64) -> bool) {
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let x = i64::try_from(centre.0).unwrap_or(0) + dx;
                let y = i64::try_from(centre.1).unwrap_or(0) + dy;
                if let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) {
                    self.furnish(x, y, on(dx.abs().max(dy.abs())));
                }
            }
        }
    }

    /// **Where (`x`, `y`) lives in a plane**, or nowhere.
    ///
    /// The `x` bound is not the vector's — a row-major offset with `x` past the
    /// side is a perfectly valid index into the *next* row, so the length check
    /// every read below already does would let a write off the right edge land
    /// silently at the left edge of the row beneath. The stamps deliberately
    /// walk off both edges, so this is the ordinary case rather than a guard
    /// against a mistake.
    fn offset(&self, x: usize, y: usize) -> Option<usize> {
        (x < self.side).then(|| y * self.side + x)
    }

    /// Write one module and record it as furniture.
    fn furnish(&mut self, x: usize, y: usize, on: bool) {
        self.set(x, y, on);
        if let Some(slot) = self.offset(x, y).and_then(|at| self.fixed.get_mut(at)) {
            *slot = true;
        }
    }

    fn set(&mut self, x: usize, y: usize, on: bool) {
        if let Some(slot) = self.offset(x, y).and_then(|at| self.dark.get_mut(at)) {
            *slot = on;
        }
    }

    fn is_fixed(&self, x: usize, y: usize) -> bool {
        self.offset(x, y)
            .and_then(|at| self.fixed.get(at))
            .copied()
            .unwrap_or(false)
    }
}

/// **Where the eighteen version bits go**, both copies, one whole copy after
/// the other — six bits of version and twelve of BCH, in a 3×6 block beside the
/// top-right finder and its transpose beside the bottom-left one. Empty below
/// version 7, which has no version block at all.
///
/// The two copies are written as one list for the same reason the format
/// block's are: reserving the modules and writing them is one geometry, and two
/// spellings of one geometry drift.
fn version_slots(plan: Plan) -> Vec<(usize, usize)> {
    if plan.version_bits().is_none() {
        return Vec::new();
    }
    let far = |index: usize| plan.side() + index % 3 - 11;
    let near = |index: usize| index / 3;
    (0..18)
        .map(|index| (far(index), near(index)))
        .chain((0..18).map(|index| (near(index), far(index))))
        .collect()
}

/// **Where the fifteen format bits go**, least significant first, both copies —
/// the second copy repeating the first so a symbol survives losing a corner.
/// Slot `n` carries bit `n % 15`, which is what makes the repetition a property
/// of the list rather than a second loop.
///
/// It is one list rather than two loops because the reservation and the writing
/// are the same geometry, and two spellings of one geometry drift.
fn format_slots(side: usize) -> Vec<(usize, usize)> {
    let end = side.saturating_sub(1);
    let first = (0..6)
        .map(|i| (8, i))
        .chain([(8, 7), (8, 8), (7, 8)])
        .chain((9..15).map(|i| (14 - i, 8)));
    let second = (0..8)
        .map(move |i| (end - i, 8))
        .chain((8..15).map(move |i| (8, side + i - 15)));
    first.chain(second).collect()
}
