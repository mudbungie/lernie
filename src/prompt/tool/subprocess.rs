//! Subprocess plumbing for the tool executor — spawn, capture, wait,
//! cascade.
//!
//! Capture is one writer thread (stdin) plus two reader threads
//! (stdout, stderr) joined after the child exits, so a tool that
//! ignores stdin or fills its stdout pipe cannot deadlock the wait
//! loop. The main thread polls `stop` every [`POLL_INTERVAL`]; on stop
//! it sends SIGTERM, waits the deadline, then SIGKILL — same pattern
//! ARCH §4.4 pins for adapters.
//!
//! Errors on the stdin writer are intentionally swallowed: the §3.3
//! contract only requires stdin be *offered* to the tool; whether the
//! tool reads it is the tool's choice.
//!
//! Every child is spawned with an explicit [`SpawnArgs::cwd`] — the
//! calling agent's worktree, resolved by the executor (§3.3 *Working
//! directory*). This layer never lets a tool inherit the harness's own
//! working directory.

use super::ExecError;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Cadence for the stop-flag polling loop. Small enough that a UI
/// cancel feels instant; large enough that an idle harness costs
/// nothing measurable.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Maximum time to spend retrying past `ETXTBSY` before treating it
/// as a real spawn failure. The race window between a sibling
/// process forking with the binary still open for write and that
/// child reaching `exec` is bounded by the kernel's exec
/// transition; a few tens of milliseconds covers worst-case under
/// heavy parallel-test load.
/// Injected per-executor ([`super::SpawnTool::with_etxtbsy_budget`])
/// so a test can decide which arm it is exercising instead of racing
/// this value with its own fixture hold: pitting a hold measured on
/// one clock against a budget measured on another makes the verdict a
/// property of machine load, and the loser is whichever agent's close
/// gate is running (bl-1c2e, bl-7a3f).
pub(super) const ETXTBSY_RETRY_BUDGET: Duration = Duration::from_millis(200);

/// Backoff between `ETXTBSY` retries. Short enough that the race
/// window closes quickly; not so short that we busy-spin the kernel.
const ETXTBSY_RETRY_INTERVAL: Duration = Duration::from_millis(2);

/// Captured tool stdio + final status.
pub(super) struct Captured {
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
    pub(super) status: ExitStatus,
}

/// Inputs to [`spawn_and_capture`]. Bundled into a struct so the call
/// site stays a single line — the function takes 7 arguments which is
/// past readable for a positional call (and past `tarpaulin`'s
/// instrumentation comfort, which mis-attributes coverage on
/// multi-line `&borrow` argument lists).
pub(super) struct SpawnArgs<'a> {
    pub(super) binary: &'a OsString,
    pub(super) args: &'a [OsString],
    pub(super) stdin_bytes: &'a [u8],
    /// (key, value) pairs added to the inherited environment per ARCH
    /// §3.3 (the harness conveys conversation context —
    /// `LERNIE_CONV_REPO`, `LERNIE_CONV_BRANCH` — to tools through
    /// env vars rather than the model-facing input schema).
    pub(super) extra_env: &'a [(&'a str, OsString)],
    /// Working directory for the child — the calling agent's worktree
    /// per ARCH §3.3 (*Working directory*). Never inherited: a tool
    /// left in the harness's own cwd would write its side effects into
    /// whatever directory the operator's shell was pointing at instead
    /// of onto the agent's branch.
    pub(super) cwd: &'a Path,
    pub(super) stop: &'a AtomicBool,
    pub(super) deadline: Duration,
    /// How long [`spawn_with_etxtbsy_retry`] rides out `ETXTBSY`.
    pub(super) etxtbsy_budget: Duration,
    pub(super) tool_name: &'a str,
}

