use super::*;
use std::cell::RefCell;
use tempfile::TempDir;

// --- check_dest --------------------------------------------------
#[test]
fn check_dest_allows_missing_path() {
    let holder = TempDir::new().unwrap();
    let missing = holder.path().join("nope");
    check_dest(&missing).unwrap();
}

#[test]
fn check_dest_allows_empty_directory() {
    let holder = TempDir::new().unwrap();
    check_dest(holder.path()).unwrap();
}

#[test]
fn check_dest_rejects_non_empty_directory() {
    let holder = TempDir::new().unwrap();
    fs::write(holder.path().join("occupant"), b"x").unwrap();
    let err = check_dest(holder.path()).unwrap_err();
    assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
}

#[test]
fn check_dest_surfaces_other_io_errors() {
    // read_dir on a regular file fails with kind != NotFound, so the
    // third arm of check_dest fires.
    let holder = TempDir::new().unwrap();
    let file = holder.path().join("actually-a-file");
    fs::write(&file, b"not a dir").unwrap();
    let err = check_dest(&file).unwrap_err();
    assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
}

// --- scaffold orchestration via stub GitRunner -------------------
/// Records every `git` subprocess run and can be programmed to fail
/// at a chosen run index.
struct StubGit {
    runs: RefCell<Vec<Vec<String>>>,
    fail_at: Option<usize>,
}

impl StubGit {
    fn ok() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: None,
        }
    }
    fn failing_at(idx: usize) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: Some(idx),
        }
    }
}

impl GitRunner for StubGit {
    fn run(&self, _dest: &Path, args: &[&str]) -> io::Result<()> {
        let mut runs = self.runs.borrow_mut();
        let idx = runs.len();
        runs.push(args.iter().map(|s| (*s).to_owned()).collect());
        if self.fail_at == Some(idx) {
            Err(io::Error::other(format!("stub fail at {idx}")))
        } else {
            Ok(())
        }
    }

    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        self.run(dest, args).map(|_| String::new())
    }
}

#[test]
fn scaffold_happy_path_runs_git_in_order() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let git = StubGit::ok();
    scaffold(&dest, &git).unwrap();
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0], vec!["init", "-b", "main"]);
    assert_eq!(runs[1], vec!["add", "-A"]);
    assert_eq!(runs[2], vec!["commit", "-m", "init conversation repo"]);
    assert!(dest.join(".agent/version").is_file());
    assert!(dest.join(".agent/system/prompts/base.md").is_file());
}

#[test]
fn scaffold_propagates_init_failure() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let err = scaffold(&dest, &StubGit::failing_at(0)).unwrap_err();
    assert!(matches!(err, ScaffoldError::Git(_)), "got {err:?}");
}

#[test]
fn scaffold_propagates_add_failure() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let err = scaffold(&dest, &StubGit::failing_at(1)).unwrap_err();
    assert!(matches!(err, ScaffoldError::Git(_)));
}

#[test]
fn scaffold_propagates_commit_failure() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let err = scaffold(&dest, &StubGit::failing_at(2)).unwrap_err();
    assert!(matches!(err, ScaffoldError::Git(_)));
}

#[test]
fn scaffold_refuses_non_empty_dest() {
    let holder = TempDir::new().unwrap();
    fs::write(holder.path().join("x"), b"x").unwrap();
    let err = scaffold(holder.path(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
}

#[test]
fn scaffold_surfaces_extract_io_error() {
    // A path segment that exists as a regular file makes include_dir's
    // `extract` fail when it tries to create the sub-directory —
    // hits the `ScaffoldError::Io` arm of scaffold().
    let holder = TempDir::new().unwrap();
    let blocker = holder.path().join("blocker");
    fs::write(&blocker, b"blocks extraction").unwrap();
    let dest = blocker.join("child");
    let err = scaffold(&dest, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
}

// --- RealGit -----------------------------------------------------
#[test]
fn realgit_default_matches_new() {
    let _ = RealGit::default();
}

#[test]
fn realgit_succeeds_on_valid_command() {
    let holder = TempDir::new().unwrap();
    RealGit::new()
        .run(holder.path(), &["init", "-b", "main"])
        .unwrap();
    assert!(holder.path().join(".git").is_dir());
}

#[test]
fn realgit_returns_error_on_nonzero_exit() {
    let holder = TempDir::new().unwrap();
    // No git repo here, so `git status` exits non-zero. That hits
    // the `!status.success()` branch without needing a missing
    // binary.
    let err = RealGit::new()
        .run(holder.path(), &["status", "--porcelain"])
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("exited with"), "unexpected: {msg}");
}

#[test]
fn realgit_returns_error_when_binary_missing() {
    let holder = TempDir::new().unwrap();
    let git = RealGit {
        bin: PathBuf::from("/no/such/lernie-test-git"),
    };
    let err = git.run(holder.path(), &["init"]).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}

#[test]
fn realgit_run_capture_returns_stdout() {
    // `git --version` prints a line to stdout that RealGit trims.
    let holder = TempDir::new().unwrap();
    let out = RealGit::new()
        .run_capture(holder.path(), &["--version"])
        .unwrap();
    assert!(out.starts_with("git "), "unexpected: {out:?}");
}

#[test]
fn stub_run_capture_delegates_to_run() {
    let holder = TempDir::new().unwrap();
    assert_eq!(
        StubGit::ok().run_capture(holder.path(), &["x"]).unwrap(),
        ""
    );
    assert!(
        StubGit::failing_at(0)
            .run_capture(holder.path(), &["x"])
            .is_err()
    );
}
