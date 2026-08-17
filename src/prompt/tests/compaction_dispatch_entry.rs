//! **A conversation past a compaction still has its opening prompt**
//! (ARCH §2.7 *the goal is not compaction-eligible*, bl-898f).
//!
//! Written from the operator's sentence, not from the mechanism: run a
//! conversation through a real compaction — a compactor forked off the
//! checkpoint commit, nominating through the real `mark_for_deletion`,
//! its product landed by the §6 hop — and assert the branch's
//! `messages/001-*` is still on disk **and still renders**, i.e. still
//! composes into the next model call's wire history (§5.2).
//!
//! Before the fix it did not. The opening prompt is written twice at
//! dispatch — as `goal.md` (§2.8) and, through the front door (§2.11), as
//! the dispatch entry `messages/001-user.md` — and the compactor's own
//! goal quotes `goal.md` verbatim (§2.7), so the one entry a model told to
//! nominate superseded files reads as *pure duplication* is the one the
//! operator reads. It was nominated, marked, squashed into the compaction
//! base, and gone.

use super::advance::{AGENT, RecLauncher, worker_config};
use super::fixtures::*;
use crate::prompt::Clock;
use crate::prompt::child_dispatch::{ChildDispatchRequest, run as dispatch_child};
use crate::prompt::compactor::tools;
use crate::prompt::dispatch::advance::{AdvanceOutcome, run};
use crate::prompt::inbox::{self, deposit_result};
use crate::prompt::{Error, PinnedDocs};
use crate::template::{GitRunner, RealGit};
use crate::workspace::agent_name::mint::test_rng;
use crate::workspace::{agent_worktree, fixture};

/// The branch's opening prompt — the text the operator typed, which
/// `goal.md` and the dispatch entry both carry (§2.8, §2.11).
const OPENING: &str = "port the parser to the new grammar\n";

/// A hyphen-free compact stamp so a dispatched child's id is a clean
/// two-token descent segment (§2.3), as [`super::advance_compaction`]'s.
struct DescentClock;
impl Clock for DescentClock {
    fn now_iso8601(&self) -> String {
        "iso".into()
    }
    fn now_compact(&self) -> String {
        "ct1".into()
    }
}

#[test]
fn a_conversation_past_a_compaction_still_has_its_opening_prompt() {
    let (_h, ws) = fixture::workspace();
    let parent = AGENT;
    let parent_wt = fixture::spawn_root(&ws, parent);
    let git = RealGit::new();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let rec = RecLauncher::default();

    // The dispatching branch at the checkpoint commit: its opening prompt
    // as the dispatch entry, its goal beside it (the two projections of
    // one dispatch), and one later exchange for the compactor to shed.
    std::fs::create_dir_all(parent_wt.join("messages")).unwrap();
    std::fs::write(parent_wt.join("messages/001-user.md"), OPENING).unwrap();
    std::fs::write(parent_wt.join("messages/002-user.md"), "any progress?\n").unwrap();
    std::fs::write(parent_wt.join("goal.md"), OPENING).unwrap();
    git.run(&parent_wt, &["add", "-A"]).unwrap();
    git.run(&parent_wt, &["commit", "-m", "checkpoint"])
        .unwrap();

    // A real compactor child, forked off that commit: its worktree
    // inherits the dispatching branch's transcript (§2.7, no dialog prune).
    let child = dispatch_child(
        &ChildDispatchRequest {
            repo: &ws,
            parent_branch: parent,
            parent_worktree: &parent_wt,
            role: "compactor",
            goal: "compact",
            name: None,
            fork_point: None,
            cwd: None,
            pins: PinnedDocs::none(),
        },
        &git,
        &DescentClock,
        &id,
        &rec,
        test_rng(),
    )
    .unwrap();
    let cwt = agent_worktree(&ws, &child);
    assert_eq!(
        std::fs::read_to_string(cwt.join("messages/001-user.md")).unwrap(),
        OPENING,
        "the compactor inherits the entry it used to delete"
    );

    // The compaction itself, through the compactor's real toolset. The
    // nomination the shipped model actually made is declined in-band and
    // stages nothing; the later entry is shed as it always was.
    tools::write_summary(&cwt, "the parser port is underway\n").unwrap();
    let declined = tools::mark_for_deletion(&cwt, "messages/001-user.md", &git).unwrap_err();
    assert!(
        matches!(&declined, Error::DispatchEntryNotEligible { path }
            if path == "messages/001-user.md"),
        "{declined:?}"
    );
    tools::mark_for_deletion(&cwt, "messages/002-user.md", &git).unwrap();
    git.run(&cwt, &["add", "-A"]).unwrap();
    git.run(&cwt, &["commit", "-m", "compaction"]).unwrap();
    let tip = git.run_capture(&cwt, &["rev-parse", "HEAD"]).unwrap();

    // The compactor returns; the hop lands the rebase-forward and steps.
    deposit_result(
        &ws,
        parent,
        &child,
        inbox::Epitaph::FinalResponse,
        tip.trim(),
        Some("compacted"),
        &clock,
        &git,
    )
    .unwrap();
    inbox::deposit(&ws, parent, "user", "carry on", &clock).unwrap();

    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&happy_response_bytes())]);
    let (sleeper, tools_stub, stub_git) = (
        StubSleeper::default(),
        StubToolExecutor::ok(),
        StubGit::ok(),
    );
    let mut deps = valid_deps(&adapter, &sleeper, &stub_git, &clock, &id, &tools_stub, &ws);
    deps.git = &git;
    deps.launcher = &rec;
    let out = run(&ws, parent, None, &deps, &mut || Ok(worker_config())).unwrap();
    assert!(matches!(out, AdvanceOutcome::Terminal), "the step ran");

    // A compaction really landed — summary in, the later entry squashed out.
    assert_eq!(
        std::fs::read_to_string(parent_wt.join("summary/001.md")).unwrap(),
        "the parser port is underway\n"
    );
    assert!(!parent_wt.join("messages/002-user.md").exists());

    // The operator's copy of the opening prompt is still there …
    assert_eq!(
        std::fs::read_to_string(parent_wt.join("messages/001-user.md")).unwrap(),
        OPENING
    );
    // … and still renders: it composes into the wire history of the step
    // the branch took after the landing (§5.2), read off the step record.
    let req: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(ws.join(format!("steps/{parent}/001/request.json"))).unwrap(),
    )
    .unwrap();
    assert!(
        req["messages"].to_string().contains("port the parser"),
        "{}",
        req["messages"]
    );
}
