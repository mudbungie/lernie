//! **The seat's one reading of time**, injected so the suite can move it by
//! hand (DESIGN §4.40).
//!
//! Two questions and nothing else: how long ago something happened
//! ([`Clock::now`], monotonic — what a held connection's silence is measured
//! against) and what the wall says ([`Clock::unix`] — what a BEP 44 sequence
//! number is drawn from, so a call written by a later run of this seat is
//! newer than one written by an earlier run without the seat keeping a file).
//! `Arc<dyn Clock>` rather than a generic, so the channel's public surface
//! carries no bound (README: *no generic bounds on a `pub` item*).

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// A source of time.
pub trait Clock: Send + Sync + std::fmt::Debug {
    /// The monotonic instant.
    fn now(&self) -> Instant;
    /// Whole seconds since the epoch.
    fn unix(&self) -> i64;
}

/// The system's clock — what every production channel reads.
#[derive(Debug)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn unix(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .unwrap_or_default()
    }
}

/// The production clock, shared.
pub fn system() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_clock_reads_the_wall_and_moves_forward() {
        let clock = system();
        let (a, unix) = (clock.now(), clock.unix());
        assert!(unix > 1_700_000_000, "{unix}");
        assert!(clock.now() >= a);
    }
}
