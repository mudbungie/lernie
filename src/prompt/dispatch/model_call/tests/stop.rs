//! The §2.9 bound on the §2.10 retry loop: once a stop is pending the
//! loop must not launch a further `bz`.
//!
//! A `bz` spawned *after* the group SIGTERM is outside the cascade's
//! reach (§2.9 step 1) — nothing would fell it — so retrying through a
//! stop spends a whole additional model call, the window that dominates
//! a stop's observed latency. The error the loop returns instead is the
//! attempt's own `AdapterError`; the caller's §2.9 step-3 check point
//! discards it and settles the branch as stopped.

use super::*;

/// Both retry check points are read: one before the backoff (the stop
/// landed during the attempt — the group signal reaches the executor and
/// `bz` together) and one after it (the stop landed inside the pause).
#[test]
fn stop_pending_before_the_backoff_ends_the_retry_loop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("steps/c/001/response.json");
    let sleeper = RecSleeper::default();
    let stop = AtomicBool::new(true);
    let (result, stdins) = run_injected(
        &path,
        StubAdapter::new(vec![Ok(error_stream(ErrorKind::Transport))]),
        retry(3),
        false,
        &sleeper,
        &stop,
    );
    // Retryable, budget to spare — but the stop outranks both.
    assert!(matches!(result, Err(Error::AdapterError { .. })));
    assert_eq!(stdins.len(), 1, "no further `bz` after a stop");
    assert_eq!(sleeper.0.borrow().len(), 0, "no backoff slept under a stop");
}

#[test]
fn stop_landing_during_the_backoff_ends_the_retry_loop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("steps/c/001/response.json");
    let stop = AtomicBool::new(false);
    let sleeper = StoppingSleeper {
        flag: &stop,
        slept: RefCell::new(Vec::new()),
    };
    let (result, stdins) = run_injected(
        &path,
        StubAdapter::new(vec![Ok(error_stream(ErrorKind::Transport))]),
        retry(3),
        false,
        &sleeper,
        &stop,
    );
    assert!(matches!(result, Err(Error::AdapterError { .. })));
    assert_eq!(sleeper.slept.borrow().len(), 1, "the backoff was entered");
    assert_eq!(
        stdins.len(),
        1,
        "the far-side re-read caught the stop before another attempt"
    );
}

/// The bound is a *stop* bound, not a retry ban: with no stop pending
/// the same script retries and recovers (the §2.10 contract intact).
#[test]
fn no_stop_leaves_the_retry_loop_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("steps/c/001/response.json");
    let sleeper = RecSleeper::default();
    let stop = AtomicBool::new(false);
    let (result, stdins) = run_injected(
        &path,
        StubAdapter::new(vec![
            Ok(error_stream(ErrorKind::Transport)),
            Ok(text_stream("recovered", FinishReason::Stop)),
        ]),
        retry(3),
        false,
        &sleeper,
        &stop,
    );
    result.unwrap();
    assert_eq!(stdins.len(), 2, "the retry ran");
    assert_eq!(sleeper.0.borrow().len(), 1);
}
