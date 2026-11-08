//! The separators' words, against a fixed clock.

use super::{THIS_MONTH, THIS_WEEK, TODAY, civil, epoch, now};

/// 2026-11-07T12:00:00Z.
const NOW: i64 = 1_794_052_800;
const DAY: i64 = 86_400;

/// **The calendar is the proleptic Gregorian one**, leap days included.
#[test]
fn the_days_land_on_their_dates() {
    assert_eq!(civil(0), (1970, 1, 1));
    assert_eq!(civil(NOW / DAY), (2026, 11, 7));
    assert_eq!(civil(11_016), (2000, 2, 29));
    assert_eq!(civil(-1), (1969, 12, 31));
}

/// **Today, this week, this month, then the month — and another year's
/// month carries its year.** A future age is today.
#[test]
fn an_age_stands_under_its_stretch() {
    assert_eq!(epoch(NOW, 3600), TODAY);
    assert_eq!(epoch(NOW, -60), TODAY);
    assert_eq!(epoch(NOW, 13 * 3600), THIS_WEEK, "yesterday");
    assert_eq!(epoch(NOW, 6 * DAY), THIS_WEEK);
    assert_eq!(
        epoch(NOW, 6 * DAY + 13 * 3600),
        "October",
        "the 31st is not this week"
    );
    assert_eq!(epoch(NOW + 20 * DAY, 10 * DAY), THIS_MONTH);
    assert_eq!(epoch(NOW, 22 * DAY), "October");
    assert_eq!(epoch(NOW, 62 * DAY), "September");
    assert_eq!(epoch(NOW, 400 * DAY), "October 2025");
}

/// The seat's clock is after the epoch.
#[test]
fn the_clock_is_read() {
    assert!(now() > NOW - 365 * DAY);
}
