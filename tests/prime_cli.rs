//! `lernie prime` end-to-end over the real binary (ARCH §2.2). Proves the
//! exact invocation yog drives — `LERNIE_HOME=<dir> lernie prime` — founds
//! a fresh nested home, is idempotent (a second run changes nothing), and
//! never clobbers a hand-edited `models.yaml`. Product-less per the stdout
//! one-product convention (§3.4): success prints nothing.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn prime(home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lernie"))
        .arg("prime")
        .env("LERNIE_HOME", home)
        .output()
        .expect("spawn lernie prime")
}

fn assert_ok_silent(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "lernie prime failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stdout.is_empty(),
        "prime is product-less; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn prime_founds_a_fresh_nested_home_idempotently() {
    let home = TempDir::new().unwrap();
    let h = home.path();

    // First run founds the substrate.
    assert_ok_silent(&prime(h));
    assert!(h.join("workflows").is_dir());
    assert!(h.join("workspaces").is_dir());
    // The seeded models.yaml is mechanism only (bl-35e2): present, but
    // naming no model.
    let models = h.join("models.yaml");
    let body = fs::read_to_string(&models).unwrap();
    assert!(
        body.contains("adapter:"),
        "the adapter override is documented"
    );
    assert!(!body.contains("claude-"), "no model id ships (bl-35e2)");
    for name in [
        "bash",
        "cd",
        "dispatch",
        "load_skill",
        "message",
        "read_file",
    ] {
        assert!(h.join("tools").join(format!("{name}.json")).is_file());
        assert!(h.join("skills").join(name).join("SKILL.md").is_file());
    }

    // Idempotency: a hand-edited models.yaml survives a second run, and
    // nothing else the second run touched changed.
    fs::write(&models, "adapter: /opt/bz\n").unwrap();
    assert_ok_silent(&prime(h));
    assert_eq!(fs::read_to_string(&models).unwrap(), "adapter: /opt/bz\n");
    assert!(h.join("skills/bash/SKILL.md").is_file());
}

#[test]
fn prime_reports_a_seeding_failure_loudly() {
    // A `LERNIE_HOME` whose parent is a regular file cannot be created
    // (`ENOTDIR`), so seeding fails: the binding prints the uniform
    // `lernie prime: …` stderr shape and exits non-zero (§3.4).
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("not-a-dir");
    fs::write(&file, b"x").unwrap();
    let out = prime(&file.join("home"));
    assert!(!out.status.success(), "seeding under a file must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("lernie prime:"), "got {stderr:?}");
}
