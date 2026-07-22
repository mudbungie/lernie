//! §5.2 manifest wiring through [`crate::prompt::run`]: the control
//! read from the config commit (resolve), the role-keyed rules lookup,
//! and the head/body composition landing in the model call's request.
//! Assembly mechanics themselves are unit-tested at
//! `prompt::dispatch::assembler` (and its `body` submodule); these
//! tests pin the runtime caller.

use super::fixtures::*;
use crate::prompt::Error;

/// A manifest whose `worker` entry orders `summary/**` (§5.2) — the
/// shape the shipped template carries, minus the structurally-homed
/// pinned entries.
const WORKER_SUMMARY_MANIFEST: &str = "roles:\n  worker:\n    pinned: []\n    order:\n      - summary/**\n    budget_tokens: 1000\n    overflow: drop\n";

#[test]
fn run_surfaces_absent_manifest_as_control_read() {
    // No manifest.yaml in the config commit's tree: the §2.2 control
    // read fails loudly with the commit-qualified path — the manifest
    // is a required control file (§2.2), like its siblings.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::remove_file(repo.path().join("manifest.yaml")).unwrap();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
}

#[test]
fn run_surfaces_manifest_parse_error_as_config() {
    // A shape-invalid manifest (zero budget, §5.2) is declined at the
    // load, before the first model call.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::write(
        repo.path().join("manifest.yaml"),
        "roles:\n  worker:\n    pinned: []\n    order: []\n    budget_tokens: 0\n    overflow: drop\n",
    )
    .unwrap();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_composes_the_manifest_body_into_the_request() {
    // End-to-end §5.2: a `summary/**` file in the branch's worktree
    // composes as a path-framed body block ahead of the delivered user
    // message, in the same (grouped) first wire message.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::write(repo.path().join("manifest.yaml"), WORKER_SUMMARY_MANIFEST).unwrap();
    // The stub git's `worktree add` materializes nothing, so pre-lay
    // the worktree with the inherited summary (§2.7 — what a compaction
    // merge leaves behind).
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join("summary")).unwrap();
    std::fs::write(wt.join("summary/001.md"), "distilled history").unwrap();

    let adapter = StubAdapter::happy(&happy_response_bytes());
    run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap();

    let request: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo.path().join("steps/ct-1-deadbeef/001/request.json")).unwrap(),
    )
    .unwrap();
    let messages = request["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], "user");
    let content = messages[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2, "body block + delivered user message");
    let body = content[0]["text"].as_str().unwrap();
    assert!(body.contains("<file path=\"summary/001.md\">"));
    assert!(body.contains("distilled history"));
    // The delivered entry follows, verbatim with its §2.11 provenance
    // frontmatter.
    assert!(content[1]["text"].as_str().unwrap().ends_with("hi"));
}
