//! End-to-end: an adapter that fails at startup says why (ARCH §2.3
//! step record, §4.4).
//!
//! Real `bz`, pointed at a malformed brazen config: it dies before it
//! can emit a single `v=1` event, so stdout is empty and the whole
//! complaint is on stderr. On disk that is indistinguishable from a
//! mid-stream kill (§2.9) — the operator-visible difference is the
//! captured stderr, quoted in the error and landed beside the empty
//! `response.json` as `stderr.log`.

use super::prompt_end_to_end::{scaffold_repo, write_global_models};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn a_bz_that_dies_at_startup_surfaces_its_stderr() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);

    // Not TOML. `bz --version` (the load-time guard, §4.4) does not read
    // the config, so the failure lands where it hurts: mid-model-call.
    let brazen_config = holder.path().join("brazen.toml");
    fs::write(&brazen_config, "this is not = valid toml [[[\n").unwrap();

    let out = Command::new(crate::test_support::lernie_binary())
        .arg("prompt")
        .arg(&dest)
        .arg("ping")
        .env("LERNIE_HOME", &harness)
        .env("BRAZEN_CONFIG", &brazen_config)
        .output()
        .expect("spawn lernie prompt");
    assert!(!out.status.success(), "a dead adapter is not a success");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("malformed config"), "{stderr}");
    assert!(stderr.contains("TOML parse error"), "{stderr}");
    assert!(stderr.contains("stderr.log"), "{stderr}");

    // The artifact holds the whole capture, beside the response that
    // never arrived (§2.3).
    let steps = dest.join("steps");
    let agent = fs::read_dir(&steps)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let log = fs::read_to_string(agent.join("001/stderr.log")).unwrap();
    assert!(log.contains("expected `.`, `=`"), "{log}");
    assert!(
        fs::read(agent.join("001/response.json"))
            .unwrap()
            .is_empty(),
        "the adapter never reached the contract"
    );
}
