//! `make install` end-to-end: harness root scaffold (ARCH §2.2),
//! adapter binary placement (ARCH §4.1, §4.4), and idempotency.
//!
//! The Makefile is the public install contract. This test pins its
//! observable shape so that re-runs never clobber rotated credentials
//! and the layout matches what the runtime resolvers expect.
//!
//! Single test: every assertion shares the same `make install` run
//! (which transitively builds the workspace in release mode), so the
//! cost is paid once instead of once-per-test under parallel
//! `cargo test`.

// Tarpaulin sets `--cfg=tarpaulin` at compile time; the test below uses
// `cfg_attr(tarpaulin, ignore)` to skip itself under instrumented runs.
// Allow the unknown cfg name so `-D warnings` clippy stays clean.
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

// `make install` shells out to `cargo build --workspace --release`,
// which contends with tarpaulin's lock on `target/` and stretches the
// test from sub-second to 15+ minutes. Skip under tarpaulin — the test
// only exercises shell glue, so excluding it from instrumented runs
// has no effect on Rust line coverage.
#[cfg_attr(tarpaulin, ignore)]
#[test]
fn make_install_lays_down_skeleton_idempotently() {
    let prefix = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    run_install(prefix.path(), home.path());

    // Harness-root skeleton (ARCH §2.2).
    for d in [
        "adapters",
        "workflows",
        "tools",
        "skills",
        "agents",
        "conversations",
    ] {
        assert!(
            home.path().join(d).is_dir(),
            "harness root subdir missing: {d}"
        );
    }

    // Path binaries land under INSTALL_PREFIX/bin.
    let bin = prefix.path().join("bin");
    assert!(bin.join("lernie").is_file(), "lernie missing from bin/");
    assert!(
        bin.join("lernie-ui-egui").is_file(),
        "lernie-ui-egui missing from bin/"
    );

    // Adapter binary lands at <harness-root>/adapters/, NOT on PATH —
    // ARCH §4.4 resolves there first; an extra PATH copy would break
    // per-harness-root credential isolation.
    assert!(
        home.path()
            .join("adapters/lernie-provider-anthropic")
            .is_file(),
        "anthropic adapter missing from adapters/"
    );
    assert!(
        !bin.join("lernie-provider-anthropic").exists(),
        "adapter must not also land on PATH"
    );

    // Default global providers.yaml (ARCH §4.1).
    let providers = home.path().join("providers.yaml");
    let body = std::fs::read_to_string(&providers).unwrap();
    assert!(body.contains("anthropic:"));
    assert!(body.contains("ANTHROPIC_API_KEY"));
    assert!(body.contains("claude-sonnet-4-7"));

    // Default agent profile (ARCH §2.2 frozen-copy bootstrap source).
    let profile = home.path().join("agents/default");
    assert!(profile.is_dir());
    assert!(profile.join("manifest.yaml").is_file());
    assert!(profile.join("workflow.yaml").is_file());
    assert!(profile.join("providers.yaml").is_file());
    assert!(profile.join("souls/worker.md").is_file());
    assert!(profile.join("souls/compactor.md").is_file());

    // Idempotency: hand-edit config, re-run, verify it survives.
    // Binaries are re-installed unconditionally (a fresh build is the
    // point of re-running install) so we don't pin those.
    std::fs::write(&providers, "providers: {}\nmodels: {}\n").unwrap();
    let agent_marker = profile.join("CANARY");
    std::fs::write(&agent_marker, b"keep me").unwrap();

    run_install(prefix.path(), home.path());

    assert_eq!(
        std::fs::read_to_string(&providers).unwrap(),
        "providers: {}\nmodels: {}\n",
        "providers.yaml was clobbered by re-install"
    );
    assert!(
        agent_marker.exists(),
        "agents/default was clobbered by re-install"
    );
}
