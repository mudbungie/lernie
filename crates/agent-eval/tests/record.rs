//! Coverage for the evaluation record (bl-36fa): save/load round-trip,
//! load failures, the stats projection, and the derived observation
//! sets.

use agent_eval::metrics::RunMetrics;
use agent_eval::record::{Provenance, Record, RecordError, RunRecord, TaskRecord};

pub fn provenance() -> Provenance {
    Provenance {
        experiment: "baseline".to_string(),
        workflow: "/x/workflow.yaml".to_string(),
        suite: "tests/suite".to_string(),
        suite_revision: Some("abc123".to_string()),
        fixture_digest: Some("00ff".to_string()),
        driver: "fake-driver".to_string(),
        driver_version: Some("fake-driver 1.0".to_string()),
        runs_per_task: 2,
    }
}

fn metrics(models: &[&str], providers: &[&str]) -> RunMetrics {
    RunMetrics {
        attempts: 1,
        tool_invocations: 2,
        input_tokens: Some(10),
        output_tokens: Some(5),
        cache_read_tokens: None,
        cache_write_tokens: None,
        models: models.iter().map(|s| s.to_string()).collect(),
        providers: providers.iter().map(|s| s.to_string()).collect(),
    }
}

fn record() -> Record {
    Record {
        provenance: provenance(),
        tasks: vec![
            TaskRecord {
                id: "a".to_string(),
                categories: vec!["early_termination".to_string()],
                runs: vec![
                    RunRecord {
                        pass: true,
                        wall_ms: 1500,
                        metrics: Some(metrics(&["m1"], &["acme"])),
                    },
                    RunRecord {
                        pass: false,
                        wall_ms: 500,
                        metrics: Some(metrics(&["m2", "m1"], &["other"])),
                    },
                ],
            },
            TaskRecord {
                id: "b".to_string(),
                categories: vec!["scope_reduction".to_string()],
                runs: vec![
                    RunRecord {
                        pass: false,
                        wall_ms: 0,
                        metrics: None,
                    },
                    RunRecord {
                        pass: false,
                        wall_ms: 100,
                        metrics: None,
                    },
                ],
            },
        ],
    }
}

#[test]
fn save_then_load_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("record.json");
    let r = record();
    r.save(&path).unwrap();
    let loaded = Record::load(&path).unwrap();
    assert_eq!(loaded, r);
}

#[test]
fn load_names_the_failure() {
    let dir = tempfile::tempdir().unwrap();
    let missing = Record::load(&dir.path().join("nope.json")).unwrap_err();
    assert!(matches!(missing, RecordError::Read { .. }));
    assert!(missing.to_string().contains("read record"));

    let bad = dir.path().join("bad.json");
    std::fs::write(&bad, "not json").unwrap();
    let parse = Record::load(&bad).unwrap_err();
    assert!(matches!(parse, RecordError::Parse { .. }));
    assert!(parse.to_string().contains("parse record"));
}

#[test]
fn task_results_project_pass_fail_for_stats() {
    let results = record().task_results();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "a");
    assert_eq!(results[0].outcomes, vec![true, false]);
    assert_eq!(results[0].categories, vec!["early_termination".to_string()]);
    assert_eq!(results[1].outcomes, vec![false, false]);
}

#[test]
fn observed_sets_are_sorted_unions_over_disclosed_runs() {
    let r = record();
    assert_eq!(
        r.observed_models(),
        vec!["m1".to_string(), "m2".to_string()]
    );
    assert_eq!(
        r.observed_providers(),
        vec!["acme".to_string(), "other".to_string()]
    );
}
