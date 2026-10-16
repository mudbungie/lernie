//! **The clock as a value**: the linger runs from when the pointer left,
//! and the knob is its length (DESIGN §4.45 item 1; §4.46 item 4).

use super::super::{Bar, LINGER};

/// **The clock, as a value**: the pointer over keeps it open; leaving starts
/// the linger, which asks for the repaint at the moment it lapses; at
/// [`LINGER`] — 0.75 s, the number read off the ruling (DESIGN §4.45 item
/// 1) — it shuts and forgets; and the pin holds it open with no repaint owed.
#[test]
fn the_linger_runs_from_when_the_pointer_left() {
    assert!((LINGER - 0.75).abs() < f32::EPSILON, "the ruling's number");
    let linger = f64::from(LINGER);
    let mut bar = Bar::default();
    assert!(!bar.open(0.0, linger), "shut at rest");
    assert_eq!(bar.heard(0.0, false, linger), None);
    assert_eq!(bar.heard(1.0, true, linger), None);
    assert!(bar.open(1.0, linger));
    assert_eq!(
        bar.heard(1.5, false, linger),
        Some(0.25),
        "a quarter second left"
    );
    assert!(bar.open(1.5, linger), "lingering");
    assert!(!bar.open(1.0 + linger, linger));
    assert_eq!(bar.heard(1.0 + linger, false, linger), None);
    assert_eq!(bar.left_at, None, "forgotten once lapsed");
    // **The linger is the knob's** (DESIGN §4.46 item 4): a longer one keeps
    // the same bar open past the ruling's number.
    bar.left_at = Some(1.0);
    assert!(bar.open(1.0 + linger, 2.0), "a longer linger");
    bar.pinned = true;
    assert!(bar.open(9.0, linger), "pinned");
    bar.left_at = Some(9.0);
    assert_eq!(
        bar.heard(9.5, false, linger),
        None,
        "a pinned bar owes no repaint"
    );
}
