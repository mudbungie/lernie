//! The suite's deterministic [`Clock`]: an origin plus an offset the test
//! advances by hand, so a held connection's whole silence bound is crossed
//! without sleeping (DESIGN §4.40).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::channel::clock::Clock;

/// A clock every handle of which shares one offset — advancing the test's
/// moves the one the channel holds.
#[derive(Clone, Debug)]
pub(crate) struct FakeClock {
    origin: Instant,
    unix: i64,
    advanced: Arc<AtomicU64>,
}

impl FakeClock {
    pub(crate) fn new() -> Self {
        Self {
            origin: Instant::now(),
            unix: 1_800_000_000,
            advanced: Arc::new(AtomicU64::new(0)),
        }
    }

    /// This clock as the shared trait object the channel takes.
    pub(crate) fn arc(&self) -> Arc<dyn Clock> {
        Arc::new(self.clone())
    }

    pub(crate) fn advance(&self, by: Duration) {
        self.advanced
            .fetch_add(u64::try_from(by.as_nanos()).unwrap(), Ordering::Relaxed);
    }

    fn offset(&self) -> Duration {
        Duration::from_nanos(self.advanced.load(Ordering::Relaxed))
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.origin + self.offset()
    }

    fn unix(&self) -> i64 {
        self.unix + i64::try_from(self.offset().as_secs()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_handle_moves_together() {
        let clock = FakeClock::new();
        let shared = clock.arc();
        let (t0, u0) = (shared.now(), shared.unix());
        clock.advance(Duration::from_secs(90));
        assert_eq!(shared.now() - t0, Duration::from_secs(90));
        assert_eq!(shared.unix() - u0, 90);
    }
}
