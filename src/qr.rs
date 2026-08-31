//! **A QR symbol, drawn by this crate** (ISO/IEC 18004): bytes in, a grid of
//! dark and light modules out.
//!
//! # Why it is here rather than in the lockfile
//!
//! The seat has to put a block of bytes on a screen where a phone camera can
//! read it. That is a QR symbol, and a QR symbol is a **fully specified
//! algorithm** — a field, a table, a zigzag and four scoring rules — rather
//! than a research problem or a moving target. The manifest's dependency set is
//! an approved list that a ball has to argue its way onto, and the argument for
//! adding one here would be that the standard is hard. It is not; it is long,
//! which is a different thing and one the 300-line cap already answers by
//! splitting it.
//!
//! # The seam, and what it deliberately does not know
//!
//! [`Symbol::encode`] takes `&[u8]`. It has no opinion about what the bytes
//! mean, what produced them or where the picture goes — which is what lets the
//! payload's own definition live somewhere else and change without touching any
//! of this.
//!
//! Out the other side is a square of booleans and its side length. Two things
//! render it: [`Symbol::block`] for a terminal, and the window, which paints
//! the modules as rectangles. Neither is the symbol, and the assertion in both
//! directions is over the **matrix** — a QR symbol has no glyphs, so the paint
//! probe (the crate's one walk over painted text) has nothing to say about one,
//! and pixels are the wrong altitude to be right at.
//!
//! # What it emits, stated once
//!
//! **Byte mode, correction level M, the smallest version that fits.** Byte mode
//! because the payload is bytes and the other modes buy density for content
//! this seat will never encode. The level is argued in [`version`]. The version
//! is a function of length, so the symbol is as small as the payload allows and
//! nothing chooses it by hand.
//!
//! The ceiling is version 40, which holds **2331 bytes** at this level, and
//! [`TooLong`] is what a payload over it gets — a value, named, never a panic.

use version::Plan;

/// The payload as a bit stream: header, padding, blocks and interleave.
mod bits;
/// The terminal rendering.
mod block;
/// GF(2⁸) and the Reed-Solomon check bytes.
mod gf;
/// The eight masks and the four rules that choose between them.
mod mask;
/// The grid: furniture, the zigzag, and the eight candidates.
mod matrix;
/// Size, block structure, and the two error-corrected headers.
mod version;

/// **One QR symbol.** A square of modules, dark where the boolean is true, with
/// no quiet zone — the quiet zone is a property of where the symbol is *drawn*,
/// and a renderer that could not add four modules of margin could not draw the
/// symbol either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    side: usize,
    dark: Vec<bool>,
    mask: u8,
}

/// The payload does not fit the largest symbol this encoder emits.
///
/// Written out rather than derived: this crate links no error-derive crate, and
/// one refusal with one sentence is not the argument for acquiring one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooLong {
    /// What was offered.
    pub len: usize,
}

impl std::fmt::Display for TooLong {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len;
        write!(
            out,
            "{len} bytes is past the {LIMIT} a version 40 symbol carries at correction level M"
        )
    }
}

impl std::error::Error for TooLong {}

/// The most bytes a symbol can carry here: version 40, correction level M.
const LIMIT: usize = 2331;

impl Symbol {
    /// **Encode `payload`**, in the smallest symbol that holds it.
    pub fn encode(payload: &[u8]) -> Result<Self, TooLong> {
        let plan = Plan::smallest_for(payload.len()).ok_or(TooLong { len: payload.len() })?;
        let mut grid = matrix::Grid::furnished(plan);
        grid.inscribe(&bits::stream(payload, plan));
        let (mask, dark) = grid.resolve(plan);
        Ok(Self {
            side: plan.side(),
            dark,
            mask,
        })
    }

    /// The symbol's side, in modules.
    pub fn side(&self) -> usize {
        self.side
    }

    /// Whether the module at (`x`, `y`) is dark. Outside the symbol is light,
    /// which is the same answer the quiet zone gives.
    pub fn dark(&self, x: usize, y: usize) -> bool {
        x < self.side && self.dark.get(y * self.side + x).copied().unwrap_or(false)
    }

    /// Which of the eight masks won.
    pub fn mask(&self) -> u8 {
        self.mask
    }

    /// The symbol as lines of text for a terminal — see [`block`].
    pub fn block(&self) -> String {
        block::render(self)
    }
}

#[cfg(test)]
mod tests;
