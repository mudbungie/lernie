//! Fork-point resolution (ARCH §2.3, §7.2): one ref out of the two
//! spellings a start may use, against a real scaffolded workspace.

use super::{Error, resolve};
use crate::template::{GitRunner, RealGit};
use crate::workspace::{self, fixture};

fn git() -> RealGit {
    RealGit::new()
}

#[test]
fn naming_nothing_is_the_default_lineage_not_a_special_case() {
    let (_h, ws) = fixture::workspace();
    assert_eq!(resolve(&ws, None, None, &git()).unwrap(), "config/default");
    // …which is exactly what naming it explicitly resolves to.
    assert_eq!(
        resolve(&ws, None, Some("default"), &git()).unwrap(),
        resolve(&ws, None, None, &git()).unwrap()
    );
}

#[test]
fn a_config_name_resolves_to_that_lineages_ref() {
    let (_h, ws) = fixture::workspace();
    let g = git();
    let strict = ws.join("strict");
    let strict_str = strict.to_string_lossy().to_string();
    g.run(
        &workspace::repo_git(&ws),
        &[
            "worktree",
            "add",
            "-b",
            "config/strict",
            strict_str.as_str(),
            "config/default",
        ],
    )
    .unwrap();
    std::fs::write(strict.join("extra.md"), "x").unwrap();
    g.run(&strict, &["add", "extra.md"]).unwrap();
    g.run(&strict, &["commit", "-m", "config: strict"]).unwrap();

    assert_eq!(
        resolve(&ws, None, Some("strict"), &g).unwrap(),
        "config/strict"
    );
}

#[test]
fn an_absent_lineage_is_declined_by_name_with_the_pool() {
    let (_h, ws) = fixture::workspace();
    let err = resolve(&ws, None, Some("strict"), &git()).unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, Error::UnknownLineage(_)), "{msg}");
    assert!(msg.contains("no config lineage \"strict\""), "{msg}");
    assert!(msg.contains("existing lineages: default"), "{msg}");
}

#[test]
fn any_ref_is_a_legal_fork_point_and_travels_verbatim() {
    let (_h, ws) = fixture::workspace();
    // A historical commit of an agent — §7.2 fork-from-history — is
    // taken as given: no prefix, no rewriting, no distinct operation.
    fixture::spawn_root(&ws, "20260101-r1");
    let tip = git()
        .run_capture(
            &workspace::repo_git(&ws),
            &["rev-parse", "--verify", "agents/20260101-r1"],
        )
        .unwrap();
    assert_eq!(resolve(&ws, Some(&tip), None, &git()).unwrap(), tip);
    assert_eq!(
        resolve(&ws, Some("agents/20260101-r1"), None, &git()).unwrap(),
        "agents/20260101-r1"
    );
}

#[test]
fn an_absent_ref_is_declined_before_anything_is_created() {
    let (_h, ws) = fixture::workspace();
    let err = resolve(&ws, Some("agents/nope"), None, &git()).unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, Error::UnknownRef(_)), "{msg}");
    assert!(msg.contains("no ref or commit \"agents/nope\""), "{msg}");
    assert!(
        msg.contains("a root agent forks off the ref you name"),
        "{msg}"
    );
    // Nothing was consulted beyond the ref: no branch, no worktree.
    assert!(!ws.join("agents").join("nope").exists());
}

#[test]
fn naming_both_spellings_is_declined_as_one_fork_point() {
    let (_h, ws) = fixture::workspace();
    let err = resolve(&ws, Some("config/default"), Some("default"), &git()).unwrap_err();
    assert!(matches!(err, Error::Conflict), "{err}");
    assert!(err.to_string().contains("not both"), "{err}");
}
