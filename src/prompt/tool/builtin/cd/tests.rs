//! Unit tests for [`super::run_with`] (ARCH §3.3 *Working directory*).
//! One branch per test so a coverage regression names the path it broke.

use super::*;
use crate::template::RealGit;
use crate::workspace::fixture;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Cursor;

/// HashMap-backed stub [`EnvLookup`] — `None` for anything not seeded.
struct StubEnv(HashMap<&'static str, OsString>);
impl EnvLookup for StubEnv {
    fn get(&self, key: &str) -> Option<OsString> {
        self.0.get(key).cloned()
    }
}

fn env(repo: &Path, branch: OsString) -> StubEnv {
    let mut m = HashMap::new();
    m.insert(ENV_CONV_REPO, repo.as_os_str().to_owned());
    m.insert(ENV_CONV_BRANCH, branch);
    StubEnv(m)
}

fn input(path: &str) -> Cursor<Vec<u8>> {
    Cursor::new(serde_json::json!({ "path": path }).to_string().into_bytes())
}

/// A real workspace with one root agent `a` — what the executor's env
/// vars name on every tool call.
fn agent() -> (tempfile::TempDir, PathBuf) {
    let (holder, ws) = fixture::workspace();
    fixture::spawn_root(&ws, "a");
    (holder, ws)
}

fn cwd_of(ws: &Path) -> Option<PathBuf> {
    workspace::cwd::read(ws, "a", &RealGit::new())
}

#[test]
fn an_absolute_directory_becomes_the_agents_working_directory() {
    let (_h, ws) = agent();
    let target = ws.join("agents/a");
    let mut out = Vec::new();
    run_with(
        &mut input(&target.to_string_lossy()),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap();

    let payload: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let canonical = std::fs::canonicalize(&target).unwrap();
    assert_eq!(payload["cwd"], canonical.to_string_lossy().as_ref());
    // The one product is the cwd; no constant `status` rides along.
    assert_eq!(payload.as_object().unwrap().len(), 1);
    assert_eq!(cwd_of(&ws), Some(canonical));
}

#[test]
fn a_relative_path_resolves_against_this_processs_own_cwd() {
    // The executor spawns the tool *in* the agent's current working
    // directory, so a relative path needs no re-derivation — which is
    // what this asserts, using the test process's own cwd as the stand-in.
    let (_h, ws) = agent();
    let mut out = Vec::new();
    run_with(
        &mut input("src/prompt"),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap();
    let expected = std::env::current_dir().unwrap().join("src/prompt");
    assert_eq!(cwd_of(&ws), Some(std::fs::canonicalize(expected).unwrap()));
}

#[test]
fn a_dot_dot_path_is_resolved_not_stored_literally() {
    let (_h, ws) = agent();
    let mut out = Vec::new();
    run_with(
        &mut input(&ws.join("agents/a/../a").to_string_lossy()),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap();
    let stored = cwd_of(&ws).unwrap();
    assert!(!stored.to_string_lossy().contains(".."), "{stored:?}");
}

#[test]
fn a_nonexistent_directory_is_declined_and_the_agent_stays_put() {
    let (_h, ws) = agent();
    let mut out = Vec::new();
    let err = run_with(
        &mut input("/no/such/place/at/all"),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("no such directory"), "{err}");
    assert!(out.is_empty());
    assert_eq!(cwd_of(&ws), None);
}

#[test]
fn a_file_is_declined_as_not_a_directory() {
    let (_h, ws) = agent();
    let file = ws.join("agents/a/goal.md");
    let mut out = Vec::new();
    let err = run_with(
        &mut input(&file.to_string_lossy()),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not a directory"), "{err}");
    assert_eq!(cwd_of(&ws), None);
}

#[test]
fn a_directory_outside_the_worktree_is_permitted() {
    // No containment: v1.0 bounds authority nowhere (§3.6 defers it to
    // the v1.1 sandbox), and `bash` could already reach here.
    let (_h, ws) = agent();
    let outside = tempfile::TempDir::new().unwrap();
    let mut out = Vec::new();
    run_with(
        &mut input(&outside.path().to_string_lossy()),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap();
    let expected = std::fs::canonicalize(outside.path()).unwrap();
    assert_eq!(cwd_of(&ws), Some(expected));
}

#[test]
fn malformed_input_json_is_declined() {
    let (_h, ws) = agent();
    let mut out = Vec::new();
    let err = run_with(
        &mut Cursor::new(b"{\"path\": 7}".to_vec()),
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("invalid input JSON"), "{err}");
}

#[test]
fn a_stdin_that_fails_mid_read_is_declined() {
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("pipe died"))
        }
    }
    let (_h, ws) = agent();
    let mut out = Vec::new();
    let err = run_with(
        &mut Broken,
        &mut out,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("read input from stdin"), "{err}");
}

#[test]
fn a_missing_workspace_env_var_is_declined() {
    let mut out = Vec::new();
    let err = run_with(
        &mut input("/tmp"),
        &mut out,
        &StubEnv(HashMap::new()),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains(ENV_CONV_REPO), "{err}");
}

#[test]
fn a_missing_branch_env_var_is_declined() {
    let (_h, ws) = agent();
    let mut m = HashMap::new();
    m.insert(ENV_CONV_REPO, ws.as_os_str().to_owned());
    let mut out = Vec::new();
    let err = run_with(&mut input("/tmp"), &mut out, &StubEnv(m), &RealGit::new()).unwrap_err();
    assert!(err.to_string().contains(ENV_CONV_BRANCH), "{err}");
}

#[test]
fn a_non_utf8_branch_is_declined_rather_than_lossily_named() {
    use std::os::unix::ffi::OsStringExt;
    let (_h, ws) = agent();
    let mut out = Vec::new();
    let err = run_with(
        &mut input("/tmp"),
        &mut out,
        &env(&ws, OsString::from_vec(b"a\xff".to_vec())),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains(ENV_CONV_BRANCH), "{err}");
}

#[test]
fn a_mark_that_cannot_be_stored_is_declined() {
    let holder = tempfile::TempDir::new().unwrap();
    let mut out = Vec::new();
    let err = run_with(
        &mut input("/tmp"),
        &mut out,
        &env(holder.path(), OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("store the working directory"),
        "{err}"
    );
    assert!(out.is_empty());
}

#[test]
fn a_stdout_that_will_not_take_the_result_is_declined() {
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let (_h, ws) = agent();
    let err = run_with(
        &mut input("/tmp"),
        &mut Broken,
        &env(&ws, OsString::from("a")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("write to stdout"), "{err}");
}

#[test]
fn the_production_entry_uses_the_real_git() {
    let (_h, ws) = agent();
    let mut out = Vec::new();
    run(&mut input("/tmp"), &mut out, &env(&ws, OsString::from("a"))).unwrap();
    assert_eq!(cwd_of(&ws), Some(std::fs::canonicalize("/tmp").unwrap()));
}
