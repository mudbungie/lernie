//! §6 role-aware resolution over dispatched agents — split from
//! [`cases`](super::cases) to hold the per-file line cap. The shared
//! real-git harness lives in [`super`].

use super::Fx;
use crate::prompt::{ChildDispatchRequest, child_dispatch};
use crate::workspace::{agent_worktree, fixture};

/// A global `models.yaml` naming an `adapter:` override (so the version
/// guard is skipped, §4.4). The roles' models are the per-repo
/// assignment's alone (§4.3) — the global file carries no models table
/// (bl-35e2). Written into `fx.cfg` so it is the resolve's config root.
fn write_models(fx: &Fx) {
    std::fs::write(fx.cfg.path().join("models.yaml"), "adapter: /bin/true\n").unwrap();
}

#[test]
fn resolve_derives_a_dispatched_compactors_role_soul_and_toolset() {
    // §6 role-aware resolution: an existing agent's role is derived from
    // its dispatch commit subject (the single authoritative home). A
    // dispatched compactor resolves `souls/compactor.md` and its
    // `compactor` providers assignment (haiku, no declared tools — the
    // injected built-in toolset is the step's, not `providers.yaml`'s).
    use crate::prompt::resolve::{ConfigSource, resolve_worker};
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-r1";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    write_models(&fx);
    let parent_wt = agent_worktree(&ws, parent);
    let req = ChildDispatchRequest {
        repo: &ws,
        parent_branch: parent,
        parent_worktree: &parent_wt,
        role: "compactor",
        goal: "compact",
        name: None,
        fork_point: None,
        cwd: None,
        pins: crate::prompt::PinnedDocs::none(),
    };
    let child = child_dispatch::run(
        &req,
        &fx.git,
        &fx.clock,
        &fx.id,
        &fx.launcher,
        crate::workspace::agent_name::mint::test_rng(),
    )
    .unwrap();

    let cfg = resolve_worker(&ws, ConfigSource::Agent(&child), &fx.deps()).unwrap();
    assert_eq!(cfg.role, "compactor");
    assert_eq!(cfg.model_id, "claude-haiku-4-5");
    assert!(cfg.tools.is_empty(), "compactor declares no such tools");
    assert!(cfg.soul.to_lowercase().contains("compact"), "{}", cfg.soul);
}

#[test]
fn resolve_defaults_a_root_agent_to_the_worker_role() {
    // A root's dispatch subject lacks the `dispatch: <role>` prefix, so the
    // role derives to `None` and the worker default applies (§6).
    use crate::prompt::resolve::{ConfigSource, resolve_worker};
    let (_h, ws) = fixture::workspace();
    let root = "20260101-r2";
    fixture::spawn_root(&ws, root);
    let fx = Fx::new();
    write_models(&fx);
    let cfg = resolve_worker(&ws, ConfigSource::Agent(root), &fx.deps()).unwrap();
    assert_eq!(cfg.role, "worker");
    assert_eq!(cfg.model_id, "claude-sonnet-5");
}
