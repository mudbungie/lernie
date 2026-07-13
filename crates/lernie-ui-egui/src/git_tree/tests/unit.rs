//! Pure-function unit tests for the detect + cmd layers.

use crate::git_tree::GitTreeError;
use crate::git_tree::cmd::{parse_log, parse_step_commits};
use crate::git_tree::detect::{PREVIEW_MAX, extract_request_preview, truncate_preview};

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
fn extract_request_preview_pulls_first_user_message_content() {
    let json = br#"{"messages":[{"role":"user","content":"hi v03"}]}"#;
    assert_eq!(extract_request_preview(json).as_deref(), Some("hi v03"));
}

#[test]
fn extract_request_preview_returns_none_on_bad_json() {
    assert!(extract_request_preview(b"not json").is_none());
}

#[test]
fn extract_request_preview_returns_none_without_messages() {
    assert!(extract_request_preview(br#"{"model":"m"}"#).is_none());
}

#[test]
fn extract_request_preview_returns_none_when_messages_empty() {
    assert!(extract_request_preview(br#"{"messages":[]}"#).is_none());
}

#[test]
fn extract_request_preview_returns_none_when_content_not_string() {
    let json = br#"{"messages":[{"role":"user","content":[]}]}"#;
    assert!(extract_request_preview(json).is_none());
}

#[test]
fn parse_log_errors_on_line_missing_subject_separator() {
    // No `\x00` between the parent column and the subject — every log
    // entry must include the merge subject for v0.3.1 detection.
    let err = parse_log(b"only-one-token\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_errors_on_line_missing_timestamp() {
    let err = parse_log(b"only-one-token\x00subject\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_errors_on_non_numeric_timestamp() {
    let err = parse_log(b"abc notanumber\x00subject\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_parses_root_commit_with_no_parents() {
    let out = parse_log(b"abc 100 \x00scaffold\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].oid, "abc");
    assert_eq!(out[0].timestamp, 100);
    assert_eq!(out[0].parent_count, 0);
    assert_eq!(out[0].subject, "scaffold");
}

#[test]
fn parse_log_parses_single_parent_commit() {
    let out = parse_log(b"abc 100 def\x00add README\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].parent_count, 1);
    assert_eq!(out[0].subject, "add README");
}

#[test]
fn parse_log_parses_merge_commit_with_two_parents() {
    let out = parse_log(b"abc 100 def ghi\x00Merge branch 'foo'\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].parent_count, 2);
    assert_eq!(out[0].subject, "Merge branch 'foo'");
}

#[test]
fn parse_step_commits_parses_valid_lines() {
    let out = parse_step_commits(b"abc 100\ndef 200\n").unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].oid, "abc");
    assert_eq!(out[0].timestamp_unix, 100);
    assert_eq!(out[1].timestamp_unix, 200);
}

#[test]
fn parse_step_commits_errors_on_malformed_line() {
    let err = parse_step_commits(b"abc\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_step_commits_errors_on_non_numeric_timestamp() {
    let err = parse_step_commits(b"abc notanumber\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn error_display_for_log_format() {
    let e = GitTreeError::LogFormat("oops".into());
    let msg = e.to_string();
    assert!(msg.contains("malformed"));
}
