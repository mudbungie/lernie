use super::*;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

const SETTLE: Duration = Duration::from_millis(300);

fn wait_quiet(watcher: &Watcher) {
    std::thread::sleep(Duration::from_millis(100));
    let _ = watcher.tick();
}

fn settle_and_tick(watcher: &Watcher) -> Vec<Change> {
    std::thread::sleep(SETTLE);
    watcher.tick()
}

/// Build an agent worktree dir under `agents/` (ARCH §2.2).
fn make_agent_worktree(repo: &Path, id: &str) {
    fs::create_dir_all(repo.join("agents").join(id)).unwrap();
}

#[test]
fn new_errors_on_missing_repo() {
    let root = tempdir().unwrap();
    let missing = root.path().join("does-not-exist");
    let err = match Watcher::new(&missing) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert!(err.to_string().contains("filesystem watcher"));
}

#[test]
fn tick_is_empty_when_nothing_changed() {
    let root = tempdir().unwrap();
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    assert!(watcher.tick().is_empty());
}

#[test]
fn detects_step_request_creation_at_conv_repo_root() {
    // v0.3.1: step records live at <conv-repo>/steps/<conv-id>/<NNN>/,
    // outside every worktree (ARCH §2.2 / §2.3).
    let root = tempdir().unwrap();
    let step_dir = root.path().join("steps/abc-1/001");
    fs::create_dir_all(&step_dir).unwrap();
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    let target = step_dir.join("request.json");
    fs::write(&target, b"{}").unwrap();
    let changes = settle_and_tick(&watcher);
    let hit = changes.iter().find(|c| c.path == target).expect("event");
    assert_eq!(hit.kind, ChangeKind::Touched);
}

#[test]
fn detects_subagent_step_record_at_conv_repo_root() {
    // Subagent step records share the same conv-repo-root `steps/`
    // tree, namespaced by the subagent's hyphenated descent (§2.2 /
    // §2.3). The watcher does not care whether the conv-id is a root
    // or a subagent.
    let root = tempdir().unwrap();
    let sub_step = root.path().join("steps/aa-bb/001");
    fs::create_dir_all(&sub_step).unwrap();
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    let target = sub_step.join("request.json");
    fs::write(&target, b"{}").unwrap();
    let changes = settle_and_tick(&watcher);
    let hit = changes.iter().find(|c| c.path == target).expect("event");
    assert_eq!(hit.kind, ChangeKind::Touched);
}

#[test]
fn detects_inbox_deposits_at_workspace_root() {
    // Inboxes live at `<workspace>/inbox/<agent-id>/` (ARCH §2.11).
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("inbox/aa-bb")).unwrap();
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    let target = root.path().join("inbox/aa-bb/user-001.md");
    fs::write(&target, b"x").unwrap();
    let changes = settle_and_tick(&watcher);
    let hit = changes.iter().find(|c| c.path == target).expect("event");
    assert_eq!(hit.kind, ChangeKind::Touched);
}

#[test]
fn detects_removal_under_summary() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("agents/aa-bb/summary")).unwrap();
    let target = root.path().join("agents/aa-bb/summary/001.md");
    fs::write(&target, b"hi").unwrap();
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    fs::remove_file(&target).unwrap();
    let changes = settle_and_tick(&watcher);
    let hit = changes.iter().find(|c| c.path == target).expect("event");
    assert_eq!(hit.kind, ChangeKind::Removed);
}

#[test]
fn ignores_paths_outside_allowlist() {
    let root = tempdir().unwrap();
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    fs::write(root.path().join("README.md"), b"x").unwrap();
    fs::create_dir_all(root.path().join("random")).unwrap();
    fs::write(root.path().join("random/x.txt"), b"x").unwrap();
    fs::create_dir_all(root.path().join("agents/aa-bb/random")).unwrap();
    fs::write(root.path().join("agents/aa-bb/random/x.txt"), b"x").unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert!(watcher.tick().is_empty());
}

#[test]
fn coalesces_rapid_writes_to_one_event() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("steps")).unwrap();
    let target = root.path().join("steps/out.log");
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    for i in 0..5 {
        fs::write(&target, format!("line {i}")).unwrap();
    }
    std::thread::sleep(Duration::from_millis(400));
    let changes = watcher.tick();
    let hits: Vec<_> = changes.iter().filter(|e| e.path == target).collect();
    assert_eq!(hits.len(), 1, "got {changes:?}");
    assert_eq!(hits[0].kind, ChangeKind::Touched);
}

