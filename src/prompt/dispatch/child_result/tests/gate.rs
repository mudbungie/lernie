//! §6 verifier-gate unit/integration coverage: the reject path, the
//! idempotent `already_gated` hold, the not-due checkpoint, and the result
//! frontmatter split. The shared real-git harness lives in [`super`].

use super::super::{interpret_pending, run_flush, split_frontmatter};
use super::{Fx, returned_child, workflow};
use crate::prompt::inbox::{Epitaph, deposit_result, inbox_dir};
use crate::prompt::{ChildDispatchRequest, child_dispatch};
use crate::template::GitRunner;
use crate::workspace::{agent_worktree, fixture};

/// Fork a verifier off `worker_tip` (as the §6 gate does) and, when
/// `verdict` is `Some`, deposit its result into the parent's inbox.
fn verifier_child(
    ws: &std::path::Path,
    parent: &str,
    worker_tip: &str,
    verdict: Option<&str>,
    fx: &Fx,
) -> String {
    let parent_wt = agent_worktree(ws, parent);
    let req = ChildDispatchRequest {
        repo: ws,
        parent_branch: parent,
        parent_worktree: &parent_wt,
        role: "verifier",
        goal: "judge",
        fork_point: Some(worker_tip),
    };
    let v = child_dispatch::run(&req, &fx.git, &fx.clock, &fx.id, &fx.launcher).unwrap();
    if let Some(response) = verdict {
        let vtip = fx
            .git
            .run_capture(&agent_worktree(ws, &v), &["rev-parse", "HEAD"])
            .unwrap();
        deposit_result(
            ws,
            parent,
            &v,
            Epitaph::FinalResponse,
            vtip.trim(),
            Some(response),
            &fx.clock,
        )
        .unwrap();
    }
    v
}

const GATE: &str =
    "events:\n  worker_return:\n    - dispatch(verifier)\n    - gate_return_on(verifier.approve)\n";

fn worker_tip(ws: &std::path::Path, worker: &str, fx: &Fx) -> String {
    fx.git
        .run_capture(&agent_worktree(ws, worker), &["rev-parse", "HEAD"])
        .unwrap()
        .trim()
        .to_string()
}

#[test]
fn a_rejecting_verifier_redispatches_the_worker_and_discards_the_result() {
    // §6 verifier_reject (unbound → the baseline `dispatch(worker, with:
    // verifier.feedback)`): a `REJECT` verdict re-dispatches a worker and
    // discards both the rejected result and the verdict message.
    let (_h, ws) = fixture::workspace();
    fixture::amend_config(&ws, &[("souls/verifier.md", "v")]);
    let parent = "20260101-g1";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let worker = returned_child(&ws, parent, "worker", "do it", ("out.txt", "x\n"), &fx);
    let wtip = worker_tip(&ws, &worker, &fx);
    let verifier = verifier_child(&ws, parent, &wtip, Some("REJECT: redo it"), &fx);

    let wt = agent_worktree(&ws, parent);
    interpret_pending(&ws, parent, &wt, &workflow(GATE), &fx.deps()).unwrap();

    // Both messages consumed; a fresh worker was dispatched (worker,
    // verifier, re-dispatched worker → three launches).
    assert!(
        !inbox_dir(&ws, parent)
            .join(format!("{worker}-001.md"))
            .exists()
    );
    assert!(
        !inbox_dir(&ws, parent)
            .join(format!("{verifier}-001.md"))
            .exists()
    );
    assert_eq!(fx.launcher.launched.borrow().len(), 3);
}

