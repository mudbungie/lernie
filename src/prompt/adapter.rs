//! Harness-side of the provider-adapter contract (ARCH §4.4).
//!
//! The harness invokes `lernie-provider-<name>` as a subprocess per model
//! call, pipes a Messages-API request to its stdin, and reads one JSON
//! document back on stdout. [`AdapterRunner`] is the trait [`super::run`]
//! depends on; [`SpawnAdapter`] is the production implementation.
//!
//! Child env is inherited by default; the harness layers explicit
//! key/value pairs on top via the `envs` argument. That covers the §4.4
//! `endpoint_env` handoff (harness-set values from `providers.yaml`)
//! while leaving credential vars (`auth_env`) to inherit naturally —
//! aligning with §4.4 "auth lives entirely inside the adapter."

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Subdirectory under the harness root where provider adapter binaries
/// live (ARCH §4.1, §4.4 — "the harness looks up
/// `lernie-provider-<name>` at `<harness-root>/adapters/` … before
/// falling back to `PATH`"). Mirrors the `<harness-root>/tools/`
/// resolution path for tool binaries (§3.3).
pub const ADAPTERS_DIR: &str = "adapters";

/// Name prefix for provider adapter binaries (ARCH §4.4).
pub const ADAPTER_PREFIX: &str = "lernie-provider-";

/// Resolve the adapter binary for `provider_name`. Apply the §4.4
/// resolution order: try `<harness_root>/adapters/lernie-provider-<name>`
/// first; if absent, return the bare name so [`Command`] can resolve
/// against `PATH`. The harness-root copy wins so per-install adapter
/// rotations stay local to one harness root.
pub fn resolve_binary(harness_root: &Path, provider_name: &str) -> OsString {
    let bare = format!("{ADAPTER_PREFIX}{provider_name}");
    let harness_path = harness_root.join(ADAPTERS_DIR).join(&bare);
    if harness_path.is_file() {
        return harness_path.into_os_string();
    }
    OsString::from(bare)
}

/// The slice of the adapter contract that the harness calls into. One
/// subprocess per [`Self::run`]; non-zero exit surfaces as an error because
/// §4.4 reserves that for adapter-side crashes, not in-band provider
/// failures.
pub trait AdapterRunner {
    /// Spawn `binary` with `args` and `envs` set on the child process,
    /// forward `stdin_bytes` to its stdin, and return its stdout bytes on
    /// exit-zero. The caller is responsible for parsing the bytes as
    /// either an upstream response or an in-band adapter error (§4.4).
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        envs: &[(&str, &str)],
        stdin_bytes: &[u8],
    ) -> io::Result<Vec<u8>>;
}

/// Default [`AdapterRunner`]. Uses [`Command`] with PATH lookup.
#[derive(Debug, Clone, Copy)]
pub struct SpawnAdapter;

impl AdapterRunner for SpawnAdapter {
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        envs: &[(&str, &str)],
        stdin_bytes: &[u8],
    ) -> io::Result<Vec<u8>> {
        let mut child = Command::new(binary)
            .args(args)
            .envs(envs.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().expect("stdin is piped");
            stdin.write_all(stdin_bytes)?;
        }
        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(io::Error::other(format!(
                "adapter {:?} exited with {}: {}",
                binary,
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(out.stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn resolve_binary_prefers_harness_root_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let adapters = tmp.path().join(ADAPTERS_DIR);
        std::fs::create_dir_all(&adapters).unwrap();
        let installed = adapters.join("lernie-provider-anthropic");
        std::fs::write(&installed, b"#!/bin/sh\nexit 0\n").unwrap();

        let resolved = resolve_binary(tmp.path(), "anthropic");
        assert_eq!(PathBuf::from(&resolved), installed);
    }

    #[test]
    fn resolve_binary_falls_back_to_bare_name_for_path_lookup() {
        let tmp = tempfile::tempdir().unwrap();
        // No adapters/ subdir — resolver must hand back the bare name
        // so Command's PATH lookup runs.
        let resolved = resolve_binary(tmp.path(), "anthropic");
        assert_eq!(resolved, OsString::from("lernie-provider-anthropic"));
    }

    #[test]
    fn resolve_binary_falls_back_when_path_is_a_directory() {
        // A directory at the expected adapter path doesn't count as the
        // adapter binary — must fall back to PATH.
        let tmp = tempfile::tempdir().unwrap();
        let bogus = tmp.path().join(ADAPTERS_DIR).join("lernie-provider-acme");
        std::fs::create_dir_all(&bogus).unwrap();

        let resolved = resolve_binary(tmp.path(), "acme");
        assert_eq!(resolved, OsString::from("lernie-provider-acme"));
    }

    #[test]
    fn spawn_adapter_runs_cat_as_echo() {
        // `cat` copies stdin to stdout — a portable stand-in for a real
        // adapter. Proves the happy path: args, stdin piping, stdout
        // capture, exit-zero.
        let bin = OsString::from("cat");
        let out = SpawnAdapter.run(&bin, &[], &[], b"hello\n").unwrap();
        assert_eq!(out, b"hello\n");
    }

    #[test]
    fn spawn_adapter_forwards_envs_to_child() {
        // `env -0` prints the child env, NUL-terminated. We can scan for
        // the var we set without picking up the inherited PATH/USER lines.
        let bin = OsString::from("env");
        let out = SpawnAdapter
            .run(&bin, &["-0"], &[("LERNIE_TEST_VAR", "passthrough-ok")], b"")
            .unwrap();
        let text = String::from_utf8_lossy(&out);
        assert!(
            text.split('\0')
                .any(|line| line == "LERNIE_TEST_VAR=passthrough-ok"),
            "env not forwarded; got: {text}"
        );
    }

    #[test]
    fn spawn_adapter_reports_spawn_failure() {
        let bin = OsString::from("/no/such/lernie-provider-nonesuch");
        let err = SpawnAdapter.run(&bin, &[], &[], b"").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn spawn_adapter_reports_nonzero_exit() {
        // `false` exits with status 1 on every POSIX system.
        let bin = OsString::from("false");
        let err = SpawnAdapter.run(&bin, &[], &[], b"").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("exited with"), "got: {msg}");
    }
}
