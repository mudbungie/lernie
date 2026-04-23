use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;
use std::time::Duration;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write_script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn collect(stream: Stream) -> (Vec<u8>, Vec<u8>, ExitInfo) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut exit = ExitInfo::Unknown;
    for chunk in stream {
        match chunk {
            Chunk::Stdout(b) => out.extend(b),
            Chunk::Stderr(b) => err.extend(b),
            Chunk::Exited(e) => exit = e,
        }
    }
    (out, err, exit)
}

#[test]
fn new_stores_binary_path() {
    let cli = Cli::new("/usr/local/bin/lernie");
    assert_eq!(cli.binary(), Path::new("/usr/local/bin/lernie"));
}

#[test]
fn resolve_uses_env_var_when_set() {
    let _guard = ENV_LOCK.lock().unwrap();
    // SAFETY: test-only; ENV_LOCK serializes all LERNIE_BINARY access.
    unsafe { std::env::set_var("LERNIE_BINARY", "/opt/lernie-test") };
    let cli = Cli::resolve();
    unsafe { std::env::remove_var("LERNIE_BINARY") };
    assert_eq!(cli.binary(), Path::new("/opt/lernie-test"));
}

#[test]
fn resolve_falls_back_to_default_when_env_empty() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("LERNIE_BINARY", "") };
    let cli = Cli::resolve();
    unsafe { std::env::remove_var("LERNIE_BINARY") };
    assert_eq!(cli.binary(), Path::new("lernie"));
}

#[test]
fn resolve_falls_back_to_default_when_env_unset() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::remove_var("LERNIE_BINARY") };
    let cli = Cli::resolve();
    assert_eq!(cli.binary(), Path::new("lernie"));
}

#[test]
fn run_errors_on_missing_binary() {
    let cli = Cli::new("/definitely/not/a/real/binary/lernie-xyz");
    let err = match cli.run(&[]) {
        Ok(_) => panic!("expected spawn failure"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("failed to spawn"), "{msg}");
}

#[test]
fn run_streams_stdout_and_reports_exit_zero() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf 'hello out\\n'\nexit 0\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let (out, err, exit) = collect(stream);
    assert_eq!(out, b"hello out\n");
    assert!(err.is_empty());
    assert_eq!(exit, ExitInfo::Code(0));
}

#[test]
fn run_streams_stderr_and_propagates_nonzero_exit() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf 'boom\\n' 1>&2\nexit 7\n",
    );
    let cli = Cli::new(bin);
    let (out, err, exit) = collect(cli.run(&["some", "args"]).unwrap());
    assert!(out.is_empty());
    assert_eq!(err, b"boom\n");
    assert_eq!(exit, ExitInfo::Code(7));
}

#[test]
fn pid_is_available_while_running() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nexit 0\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let pid = stream.pid();
    assert!(pid.is_some());
    drop(stream);
}

#[test]
fn drop_terminates_long_running_child() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nsleep 30\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let pid = stream.pid().unwrap();
    drop(stream);
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("child pid {pid} still alive after drop");
}

#[test]
fn drop_escalates_to_sigkill_if_sigterm_ignored() {
    let dir = tempdir().unwrap();
    // Trap SIGTERM to ignore it; only SIGKILL stops the process.
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\ntrap '' TERM\nwhile :; do sleep 1; done\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let pid = stream.pid().unwrap();
    drop(stream);
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("child pid {pid} survived SIGKILL escalation");
}

#[test]
fn exit_info_reports_signal_when_child_killed_mid_flight() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf 'hi\\n'\nkill -USR1 $$\nsleep 5\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let (_out, _err, exit) = collect(stream);
    assert!(
        matches!(exit, ExitInfo::Signal(_)),
        "expected Signal, got {exit:?}"
    );
}

#[test]
fn iterator_returns_none_after_exited() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nexit 0\n",
    );
    let cli = Cli::new(bin);
    let mut stream = cli.run(&[]).unwrap();
    while let Some(chunk) = stream.next() {
        if matches!(chunk, Chunk::Exited(_)) {
            break;
        }
    }
    assert!(stream.next().is_none());
    assert!(stream.next().is_none());
}

#[test]
fn exit_info_unknown_when_status_missing() {
    assert_eq!(exit_info(None), ExitInfo::Unknown);
}

#[test]
fn exit_info_unknown_for_stopped_status() {
    use std::os::unix::process::ExitStatusExt;
    // Raw wait status 0x7f = WIFSTOPPED. On Linux this produces
    // code() == None && signal() == None (stopped_signal is separate),
    // which is our Unknown branch.
    let stopped = std::process::ExitStatus::from_raw(0x7f);
    assert_eq!(exit_info(Some(stopped)), ExitInfo::Unknown);
}

#[test]
fn pump_step_returns_false_on_eof() {
    let (tx, _rx) = mpsc::channel::<Chunk>();
    let mut reader: &[u8] = &[];
    let mut buf = [0u8; 16];
    assert!(!pump_step(&mut reader, &tx, &mut buf, Chunk::Stdout));
}

#[test]
fn pump_step_returns_false_when_receiver_dropped() {
    let (tx, rx) = mpsc::channel::<Chunk>();
    drop(rx);
    let mut reader: &[u8] = b"data";
    let mut buf = [0u8; 16];
    assert!(!pump_step(&mut reader, &tx, &mut buf, Chunk::Stdout));
}

#[test]
fn pump_step_returns_true_and_forwards_chunk() {
    let (tx, rx) = mpsc::channel::<Chunk>();
    let mut reader: &[u8] = b"abc";
    let mut buf = [0u8; 16];
    assert!(pump_step(&mut reader, &tx, &mut buf, Chunk::Stderr));
    assert_eq!(rx.try_recv().unwrap(), Chunk::Stderr(b"abc".to_vec()));
}

#[test]
fn pump_step_returns_false_on_read_error() {
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("boom"))
        }
    }
    let (tx, _rx) = mpsc::channel::<Chunk>();
    let mut reader = Failing;
    let mut buf = [0u8; 16];
    assert!(!pump_step(&mut reader, &tx, &mut buf, Chunk::Stdout));
}

fn process_exists(pid: u32) -> bool {
    // kill(pid, 0) returns 0 if alive and we can signal it, -1 with
    // errno=ESRCH if gone. After the child exits and we've wait()ed it,
    // its /proc entry is gone too.
    Path::new(&format!("/proc/{pid}")).exists()
}
