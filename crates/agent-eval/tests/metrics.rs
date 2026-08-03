//! Coverage for the per-run metrics derivation and the efficiency
//! aggregate (bl-36fa, ARCH §9.3) — all over fabricated `steps/` slices
//! and run homes, never a live workspace.

use agent_eval::metrics::{self, Efficiency, RunMetrics};
use agent_eval::record::RunRecord;
use std::fs;
use std::path::Path;

const MSG_M1: &str = r#"{"type":"message_start","v":1,"id":null,"model":"m1","role":"assistant"}"#;
const MSG_NO_MODEL: &str =
    r#"{"type":"message_start","v":1,"id":null,"model":null,"role":"assistant"}"#;
const TOOL_START: &str =
    r#"{"type":"content_start","index":0,"kind":{"tool_use":{"id":"t","name":"bash"}}}"#;
const TEXT_START: &str = r#"{"type":"content_start","index":1,"kind":{"text":{}}}"#;
const USAGE_IN5: &str = r#"{"type":"usage","input_tokens":5,"output_tokens":null,"cache_read_tokens":null,"cache_write_tokens":null}"#;
const USAGE_FULL: &str = r#"{"type":"usage","input_tokens":10,"output_tokens":4,"cache_read_tokens":2,"cache_write_tokens":1}"#;
const END: &str = r#"{"type":"end"}"#;

fn write_step(root: &Path, agent_dir: &str, seq: &str, lines: &[&str]) {
    let dir = root.join("steps").join(agent_dir).join(seq);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("response.json"), format!("{}\n", lines.join("\n"))).unwrap();
}

#[test]
fn collect_walks_the_descent_and_derives_every_counter() {
    let ws = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    // Step with two attempt segments: the superseded first segment's
    // tool invocation is NOT counted; both segments' usage IS.
    write_step(
        ws.path(),
        "a1",
        "001",
        &[
            MSG_M1, TOOL_START, USAGE_IN5, END, // superseded attempt
            MSG_M1, TOOL_START, TEXT_START, USAGE_FULL, END, // authoritative
        ],
    );
    // A hyphen-descendant child's step counts toward the same run.
    write_step(
        ws.path(),
        "a1-child",
        "001",
        &[MSG_NO_MODEL, USAGE_IN5, END],
    );
    // A sibling agent outside the descent is ignored.
    write_step(ws.path(), "other", "001", &[USAGE_FULL, END]);
    // Non-step dirs and malformed content are skipped, not errors.
    write_step(ws.path(), "a1", "junk", &[END]);
    write_step(ws.path(), "a1", "002", &["not json", ""]);
    fs::create_dir_all(ws.path().join("steps/a1/003")).unwrap(); // no response.json
    fs::write(
        home.path().join("models.yaml"),
        "models:\n  m1:\n    provider: acme\n    model_id: m1-wire\n",
    )
    .unwrap();

    let m = metrics::collect(ws.path(), "a1", home.path());
    assert_eq!(m.attempts, 3); // 2 in 001 + 1 in the child step
    assert_eq!(m.tool_invocations, 1); // authoritative segment only
    assert_eq!(m.input_tokens, Some(20)); // 5 + 10 + 5, every segment billed
    assert_eq!(m.output_tokens, Some(4));
    assert_eq!(m.cache_read_tokens, Some(2));
    assert_eq!(m.cache_write_tokens, Some(1));
    assert_eq!(m.models, vec!["m1".to_string()]);
    assert_eq!(m.providers, vec!["acme".to_string()]);
}

#[test]
fn a_bare_workspace_yields_observed_zeros_and_nothing_resolved() {
    let ws = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    // No steps/ tree at all: zeros are what was observed; the usage
    // counters stay unreported (None), never fabricated zeros.
    let m = metrics::collect(ws.path(), "a1", home.path());
    assert_eq!(m, RunMetrics::default());
}

