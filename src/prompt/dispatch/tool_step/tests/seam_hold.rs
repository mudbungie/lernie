//! The hold-mark lifecycle at the window (ARCH §3.3 *Tool control*):
//! park, resume-skip, lift, restate, and the mark's git-failure arms.
//! Split from [`super::seam`] to hold the 300-line code-file cap.

use super::Resolution;
use super::seam::{Rig, control_script, found_mark_repo, gated, tool_use};
use crate::prompt::Error;
use crate::prompt::dispatch::tool_step::ToolWindow;
use crate::template::{GitRunner, RealGit};
use crate::workspace::hold;
use brazen::Content;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn a_hold_parks_the_window_with_the_mark_and_nothing_past_it() {
    let rig = Rig::new("agent-hold");
    found_mark_repo(rig.ws.path());
    let control = control_script(
        rig.ws.path(),
        "echo '{\"verdict\":\"hold\",\"reason\":\"awaiting operator review\"}'",
    );
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let git = RealGit::new();
    let window = rig
        .run(
            "agent-hold",
            &resolution,
            &[tool_use("t1", "bash"), tool_use("t2", "read_file")],
            &git,
        )
        .unwrap();
    assert!(matches!(window, ToolWindow::Held));
    // Nothing executed, nothing committed — the park precedes execution.
    assert!(rig.executed().is_empty());
    assert!(!rig.worktree.join("messages/002-tool.json").exists());
    let mark = hold::read(rig.ws.path(), "agent-hold", &git).unwrap();
    assert_eq!(mark.tool_use_id, "t1");
    assert_eq!(mark.tool, "bash");
    assert_eq!(mark.reason, "awaiting operator review");
}

#[test]
fn a_resume_skips_committed_results_and_a_pass_lifts_the_mark() {
    // The parked window's frontier: t1's result committed before the
    // park, t2 held. On resume the control passes — t1 is skipped (its
    // result is the transcript's), t2 executes, the mark lifts.
    let rig = Rig::new("agent-resume");
    found_mark_repo(rig.ws.path());
    let git = RealGit::new();
    let t1_result = serde_json::to_string(&[Content::ToolResult {
        tool_use_id: "t1".into(),
        content: vec![Content::Text("done".into())],
        is_error: false,
    }])
    .unwrap();
    std::fs::write(rig.worktree.join("messages/002-tool.json"), t1_result).unwrap();
    git.run(&rig.worktree, &["add", "-A"]).unwrap();
    git.run(&rig.worktree, &["commit", "-m", "t1 result"])
        .unwrap();
    hold::write(
        rig.ws.path(),
        "agent-resume",
        &hold::Held {
            tool_use_id: "t2".into(),
            tool: "read_file".into(),
            reason: "review".into(),
        },
        &git,
    )
    .unwrap();
    let control = control_script(rig.ws.path(), "echo '{\"verdict\":\"pass\"}'");
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let window = rig
        .run(
            "agent-resume",
            &resolution,
            &[tool_use("t1", "bash"), tool_use("t2", "read_file")],
            &git,
        )
        .unwrap();
    assert!(matches!(window, ToolWindow::Completed));
    assert_eq!(rig.executed(), vec!["read_file"]);
    assert!(rig.worktree.join("messages/003-tool.json").exists());
    assert_eq!(hold::read(rig.ws.path(), "agent-resume", &git), None);
}

#[test]
fn a_hold_again_on_resume_restates_the_mark() {
    let rig = Rig::new("agent-rehold");
    found_mark_repo(rig.ws.path());
    let git = RealGit::new();
    hold::write(
        rig.ws.path(),
        "agent-rehold",
        &hold::Held {
            tool_use_id: "t1".into(),
            tool: "bash".into(),
            reason: "first look".into(),
        },
        &git,
    )
    .unwrap();
    let control = control_script(
        rig.ws.path(),
        "echo '{\"verdict\":\"hold\",\"reason\":\"still under review\"}'",
    );
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let window = rig
        .run("agent-rehold", &resolution, &[tool_use("t1", "bash")], &git)
        .unwrap();
    assert!(matches!(window, ToolWindow::Held));
    let mark = hold::read(rig.ws.path(), "agent-rehold", &git).unwrap();
    assert_eq!(mark.reason, "still under review");
}

