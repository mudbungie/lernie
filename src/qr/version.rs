//! **How big the symbol has to be**, and the four pieces of geometry that
//! follow from the answer: the block structure, the side, where the alignment
//! patterns sit, and the two error-corrected headers a decoder reads first.
//!
//! # One correction level, chosen once
//!
//! The standard offers four — recovering 7%, 15%, 25% and 30% of the symbol —
//! and this encoder emits **M** only. A level is a column of the block table,
//! so supporting all four is four times the data for a choice nothing in this
//! crate would ever make differently, and a table nobody exercises is a table
//! nobody can trust.
//!
//! M is the one to fix. The symbol is shown on a screen and read by a phone
//! camera at arm's length, against glare and a pixel grid that beats against
//! the module grid; the lowest level's 7% is thin for that. The highest costs
//! nearly twice the data area for content that is **re-displayable at will** —
//! the seat holds no copy of what it drew, so a scan that fails is answered by
//! asking again, which is the cheapest recovery there is.
//!
//! # The table is three numbers a version, and the rest derives
//!
//! [`STRUCTURE`] carries, per version: total codewords, check bytes per block,
//! and block count. Everything else is arithmetic on those — the data
//! codewords are what the check bytes do not occupy, and the standard's split
//! of them into short and long blocks is exactly the even division with the
//! remainder given to the last blocks. A table of the split itself would be two
//! more columns saying what one subtraction and one division already say.

/// Per version 1–40 at correction level M: **(total codewords, check bytes per
/// block, blocks)**.
const STRUCTURE: [(u16, u8, u8); 40] = [
    (26, 10, 1),
    (44, 16, 1),
    (70, 26, 1),
    (100, 18, 2),
    (134, 24, 2),
    (172, 16, 4),
    (196, 18, 4),
    (242, 22, 4),
    (292, 22, 5),
    (346, 26, 5),
    (404, 30, 5),
    (466, 22, 8),
    (532, 22, 9),
    (581, 24, 9),
    (655, 24, 10),
    (733, 28, 10),
    (815, 28, 11),
    (901, 26, 13),
    (991, 26, 14),
    (1085, 26, 16),
    (1156, 26, 17),
    (1258, 28, 17),
    (1364, 28, 18),
    (1474, 28, 20),
    (1588, 28, 21),
    (1706, 28, 23),
    (1828, 28, 25),
    (1921, 28, 26),
    (2051, 28, 28),
    (2185, 28, 29),
    (2323, 28, 31),
    (2465, 28, 33),
    (2611, 28, 35),
    (2761, 28, 37),
    (2876, 28, 38),
    (3034, 28, 40),
    (3196, 28, 43),
    (3362, 28, 45),
    (3532, 28, 47),
    (3706, 28, 49),
];

/// The four-bit mode indicator for a byte-mode segment.
pub(super) const BYTE_MODE: u32 = 0b0100;

/// Everything the rest of the encoder needs to know about one symbol's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Plan {
    /// 1–40.
    pub(super) version: usize,
    /// Data codewords, over every block.
    pub(super) data: usize,
    /// Check bytes per block — the same for every block of a version.
    pub(super) check: usize,
    /// How many blocks the data is split into.
    pub(super) blocks: usize,
}

impl Plan {
    /// **The smallest symbol that holds `len` payload bytes**, or `None` when
    /// no version does.
    ///
    /// The header a byte segment carries is the four-bit mode indicator plus a
    /// character count, and the count is eight bits through version 9 and
    /// sixteen from version 10 — so the question "does it fit" is asked in
    /// bits and answered against the version being asked about, never against
    /// a byte capacity computed once.
    pub(super) fn smallest_for(len: usize) -> Option<Self> {
        (1..=STRUCTURE.len()).find_map(|version| {
            let plan = Self::at(version)?;
            let needed = 4 + plan.count_bits() + len.checked_mul(8)?;
            (needed <= plan.data.checked_mul(8)?).then_some(plan)
        })
    }

    /// The plan for one version, or `None` outside 1–40.
    pub(super) fn at(version: usize) -> Option<Self> {
        let &(total, check, blocks) = STRUCTURE.get(version.checked_sub(1)?)?;
        let (check, blocks) = (usize::from(check), usize::from(blocks));
        Some(Self {
            version,
            data: usize::from(total).checked_sub(check.checked_mul(blocks)?)?,
            check,
            blocks,
        })
    }

    /// The width of the character count that follows the mode indicator.
    pub(super) fn count_bits(self) -> usize {
        if self.version <= 9 { 8 } else { 16 }
    }

    /// The symbol's side in modules: 21 at version 1, four more each version.
    pub(super) fn side(self) -> usize {
        17 + 4 * self.version
    }

    /// **How the data codewords divide into blocks**, shortest first.
    ///
    /// The standard's two groups are the even division and its remainder: every
    /// block gets `data / blocks`, and the last `data % blocks` of them get one
    /// more. The order matters — interleaving reads column-wise across the
    /// blocks, so the long ones have to be the last.
    pub(super) fn block_lengths(self) -> Vec<usize> {
        let short = self.data.checked_div(self.blocks).unwrap_or(0);
        let long = self.data.checked_rem(self.blocks).unwrap_or(0);
        (0..self.blocks)
            .map(|at| {
                if at + long < self.blocks {
                    short
                } else {
                    short + 1
                }
            })
            .collect()
    }

    /// **Where the alignment patterns' centres sit**, as coordinates that apply
    /// to both axes. Empty at version 1, which has none.
    ///
    /// The standard prints a table of these, and a table is what they are: the
    /// arithmetic below reproduces all thirty-nine rows of it **except version
    /// 32**, whose row no rule generates. That one row is named here rather
    /// than smuggled into a formula that would then be wrong in a way nothing
    /// reading it could see.
    pub(super) fn alignment(self) -> Vec<usize> {
        if self.version < 2 {
            return Vec::new();
        }
        if self.version == 32 {
            return vec![6, 34, 60, 86, 112, 138];
        }
        let count = self.version / 7 + 2;
        let last = 4 * self.version + 10;
        let span = last.saturating_sub(6);
        let gaps = count.saturating_sub(1);
        let step = 2 * span.div_ceil(2 * gaps);
        std::iter::once(6)
            .chain((0..gaps).map(|at| last - (gaps - 1 - at) * step))
            .collect()
    }

    /// **The version block**, present from version 7 and absent below it: six
    /// bits of version number carrying twelve of BCH check bits.
    pub(super) fn version_bits(self) -> Option<u32> {
        let version = u32::try_from(self.version).ok()?;
        (self.version >= 7).then(|| bch(version, 0x1f25, 12))
    }
}

/// **The format block**: two bits of correction level (M is `00`) and three of
/// mask, carrying ten of BCH check bits, the whole xored by a constant so that
/// the all-zero selection is not an all-zero block.
pub(super) fn format_bits(mask: u32) -> u32 {
    bch(mask & 0b111, 0x537, 10) ^ 0x5412
}

/// One BCH codeword: `value` shifted up by `bits` and carrying the remainder of
/// its division by `generator`, appended.
fn bch(value: u32, generator: u32, bits: u32) -> u32 {
    let width = |of: u32| 32 - of.leading_zeros();
    let mut rem = value << bits;
    while width(rem) >= width(generator) {
        rem ^= generator << (width(rem) - width(generator));
    }
    (value << bits) | rem
}
