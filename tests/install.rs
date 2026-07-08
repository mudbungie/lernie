//! `make install` end-to-end: harness root scaffold (ARCH §2.2), the
//! global `models.yaml` (ARCH §4.2), and idempotency.
//!
//! The Makefile is the public install contract. This test pins its
//! observable shape so re-runs never clobber hand-edited config and the
//! layout matches what the runtime resolvers expect. The provider
//! adapter is brazen's `bz`, installed by `cargo install brazen` onto
//! the user's cargo bin (§4.4) — not into the harness root — so this
//! test asserts the harness-owned layout only.

// Tarpaulin sets `--cfg=tarpaulin` at compile time; the test below uses
// `cfg_attr(tarpaulin, ignore)` to skip itself under instrumented runs.
#![allow(unexpected_cfgs)]

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_install(prefix: &Path, home: &Path) {
    let out = Command::new("make")
        .current_dir(repo_root())
        .arg("install")
        .arg(format!("INSTALL_PREFIX={}", prefix.display()))
        .arg(format!("LERNIE_HOME={}", home.display()))
        .env("GIT_AUTHOR_NAME", "lernie-test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "lernie-test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .output()
        .expect("invoke make install");
    assert!(
        out.status.success(),
        "make install failed (status {}):\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

// `make install` shells out to `cargo build --workspace --release` (and
// `cargo install brazen`), which contends with tarpaulin's `target/`
// lock. Skip under tarpaulin — the test only exercises shell glue, so
// excluding it from instrumented runs has no effect on Rust line
// coverage.
#[cfg_attr(tarpaulin, ignore)]
#[test]
fn make_install_lays_down_skeleton_idempotently() {
    let prefix = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    run_install(prefix.path(), home.path());

    // Harness-root skeleton (ARCH §2.2). No `adapters/` — the adapter is
    // brazen's `bz` on PATH now (§4.4).
    for d in ["workflows", "tools", "skills", "agents", "conversations"] {
        assert!(
            home.path().join(d).is_dir(),
            "harness root subdir missing: {d}"
        );
    }
    assert!(
        !home.path().join("adapters").exists(),
        "the retired per-provider adapters/ dir must not be created"
    );

    // Path binaries land under INSTALL_PREFIX/bin.
    let bin = prefix.path().join("bin");
    assert!(bin.join("lernie").is_file(), "lernie missing from bin/");
    assert!(
        bin.join("lernie-ui-egui").is_file(),
        "lernie-ui-egui missing from bin/"
    );

    // Default global models.yaml (ARCH §4.2) — capabilities, no auth.
    let models = home.path().join("models.yaml");
    let body = std::fs::read_to_string(&models).unwrap();
    assert!(body.contains("provider: anthropic"));
    assert!(body.contains("claude-sonnet-4-7"));
    assert!(
        !body.contains("ANTHROPIC_API_KEY"),
        "auth material must not live in models.yaml (§4.1)"
    );

    // Default agent profile (ARCH §2.2 frozen-copy bootstrap source).
    let profile = home.path().join("agents/default");
    assert!(profile.join("manifest.yaml").is_file());
    assert!(profile.join("workflow.yaml").is_file());
    assert!(profile.join("providers.yaml").is_file());
    assert!(profile.join("souls/worker.md").is_file());
    assert!(profile.join("souls/compactor.md").is_file());

    // Idempotency: hand-edit config, re-run, verify it survives.
    std::fs::write(&models, "models: {}\n").unwrap();
    let agent_marker = profile.join("CANARY");
    std::fs::write(&agent_marker, b"keep me").unwrap();

    run_install(prefix.path(), home.path());

    assert_eq!(
        std::fs::read_to_string(&models).unwrap(),
        "models: {}\n",
        "models.yaml was clobbered by re-install"
    );
    assert!(
        agent_marker.exists(),
        "agents/default was clobbered by re-install"
    );
}