// The hold-mark git failure arms, driven by a runner that delegates to
// real git except for one poisoned subcommand.
struct FailOn {
    inner: RealGit,
    needle: &'static str,
}
impl FailOn {
    fn new(needle: &'static str) -> Self {
        Self {
            inner: RealGit::new(),
            needle,
        }
    }
}
impl GitRunner for FailOn {
    fn run(&self, dest: &Path, args: &[&str]) -> std::io::Result<()> {
        if args.contains(&self.needle) {
            return Err(std::io::Error::other(format!("poisoned {}", self.needle)));
        }
        self.inner.run(dest, args)
    }
    fn run_capture(&self, dest: &Path, args: &[&str]) -> std::io::Result<String> {
        if args.contains(&self.needle) {
            return Err(std::io::Error::other(format!("poisoned {}", self.needle)));
        }
        self.inner.run_capture(dest, args)
    }
}

#[test]
fn a_failed_mark_write_surfaces_as_the_git_error_it_is() {
    let rig = Rig::new("agent-badwrite");
    found_mark_repo(rig.ws.path());
    let control = control_script(
        rig.ws.path(),
        "echo '{\"verdict\":\"hold\",\"reason\":\"r\"}'",
    );
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let git = FailOn::new("hash-object");
    let err = rig
        .run(
            "agent-badwrite",
            &resolution,
            &[tool_use("t1", "bash")],
            &git,
        )
        .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "hold mark write",
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn a_failed_mark_lift_surfaces_as_the_git_error_it_is() {
    let rig = Rig::new("agent-badclear");
    found_mark_repo(rig.ws.path());
    let real = RealGit::new();
    hold::write(
        rig.ws.path(),
        "agent-badclear",
        &hold::Held {
            tool_use_id: "t1".into(),
            tool: "bash".into(),
            reason: "r".into(),
        },
        &real,
    )
    .unwrap();
    let control = control_script(rig.ws.path(), "echo '{\"verdict\":\"pass\"}'");
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let git = FailOn::new("-d");
    let err = rig
        .run(
            "agent-badclear",
            &resolution,
            &[tool_use("t1", "bash")],
            &git,
        )
        .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "hold mark clear",
                ..
            }
        ),
        "{err:?}"
    );
    assert!(rig.executed().is_empty(), "the lift precedes the executor");
}

#[test]
fn committed_result_ids_reads_only_tool_entries_and_an_absent_dir_is_empty() {
    use crate::prompt::dispatch::transcript::committed_result_ids;
    let empty = TempDir::new().unwrap();
    assert!(committed_result_ids(empty.path()).unwrap().is_empty());
    let wt = TempDir::new().unwrap();
    std::fs::create_dir_all(wt.path().join("messages")).unwrap();
    std::fs::write(wt.path().join("messages/001-user.md"), "hi").unwrap();
    std::fs::write(
        wt.path().join("messages/002-claude-sonnet-5.json"),
        serde_json::to_string(&[Content::Text("t".into())]).unwrap(),
    )
    .unwrap();
    std::fs::write(
        wt.path().join("messages/003-tool.json"),
        serde_json::to_string(&[Content::ToolResult {
            tool_use_id: "t9".into(),
            content: vec![],
            is_error: false,
        }])
        .unwrap(),
    )
    .unwrap();
    let ids = committed_result_ids(wt.path()).unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("t9"));
    // An unreadable `messages/` (a file where the directory belongs)
    // surfaces as the I/O error it is, never an empty set.
    let broken = TempDir::new().unwrap();
    std::fs::write(broken.path().join("messages"), "not a dir").unwrap();
    assert!(committed_result_ids(broken.path()).is_err());
}