#[test]
fn coalesces_atomic_rename_to_destination() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("steps/abc/001")).unwrap();
    let tmp = root.path().join("steps/abc/001/request.json.tmp");
    let final_path = root.path().join("steps/abc/001/request.json");
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    fs::write(&tmp, b"{}").unwrap();
    fs::rename(&tmp, &final_path).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let changes = watcher.tick();
    let finals: Vec<_> = changes.iter().filter(|e| e.path == final_path).collect();
    let tmps: Vec<_> = changes.iter().filter(|e| e.path == tmp).collect();
    assert_eq!(
        finals.len(),
        1,
        "expected exactly one event for destination: {changes:?}"
    );
    assert_eq!(finals[0].kind, ChangeKind::Touched);
    assert!(
        tmps.is_empty(),
        "rename source should not surface: {tmps:?}"
    );
}

#[test]
fn detects_goal_md_update_in_an_agent_worktree() {
    let root = tempdir().unwrap();
    make_agent_worktree(root.path(), "aa-bb");
    let target = root.path().join("agents/aa-bb/goal.md");
    let watcher = Watcher::new(root.path()).unwrap();
    wait_quiet(&watcher);
    fs::write(&target, b"hi").unwrap();
    let changes = settle_and_tick(&watcher);
    assert!(
        changes
            .iter()
            .any(|e| e.path == target && e.kind == ChangeKind::Touched)
    );
}

#[test]
fn is_watched_admits_root_control_files() {
    let r = Path::new("/r");
    for prefix in ROOT_CONTROL_PREFIXES {
        assert!(is_watched(r, &r.join(prefix)), "{prefix}");
        assert!(
            is_watched(r, &r.join(prefix).join("child")),
            "{prefix}/child"
        );
    }
}

#[test]
fn is_watched_admits_per_worktree_paths_under_any_agent_id() {
    let r = Path::new("/r");
    for prefix in WORKTREE_PREFIXES {
        for id in ["aa-bb", "20260424T120000Z-deadbeef"] {
            let path = r.join("agents").join(id).join(prefix);
            assert!(is_watched(r, &path), "agents/{id}/{prefix}");
            assert!(
                is_watched(r, &path.join("child")),
                "agents/{id}/{prefix}/child"
            );
        }
    }
}

#[test]
fn is_watched_admits_refs_and_head() {
    let r = Path::new("/r");
    for prefix in REFS_PREFIXES {
        assert!(is_watched(r, &r.join(prefix)), "{prefix}");
    }
    assert!(is_watched(r, &r.join("repo.git/refs/heads/agents/aa-bb")));
}

#[test]
fn is_watched_rejects_unrelated_top_level_files() {
    let r = Path::new("/r");
    assert!(!is_watched(r, &r.join("README.md")));
    assert!(!is_watched(r, Path::new("/other/manifest.yaml")));
}

#[test]
fn is_watched_rejects_arbitrary_subdir_files() {
    let r = Path::new("/r");
    assert!(!is_watched(r, &r.join("aa-bb/random.txt")));
    assert!(!is_watched(r, &r.join("agents/aa-bb/notes/x.md")));
    // A bare `agents/<id>` file (no per-worktree tail) is not watched.
    assert!(!is_watched(r, &r.join("agents/aa-bb")));
}

#[test]
fn classify_removed_event_is_removed() {
    use notify::event::RemoveKind;
    let k = classify(EventKind::Remove(RemoveKind::File), Path::new("/nope/xyz"));
    assert_eq!(k, ChangeKind::Removed);
}

#[test]
fn ingest_splits_name_both_into_from_and_to() {
    let mut raw = Vec::new();
    ingest(
        Event {
            kind: EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            paths: vec![PathBuf::from("/a"), PathBuf::from("/b")],
            attrs: Default::default(),
        },
        &mut raw,
    );
    assert_eq!(raw.len(), 2);
    assert!(matches!(
        raw[0].1,
        EventKind::Modify(ModifyKind::Name(RenameMode::From))
    ));
    assert!(matches!(
        raw[1].1,
        EventKind::Modify(ModifyKind::Name(RenameMode::To))
    ));
}

#[test]
fn coalesce_drops_prior_events_when_rename_from_arrives() {
    let repo = Path::new("/r");
    let p = PathBuf::from("/r/steps/abc/001/request.json");
    let raw = vec![
        (
            p.clone(),
            EventKind::Create(notify::event::CreateKind::File),
        ),
        (
            p.clone(),
            EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        ),
    ];
    assert!(coalesce(repo, raw).is_empty());
}

#[test]
fn classify_unknown_kind_uses_path_existence() {
    let root = tempdir().unwrap();
    let present = root.path().join("x");
    fs::write(&present, b"").unwrap();
    assert_eq!(classify(EventKind::Any, &present), ChangeKind::Touched);
    assert_eq!(
        classify(EventKind::Any, Path::new("/no/such/path")),
        ChangeKind::Removed
    );
}
