//! Shared test scaffolding for the dispatch built-in: env-var stub,
//! subprocess-spawn stub, fake-conv-repo helper, and a few small
//! constructors that keep the per-test setup terse.

use super::super::*;
use std::cell::RefCell;
use std::collections::HashMap;
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

/// Stub spawner records the call args and returns a canned outcome.
pub(super) struct StubSpawner {
    pub(super) out: DispatchOutput,
    pub(super) calls: RefCell<Vec<(String, PathBuf, String, String)>>,
}

impl StubSpawner {
    pub(super) fn ok(handle: &str) -> Self {
        Self {
            out: DispatchOutput {
                stdout: format!("{handle}\n"),
                stderr: String::new(),
                exit: 0,
            },
            calls: RefCell::new(Vec::new()),
        }
    }
    pub(super) fn failing(stderr: &str, exit: i32) -> Self {
        Self {
            out: DispatchOutput {
                stdout: String::new(),
                stderr: stderr.to_string(),
                exit,
            },
            calls: RefCell::new(Vec::new()),
        }
    }
    pub(super) fn empty_stdout() -> Self {
        Self {
            out: DispatchOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit: 0,
            },
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl Spawner for StubSpawner {
    fn dispatch(
        &self,
        role: &str,
        repo: &Path,
        branch: &str,
        goal: &str,
    ) -> Result<DispatchOutput, io::Error> {
        self.calls.borrow_mut().push((
            role.to_string(),
            repo.to_path_buf(),
            branch.to_string(),
            goal.to_string(),
        ));
        Ok(DispatchOutput {
            stdout: self.out.stdout.clone(),
            stderr: self.out.stderr.clone(),
            exit: self.out.exit,
        })
    }
}

/// Spawner whose `dispatch` always fails at the io layer — exercises
/// [`Error::Spawn`].
pub(super) struct ErrSpawner;
impl Spawner for ErrSpawner {
    fn dispatch(
        &self,
        _role: &str,
        _repo: &Path,
        _branch: &str,
        _goal: &str,
    ) -> Result<DispatchOutput, io::Error> {
        Err(io::Error::new(io::ErrorKind::NotFound, "no lernie binary"))
    }
}

/// Build a fake conv-repo with a `providers.yaml` that lists `role`
/// and a soul at `souls/<role>.md`. Mirrors the v0.4 templated
/// scaffold's per-repo shape (ARCH §4.3) — minimal because validation
/// only inspects the `roles:` keys and the soul-file existence.
pub(super) fn fake_repo(role: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();
    let yaml = format!("roles:\n  {role}:\n    provider: anthropic\n    model: sonnet\n",);
    std::fs::write(repo.join("providers.yaml"), yaml).unwrap();
    std::fs::create_dir_all(repo.join("souls")).unwrap();
    std::fs::write(repo.join("souls").join(format!("{role}.md")), "soul body\n").unwrap();
    dir
}

pub(super) fn input_for(role: &str, goal: &str) -> Vec<u8> {
    serde_json::json!({ "role": role, "goal": goal })
        .to_string()
        .into_bytes()
}
