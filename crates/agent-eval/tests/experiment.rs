//! Coverage for experiment resolution (ARCH §9.3).

use agent_eval::experiment::{self, ExperimentError};

#[test]
fn resolves_present_experiment() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join("baseline");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workflow.yaml"), "events: {}\n").unwrap();

    let exp = experiment::resolve("baseline", d.path()).unwrap();
    assert_eq!(exp.name, "baseline");
    assert_eq!(exp.workflow, dir.join("workflow.yaml"));
}

#[test]
fn missing_experiment_errors() {
    let d = tempfile::tempdir().unwrap();
    let err = experiment::resolve("ghost", d.path()).unwrap_err();
    assert!(matches!(err, ExperimentError::Missing { .. }));
    assert!(err.to_string().contains("ghost"));
    assert!(err.to_string().contains("workflow.yaml"));
}

#[test]
fn resolves_the_shipped_baseline() {
    // The repo ships experiments/baseline/workflow.yaml (§9.3).
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../experiments");
    let exp = experiment::resolve("baseline", &root).unwrap();
    assert!(exp.workflow.is_file());
}
