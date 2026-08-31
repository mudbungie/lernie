//! **The payload as a bit stream**: the segment header, the padding that fills
//! the symbol, the split into blocks, and the interleave that is the order the
//! modules are actually written in.
//!
//! # Why the bits are a `Vec<bool>` and not packed
//!
//! Everything upstream of placement is byte-aligned — a byte-mode segment's
//! header is 12 or 20 bits, both multiples of four, and every codeword is
//! eight. So the packing would buy nothing but the unpacking, and placement
//! wants one bit at a time in a zigzag that is nothing like byte order.
//!
//! # The stream is short and that is not an error
//!
//! Some versions have three, four or seven modules left over after the last
//! codeword — the symbol's area is not a whole number of codewords. Every
//! implementation that carries a table of those *remainder bits* is carrying a
//! table of zeroes: the modules are written light, which is what reading past
//! the end of this stream already gives you. Placement asks for bit `n` and
//! takes `false` when there is none, and the table dissolves.

use super::gf;
use super::version::{BYTE_MODE, Plan};

/// The two padding codewords the standard names, alternating, that fill a
/// symbol the payload does not.
const PAD_FIRST: u8 = 0b1110_1100;
const PAD_SECOND: u8 = 0b0001_0001;

/// **The final codeword order**: every block's data interleaved, then every
/// block's check bytes interleaved, as one stream of bits.
pub(super) fn stream(payload: &[u8], plan: Plan) -> Vec<bool> {
    let blocks = split(&codewords(payload, plan), plan);
    let checks: Vec<Vec<u8>> = blocks.iter().map(|b| gf::check(b, plan.check)).collect();
    interleave(&blocks)
        .into_iter()
        .chain(interleave(&checks))
        .flat_map(|byte| (0..8).rev().map(move |at| byte >> at & 1 == 1))
        .collect()
}

/// **The data codewords**: the header, the payload, a terminator, and padding
/// to the version's full data capacity.
///
/// The terminator is four zero bits and it is **truncated rather than
/// dropped** when the symbol has fewer than four left — a decoder reading a
/// full symbol stops at the capacity it already knows, so the terminator is
/// what tells it to stop *early*, and where there is no early to stop at there
/// is nothing to say.
fn codewords(payload: &[u8], plan: Plan) -> Vec<u8> {
    let mut bits = Bits::new();
    bits.push(BYTE_MODE, 4);
    bits.push(
        u32::try_from(payload.len()).unwrap_or(u32::MAX),
        plan.count_bits(),
    );
    for &byte in payload {
        bits.push(u32::from(byte), 8);
    }
    let room = plan.data * 8;
    bits.push(0, 4.min(room.saturating_sub(bits.len)));
    bits.pad_to_byte();
    let mut out = bits.bytes;
    out.truncate(plan.data);
    let mut pad = [PAD_FIRST, PAD_SECOND].into_iter().cycle();
    while out.len() < plan.data {
        out.push(pad.next().unwrap_or(PAD_FIRST));
    }
    out
}

/// The data codewords cut into the version's blocks, shortest first.
fn split(data: &[u8], plan: Plan) -> Vec<Vec<u8>> {
    let mut rest = data;
    plan.block_lengths()
        .into_iter()
        .map(|len| {
            let (block, tail) = rest.split_at(len.min(rest.len()));
            rest = tail;
            block.to_vec()
        })
        .collect()
}

/// **Column-wise across the blocks**: the first byte of every block, then the
/// second of every block, and so on — a block that has run out contributing
/// nothing, which is what makes the short and long blocks interleave correctly
/// with no case for either.
fn interleave(blocks: &[Vec<u8>]) -> Vec<u8> {
    let longest = blocks.iter().map(Vec::len).max().unwrap_or(0);
    (0..longest)
        .flat_map(|at| {
            blocks
                .iter()
                .filter_map(move |block| block.get(at).copied())
        })
        .collect()
}

/// A bit stream being written most-significant bit first, packed as it goes.
struct Bits {
    bytes: Vec<u8>,
    len: usize,
}

impl Bits {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            len: 0,
        }
    }

    /// Append the low `width` bits of `value`, highest first.
    fn push(&mut self, value: u32, width: usize) {
        for at in (0..width).rev() {
            if self.len.is_multiple_of(8) {
                self.bytes.push(0);
            }
            let set = u8::from(value >> at & 1 == 1) << (7 - self.len % 8);
            if let Some(byte) = self.bytes.last_mut() {
                *byte |= set;
            }
            self.len += 1;
        }
    }

    /// Round the stream up to a whole codeword with light modules.
    fn pad_to_byte(&mut self) {
        self.push(0, (8 - self.len % 8) % 8);
    }
}
