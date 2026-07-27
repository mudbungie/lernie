//! The §6 delivered-child-result and checkpoint-flush test cases. The
//! shared real-git harness (stubs, `Fx`, `returned_child`) lives in the
//! parent [`super`] module.

use super::super::{has_pending_result, interpret_pending, run_flush};
use super::{Fx, returned_child, workflow};
use crate::prompt::{ChildDispatchRequest, Error, SystemClock, child_dispatch};
use crate::template::GitRunner;
use crate::workspace::{agent_worktree, fixture};

#[test]
fn a_worker_result_delivers_by_default_transfer_plus_transcript() {
    // §6 worker_return baseline (unbound → deliver_result): the child's
    // work product transfers to the parent tree and its result message
    // lands in the transcript; the inbox file is consumed.
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p1";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let child = returned_child(&ws, parent, "worker", "do it", ("out.txt", "result\n"), &fx);

    let wt = agent_worktree(&ws, parent);
    interpret_pending(&ws, parent, &wt, &workflow("events: {}\n"), &fx.deps()).unwrap();

    assert_eq!(
        std::fs::read_to_string(wt.join("out.txt")).unwrap(),
        "result\n"
    );
    let delivered = wt.join(format!("messages/001-{child}.md"));
    assert!(delivered.exists(), "result message delivered to transcript");
    assert!(!has_pending_result(&ws, parent).unwrap(), "inbox consumed");
}

#[test]
fn a_compactor_result_lands_the_compaction_merge_and_consumes_the_message() {
    // §6 compactor_return baseline (unbound → compaction_merge): the
    // compactor's summary merges into the parent tree, and the trigger
    // message is removed (not delivered — the merge commit is the record).
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p2";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let child = returned_child(
        &ws,
        parent,
        "compactor",
        "compact",
        ("summary/001.md", "sum\n"),
        &fx,
    );

    let wt = agent_worktree(&ws, parent);
    interpret_pending(&ws, parent, &wt, &workflow("events: {}\n"), &fx.deps()).unwrap();

    assert_eq!(
        std::fs::read_to_string(wt.join("summary/001.md")).unwrap(),
        "sum\n"
    );
    // A merge commit landed on the parent branch.
    let subj = fx
        .git
        .run_capture(&wt, &["log", "-1", "--format=%s"])
        .unwrap();
    assert!(
        subj.contains(&format!("compaction merge [{child}]")),
        "{subj}"
    );
    assert!(
        !has_pending_result(&ws, parent).unwrap(),
        "trigger consumed"
    );
    // No transcript delivery for the compactor's result.
    assert!(!wt.join(format!("messages/001-{child}.md")).exists());
}

#[test]
fn an_explicit_worker_return_binding_is_honored() {
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p3";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let child = returned_child(&ws, parent, "worker", "do it", ("out.txt", "x\n"), &fx);
    let wt = agent_worktree(&ws, parent);
    let wf = workflow("events:\n  worker_return:\n    - deliver_result\n");
    interpret_pending(&ws, parent, &wt, &wf, &fx.deps()).unwrap();
    assert!(wt.join(format!("messages/001-{child}.md")).exists());
}

#[test]
fn an_unsupported_child_result_action_is_declined_loudly() {
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p4";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    returned_child(&ws, parent, "worker", "do it", ("out.txt", "x\n"), &fx);
    let wt = agent_worktree(&ws, parent);
    // A worker_return bound to an action with no child-result executor
    // (a ref-mark action belongs to the branch's own terminal) is declined
    // here, never silently no-oped.
    let wf = workflow("events:\n  worker_return:\n    - notify_ui\n");
    let err = interpret_pending(&ws, parent, &wt, &wf, &fx.deps()).unwrap_err();
    assert!(matches!(err, Error::ActionUnsupported { .. }), "{err:?}");
}

#[test]
fn has_pending_result_is_false_without_a_result_message() {
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p5";
    fixture::spawn_root(&ws, parent);
    // An ordinary steering deposit is not a result message.
    crate::prompt::inbox::deposit(&ws, parent, "user", "hi", &SystemClock).unwrap();
    assert!(!has_pending_result(&ws, parent).unwrap());
}

