//! Shared test scaffolding for the await built-in: env-var stub,
//! no-op sleeper, on-disk conv-repo helper that boots a real git
//! repository so the production [`crate::template::RealGit`] runs
//! end-to-end against a tempdir.

use super::super::*;
use crate::template::{GitRunner, ROOT_WORKTREE, RealGit};
use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

/// Minimal stub [`EnvLookup`] backed by a HashMap so tests can pin
/// `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` without touching the
/// process env (cargo test runs in parallel; mutating env is racy).
pub(super) struct StubEnv(pub(super) HashMap<&'static str, OsString>);

impl EnvLookup for StubEnv {
    fn get(&self, key: &str) -> Option<OsString> {
        self.0.get(key).cloned()
    }
}

pub(super) fn env(repo: &Path, branch: &str) -> StubEnv {
    let mut m = HashMap::new();
    m.insert(
        crate::prompt::tool::ENV_CONV_REPO,
        repo.as_os_str().to_owned(),
    );
    m.insert(crate::prompt::tool::ENV_CONV_BRANCH, OsString::from(branch));
    StubEnv(m)
}

/// Sleeper that records its call count instead of actually sleeping.
/// `await::run` calls it on each `InFlight` poll; tests that drive
/// the loop more than once read `count.get()` to assert the cadence
/// without burning real wallclock time. Callers reach the body
/// through [`ConflictOnFirstSleep`] in InFlight tests (it wraps a
/// `NoopSleeper` for its own count) or directly when they want to
/// assert "no sleep happened" on the first-poll-terminates path.
pub(super) struct NoopSleeper {
    pub(super) count: Cell<usize>,
}

impl NoopSleeper {
    pub(super) fn new() -> Self {
        Self {
            count: Cell::new(0),
        }
    }
}

impl Sleeper for NoopSleeper {
    fn sleep(&self, _dur: Duration) {
        self.count.set(self.count.get() + 1);
    }
}

#[test]
fn noop_sleeper_records_sleep_calls() {
    let s = NoopSleeper::new();
    s.sleep(Duration::from_secs(0));
    s.sleep(Duration::from_secs(0));
    assert_eq!(s.count.get(), 2);
}

/// A live conv-repo on disk, primed for await tests. Returns the
/// tempdir + a git runner pointed at the repo's `root/` worktree
/// (where `.git` lives, ARCH §2.2). Tests use [`Self::run_git`] to
/// stage refs/branches/commits per scenario.
pub(super) struct LiveRepo {
    pub(super) dir: TempDir,
    pub(super) git: RealGit,
}

impl LiveRepo {
    /// `git init -b main` inside `<tempdir>/root/`, with a single
    /// empty initial commit so subsequent branches have something to
    /// be a child of. The user identity is pinned to a deterministic
    /// value so commits do not depend on the host's `git config`.
    pub(super) fn new() -> Self {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(ROOT_WORKTREE)).unwrap();
        let git = RealGit::new();
        let cwd = dir.path().join(ROOT_WORKTREE);
        run(&git, &cwd, &["init", "-b", "main"]);
        run(&git, &cwd, &["config", "user.email", "await@test.lernie"]);
        run(&git, &cwd, &["config", "user.name", "await-test"]);
        run(&git, &cwd, &["commit", "--allow-empty", "-m", "init"]);
        Self { dir, git }
    }

    pub(super) fn repo(&self) -> &Path {
        self.dir.path()
    }

    pub(super) fn root(&self) -> PathBuf {
        self.dir.path().join(ROOT_WORKTREE)
    }

    /// Run `git <args>` against the repo's `root/` worktree. Panics
    /// on failure — tests want a loud signal if the fixture is
    /// broken.
    pub(super) fn run_git(&self, args: &[&str]) {
        run(&self.git, &self.root(), args);
    }

    /// Branch off `parent` as `child`, drop a marker file so the
    /// branch has its own commit, return the child's tip sha.
    /// Useful for setting up unmerged subagent branches.
    pub(super) fn branch_and_commit(&self, parent: &str, child: &str, marker: &str) -> String {
        let cwd = self.root();
        run(&self.git, &cwd, &["checkout", parent]);
        run(&self.git, &cwd, &["checkout", "-b", child]);
        std::fs::write(cwd.join(marker), b"x\n").unwrap();
        run(&self.git, &cwd, &["add", marker]);
        run(&self.git, &cwd, &["commit", "-m", "child commit"]);
        let sha = self
            .git
            .run_capture(&cwd, &["rev-parse", child])
            .unwrap()
            .trim()
            .to_owned();
        run(&self.git, &cwd, &["checkout", parent]);
        sha
    }

    /// Write `summary/<NNN>.md` on `branch` and commit it. Used by
    /// merged-path tests: the terminal compactor would normally do
    /// this through the `write_summary` tool (ARCH §2.7); the
    /// fixture short-circuits that to keep the test scoped to await.
    pub(super) fn write_summary_on(&self, branch: &str, seq: u32, contents: &str) {
        let cwd = self.root();
        run(&self.git, &cwd, &["checkout", branch]);
        std::fs::create_dir_all(cwd.join(SUMMARY_DIR)).unwrap();
        let rel = format!("{SUMMARY_DIR}/{seq:03}.md");
        std::fs::write(cwd.join(&rel), contents).unwrap();
        run(&self.git, &cwd, &["add", &rel]);
        run(&self.git, &cwd, &["commit", "-m", "summary"]);
    }

    /// Drop a `response.json` at `<repo>/steps/<conv-id>/<NNN>/`
    /// matching ARCH §2.3. The file lives outside any worktree.
    pub(super) fn write_response(&self, conv_id: &str, seq: u32, contents: &str) {
        let dir = self
            .repo()
            .join(STEPS_DIR)
            .join(conv_id)
            .join(format!("{seq:03}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(RESPONSE_FILE), contents).unwrap();
    }
}

fn run(git: &RealGit, cwd: &Path, args: &[&str]) {
    git.run(cwd, args)
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
}

pub(super) fn input_for(handle: &str) -> Vec<u8> {
    serde_json::json!({ "handle": handle })
        .to_string()
        .into_bytes()
}

/// Sleeper that, on its first invocation, writes the conflicted ref
/// for `handle` against the live repo. Drives any in-flight test to
/// a deterministic `conflicted` resolution on the next poll without
/// burning real wallclock time.
pub(super) struct ConflictOnFirstSleep<'a> {
    pub(super) live: &'a LiveRepo,
    pub(super) handle: &'a str,
    pub(super) count: std::cell::RefCell<usize>,
}

impl<'a> ConflictOnFirstSleep<'a> {
    pub(super) fn new(live: &'a LiveRepo, handle: &'a str) -> Self {
        Self {
            live,
            handle,
            count: std::cell::RefCell::new(0),
        }
    }
}

impl<'a> Sleeper for ConflictOnFirstSleep<'a> {
    fn sleep(&self, _dur: Duration) {
        if *self.count.borrow() == 0 {
            let ref_name = format!("refs/lernie/conflicted/{}", self.handle);
            self.live
                .run_git(&["update-ref", ref_name.as_str(), self.handle]);
        }
        *self.count.borrow_mut() += 1;
    }
}
