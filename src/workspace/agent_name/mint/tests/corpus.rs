//! What the embedded `words.txt` promises its consumer, and what makes
//! adding a word an explicit review event (bl-b59c).
//!
//! A minted name is worn in front of an operator and their peers, so the
//! pool is not merely *shaped* data — it is *approved* data. Three
//! properties, in three tests: the shape a name must have to be a legal
//! name at all, the approval pin (count + digest) that turns any edit
//! into a deliberate act, and the semantic rule the approval is applied
//! against, mechanised so a future addition is checked and not merely
//! trusted.

use super::super::wordlist;
use std::collections::HashSet;

/// The 64-bit FNV-1a of the pool, newline-joined. A digest rather than a
/// dependency: the property wanted is "these exact words in this exact
/// order", and FNV-1a buys it in three lines of wrapping arithmetic —
/// the same trade the mint's own [`super::SplitMix64`] makes against
/// pulling in `rand`. It is a change detector, not a security primitive;
/// nothing here defends against a hostile editor, only a careless one.
fn digest(words: &[&str]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in words.join("\n").bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The shape every entry must hold for a minted word to be a legal name:
/// `^[a-z]{3,9}$` — path-safe on every supported filesystem, shell-safe
/// unquoted, case-unambiguous, and always past `require_available`'s own
/// gates. Plus the parse rule and the file's reviewability: unique, and
/// sorted, so a diff of this file reads as an insertion.
#[test]
fn every_word_is_lowercase_ascii_three_to_nine_and_the_file_is_sorted() {
    let words = wordlist();
    // The `#` header and blank lines are gone; nothing else is.
    assert!(!words.iter().any(|w| w.starts_with('#') || w.is_empty()));
    for w in &words {
        assert!(
            (3..=9).contains(&w.len()) && w.chars().all(|c| c.is_ascii_lowercase()),
            "{w:?} violates ^[a-z]{{3,9}}$"
        );
    }
    let unique: HashSet<&&str> = words.iter().collect();
    assert_eq!(unique.len(), words.len(), "duplicate word in words.txt");
    let mut sorted = words.clone();
    sorted.sort_unstable();
    assert_eq!(words, sorted, "words.txt is not sorted");
}

/// **The approval pin.** The pool is a reviewed artifact: authored for
/// this repository, read end to end by a human, and licensed under the
/// crate's own licence with no third-party corpus behind it (the file's
/// own header). These two lines are the record of that review — an edit
/// to the list fails here until both are updated by hand, which is
/// exactly the explicit review event bl-b59c asks for. The count is also
/// load-bearing at runtime: it is the whole pool, and so the bound on
/// the collision-retry scan.
#[test]
fn the_approved_pool_is_pinned_by_count_and_digest() {
    let words = wordlist();
    assert_eq!(words.len(), 541, "the approved word count changed");
    assert_eq!(
        digest(&words),
        0x1e02_5c48_f3b9_36aa,
        "words.txt changed — re-review the list, then update this digest and the count above"
    );
}

/// **The semantic rule, mechanised.** The pool is concrete, neutral,
/// everyday English: nothing violent, sexual, medical, political or
/// insulting, no proper noun, no brand, no personal name. The previous
/// EFF-derived pool asserted none of this and minted `humiliate` and
/// `wrath` as real agent names (bl-b59c).
///
/// The stem check is **substring**, not whole-word, and deliberately has
/// no exception list: an invariant with exceptions is an invariant
/// nobody can apply. It costs a handful of innocent words — `warm` and
/// `paint` are unmintable because of `war` and `pain` — and that is the
/// trade, because the alternative is a per-word argument every time
/// someone extends the list.
#[test]
fn no_word_is_hostile_or_a_human_identity() {
    // Harm vocabulary, as stems: no word may *contain* any of these.
    const HOSTILE: &[&str] = &[
        "abuse", "angry", "blade", "blood", "bomb", "carnage", "choke", "corpse", "cruel", "curse",
        "dead", "death", "deceit", "deprav", "despair", "drug", "evil", "fever", "filth", "greed",
        "hate", "hostil", "humiliat", "hurt", "kill", "knife", "murder", "nude", "obscen", "pain",
        "panic", "poison", "rage", "rape", "sexy", "sick", "slave", "stench", "sword", "terror",
        "threat", "toxic", "traitor", "trash", "tumor", "venom", "victim", "vile", "vulgar", "war",
        "weapon", "wound", "wrath",
    ];
    // Identities a minted name must never impersonate: the personal names
    // that hide in nature vocabulary (`willow`, `hazel`, `robin`…) and the
    // system accounts a claimant string is stamped with — including bl's
    // own `--as` fallback literal, `unknown`.
    const IDENTITIES: &[&str] = &[
        "admin", "amber", "basil", "brook", "clay", "daemon", "daisy", "dawn", "flint", "guest",
        "hazel", "heather", "holly", "iris", "ivy", "jade", "jasmine", "lily", "nobody", "olive",
        "opal", "pearl", "poppy", "river", "robin", "root", "rose", "ruby", "sage", "summer",
        "unknown", "violet", "willow", "winter", "wren",
    ];
    let words = wordlist();
    for w in &words {
        if let Some(stem) = HOSTILE.iter().find(|s| w.contains(**s)) {
            panic!("{w:?} carries the harm stem {stem:?} — the mint may not put it in a name");
        }
    }
    let pool: HashSet<&&str> = words.iter().collect();
    for id in IDENTITIES {
        assert!(!pool.contains(id), "{id:?} reads as a human identity");
    }
}