#[test]
fn interpret_pending_skips_a_steering_deposit() {
    // The same steering-vs-result split inside the interpreter itself:
    // a deposit with no `terminal_ref:` is not a child result, so the
    // interpreter loads nothing and touches no dep (all of Fx's deps
    // are unreachable stubs — reaching one would panic).
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p8";
    let wt = fixture::spawn_root(&ws, parent);
    crate::prompt::inbox::deposit(&ws, parent, "user", "hi", &SystemClock).unwrap();
    let fx = Fx::new();
    let wf = workflow("events: {}\n");
    interpret_pending(&ws, parent, &wt, &wf, &fx.deps()).unwrap();
    // The steering message is left where it was deposited, undrained.
    assert!(!has_pending_result(&ws, parent).unwrap());
}

#[test]
fn run_flush_dispatches_a_compactor_when_the_checkpoint_is_due() {
    // §2.7/§6: a `compaction:` clock due at the boundary runs worker_flush
    // → dispatch(compactor), forking a compactor off the tip C.
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p6";
    let wt = fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    // every_n_commits n=1: the parent already has commits, so it is due.
    let wf = workflow(
        "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 1\n",
    );
    run_flush(&ws, parent, &wt, &wf, &fx.deps()).unwrap();
    // A compactor child was launched through the front door.
    let launched = fx.launcher.launched.borrow();
    assert_eq!(launched.len(), 1);
    assert!(
        launched[0].starts_with(&format!("{parent}-")),
        "{launched:?}"
    );
}

#[test]
fn run_flush_is_a_noop_without_a_compaction_block() {
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p7";
    let wt = fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    run_flush(&ws, parent, &wt, &workflow("events: {}\n"), &fx.deps()).unwrap();
    assert!(fx.launcher.launched.borrow().is_empty());
}

/// A global `models.yaml` naming an `adapter:` override (so the version
/// guard is skipped, §4.4) and carrying both the worker's sonnet and the
/// compactor's haiku (§4.2). Written into `fx.cfg` so it is the resolve's
/// config root.
fn write_models(fx: &Fx) {
    let yaml = "adapter: /bin/true\nmodels:\n  \
        claude-sonnet-5: {provider: anthropic, model_id: claude-sonnet-5, \
        capabilities: [tool_use_native], context_window: 200000}\n  \
        claude-haiku-4-5: {provider: anthropic, model_id: claude-haiku-4-5, \
        capabilities: [tool_use_native], context_window: 200000}\n";
    std::fs::write(fx.cfg.path().join("models.yaml"), yaml).unwrap();
}

#[test]
fn resolve_derives_a_dispatched_compactors_role_soul_and_toolset() {
    // §6 role-aware resolution: an existing agent's role is derived from
    // its dispatch commit subject (the single authoritative home). A
    // dispatched compactor resolves `souls/compactor.md`, its `compactor`
    // providers assignment (haiku, no declared tools — the injected
    // built-in toolset is the step's, not `providers.yaml`'s).
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
        fork_point: None,
    };
    let child = child_dispatch::run(&req, &fx.git, &fx.clock, &fx.id, &fx.launcher).unwrap();

    let cfg = resolve_worker(&ws, ConfigSource::Agent(&child), &fx.deps()).unwrap();
    assert_eq!(cfg.role, "compactor");
    assert_eq!(cfg.model.model_id, "claude-haiku-4-5");
    assert!(
        cfg.tools.is_empty(),
        "compactor declares no providers.yaml tools"
    );
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
    assert_eq!(cfg.model.model_id, "claude-sonnet-5");
}

#[test]
fn run_flush_declines_an_unsupported_flush_action() {
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-p8";
    let wt = fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    // Due, but worker_flush bound to a non-dispatch action → declined.
    let wf = workflow(
        "events:\n  worker_flush:\n    - notify_ui\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 1\n",
    );
    let err = run_flush(&ws, parent, &wt, &wf, &fx.deps()).unwrap_err();
    assert!(matches!(err, Error::ActionUnsupported { .. }), "{err:?}");
}
