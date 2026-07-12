//! The executor lock (ARCH §2.11 *The executor lock*).
//!
//! `flock(2)` on the agent's **inbox directory** fd, acquired
//! non-blocking when an executor starts and held for the whole step
//! loop. The lock is kernel state bound to process lifetime: released by
//! the kernel on any death, observable but never written — there is no
//! stale-lock cleanup because there is nothing on disk to go stale
//! (PRINCIPLES "Single source of truth"). The inbox directory lives at
//! the workspace root and persists across worktree teardown (§2.3
//! step 6), so the lock's home outlives the substrate's materialization.
//!
//! Two open file descriptions on the same directory contend even inside
//! one process (`flock(2)`: descriptors from separate `open` calls are
//! treated independently), so `try_acquire` is a true mutual-exclusion
//! probe: the caller who wins holds the lease, everyone else observes
//! `None` and steps aside (Writer/driver totality, §2.11).

use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;

/// A held executor lease. Dropping it closes the underlying fd, which
/// releases the `flock`; nothing is written on release. The `File` is
/// the whole state — the guard exists only to tie the kernel lease to a
/// Rust lifetime.
#[derive(Debug)]
pub struct ExecutorLock {
    // Held solely to keep the fd (and thus the lease) alive; never read.
    _fd: File,
}

/// Try to acquire the executor lock for the agent whose inbox is
/// `inbox_dir`. Non-blocking: `Ok(Some(_))` means the lease is now held
/// by the returned guard; `Ok(None)` means another executor holds it
/// (the branch is being driven); `Err` is an I/O failure opening the
/// inbox fd. The inbox directory is created on demand — a fresh agent
/// with no deposited messages still has a lock home (§2.3 step 6).
pub fn try_acquire(inbox_dir: &Path) -> io::Result<Option<ExecutorLock>> {
    let fd = open_inbox_fd(inbox_dir)?;
    lock_or_none(fd)
}

/// Open (creating if needed) the inbox directory and return an fd on it.
/// The only branch here is `create_dir_all`'s; `File::open` on a
/// just-ensured directory returns its `Result` straight through.
fn open_inbox_fd(inbox_dir: &Path) -> io::Result<File> {
    std::fs::create_dir_all(inbox_dir)?;
    File::open(inbox_dir)
}

/// `flock(LOCK_EX | LOCK_NB)` the fd, mapping the outcome to a guard.
/// The error interpretation is factored into [`interpret_lock`] so all
/// three arms are unit-testable without provoking a real syscall
/// failure.
pub(super) fn lock_or_none(fd: File) -> io::Result<Option<ExecutorLock>> {
    // SAFETY: `flock` takes a valid fd (owned by `fd`, alive for the
    // call) and a flag constant; it has no memory effects.
    let ret = unsafe { libc::flock(fd.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    interpret_lock(fd, ret, io::Error::last_os_error())
}

/// Classify a `flock` return: `0` → lease held; `EWOULDBLOCK` → someone
/// else drives; any other errno → propagate. Kept pure (takes the fd,
/// the raw return, and the captured errno) so the Err arm is reachable
/// in a test without a genuine syscall failure.
pub(super) fn interpret_lock(
    fd: File,
    ret: i32,
    err: io::Error,
) -> io::Result<Option<ExecutorLock>> {
    if ret == 0 {
        Ok(Some(ExecutorLock { _fd: fd }))
    } else if err.raw_os_error() == Some(libc::EWOULDBLOCK) {
        Ok(None)
    } else {
        Err(err)
    }
}
