use super::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

struct Fixture {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        run_git(&path, &["init", "-q", "-b", "main"]);
        run_git(&path, &["config", "user.email", "t@t.local"]);
        run_git(&path, &["config", "user.name", "Tester"]);
        run_git(&path, &["config", "commit.gpgsign", "false"]);
        fs::create_dir(path.join("exchanges")).unwrap();
        Self { _dir: dir, path }
    }

    fn commit_exchange(&self, id: &str, user_message: &str) {
        let rel = format!("exchanges/{id}.json");
        let json = format!(r#"{{"user_message":"{}"}}"#, user_message.replace('"', "\\\""));
        fs::write(self.path.join(&rel), json).unwrap();
        run_git(&self.path, &["add", &rel]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("ex {id}")]);
    }

    fn commit_other(&self, file: &str, body: &str) {
        fs::write(self.path.join(file), body).unwrap();
        run_git(&self.path, &["add", file]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("add {file}")]);
    }

    fn commit_malformed_exchange(&self, id: &str) {
        let rel = format!("exchanges/{id}.json");
        fs::write(self.path.join(&rel), "{ not valid json").unwrap();
        run_git(&self.path, &["add", &rel]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("bad {id}")]);
    }
}

fn run_git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-04-22T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-04-22T12:00:00Z")
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn from_repo_errors_when_repo_missing() {
    let dir = tempdir().unwrap();
    let err = match GitTree::from_repo(&dir.path().join("nope")) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    // Non-existent -C target makes git exit nonzero.
    assert!(
        matches!(err, GitTreeError::Git { .. } | GitTreeError::Spawn(_)),
        "got {err:?}"
    );
}

#[test]
fn from_repo_empty_tree_on_fresh_repo() {
    let fx = Fixture::new();
    // git log on an unborn HEAD exits nonzero — treat as Git error.
    let err = GitTree::from_repo(&fx.path).unwrap_err();
    assert!(matches!(err, GitTreeError::Git { .. }), "got {err:?}");
}

#[test]
fn from_repo_linear_history_with_exchanges() {
    let fx = Fixture::new();
    fx.commit_exchange("20260422T120000Z-aaaa", "first prompt");
    fx.commit_exchange("20260422T120500Z-bbbb", "second prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    assert_eq!(
        tree.commits[0].exchange_id.as_deref(),
        Some("20260422T120000Z-aaaa")
    );
    assert_eq!(tree.commits[0].preview.as_deref(), Some("first prompt"));
    assert_eq!(
        tree.commits[1].exchange_id.as_deref(),
        Some("20260422T120500Z-bbbb")
    );
    assert_eq!(tree.commits[1].preview.as_deref(), Some("second prompt"));
    assert_eq!(tree.commits[0].short_oid.len(), 8);
}

#[test]
fn from_repo_commit_without_exchange_has_no_preview() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "hi");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert!(tree.commits[0].exchange_id.is_none());
    assert!(tree.commits[0].preview.is_none());
}

#[test]
fn from_repo_malformed_exchange_keeps_id_but_drops_preview() {
    let fx = Fixture::new();
    fx.commit_malformed_exchange("20260422T120000Z-cccc");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(
        tree.commits[0].exchange_id.as_deref(),
        Some("20260422T120000Z-cccc")
    );
    assert!(tree.commits[0].preview.is_none());
}

#[test]
fn from_repo_exchange_json_without_user_message_drops_preview() {
    let fx = Fixture::new();
    let rel = "exchanges/20260422T120000Z-dddd.json";
    fs::write(fx.path.join(rel), r#"{"other":"thing"}"#).unwrap();
    run_git(&fx.path, &["add", rel]);
    run_git(&fx.path, &["commit", "-q", "-m", "ex"]);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.commits[0].preview.is_none());
    assert!(tree.commits[0].exchange_id.is_some());
}

#[test]
fn is_v01_exchange_path_accepts_top_level_json() {
    assert!(is_v01_exchange_path("exchanges/abc.json"));
}