#[test]
fn an_already_gated_worker_does_not_redispatch_a_verifier() {
    // §6 hold idempotency: with a verifier already gating the worker
    // (dispatched, in-flight — no verdict yet), a re-interpretation of the
    // worker return does not dispatch a second verifier.
    let (_h, ws) = fixture::workspace();
    fixture::amend_config(&ws, &[("souls/verifier.md", "v")]);
    let parent = "20260101-g2";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let worker = returned_child(&ws, parent, "worker", "do it", ("out.txt", "x\n"), &fx);
    let wtip = worker_tip(&ws, &worker, &fx);
    verifier_child(&ws, parent, &wtip, None, &fx); // in-flight verifier

    let wt = agent_worktree(&ws, parent);
    interpret_pending(&ws, parent, &wt, &workflow(GATE), &fx.deps()).unwrap();

    // No third launch: worker + the one in-flight verifier only, and the
    // worker result stays held.
    assert_eq!(fx.launcher.launched.borrow().len(), 2);
    assert!(
        inbox_dir(&ws, parent)
            .join(format!("{worker}-001.md"))
            .exists()
    );
}

#[test]
fn run_flush_is_a_noop_when_the_clock_is_below_threshold() {
    // A `compaction:` block present but not yet due (§2.7): the git state is
    // derived, `due` is false, and no compactor is dispatched.
    let (_h, ws) = fixture::workspace();
    let parent = "20260101-g3";
    let wt = fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let wf = workflow(
        "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 1000000\n",
    );
    run_flush(&ws, parent, &wt, &wf, &fx.deps()).unwrap();
    assert!(fx.launcher.launched.borrow().is_empty());
}

#[test]
fn a_verdict_action_without_an_executor_is_declined() {
    // A verifier_approve bound to an action the verdict executor does not
    // handle (a ref mark) is declined loudly, not silently no-oped.
    let (_h, ws) = fixture::workspace();
    fixture::amend_config(&ws, &[("souls/verifier.md", "v")]);
    let parent = "20260101-g4";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let worker = returned_child(&ws, parent, "worker", "do it", ("out.txt", "x\n"), &fx);
    let wtip = worker_tip(&ws, &worker, &fx);
    verifier_child(&ws, parent, &wtip, Some("APPROVE"), &fx);

    let wt = agent_worktree(&ws, parent);
    let wf = workflow("events:\n  verifier_approve:\n    - notify_ui\n");
    let err = interpret_pending(&ws, parent, &wt, &wf, &fx.deps()).unwrap_err();
    assert!(
        matches!(err, crate::prompt::Error::ActionUnsupported { .. }),
        "{err:?}"
    );
}

#[test]
fn verifier_reject_honors_a_literal_with_value_as_feedback() {
    // `dispatch(worker, with: <literal>)` uses the literal as the
    // re-dispatched worker's goal (not the verifier response).
    let (_h, ws) = fixture::workspace();
    fixture::amend_config(&ws, &[("souls/verifier.md", "v")]);
    let parent = "20260101-g5";
    fixture::spawn_root(&ws, parent);
    let fx = Fx::new();
    let worker = returned_child(&ws, parent, "worker", "do it", ("out.txt", "x\n"), &fx);
    let wtip = worker_tip(&ws, &worker, &fx);
    verifier_child(&ws, parent, &wtip, Some("REJECT: bad"), &fx);

    let wt = agent_worktree(&ws, parent);
    let wf = workflow("events:\n  verifier_reject:\n    - \"dispatch(worker, with: fixit)\"\n");
    interpret_pending(&ws, parent, &wt, &wf, &fx.deps()).unwrap();

    // A fresh worker was dispatched with the literal feedback as its goal.
    let redispatched = fx.launcher.launched.borrow()[2].clone();
    let goal = std::fs::read_to_string(agent_worktree(&ws, &redispatched).join("goal.md")).unwrap();
    assert_eq!(goal, "fixit");
}

#[test]
fn split_frontmatter_reads_epitaph_and_body() {
    assert_eq!(split_frontmatter("no frontmatter"), (String::new(), None));
    let (ep, body) =
        split_frontmatter("---\nepitaph: final-response\nterminal_ref: x\n---\nAPPROVE");
    assert_eq!(ep, "final-response");
    assert_eq!(body.as_deref(), Some("APPROVE"));
    let (ep2, body2) = split_frontmatter("---\nepitaph: stopped\n---\n");
    assert_eq!(ep2, "stopped");
    assert_eq!(body2, None);
}
