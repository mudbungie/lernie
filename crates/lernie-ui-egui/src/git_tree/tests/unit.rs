//! Pure-function unit tests for the detect + cmd layers.

use crate::git_tree::GitTreeError;
use crate::git_tree::cmd::{parse_log, parse_step_commits};
use crate::git_tree::detect::{
    PREVIEW_MAX, exchange_id_from_branch, exchange_id_from_v01_path, extract_v01_preview,
    extract_v02_preview, is_v01_exchange_path, truncate_preview, v02_exchange_id_from_path,
};

#[test]
fn is_v01_exchange_path_accepts_top_level_json() {
    assert!(is_v01_exchange_path("exchanges/abc.json"));
}

#[test]
fn is_v01_exchange_path_rejects_nested_steps() {
    assert!(!is_v01_exchange_path(
        "exchanges/abc/steps/001/request.json"
    ));
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
fn v02_exchange_id_from_path_matches_request_json() {
    assert_eq!(
        v02_exchange_id_from_path("exchanges/abc/steps/001/request.json"),
        Some("abc")
    );
}

#[test]
fn v02_exchange_id_from_path_matches_response_json() {
    assert_eq!(
        v02_exchange_id_from_path("exchanges/xyz/steps/002/response.json"),
        Some("xyz")
    );
}

#[test]
fn v02_exchange_id_from_path_rejects_v01_shape() {
    assert_eq!(v02_exchange_id_from_path("exchanges/abc.json"), None);
}

#[test]
fn v02_exchange_id_from_path_rejects_bare_steps_dir() {
    assert_eq!(v02_exchange_id_from_path("exchanges/abc/steps/"), None);
}

#[test]
fn v02_exchange_id_from_path_rejects_outside_exchanges() {
    assert_eq!(v02_exchange_id_from_path("artifacts/abc/steps/001/x"), None);
}

#[test]
fn exchange_id_from_v01_path_strips_dir_and_suffix() {
    assert_eq!(
        exchange_id_from_v01_path("exchanges/20260422T120000Z-aaaa.json"),
        "20260422T120000Z-aaaa"
    );
}

#[test]
fn exchange_id_from_v01_path_falls_back_for_unexpected_shape() {
    assert_eq!(exchange_id_from_v01_path("weird"), "weird");
}

#[test]
fn exchange_id_from_branch_strips_prefix() {
    assert_eq!(exchange_id_from_branch("ex/abc-1"), "abc-1");
}

#[test]
fn exchange_id_from_branch_falls_back_without_prefix() {
    assert_eq!(exchange_id_from_branch("other"), "other");
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
fn extract_v01_preview_returns_none_on_bad_json() {
    assert!(extract_v01_preview(b"not json").is_none());
}

#[test]
fn extract_v01_preview_returns_none_when_user_message_not_string() {
    assert!(extract_v01_preview(br#"{"user_message":42}"#).is_none());
}

#[test]
fn extract_v01_preview_returns_trimmed_text() {
    assert_eq!(
        extract_v01_preview(br#"{"user_message":"  hello  "}"#).as_deref(),
        Some("hello")
    );
}

#[test]
fn extract_v02_preview_pulls_first_user_message_content() {
    let json = br#"{"messages":[{"role":"user","content":"hi v02"}]}"#;
    assert_eq!(extract_v02_preview(json).as_deref(), Some("hi v02"));
}

#[test]
fn extract_v02_preview_returns_none_on_bad_json() {
    assert!(extract_v02_preview(b"not json").is_none());
}

#[test]
fn extract_v02_preview_returns_none_without_messages() {
    assert!(extract_v02_preview(br#"{"model":"m"}"#).is_none());
}

#[test]
fn extract_v02_preview_returns_none_when_messages_empty() {
    assert!(extract_v02_preview(br#"{"messages":[]}"#).is_none());
}

#[test]
fn extract_v02_preview_returns_none_when_content_not_string() {
    let json = br#"{"messages":[{"role":"user","content":[]}]}"#;
    assert!(extract_v02_preview(json).is_none());
}

#[test]
fn parse_log_errors_on_line_missing_timestamp() {
    let err = parse_log(b"only-one-token\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_errors_on_non_numeric_timestamp() {
    let err = parse_log(b"abc notanumber\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_parses_root_commit_with_no_parents() {
    let out = parse_log(b"abc 100 \n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].oid, "abc");
    assert_eq!(out[0].timestamp, 100);
    assert_eq!(out[0].parent_count, 0);
}

#[test]
fn parse_log_parses_single_parent_commit() {
    let out = parse_log(b"abc 100 def\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].parent_count, 1);
}

#[test]
fn parse_log_parses_merge_commit_with_two_parents() {
    let out = parse_log(b"abc 100 def ghi\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].parent_count, 2);
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
