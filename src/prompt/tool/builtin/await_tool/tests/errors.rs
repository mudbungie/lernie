//! Error-path tests: every variant of [`super::super::Error`].

use super::super::*;
use super::fixtures::{LiveRepo, NoopSleeper, StubPgidFinder, env, input_for};
use std::io::Cursor;

struct EmptyEnv;
impl EnvLookup for EmptyEnv {
    fn get(&self, _key: &str) -> Option<OsString> {
        None
    }
}

struct PartialEnv {
    only_repo: bool,
    repo: std::path::PathBuf,
}
impl EnvLookup for PartialEnv {
    fn get(&self, key: &str) -> Option<OsString> {
        if self.only_repo && key == crate::prompt::tool::ENV_CONV_REPO {
            Some(self.repo.as_os_str().to_owned())
        } else {
            None
        }
    }
}

#[test]
fn invalid_input_json_surfaces_invalid_json() {
    let mut stdin = Cursor::new(b"not json".to_vec());
    let mut stdout = Vec::new();
    let live = LiveRepo::new();
    let err = run(
        &mut stdin,
        &mut stdout,
        &EmptyEnv,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err}");
}

#[test]
fn missing_handle_field_surfaces_invalid_json() {
    let mut stdin = Cursor::new(serde_json::json!({}).to_string().into_bytes());
    let mut stdout = Vec::new();
    let live = LiveRepo::new();
    let err = run(
        &mut stdin,
        &mut stdout,
        &EmptyEnv,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err}");
}

#[test]
fn extra_field_surfaces_invalid_json() {
    let body = serde_json::json!({ "handle": "p1-sub", "extra": 1 })
        .to_string()
        .into_bytes();
    let mut stdin = Cursor::new(body);
    let mut stdout = Vec::new();
    let live = LiveRepo::new();
    let err = run(
        &mut stdin,
        &mut stdout,
        &EmptyEnv,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err}");
}

#[test]
fn missing_conv_repo_env_surfaces_missing_env() {
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let live = LiveRepo::new();
    let err = run(
        &mut stdin,
        &mut stdout,
        &EmptyEnv,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    match err {
        Error::MissingEnv(name) => assert_eq!(name, crate::prompt::tool::ENV_CONV_REPO),
        other => panic!("{other}"),
    }
}

#[test]
fn missing_conv_branch_env_surfaces_missing_env() {
    let live = LiveRepo::new();
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = PartialEnv {
        only_repo: true,
        repo: live.repo().to_path_buf(),
    };
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    match err {
        Error::MissingEnv(name) => assert_eq!(name, crate::prompt::tool::ENV_CONV_BRANCH),
        other => panic!("{other}"),
    }
}

#[test]
fn handle_not_a_descendant_is_rejected() {
    let live = LiveRepo::new();
    let mut stdin = Cursor::new(input_for("foreign-branch"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    match err {
        Error::NotADescendant { handle, parent } => {
            assert_eq!(handle, "foreign-branch");
            assert_eq!(parent, "p1");
        }
        other => panic!("{other}"),
    }
}

#[test]
fn handle_equal_to_parent_is_rejected() {
    // `<parent>` itself is not a child of `<parent>`. Catches the
    // off-by-one where `starts_with("<parent>-")` would only fail
    // when the trailing `-` is present.
    let live = LiveRepo::new();
    let mut stdin = Cursor::new(input_for("p1"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::NotADescendant { .. }), "{err}");
}

#[test]
fn handle_with_just_trailing_dash_is_rejected() {
    // `<parent>-` (empty sub-id) is also not a real descendant.
    let live = LiveRepo::new();
    let mut stdin = Cursor::new(input_for("p1-"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::NotADescendant { .. }), "{err}");
}

#[test]
fn git_failure_on_missing_handle_surfaces_typed() {
    // `handle` is well-formed (descendant prefix) but the branch
    // does not exist — the conflicted-ref check succeeds with empty
    // output, the merged check then runs `rev-parse` which fails.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    let mut stdin = Cursor::new(input_for("p1-nonexistent"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "rev-parse handle",
                ..
            }
        ),
        "{err}"
    );
}

#[test]
fn process_env_reads_live_var() {
    // Production [`ProcessEnv`] just defers to std::env. PATH is
    // always set on Linux test runs.
    let p = ProcessEnv;
    assert!(p.get("PATH").is_some());
    assert!(
        p.get("DEFINITELY_NOT_SET_LERNIE_AWAIT_TEST_xxxxx")
            .is_none()
    );
}

#[test]
fn thread_sleeper_zero_duration_returns() {
    // Smoke-test that ThreadSleeper actually wraps thread::sleep
    // without panicking. Zero duration so the test runs fast.
    let s = ThreadSleeper;
    s.sleep(std::time::Duration::from_millis(0));
}

#[test]
fn write_failure_surfaces_as_write_error() {
    // Construct a stdout that always errors on write. A merged-state
    // fixture drives the loop to the write step.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.write_summary_on("p1-sub", 1, "summary\n");
    live.run_git(&["checkout", "p1"]);
    live.run_git(&["merge", "--no-ff", "-m", "merge", "p1-sub"]);

    let mut stdin = Cursor::new(input_for("p1-sub"));
    struct FailingWrite;
    impl std::io::Write for FailingWrite {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut stdout = FailingWrite;
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Write(_)), "{err}");
}

#[test]
fn stdin_read_failure_surfaces_as_stdin_read() {
    struct FailingRead;
    impl std::io::Read for FailingRead {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("io fail"))
        }
    }
    let mut stdin = FailingRead;
    let mut stdout = Vec::new();
    let live = LiveRepo::new();
    let err = run(
        &mut stdin,
        &mut stdout,
        &EmptyEnv,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::StdinRead(_)), "{err}");
}
