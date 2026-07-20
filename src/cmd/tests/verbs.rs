//! Authoring and driver verbs driven against a constructed
//! [`Fx`](crate::cmd::Fx): `new`, `config`, `prompt`, `dispatch`,
//! `stop`, `message`. Each has a hermetic success path where one exists,
//! plus a cheap early-error path pinning the one-conversion failure shape
//! (`lernie <prefix>: …`). Detached launches use `"true"` as the driver
//! target (spawned, harmless).

use super::{assert_prefixed, noop_editor, with_fx, writing_editor};
use crate::cmd::{Outcome, config, dispatch, message, new, prompt, stop};
use crate::workspace::fixture;
use tempfile::TempDir;

#[test]
fn new_scaffolds_and_prints_the_destination() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("ws");
    let (r, ..) = with_fx("lernie", b"", &noop_editor, |fx| {
        new::run(
            new::Args {
                path: Some(dest.clone()),
            },
            fx,
        )
    });
    let Outcome::Line(line) = r.unwrap() else {
        panic!("new prints its destination")
    };
    assert_eq!(line, dest.display().to_string());
    assert!(dest.join("repo.git").is_dir());
}

#[test]
fn new_reports_a_scaffold_failure() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("ws");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::write(dest.join("occupied"), b"x").unwrap();
    let (r, ..) = with_fx("lernie", b"", &noop_editor, |fx| {
        new::run(new::Args { path: Some(dest) }, fx)
    });
    assert_prefixed(r.unwrap_err(), "new");
}

#[test]
fn config_authors_a_commit() {
    let (_h, ws) = fixture::workspace();
    let (r, ..) = with_fx("lernie", b"", &writing_editor, |fx| {
        config::run(
            config::Args {
                workspace: ws.clone(),
                name: None,
                from: None,
                orphan: false,
            },
            fx,
        )
    });
    assert!(matches!(r.unwrap(), Outcome::Quiet));
}

#[test]
fn config_reports_a_non_workspace() {
    let tmp = TempDir::new().unwrap();
    let (r, ..) = with_fx("lernie", b"", &noop_editor, |fx| {
        config::run(
            config::Args {
                workspace: tmp.path().to_path_buf(),
                name: None,
                from: None,
                orphan: false,
            },
            fx,
        )
    });
    assert_prefixed(r.unwrap_err(), "config");
}

#[test]
fn prompt_reports_a_non_workspace() {
    let tmp = TempDir::new().unwrap();
    let (r, ..) = with_fx("lernie", b"", &noop_editor, |fx| {
        prompt::run(
            prompt::Args {
                repo: tmp.path().to_path_buf(),
                message: "hi".into(),
            },
            fx,
        )
    });
    assert_prefixed(r.unwrap_err(), "prompt");
}

#[test]
fn dispatch_forks_a_child_through_the_front_door() {
    let (_h, ws) = fixture::workspace();
    fixture::spawn_root(&ws, "20260101-p1");
    let (r, ..) = with_fx("true", b"", &noop_editor, |fx| {
        dispatch::run(
            dispatch::Args {
                role: "worker".into(),
                repo: ws.clone(),
                branch: "20260101-p1".into(),
                goal: Some("do the thing".into()),
            },
            fx,
        )
    });
    assert!(matches!(r.unwrap(), Outcome::Quiet));
}

#[test]
fn dispatch_reports_an_undefined_role_with_its_prefix() {
    let (_h, ws) = fixture::workspace();
    fixture::spawn_root(&ws, "p1");
    let (r, ..) = with_fx("true", b"", &noop_editor, |fx| {
        dispatch::run(
            dispatch::Args {
                role: "no-such".into(),
                repo: ws.clone(),
                branch: "p1".into(),
                goal: Some("g".into()),
            },
            fx,
        )
    });
    assert_prefixed(r.unwrap_err(), "dispatch no-such");
}

#[test]
fn stop_is_idempotent_with_no_executor() {
    let (_h, ws) = fixture::workspace();
    fixture::spawn_root(&ws, "20260101-a1");
    let (r, ..) = with_fx("lernie", b"", &noop_editor, |fx| {
        stop::run(
            stop::Args {
                repo: ws.clone(),
                branch: "20260101-a1".into(),
                stop_children: false,
            },
            fx,
        )
    });
    assert!(matches!(r.unwrap(), Outcome::Quiet));
}

#[test]
fn stop_reports_a_non_workspace() {
    let tmp = TempDir::new().unwrap();
    let (r, ..) = with_fx("lernie", b"", &noop_editor, |fx| {
        stop::run(
            stop::Args {
                repo: tmp.path().to_path_buf(),
                branch: "b".into(),
                stop_children: false,
            },
            fx,
        )
    });
    assert_prefixed(r.unwrap_err(), "stop");
}

#[test]
fn message_deposits_and_probes() {
    let (_h, ws) = fixture::workspace();
    let (r, ..) = with_fx("true", b"", &noop_editor, |fx| {
        message::run(
            message::Args {
                workspace: ws.clone(),
                agent: "20260101-a1".into(),
                content: "hi".into(),
            },
            fx,
        )
    });
    assert!(matches!(r.unwrap(), Outcome::Quiet));
}

#[test]
fn message_reports_a_non_workspace() {
    let tmp = TempDir::new().unwrap();
    let (r, ..) = with_fx("true", b"", &noop_editor, |fx| {
        message::run(
            message::Args {
                workspace: tmp.path().to_path_buf(),
                agent: "a".into(),
                content: "c".into(),
            },
            fx,
        )
    });
    assert_prefixed(r.unwrap_err(), "message");
}
