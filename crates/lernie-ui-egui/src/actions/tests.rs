use super::*;
use crate::git_tree::{BranchState, ConversationBranch};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::tempdir;

// Spawn discipline rationale and the binary-wide lock live in
// crate::test_support — one static for every test module, because
// per-module locks do not exclude each other's threads.
use crate::test_support::SPAWN_LOCK;

fn write_argv_recorder(dir: &Path, name: &str, log: &Path) -> PathBuf {
    let path = dir.join(name);
    let body = format!(
        "#!/bin/sh\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\" >> {}; done\nexit 0\n",
        log.display()
    );
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn drain(stream: Stream) {
    for _ in stream {}
}

fn branch(name: &str, state: BranchState) -> ConversationBranch {
    ConversationBranch {
        branch_name: name.to_string(),
        conv_id: name.to_string(),
        tip_oid: "0".repeat(40),
        tip_short_oid: "0000000".to_string(),
        tip_timestamp_unix: 0,
        steps: vec![],
        preview: None,
        streaming_text: None,
        tool_calls: vec![],
        state,
    }
}

#[test]
fn new_prompt_enabled_when_text_present() {
    assert!(new_prompt_enabled("hi"));
    assert!(new_prompt_enabled("  surrounded by spaces  "));
}

#[test]
fn new_prompt_disabled_for_empty_input() {
    assert!(!new_prompt_enabled(""));
}

#[test]
fn new_prompt_disabled_for_whitespace_only() {
    assert!(!new_prompt_enabled("   \t\n"));
}

#[test]
fn stop_disabled_when_selection_is_none() {
    let bs = vec![branch("foo", BranchState::InFlight)];
    assert!(!stop_enabled(None, &bs));
}

#[test]
fn stop_disabled_when_selection_not_in_branches() {
    let bs = vec![branch("foo", BranchState::InFlight)];
    assert!(!stop_enabled(Some("bar"), &bs));
}

#[test]
fn stop_disabled_when_selected_branch_stopped() {
    let bs = vec![branch("foo", BranchState::Stopped)];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_disabled_when_selected_branch_merged() {
    let bs = vec![branch("foo", BranchState::Merged)];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_disabled_when_selected_branch_conflicted() {
    let bs = vec![branch("foo", BranchState::Conflicted)];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_enabled_when_selection_is_in_flight() {
    let bs = vec![branch("foo", BranchState::InFlight)];
    assert!(stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_disabled_when_branches_empty() {
    let bs: Vec<ConversationBranch> = vec![];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_picks_correct_branch_among_several() {
    let bs = vec![
        branch("a", BranchState::Stopped),
        branch("b", BranchState::InFlight),
        branch("c", BranchState::Merged),
    ];
    assert!(stop_enabled(Some("b"), &bs));
    assert!(!stop_enabled(Some("a"), &bs));
    assert!(!stop_enabled(Some("c"), &bs));
}

#[test]
fn dispatch_new_prompt_emits_subcommand_repo_message_argv() {
    let _g = SPAWN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempdir().unwrap();
    let log = dir.path().join("argv.log");
    let bin = write_argv_recorder(dir.path(), "lernie", &log);
    let cli = Cli::new(bin);
    drain(dispatch_new_prompt(&cli, Path::new("/tmp/some-repo"), "hello world").unwrap());
    let recorded = fs::read_to_string(&log).unwrap();
    assert_eq!(recorded, "prompt\n/tmp/some-repo\nhello world\n");
}

#[test]
fn dispatch_stop_emits_subcommand_repo_branch_argv() {
    let _g = SPAWN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempdir().unwrap();
    let log = dir.path().join("argv.log");
    let bin = write_argv_recorder(dir.path(), "lernie", &log);
    let cli = Cli::new(bin);
    drain(dispatch_stop(&cli, Path::new("/tmp/repo"), "branch-id-abc").unwrap());
    let recorded = fs::read_to_string(&log).unwrap();
    assert_eq!(recorded, "stop\n/tmp/repo\nbranch-id-abc\n");
}

#[test]
fn dispatch_new_prompt_propagates_spawn_error() {
    let cli = Cli::new("/definitely/not/a/real/lernie-xyz");
    assert!(dispatch_new_prompt(&cli, Path::new("/tmp/r"), "x").is_err());
}

#[test]
fn dispatch_stop_propagates_spawn_error() {
    let cli = Cli::new("/definitely/not/a/real/lernie-xyz");
    assert!(dispatch_stop(&cli, Path::new("/tmp/r"), "b").is_err());
}

#[test]
fn actions_state_default_is_empty() {
    let s = ActionsState::default();
    assert!(s.new_prompt_input.is_empty());
    assert!(s.selected_branch.is_none());
}

#[test]
fn actions_state_clone_eq() {
    let s = ActionsState {
        new_prompt_input: "hi".to_string(),
        selected_branch: Some("b".to_string()),
    };
    let s2 = s.clone();
    assert_eq!(s, s2);
}