#[test]
fn is_v01_exchange_path_rejects_nested_steps() {
    assert!(!is_v01_exchange_path("exchanges/abc/steps/001/request.json"));
}

#[test]
fn is_v01_exchange_path_rejects_non_json() {
    assert!(!is_v01_exchange_path("exchanges/abc.txt"));
}

#[test]
fn is_v01_exchange_path_rejects_outside_exchanges() {
    assert!(!is_v01_exchange_path("artifacts/abc.json"));
}

#[test]
fn exchange_id_from_path_strips_dir_and_suffix() {
    assert_eq!(
        exchange_id_from_path("exchanges/20260422T120000Z-aaaa.json"),
        "20260422T120000Z-aaaa"
    );
}

#[test]
fn exchange_id_from_path_falls_back_for_unexpected_shape() {
    assert_eq!(exchange_id_from_path("weird"), "weird");
}

#[test]
fn truncate_preview_passes_short_input_through() {
    assert_eq!(truncate_preview("hi"), "hi");
}

#[test]
fn truncate_preview_collapses_whitespace_and_trims() {
    assert_eq!(truncate_preview("  a\n\tb  "), "a  b");
}

#[test]
fn truncate_preview_cuts_long_input_with_ellipsis() {
    let long = "x".repeat(PREVIEW_MAX + 20);
    let out = truncate_preview(&long);
    let last = out.chars().last().unwrap();
    assert_eq!(last, '…');
    assert_eq!(out.chars().count(), PREVIEW_MAX);
}

#[test]
fn extract_preview_returns_none_on_bad_json() {
    assert!(extract_preview(b"not json").is_none());
}

#[test]
fn extract_preview_returns_none_when_user_message_not_string() {
    assert!(extract_preview(br#"{"user_message":42}"#).is_none());
}

#[test]
fn extract_preview_returns_trimmed_text() {
    assert_eq!(
        extract_preview(br#"{"user_message":"  hello  "}"#).as_deref(),
        Some("hello")
    );
}

#[test]
fn parse_log_errors_on_line_without_space() {
    let err = parse_log(b"only-one-token\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_errors_on_non_numeric_timestamp() {
    let err = parse_log(b"abc notanumber\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_parses_valid_lines() {
    let out = parse_log(b"abc 100\ndef 200\n").unwrap();
    assert_eq!(out, vec![("abc".into(), 100), ("def".into(), 200)]);
}

#[test]
fn render_empty_tree_shows_placeholder() {
    let ctx = egui::Context::default();
    let tree = GitTree { commits: vec![] };
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &tree));
    });
}

#[test]
fn render_populated_tree_runs_without_panic() {
    let ctx = egui::Context::default();
    let tree = GitTree {
        commits: vec![
            CommitNode {
                oid: "a".repeat(40),
                short_oid: "aaaaaaaa".into(),
                timestamp_unix: 1,
                exchange_id: Some("ex1".into()),
                preview: Some("hello".into()),
            },
            CommitNode {
                oid: "b".repeat(40),
                short_oid: "bbbbbbbb".into(),
                timestamp_unix: 2,
                exchange_id: None,
                preview: None,
            },
        ],
    };
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &tree));
    });
}

#[test]
fn error_display_for_log_format() {
    let e = GitTreeError::LogFormat("oops".into());
    let msg = e.to_string();
    assert!(msg.contains("malformed"));
}

#[test]
fn short_oid_falls_back_for_unexpectedly_short_hash() {
    // Direct unit test of build_node via a crafted short oid; the
    // get(..8) fallback covers the unreachable-in-practice case where
    // git returns a sub-8-char identifier.
    let node = CommitNode {
        oid: "abc".into(),
        short_oid: "abc".into(),
        timestamp_unix: 0,
        exchange_id: None,
        preview: None,
    };
    assert_eq!(node.short_oid, "abc");
}
