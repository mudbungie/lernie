//! **Where a row's age stands on the calendar** (DESIGN §4.48 item 5): the
//! words the bar parts its listing with — `today`, `this week`, `this
//! month`, then the month's name — read off [`super::age`]'s own input, the
//! seconds since the row moved, against the seat's clock.
//!
//! The calendar is UTC's: the seat carries no time zone, and a separator a
//! few hours off at midnight parts the same rows in the same order.

/// The words, before the month's name.
pub const TODAY: &str = "today";
pub const THIS_WEEK: &str = "this week";
pub const THIS_MONTH: &str = "this month";

/// The months' names, January first.
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Seconds in a day.
const DAY: i64 = 86_400;

/// **The year, month (1–12) and day of the day `days` after 1970-01-01** —
/// the proleptic Gregorian calendar, by eras of 400 years.
fn civil(days: i64) -> (i64, usize, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, usize::try_from(month).unwrap_or(1), day)
}

/// **The separator a row `age_secs` old stands under, at `now`** (seconds
/// since the epoch). An age in the future is today, as [`super::age`]
/// clamps it; a month of another year carries the year.
pub fn epoch(now: i64, age_secs: i64) -> String {
    let (today, then) = (now.div_euclid(DAY), (now - age_secs.max(0)).div_euclid(DAY));
    let ((year, month, _), (was_year, was_month, _)) = (civil(today), civil(then));
    if then == today {
        TODAY.to_owned()
    } else if today - then < 7 {
        THIS_WEEK.to_owned()
    } else if (year, month) == (was_year, was_month) {
        THIS_MONTH.to_owned()
    } else {
        let name = MONTHS.get(was_month - 1).copied().unwrap_or_default();
        if was_year == year {
            name.to_owned()
        } else {
            format!("{name} {was_year}")
        }
    }
}

/// **The seat's clock**, in seconds since the epoch — zero on a clock set
/// before it, which parts every row under the epoch's own month.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        })
}

#[cfg(test)]
mod tests;
