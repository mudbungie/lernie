+++
title = "a QR code the seat draws itself: the byte-mode encoder, and the module matrix it answers"
created = 1788147675
updated = 1788148431
parent = "bl-1f10"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The encoder is **written, and proven, and it cannot land by itself** — see the last section, which is the state of the work rather than a plan.

## Shape

`src/qr.rs` and six modules under it. The seam is `Symbol::encode(&[u8]) -> Result<Symbol, TooLong>`: bytes in, a square of booleans and its side out. It has no opinion about what the bytes mean or where the picture goes, which is what lets the payload be defined elsewhere and change without touching any of it.

| module | what it holds |
|---|---|
| `qr.rs` | the symbol, the ceiling, and what is emitted, stated once |
| `qr/gf.rs` | GF(2⁸) and the Reed-Solomon check bytes |
| `qr/version.rs` | the block table, the side, the alignment centres, the two BCH headers |
| `qr/bits.rs` | header, terminator, padding, blocks, interleave |
| `qr/matrix.rs` | the furniture, the zigzag, the eight candidates |
| `qr/mask.rs` | the eight masks and the four penalty rules |
| `qr/block.rs` | the terminal rendering |

## What it emits, and the rulings behind it

**Byte mode, correction level M, the smallest version that fits.** One level rather than four, because a level is a column of the block table and a table nobody exercises is a table nobody can trust. M rather than the lowest: the symbol is read by a phone camera against screen glare and a pixel grid beating against the module grid. Rather than the highest: that costs nearly twice the area for content that is **re-displayable at will**, since the seat holds no copy of what it drew and a failed scan is answered by asking again.

Three tables the standard prints are *derived* instead. The block split is the even division of the data codewords and its remainder. The remainder-bit table is a table of zeroes, dissolved by reading past the end of the bit stream as light. And no logarithm tables: the field multiply is the carry-less multiply itself, which needs no initialization and no indexing in a crate where unchecked indexing is denied.

The alignment centres are the one place an exception survives — the arithmetic reproduces thirty-eight of the standard's thirty-nine rows and version 32 is named, because no rule generates it and smuggling it into a formula would be wrong in a way nothing reading it could see.

## Two things the standard leaves open, and both bit

Building this against **two** independent implementations was worth more than building it against one, because they disagree, and where they disagree is exactly where a lone oracle would have made this encoder wrong.

- **The pad codeword.** One of them appends a spurious zero codeword whenever the message plus its four-bit terminator lands on a byte boundary — which in byte mode is *always*. The standard adds padding bits only when the stream does **not** end at a codeword boundary. The symbol still decodes (nothing reads past the terminator), so it is invisible, and it means that library can never be a byte-for-byte oracle for this one. The other agrees with this encoder to the bit.
- **Penalty rule 3.** The standard prints two eleven-module patterns and also describes them in prose as a core "preceded or followed by" four light modules, and the two readings differ on a core with light on both sides — two occurrences or one. This encoder follows the patterns as printed, which is the reading that leaves an oracle standing. Rule 3 only changes *which* mask wins and all eight decode, so it is a choice rather than a correctness question, and it is recorded as one at the rule.

One ordering was also a defect and is now stated at the site: the timing lines go down **before** the finders overwrite their ends, and both error-corrected headers are written **after** the mask scoring, never before — the penalties judge the masked data, and a header whose content is a function of the mask choice cannot also be an input to it.

## Proven

- **Three whole matrices pinned module for module** — versions 1, 7 and 10, each chosen for something nothing smaller exercises: one block, the first version block, the first sixteen-bit character count and the first two-length interleave.
- **All forty versions**, filled to their exact byte capacity: side, chosen mask and dark-module count. The count is weak alone and strong in company — it moves if a codeword moves, a block splits differently, an alignment pattern lands a module off, or the mask choice changes.
- **All thirty-nine alignment rows**, the published Reed-Solomon worked example, the published format table for all eight masks, and the published version block at 7 and at 40.
- **Verified out of band and reported rather than pinned**: at 1500 bytes and at the 2331-byte ceiling this encoder's matrix is byte-identical to the reference implementation's, and a real camera-grade decoder reads back an enrollment-shaped payload exactly. (The same decoder fails on *both* encoders above version 30, which is the detector's limit rather than either encoder's — the identical-matrix check is what says so.)

## Why it is not closed, and where it is

**A crate that denies dead code will not accept a module with no caller, and it is right.** `warnings = "deny"` refuses every item here, and the only ways past it are an inline suppression (forbidden — policy lives in the manifest) or a crate-wide `dead_code = "allow"` (a real relaxation bought for one module). Marking the module `pub` to dodge it would be a lie: `pub` means the surface the integration crate and the binary consume, and nothing consumes this yet.

So the encoder lands **with its caller**, which is bl-1f10 — blocked on the protocol authority. The work is **committed on this ball's own claim worktree**, gate-clean on every step except that one: `cargo fmt --check`, the line cap, the disclosure scan and `clippy --all-targets` are all green, and the module's own tests pass.

Whoever takes bl-1f10 should resume this claim under its original identity, fold the worktree into the surface's, and close them together. The encoder needs no further work; it needs the line that calls it.