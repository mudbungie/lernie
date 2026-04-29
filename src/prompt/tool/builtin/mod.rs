//! Built-in tools — the in-process implementations behind
//! `lernie tool <name>` (ARCH §3.3, §12 v0.3 toolset).
//!
//! Each tool is a pure function over [`Read`]/[`Write`] so unit tests
//! drive it without touching real stdio. The `lernie tool` subcommand
//! is a thin shim that locks the process's stdio handles and delegates
//! to [`run`]; the §3.3 stdio contract (stdin = `tool_use.input` JSON,
//! stdout = raw result bytes, exit code = is_error) is enforced here.
//!
//! v0.3 shipped two built-ins (`read_file`, `bash`); v0.4 Phase 2 adds
//! [`dispatch`] (the subagent-spawning tool, ARCH §2.5). Adding a new
//! one is a match arm in [`run`] plus a sibling module.

use std::io::{Read, Write};
use thiserror::Error;

pub mod bash;
pub mod dispatch;
pub mod read_file;

/// Reasons [`run`] can fail. Each in-process tool surfaces its own
/// error variant; an unknown tool name is the dispatcher-level case.
#[derive(Debug, Error)]
pub enum Error {
    /// The lernie binary was invoked as `lernie tool <name>` for a
    /// `<name>` that isn't a built-in. The harness only routes here
    /// after external resolution misses (§3.3), so this is "no tool
    /// of that name exists at all".
    #[error("unknown built-in tool: {0:?}")]
    Unknown(String),
    /// `read_file` failed; carries the inner reason for the operator's
    /// `eprintln!`. The §3.3 stdio contract concats stderr after
    /// stdout into `tool_result.content` when exit code is non-zero,
    /// so the message reaches the model verbatim.
    #[error(transparent)]
    ReadFile(#[from] read_file::Error),
    /// `bash` failed at the harness layer (bad input JSON, spawn
    /// failure, broken pipe, etc.). In-band shell failures — the
    /// command ran and exited non-zero — are *not* this variant; they
    /// flow through the returned exit code.
    #[error(transparent)]
    Bash(#[from] bash::Error),
    /// `dispatch` failed (bad input JSON, missing role / soul,
    /// `lernie dispatch <role>` exit non-zero, etc., per
    /// [`dispatch::Error`]). The §3.3 stdio contract concats stderr
    /// after stdout so the agent sees the failure verbatim.
    #[error(transparent)]
    Dispatch(#[from] dispatch::Error),
}

/// Dispatch one in-process tool call. `name` is the tool name as the
/// model spelled it (and as the harness passed via `lernie tool
/// <name>`); `stdin` carries the `tool_use.input` JSON; `stdout`
/// receives the bytes the executor will surface as
/// `tool_result.content` on success; `stderr` receives the bytes that
/// — per §3.3 — concatenate after stdout when the exit code is
/// non-zero. The returned `i32` is the desired process exit code:
/// `read_file` always returns 0 on success and lets [`Error`] carry
/// failure; `bash` propagates the shell's own exit code so a non-zero
/// command can flow through without being misclassified as a harness
/// fault.
pub fn run<R: Read, W: Write, E: Write>(
    name: &str,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<i32, Error> {
    // `current_exe` failure here is exotic (mostly unusual platforms
    // / `proc` mounts); panicking is consistent with the harness-wide
    // pattern for unrecoverable startup invariants.
    let spawner = dispatch::SubprocessSpawner::new().expect("current_exe resolves");
    run_with(name, stdin, stdout, stderr, &dispatch::ProcessEnv, &spawner)
}

/// Same as [`run`] but with the `dispatch`-tool dependencies (env
/// lookup + subprocess spawner) injected. Production wires these to
/// [`dispatch::ProcessEnv`] + [`dispatch::SubprocessSpawner`] via
/// [`run`]; tests inject stubs to exercise the dispatch arm without
/// real subprocess fan-out.
pub fn run_with<R: Read, W: Write, E: Write>(
    name: &str,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    env: &dyn dispatch::EnvLookup,
    spawner: &dyn dispatch::Spawner,
) -> Result<i32, Error> {
    if name == "read_file" {
        return read_file::run(stdin, stdout)
            .map(|()| 0)
            .map_err(Error::ReadFile);
    }
    if name == "bash" {
        return bash::run(stdin, stdout, stderr).map_err(Error::Bash);
    }
    if name == "dispatch" {
        return dispatch::run(stdin, stdout, env, spawner)
            .map(|()| 0)
            .map_err(Error::Dispatch);
    }
    Err(Error::Unknown(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn unknown_tool_name_surfaces_unknown_variant() {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let err = run("not_a_tool", &mut stdin, &mut stdout, &mut stderr).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not_a_tool"), "{msg}");
        assert!(msg.contains("unknown"), "{msg}");
    }

    #[test]
    fn read_file_routed_to_inner_module() {
        // A minimal-but-valid input that drives the inner module's
        // happy path. Exercising the dispatch arm for read_file.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"hi").unwrap();
        let input = serde_json::json!({ "path": tmp.path() }).to_string();
        let mut stdin = Cursor::new(input.into_bytes());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run("read_file", &mut stdin, &mut stdout, &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert_eq!(stdout, b"hi");
    }

    #[test]
    fn read_file_error_is_carried_through_dispatcher() {
        // Bad JSON on stdin — read_file::Error::InvalidJson — should
        // surface through the From conversion as Error::ReadFile.
        let mut stdin = Cursor::new(b"not json".to_vec());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let err = run("read_file", &mut stdin, &mut stdout, &mut stderr).unwrap_err();
        assert!(matches!(err, Error::ReadFile(_)), "{err}");
    }

    #[test]
    fn bash_routed_to_inner_module() {
        // Drives the dispatch arm for bash through a trivial command.
        let input = serde_json::json!({ "command": "printf hi" }).to_string();
        let mut stdin = Cursor::new(input.into_bytes());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run("bash", &mut stdin, &mut stdout, &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert_eq!(stdout, b"hi");
    }

    #[test]
    fn bash_error_is_carried_through_dispatcher() {
        // Bad JSON on stdin — bash::Error::InvalidJson — should
        // surface through the From conversion as Error::Bash.
        let mut stdin = Cursor::new(b"not json".to_vec());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let err = run("bash", &mut stdin, &mut stdout, &mut stderr).unwrap_err();
        assert!(matches!(err, Error::Bash(_)), "{err}");
    }

    /// Test-only stub for the dispatch tool's [`Spawner`] dependency.
    /// Returns a fixed handle on stdout, exit 0 — exercising the
    /// happy-path arm of [`run_with`] without spawning a real
    /// subprocess.
    struct StubSpawner;
    impl dispatch::Spawner for StubSpawner {
        fn dispatch(
            &self,
            _role: &str,
            _repo: &std::path::Path,
            _branch: &str,
            _goal: &str,
        ) -> std::io::Result<dispatch::DispatchOutput> {
            Ok(dispatch::DispatchOutput {
                stdout: "p1-sub\n".to_string(),
                stderr: String::new(),
                exit: 0,
            })
        }
    }

    /// Stub env that returns the same conv-repo / conv-branch
    /// regardless of which key is asked for. Real lookups discriminate
    /// on key; the dispatch tool only requests the two we care about
    /// so a key-blind stub is sound for the routing test.
    struct StubEnv {
        repo: std::path::PathBuf,
        branch: String,
    }
    impl dispatch::EnvLookup for StubEnv {
        fn get(&self, key: &str) -> Option<std::ffi::OsString> {
            if key == crate::prompt::tool::ENV_CONV_REPO {
                Some(self.repo.as_os_str().to_owned())
            } else if key == crate::prompt::tool::ENV_CONV_BRANCH {
                Some(self.branch.as_str().into())
            } else {
                None
            }
        }
    }

    #[test]
    fn dispatch_routed_to_inner_module() {
        let repo = tempfile::TempDir::new().unwrap();
        std::fs::write(
            repo.path().join("providers.yaml"),
            "roles:\n  worker:\n    provider: anthropic\n    model: m\n",
        )
        .unwrap();
        std::fs::create_dir_all(repo.path().join("souls")).unwrap();
        std::fs::write(repo.path().join("souls").join("worker.md"), "soul").unwrap();

        let input = serde_json::json!({"role":"worker","goal":"g"}).to_string();
        let mut stdin = Cursor::new(input.into_bytes());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let env = StubEnv {
            repo: repo.path().to_path_buf(),
            branch: "p1".into(),
        };
        let code = run_with(
            "dispatch",
            &mut stdin,
            &mut stdout,
            &mut stderr,
            &env,
            &StubSpawner,
        )
        .unwrap();
        assert_eq!(code, 0);
        let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(payload["status"], "in_progress");
        assert_eq!(payload["handle"], "p1-sub");
    }

    #[test]
    fn dispatch_error_is_carried_through_dispatcher() {
        // No env vars set on the StubEnv variant below — surfaces as
        // dispatch::Error::MissingEnv via #[from] into Error::Dispatch.
        struct EmptyEnv;
        impl dispatch::EnvLookup for EmptyEnv {
            fn get(&self, _key: &str) -> Option<std::ffi::OsString> {
                None
            }
        }
        let input = serde_json::json!({"role":"worker","goal":"g"}).to_string();
        let mut stdin = Cursor::new(input.into_bytes());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let err = run_with(
            "dispatch",
            &mut stdin,
            &mut stdout,
            &mut stderr,
            &EmptyEnv,
            &StubSpawner,
        )
        .unwrap_err();
        assert!(matches!(err, Error::Dispatch(_)), "{err}");
    }
}
