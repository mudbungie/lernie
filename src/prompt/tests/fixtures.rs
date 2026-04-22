//! Shared stubs and fixtures for `prompt::tests::*`.

use crate::prompt::{AdapterRunner, Clock, Deps, IdGen};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use tempfile::TempDir;

/// Deterministic [`Clock`] — counts how many times each method was called
/// and returns a formatted counter, so `started_at` / `ended_at` and the
/// filename ts are all distinct and observable.
pub(super) struct FixedClock {
    iso_calls: RefCell<u32>,
    compact_calls: RefCell<u32>,
}

impl FixedClock {
    pub(super) fn new() -> Self {
        Self {
            iso_calls: RefCell::new(0),
            compact_calls: RefCell::new(0),
        }
    }
}

impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        *self.iso_calls.borrow_mut() += 1;
        format!("iso-{}", self.iso_calls.borrow())
    }
    fn now_compact(&self) -> String {
        *self.compact_calls.borrow_mut() += 1;
        format!("ct-{}", self.compact_calls.borrow())
    }
}

pub(super) struct FixedIdGen;
impl IdGen for FixedIdGen {
    fn short(&self) -> String {
        "deadbeef".into()
    }
}

/// Scripted [`AdapterRunner`] — returns a canned stdout or an error.
enum AdapterReply {
    Ok(Vec<u8>),
    Err(io::Error),
}

/// Snapshot of a single adapter invocation captured by [`StubAdapter`]: the
/// binary name, argv, and stdin bytes.
pub(super) type AdapterCall = (OsString, Vec<String>, Vec<u8>);

pub(super) struct StubAdapter {
    reply: RefCell<Option<AdapterReply>>,
    pub(super) observed: RefCell<Option<AdapterCall>>,
}

impl StubAdapter {
    pub(super) fn returning_ok(bytes: &[u8]) -> Self {
        Self {
            reply: RefCell::new(Some(AdapterReply::Ok(bytes.to_vec()))),
            observed: RefCell::new(None),
        }
    }
    pub(super) fn returning_err(kind: io::ErrorKind, msg: &str) -> Self {
        Self {
            reply: RefCell::new(Some(AdapterReply::Err(io::Error::new(
                kind,
                msg.to_string(),
            )))),
            observed: RefCell::new(None),
        }
    }
}

impl AdapterRunner for StubAdapter {
    fn run(&self, binary: &OsString, args: &[&str], stdin_bytes: &[u8]) -> io::Result<Vec<u8>> {
        *self.observed.borrow_mut() = Some((
            binary.clone(),
            args.iter().map(|s| (*s).to_owned()).collect(),
            stdin_bytes.to_vec(),
        ));
        match self.reply.borrow_mut().take() {
            Some(AdapterReply::Ok(b)) => Ok(b),
            Some(AdapterReply::Err(e)) => Err(e),
            None => panic!("StubAdapter::run called twice"),
        }
    }
}

/// Scripted [`GitRunner`] — records calls, can fail at a chosen index, and
/// returns canned stdout for `run_capture`.
pub(super) struct StubGit {
    pub(super) runs: RefCell<Vec<Vec<String>>>,
    fail_at: Option<usize>,
    capture_reply: String,
}

impl StubGit {
    pub(super) fn ok() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: None,
            capture_reply: "sha-123".into(),
        }
    }
    pub(super) fn failing_at(idx: usize) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: Some(idx),
            capture_reply: "".into(),
        }
    }
}

impl GitRunner for StubGit {
    fn run(&self, _dest: &Path, args: &[&str]) -> io::Result<()> {
        let mut runs = self.runs.borrow_mut();
        let idx = runs.len();
        runs.push(args.iter().map(|s| (*s).to_owned()).collect());
        if self.fail_at == Some(idx) {
            Err(io::Error::other(format!("stub git fail at {idx}")))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        self.run(dest, args)?;
        Ok(self.capture_reply.clone())
    }
}

// --- Fixtures ---------------------------------------------------------

pub(super) const VALID_PROVIDERS_YAML: &str = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth:
      type: api_key
      env: ANTHROPIC_API_KEY
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native]
    context_window: 200000
"#;

pub(super) const VALID_AGENTS_YAML: &str = r#"
agents:
  worker:
    model: claude-sonnet-4-7
    system_prompt: prompts/worker.md
"#;

pub(super) const HAPPY_RESPONSE_JSON: &str = r#"{
    "id":"msg_01","model":"claude-sonnet-4-7","stop_reason":"end_turn",
    "content":[{"type":"text","text":"hi there"}],
    "usage":{"input_tokens":3,"output_tokens":2}
}"#;

pub(super) fn scaffold_repo(
    providers_yaml: &str,
    agents_yaml: &str,
    worker_prompt: Option<&str>,
) -> TempDir {
    let tmp = TempDir::new().unwrap();
    let agent = tmp.path().join(".agent");
    let prompts = agent.join("system").join("prompts");
    std::fs::create_dir_all(&prompts).unwrap();
    std::fs::write(agent.join("providers.yaml"), providers_yaml).unwrap();
    std::fs::write(agent.join("agents.yaml"), agents_yaml).unwrap();
    if let Some(body) = worker_prompt {
        std::fs::write(prompts.join("worker.md"), body).unwrap();
    }
    tmp
}

pub(super) fn valid_deps<'a>(
    adapter: &'a StubAdapter,
    git: &'a StubGit,
    clock: &'a FixedClock,
    id: &'a FixedIdGen,
) -> Deps<'a> {
    Deps {
        adapter,
        git,
        clock,
        id_gen: id,
    }
}
