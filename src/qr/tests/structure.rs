//! **The pieces under the symbol**, each asserted where a failure names it.
//!
//! The whole-matrix comparison next door says the encoder is right. It does not
//! say *which* part was wrong when it stops being right, and a 177-line diff is
//! not an answer. So the field, the tables and the two error-corrected headers
//! are each pinned here against something outside this crate's arithmetic:
//! published constants where the standard publishes them, and the same
//! independent implementation's table where it does not.

use super::super::gf;
use super::super::version::{Plan, format_bits};
use super::tables;

/// **The field is a field**, which is four laws and not an opinion: 1 is the
/// identity, 0 annihilates, multiplication commutes, and every non-zero element
/// has an inverse. The last is the one that would actually catch a wrong
/// reduction polynomial — a broken field has elements that multiply to zero.
#[test]
fn multiplication_in_the_field_obeys_the_field_s_own_laws() {
    for a in 0..=255_u8 {
        assert_eq!(gf::mul(a, 1), a, "one is the identity");
        assert_eq!(gf::mul(a, 0), 0, "zero annihilates");
        assert_eq!(gf::mul(a, 7), gf::mul(7, a), "and it commutes");
    }
    let mut inverted = 0;
    for a in 1..=255_u8 {
        if (1..=255_u8).any(|b| gf::mul(a, b) == 1) {
            inverted += 1;
        }
    }
    assert_eq!(inverted, 255, "every non-zero element has an inverse");
}

/// The published worked example of a Reed-Solomon block, from the standard's
/// own annex: sixteen data codewords and the ten check bytes they produce at
/// version 1, correction level M.
#[test]
fn the_check_bytes_match_the_standard_s_worked_example() {
    let data = [
        0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec,
        0x11,
    ];
    assert_eq!(
        gf::check(&data, 10),
        vec![0xa5, 0x24, 0xd4, 0xc1, 0xed, 0x36, 0xc7, 0x87, 0x2c, 0x55]
    );
}

/// **No check bytes is no work**, and it is the one call that would otherwise
/// walk a register with nothing in it. Not reachable from the encoder — every
/// version asks for at least ten — which is exactly why it is asserted rather
/// than assumed.
#[test]
fn a_block_with_no_check_bytes_answers_nothing() {
    assert_eq!(gf::check(&[1, 2, 3], 0), Vec::<u8>::new());
}

/// **The format block, all eight masks**, against the standard's published
/// table for correction level M. These are the fifteen bits a decoder reads
/// before anything else, so a wrong one is a symbol nothing can even begin.
#[test]
fn the_format_block_matches_the_standard_s_published_table() {
    let published = [
        0x5412, 0x5125, 0x5e7c, 0x5b4b, 0x45f9, 0x40ce, 0x4f97, 0x4aa0,
    ];
    for (mask, want) in published.into_iter().enumerate() {
        assert_eq!(
            format_bits(u32::try_from(mask).unwrap_or(0)),
            want,
            "mask {mask}"
        );
    }
}

/// **The version block, at both ends of where it exists.** Absent below version
/// 7 — which is not a special case but the standard's own rule — and the
/// published eighteen bits at 7 and at 40.
#[test]
fn the_version_block_is_absent_below_seven_and_published_above_it() {
    for version in 1..7 {
        assert_eq!(Plan::at(version).and_then(Plan::version_bits), None);
    }
    assert_eq!(Plan::at(7).and_then(Plan::version_bits), Some(0x07c94));
    assert_eq!(Plan::at(40).and_then(Plan::version_bits), Some(0x28c69));
}

/// **Every alignment table row**, because the arithmetic that generates them
/// has one exception — version 32 — and a rule with an exception is a rule that
/// has to be checked at every row rather than at the exception.
#[test]
fn every_version_s_alignment_centres_match_the_oracle_s_table() {
    for &(version, want) in &tables::ALIGNMENT {
        let plan = Plan::at(version).expect("a version in range");
        assert_eq!(plan.alignment(), want, "version {version}");
    }
    assert_eq!(Plan::at(1).map(Plan::alignment), Some(Vec::new()));
    assert_eq!(tables::ALIGNMENT.len(), 39, "the walk enumerated nothing");
}

/// Outside 1–40 there is no plan, which is what makes the ceiling a refusal
/// rather than an out-of-range read.
#[test]
fn no_version_exists_outside_one_to_forty() {
    assert_eq!(Plan::at(0), None);
    assert_eq!(Plan::at(41), None);
}

/// **The block split is the even division and its remainder**, at every
/// version: the lengths sum to the data capacity, differ by at most one, and
/// the long ones come last — which is the order the interleave depends on.
#[test]
fn every_version_splits_its_data_into_blocks_that_sum_and_are_sorted() {
    for version in 1..=40 {
        let plan = Plan::at(version).expect("a version in range");
        let lengths = plan.block_lengths();
        assert_eq!(lengths.len(), plan.blocks, "version {version} block count");
        assert_eq!(
            lengths.iter().sum::<usize>(),
            plan.data,
            "version {version}"
        );
        assert!(lengths.windows(2).all(|p| p[1] >= p[0]), "shortest first");
        let span = lengths.last().unwrap_or(&0) - lengths.first().unwrap_or(&0);
        assert!(span <= 1, "version {version} splits by more than one");
    }
}
