+++
title = "a QR code the seat draws itself: the byte-mode encoder, and the module matrix it answers"
created = 1788147675
updated = 1788147675
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The seat has to show an operator a block of bytes that a phone camera can read. That is a QR symbol, and it arrives with **no new dependency** — the house default is zero, and a QR encoder is a fully specified algorithm rather than a research problem.

## The seam this ball owns

Bytes in, a module matrix out. Nothing about enrollment, nothing about the wire, nothing about what the bytes mean. That is what makes it landable on its own and what makes it survive any later decision about the payload: whatever is encoded, it is `&[u8]` by the time it reaches here.

Two renderings sit on the matrix and neither is the matrix:

- **In-window**, painted as geometry. The paint probe is the one walk over painted GLYPHS and a QR symbol has none, so the assertion is over the module matrix, never over pixels.
- **On a terminal**, as a text block for the CLI verb — two rows of modules per character cell using the half-block, so the symbol comes out square in a cell grid that is not.

## Scope, and what is deliberately not in it

Byte mode only. Alphanumeric and kanji modes buy density for content this seat will never encode, and a mode nothing produces is a table nobody can test.

Version and error-correction level are chosen for the payload rather than fixed: a measured enrollment envelope is on the order of 1.5 kB of compact JSON, which is a mid-forties version at the lowest correction level and comfortably inside the format. The choice is a function of length, stated once.

Masking is evaluated properly rather than pinned. A conformant encoder may emit any mask — the decoder reads which one out of the format information — so pinning one is legal and it is also how a symbol that a camera cannot lock onto gets shipped. The eight are scored by the specification's four penalty rules.

## How it is proven, given there is nothing to compare against in the crate

An encoder that agrees with itself proves nothing, and a decoder written beside it shares its tables and therefore its mistakes. So the oracle is INDEPENDENT and it is spent at development time: a separate, established implementation generates the ground-truth matrix for each chosen input, and that matrix is committed as a text fixture the suite pins. The suite then needs no tool, no network and no second implementation — it needs the fixture, which is what an oracle is for.

Both directions, as everywhere else here: a fixture that stops being compared, or a comparison that enumerates nothing, is a test that passes forever.