/// Spawn `binary args`, write `stdin_bytes`, capture stdio in
/// background threads, and reap the child. Polls `stop` every
/// [`POLL_INTERVAL`]; when set, sends SIGTERM and waits up to
/// `deadline` before SIGKILL.
pub(super) fn spawn_and_capture(req: &SpawnArgs<'_>) -> Result<Captured, ExecError> {
    let mut child = spawn_with_etxtbsy_retry(req)?;

    let stdin_data = req.stdin_bytes.to_vec();
    let mut child_stdin = child.stdin.take().expect("stdin is piped");
    let stdin_thread = thread::spawn(move || {
        let _ = child_stdin.write_all(&stdin_data);
        drop(child_stdin);
    });

    let mut child_stdout = child.stdout.take().expect("stdout is piped");
    let mut child_stderr = child.stderr.take().expect("stderr is piped");
    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = child_stdout.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = child_stderr.read_to_end(&mut buf);
        buf
    });

    let status = wait_with_stop(&mut child, req.stop, req.deadline);
    stdin_thread.join().expect("stdin writer did not panic");
    let stdout = stdout_thread.join().expect("stdout reader did not panic");
    let stderr = stderr_thread.join().expect("stderr reader did not panic");

    Ok(Captured {
        stdout,
        stderr,
        status,
    })
}

/// `Command::spawn` with a bounded retry past `ETXTBSY` ("text file
/// busy"). The race target is a sibling process holding the binary's
/// write fd open across `fork`+`exec`: the parent has closed its fd,
/// but a forked child has briefly inherited it and not yet reached
/// `exec` to release it (CLOEXEC fires at exec, not fork). The kernel
/// rejects the parallel exec until that child transitions, which is
/// bounded by the kernel's own exec scheduling — typically
/// sub-millisecond, occasionally tens of ms under load. A short retry
/// budget keeps the harness robust without masking real spawn
/// failures.
fn spawn_with_etxtbsy_retry(req: &SpawnArgs<'_>) -> Result<Child, ExecError> {
    let deadline = Instant::now() + req.etxtbsy_budget;
    loop {
        let mut cmd = Command::new(req.binary);
        cmd.args(req.args)
            .current_dir(req.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in req.extra_env {
            cmd.env(k, v);
        }
        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(e) if e.raw_os_error() == Some(libc::ETXTBSY) && Instant::now() < deadline => {
                thread::sleep(ETXTBSY_RETRY_INTERVAL);
            }
            Err(source) => {
                return Err(ExecError::Spawn {
                    name: req.tool_name.to_string(),
                    source,
                });
            }
        }
    }
}

/// Poll `child` for completion. When `stop` flips, send SIGTERM, wait
/// up to `deadline`, then SIGKILL. Returns whatever final
/// [`ExitStatus`] the kernel reports — a tool that exits cleanly
/// inside the deadline reports its real exit code (and is therefore
/// not flagged `KilledBySignal` upstream).
/// The reap comes *before* the flag read, and the poll interval between
/// them, for the reason spelled out on `builtin::bash::wait_with_cascade`:
/// a running child is what puts us in the interval, so the line is
/// reached because of the child rather than because a stop had not
/// landed yet — the latter is a race the machine's load decides, and it
/// costs the 100% floor a line (bl-1c2e).
fn wait_with_stop(child: &mut Child, stop: &AtomicBool, deadline: Duration) -> ExitStatus {
    loop {
        if let Some(status) = try_reap(child) {
            return status;
        }
        thread::sleep(POLL_INTERVAL);
        if stop.load(Ordering::SeqCst) {
            return cascade_terminate(child, deadline);
        }
    }
}

/// `try_wait`-as-`Option` wrapper. An I/O error reaping a child the
/// kernel still owns is treated as "not exited yet" — the next poll
/// retries.
fn try_reap(child: &mut Child) -> Option<ExitStatus> {
    child.try_wait().ok().flatten()
}

/// Send SIGTERM, wait `deadline`, then SIGKILL. SIGKILL is
/// uncatchable so the final [`Child::wait`] is bounded by kernel reap
/// latency (microseconds).
fn cascade_terminate(child: &mut Child, deadline: Duration) -> ExitStatus {
    let pid = child.id() as i32;
    // SAFETY: kill on a child PID we own; signo is a constant.
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    let term_until = Instant::now() + deadline;
    while Instant::now() < term_until {
        if let Some(status) = try_reap(child) {
            return status;
        }
        thread::sleep(POLL_INTERVAL);
    }
    // SAFETY: same as above; SIGKILL is uncatchable.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
    child.wait().expect("kernel reaps SIGKILL'd child")
}
