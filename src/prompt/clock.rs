//! Wall-clock and short-id generation — behind traits so
//! [`super::run`] is deterministic in tests.
//!
//! The two concerns travel together because a v0.1 exchange filename is
//! `<ts>-<short-id>.json` (ARCH §12 plus the bl-e048 task spec): the ts is
//! human-readable, the short-id breaks ties when two exchanges land in the
//! same second.

use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Wall-clock abstraction. [`SystemClock`] is the real one; tests inject
/// [`FixedClock`] so file names and `started_at`/`ended_at` are
/// deterministic.
pub trait Clock {
    /// ISO-8601 timestamp with second precision, UTC — e.g.
    /// `2026-04-22T06:54:32Z`. Used for `started_at` / `ended_at` fields
    /// in the exchange record.
    fn now_iso8601(&self) -> String;

    /// Compact filename timestamp — e.g. `20260422T065432Z`. Sorted
    /// lexically, safe in a filename on every platform we care about.
    fn now_compact(&self) -> String;
}

/// Short (hex) identifier for exchange filenames. The real impl derives
/// entropy from the wall clock at nanosecond granularity, which is enough
/// to prevent collisions between v0.1's single-threaded, human-paced
/// `lernie prompt` invocations.
pub trait IdGen {
    /// Eight hex characters. The length is a format contract — the
    /// filename convention uses it verbatim.
    fn short(&self) -> String;
}

/// Production [`Clock`] backed by `chrono::Utc::now`.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_iso8601(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    fn now_compact(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y%m%dT%H%M%SZ").to_string()
    }
}

/// Production [`IdGen`]. 32 low bits of `SystemTime`'s nanoseconds,
/// formatted as 8 hex chars.
#[derive(Debug, Clone, Copy)]
pub struct NanoIdGen;

impl IdGen for NanoIdGen {
    fn short(&self) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        format!("{nanos:08x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_formats_match_spec() {
        let c = SystemClock;
        let iso = c.now_iso8601();
        // Shape: "YYYY-MM-DDTHH:MM:SSZ" — 20 chars, ends with Z.
        assert_eq!(iso.len(), 20, "{iso}");
        assert!(iso.ends_with('Z'));
        let compact = c.now_compact();
        // Shape: "YYYYMMDDTHHMMSSZ" — 16 chars, ends with Z.
        assert_eq!(compact.len(), 16, "{compact}");
        assert!(compact.ends_with('Z'));
    }

    #[test]
    fn nano_id_gen_produces_eight_hex_chars() {
        let id = NanoIdGen.short();
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
