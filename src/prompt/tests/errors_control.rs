//! Control-resolution error paths for [`crate::prompt::run`] (ARCH
//! §2.2): the workspace layout guard (pre-v1 clean break, §10) and the
//! config-commit control reads — `providers.yaml`, `workflow.yaml`,
//! and the role soul. Split from [`super::errors`] for the per-file
//! line cap.

use super::fixtures::*;
use crate::prompt::Error;
use tempfile::TempDir;

#[test]
fn run_refuses_a_non_workspace_via_the_layout_guard() {
    // Pre-v1 clean break (§2.2, §10): a bare directory is not a
    // workspace, and the refusal lands before any config or git work.
    let tmp = TempDir::new().unwrap();
    let err = run_with_stubs(tmp.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Layout(_)), "got {err:?}");
}

#[test]
fn run_surfaces_per_repo_providers_yaml_load_error() {
    // A legacy `providers:` block in the config commit's providers.yaml
    // is a hard load error (§4.1).
    let repo = scaffold_repo("providers: {}\n", Some("body"));
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_absent_providers_yaml_as_control_read() {
    // No providers.yaml in the config commit's tree: the §2.2 control
    // read fails loudly with the commit-qualified path.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::remove_file(repo.path().join("providers.yaml")).unwrap();
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
}

#[test]
fn run_surfaces_workflow_load_error() {
    // Version guard passes, then the retry-policy parse fails on
    // malformed workflow YAML from the config commit.
    let repo = scaffold_repo_with_workflow(
        VALID_PER_REPO_PROVIDERS_YAML,
        "events: [not, a, map]\n",
        Some("body"),
    );
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_absent_workflow_as_control_read() {
    // Version guard passes, then the workflow control read fails —
    // workflow.yaml is absent from the config commit's tree (§2.2).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::remove_file(repo.path().join("workflow.yaml")).unwrap();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
}

#[test]
fn run_surfaces_missing_soul() {
    // Version guard passes, then the soul control read fails (§2.2).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, None);
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
}

#[test]
fn run_declines_a_config_schema_version_this_build_cannot_read() {
    // ARCH §10: the config commit's `version` file is read before
    // anything it could misparse. A version above the supported one was
    // authored by a newer harness and is declined loudly, before the
    // adapter is ever reached — hence `unreachable_adapter`.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let newer = format!("{}\n", crate::config::version::SUPPORTED + 1);
    std::fs::write(repo.path().join("version"), newer).unwrap();
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_absent_version_as_control_read() {
    // No `version` in the config commit's tree: the §2.2 control read
    // fails loudly with the commit-qualified path rather than defaulting.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::remove_file(repo.path().join("version")).unwrap();
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
}

#[test]
fn run_declines_a_workflow_dispatching_an_undeclared_role() {
    // ARCH §4.3: `dispatch(<role>)` bindings are cross-validated against
    // the same config commit's `roles:` map at config load. The fixture
    // declares only `worker`, so a `dispatch(verifier)` binding is
    // declined here — before the first model call, not at the hop that
    // would finally reach the binding.
    let repo = scaffold_repo_with_workflow(
        VALID_PER_REPO_PROVIDERS_YAML,
        "events:\n  worker_return:\n    - dispatch(verifier)\n",
        Some("body"),
    );
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}
