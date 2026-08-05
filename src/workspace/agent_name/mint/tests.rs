//! The mint, ported semantics-intact from yog `src/names` (bl-aca4):
//! RNG-start wraparound draw, bounded collision retry, loud exhaustion —
//! plus the settle-the-name pre-flight over a real workspace.

use super::*;
use crate::template::{GitRunner, RealGit};
use crate::workspace::fixture;

/// An [`Rng`] that yields one scripted draw forever — the mint takes
/// exactly one, so a fixed value pins the scan's start index.
struct Fixed(u64);

impl Rng for Fixed {
    fn next_u64(&self) -> u64 {
        self.0
    }
}

fn set(names: &[&str]) -> HashSet<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

/// Three words ⇒ a three-name pool, indices 0..3 in scan order: `ash bay cove`.
const ABC: &[&str] = &["ash", "bay", "cove"];

/// A wordlist, the RNG draw, the occupied names, and the expected mint.
type Case = (
    &'static [&'static str],
    u64,
    &'static [&'static str],
    Result<&'static str, MintError>,
);

#[test]
fn mint_scans_from_the_draw_and_retries_past_collisions() {
    let cases: &[Case] = &[
        // Draw picks the start index; nothing occupied ⇒ that word is the name.
        (ABC, 0, &[], Ok("ash")),
        (ABC, 2, &[], Ok("cove")),
        // A draw wider than the pool wraps into it (5 % 3 == 2).
        (ABC, 5, &[], Ok("cove")),
        // Collision retry: the start is taken, discarded, the scan re-samples
        // the next word.
        (ABC, 0, &["ash"], Ok("bay")),
        // Retry wraps past the end of the pool: "cove" then "ash" taken.
        (ABC, 2, &["cove", "ash"], Ok("bay")),
        // Pool exhaustion: every word of a two-word list is occupied — the
        // retry is bounded by the pool, so this errors instead of looping.
        (
            &["ash", "bay"],
            0,
            &["ash", "bay"],
            Err(MintError::Exhausted(2)),
        ),
        // The empty list is the general path with no inputs — an empty pool.
        (&[], 0, &[], Err(MintError::Exhausted(0))),
    ];
    for (words, draw, taken, expect) in cases {
        let got = mint_from(words, &Fixed(*draw), &set(taken));
        let want = expect.clone().map(str::to_owned);
        assert_eq!(got, want, "words={words:?} draw={draw} taken={taken:?}");
    }
}

#[test]
fn exhaustion_is_loud_at_both_altitudes() {
    let err = MintError::Exhausted(2);
    assert_eq!(
        err.to_string(),
        "name pool exhausted: all 2 words are occupied"
    );
    // The pre-flight's projection is transparent: the caller's uniform
    // failure line carries the same words.
    assert_eq!(Unavailable::from(err.clone()).to_string(), err.to_string());
}

/// The properties the embedded `words.txt` promises its consumer (yog
/// bl-ccf7, carried over intact). Cheap, and it pins the artifact against
/// a careless future edit — a minted name's path-safety and its "never
/// mistakable for a human identity" guarantee rest entirely on the data.
#[test]
fn embedded_wordlist_holds_its_invariants() {
    let words = wordlist();
    // (1) Parse rule: the `#` header and blank lines are gone, nothing else is.
    assert!(!words.iter().any(|w| w.starts_with('#') || w.is_empty()));
    // (2) Charset `^[a-z]{3,9}$` — what makes a minted word path-safe as a
    // name blob line and always past `require_available`'s shape gates.
    for w in &words {
        assert!(
            (3..=9).contains(&w.len()) && w.chars().all(|c| c.is_ascii_lowercase()),
            "{w:?} violates ^[a-z]{{3,9}}$"
        );
    }
    let unique: HashSet<&&str> = words.iter().collect();
    assert_eq!(unique.len(), words.len(), "duplicate word in words.txt");
    // (3) No human-identity collision: bl's own `--as` fallback literal is
    // the one word that must never be mintable.
    assert!(!unique.contains(&"unknown"));
    // (4) The curated count — the whole one-word pool, and the bound on the
    // collision-retry scan. An edit to the list is meant to update this
    // line — that is the canary, not a nuisance.
    assert_eq!(words.len(), 7395);
}

#[test]
fn mint_over_the_embedded_list_is_deterministic_and_avoids_the_occupied_set() {
    // Deterministic per seed: the same generator state mints the same name.
    let name = mint(&SplitMix64::from_seed(7), &HashSet::new()).unwrap();
    let again = mint(&SplitMix64::from_seed(7), &HashSet::new()).unwrap();
    assert_eq!(name, again);
    // The one-word shape: a single wordlist entry, never a compound.
    assert!(!name.contains('-'));
    assert!(wordlist().contains(&name.as_str()));
    // Occupying that name pushes the mint to a different one.
    let next = mint(&SplitMix64::from_seed(7), &set(&[&name])).unwrap();
    assert_ne!(next, name);
}

#[test]
fn splitmix64_advances_and_seeds_from_entropy() {
    let rng = SplitMix64::from_seed(0);
    let draws: HashSet<u64> = (0..64).map(|_| rng.next_u64()).collect();
    assert_eq!(draws.len(), 64, "generator repeated within 64 draws");
    // The entropy path is real seeding, not a constant.
    assert_ne!(
        SplitMix64::from_entropy().next_u64(),
        SplitMix64::from_seed(0).next_u64()
    );
}

#[test]
fn preflight_validates_a_supplied_name_and_mints_on_omission() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    // A living agent wearing a name occupies it for both arms.
    let wt = fixture::spawn_root(&ws, "20260101T000000Z-aaaaaaaa");
    super::super::settle(&wt, Some("pale-otter"), &git).unwrap();
    git.run(&wt, &["commit", "-m", "settle name"]).unwrap();

    // Supplied → the ordinary uniqueness gate, unchanged.
    assert_eq!(
        preflight(&ws, Some("brook"), &git, &Fixed(0)).unwrap(),
        "brook"
    );
    assert!(matches!(
        preflight(&ws, Some("pale-otter"), &git, &Fixed(0)),
        Err(Unavailable::Taken { .. })
    ));
    // Absent → minted against the same living-names scan, deterministic
    // under the injected RNG, and never a name a living agent wears.
    let minted = preflight(&ws, None, &git, &SplitMix64::from_seed(7)).unwrap();
    assert_eq!(
        minted,
        preflight(&ws, None, &git, &SplitMix64::from_seed(7)).unwrap()
    );
    assert!(wordlist().contains(&minted.as_str()));
    assert_ne!(minted, "pale-otter");
    // An unreadable workspace is the scan's refusal, not a raw git error.
    assert!(matches!(
        preflight(Path::new("/nonexistent"), None, &git, &Fixed(0)),
        Err(Unavailable::Scan(_))
    ));
}