#[test]
fn a_segmentless_response_counts_no_attempt_and_no_tools() {
    let ws = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    // Events but no terminal end: no complete segment, so no attempt
    // and no authoritative tool count — but usage is still billed.
    write_step(ws.path(), "a1", "001", &[MSG_M1, TOOL_START, USAGE_IN5]);
    // A descent-named entry that is a file, not a directory: skipped.
    fs::write(ws.path().join("steps/a1-stray"), "x").unwrap();
    let m = metrics::collect(ws.path(), "a1", home.path());
    assert_eq!(m.attempts, 0);
    assert_eq!(m.tool_invocations, 0);
    assert_eq!(m.input_tokens, Some(5));
    assert_eq!(m.output_tokens, None);
}

#[test]
fn provider_resolution_matches_key_or_model_id_and_tolerates_junk() {
    let ws = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    // Two models observed: one resolves by model_id, one matches a key
    // whose row has no provider line, and the yaml carries extras.
    write_step(
        ws.path(),
        "a1",
        "001",
        &[
            r#"{"type":"message_start","v":1,"id":null,"model":"wire-x","role":"assistant"}"#,
            r#"{"type":"message_start","v":1,"id":null,"model":"bare","role":"assistant"}"#,
            r#"{"type":"message_start","v":1,"id":null,"model":"unknown","role":"assistant"}"#,
            END,
        ],
    );
    fs::write(
        home.path().join("models.yaml"),
        concat!(
            "models:\n",
            "  x:\n    provider: acme\n    model_id: wire-x\n    context_window: 1\n",
            "  bare:\n    model_id: bare\n",
        ),
    )
    .unwrap();
    let m = metrics::collect(ws.path(), "a1", home.path());
    assert_eq!(
        m.models,
        vec![
            "bare".to_string(),
            "unknown".to_string(),
            "wire-x".to_string()
        ]
    );
    assert_eq!(m.providers, vec!["acme".to_string()]);

    // An unparseable models.yaml resolves nothing — never a guess.
    fs::write(home.path().join("models.yaml"), "models: [unterminated").unwrap();
    let m = metrics::collect(ws.path(), "a1", home.path());
    assert!(m.providers.is_empty());
}

fn run(pass: bool, wall_ms: u64, metrics: Option<RunMetrics>) -> RunRecord {
    RunRecord {
        pass,
        wall_ms,
        metrics,
    }
}

#[test]
fn efficiency_aggregates_and_distinguishes_missing_from_zero() {
    let disclosed = RunMetrics {
        attempts: 3,
        tool_invocations: 7,
        input_tokens: Some(100),
        output_tokens: None,
        cache_read_tokens: Some(0),
        cache_write_tokens: None,
        models: vec![],
        providers: vec![],
    };
    let runs = [
        run(true, 2000, Some(disclosed.clone())),
        run(false, 1000, Some(disclosed)),
        run(false, 3000, None), // undisclosed: contributes wall only
    ];
    let e = Efficiency::over(&runs);
    assert_eq!(e.runs, 3);
    assert_eq!(e.disclosed, 2);
    assert!((e.wall_mean_s() - 2.0).abs() < 1e-9);
    assert_eq!(e.attempts, Some(6));
    assert_eq!(e.attempts_mean(), Some(3.0));
    assert_eq!(e.tools_mean(), Some(7.0));
    assert_eq!(e.input_tokens, Some(200));
    assert_eq!(e.output_tokens, None); // never reported: stays missing
    assert_eq!(e.cache_read_tokens, Some(0)); // reported zero: stays zero
}

#[test]
fn efficiency_over_nothing_reports_nothing() {
    let e = Efficiency::over(&[]);
    assert_eq!(e.wall_mean_s(), 0.0);
    assert_eq!(e.attempts_mean(), None);
    assert_eq!(e.tools_mean(), None);
    let undisclosed = [run(true, 500, None)];
    let e = Efficiency::over(&undisclosed);
    assert_eq!(e.disclosed, 0);
    assert_eq!(e.attempts, None);
    assert_eq!(e.input_tokens, None);
}
