//! Unit tests for the workspace physical model (ARCH §2.2–§2.3):
//! path/ref arithmetic, the clean-break layout guard, and
//! governing-config resolution over real git ancestry.

use super::fixture::{spawn_agent, workspace};
use super::*;
use crate::template::{GitRunner, RealGit};

pub(super) fn git() -> RealGit {
    RealGit::new()
}

/// `config/default` — the ref the fixtures fork off (§2.3).
pub(super) fn default_ref() -> String {
    config_ref(DEFAULT_CONFIG_NAME)
}

/// A revision's commit sha — the fact the ancestry derivation must
/// agree with.
pub(super) fn head(ws: &std::path::Path, rev: &str) -> String {
    git()
        .run_capture(&repo_git(ws), &["rev-parse", "--verify", rev])
        .unwrap()
}

#[test]
fn paths_and_refs_derive_from_the_id() {
    let ws = std::path::Path::new("/w");
    assert_eq!(repo_git(ws), std::path::Path::new("/w/repo.git"));
    assert_eq!(
        agent_worktree(ws, "a-b"),
        std::path::Path::new("/w/agents/a-b")
    );
    assert_eq!(agent_ref("a-b"), "agents/a-b");
    assert_eq!(config_ref("strict-verifier"), "config/strict-verifier");
}

#[test]
fn agent_ids_enumerates_the_agents_namespace_only() {
    let (_h, ws) = workspace();
    // No agents yet: empty, and config/default is never a candidate.
    assert!(agent_ids(&ws, &git()).unwrap().is_empty());
    spawn_agent(&ws, "20260101-r1", &default_ref());
    spawn_agent(&ws, "20260101-r1-20260102-c1", "agents/20260101-r1");
    let ids = agent_ids(&ws, &git()).unwrap();
    assert_eq!(ids, vec!["20260101-r1", "20260101-r1-20260102-c1"]);
}

#[test]
fn governing_config_is_the_fork_point_for_a_fresh_agent() {
    let (_h, ws) = workspace();
    spawn_agent(&ws, "20260101-r1", &default_ref());
    let gov = governing_config(&ws, &agent_ref("20260101-r1"), &git()).unwrap();
    assert_eq!(gov, head(&ws, &default_ref()));
}

#[test]
fn governing_config_picks_the_nearest_config_ancestor() {
    let (_h, ws) = workspace();
    let g = git();
    // Advance a second config branch from config/default's head, then
    // fork the agent off it: both config heads yield candidates, and
    // the descendant (config/strict's head) is the governing commit.
    let strict_wt = ws.join("strict");
    let strict_str = strict_wt.to_string_lossy().to_string();
    g.run(
        &repo_git(&ws),
        &[
            "worktree",
            "add",
            "-b",
            "config/strict",
            strict_str.as_str(),
            &default_ref(),
        ],
    )
    .unwrap();
    std::fs::write(strict_wt.join("extra.md"), "x").unwrap();
    g.run(&strict_wt, &["add", "extra.md"]).unwrap();
    g.run(&strict_wt, &["commit", "-m", "config: strict"])
        .unwrap();

    spawn_agent(&ws, "20260101-r1", "config/strict");
    let gov = governing_config(&ws, &agent_ref("20260101-r1"), &git()).unwrap();
    assert_eq!(gov, head(&ws, "config/strict"));
    // Symmetric enumeration order: an agent still on config/default
    // resolves to config/default's head even with config/strict extant
    // (exercises the keep-the-nearer arm in both directions).
    spawn_agent(&ws, "20260101-r2", &default_ref());
    let gov2 = governing_config(&ws, &agent_ref("20260101-r2"), &git()).unwrap();
    assert_eq!(gov2, head(&ws, &default_ref()));
    // Reverse enumeration order: a config branch that sorts *before*
    // config/default (`config/aaa`) yields the nearer candidate first,
    // so the later, farther candidate must lose (the keep-`a` arm).
    let aaa_wt = ws.join("aaa");
    let aaa_str = aaa_wt.to_string_lossy().to_string();
    g.run(
        &repo_git(&ws),
        &[
            "worktree",
            "add",
            "-b",
            "config/aaa",
            aaa_str.as_str(),
            &default_ref(),
        ],
    )
    .unwrap();
    std::fs::write(aaa_wt.join("aaa.md"), "x").unwrap();
    g.run(&aaa_wt, &["add", "aaa.md"]).unwrap();
    g.run(&aaa_wt, &["commit", "-m", "config: aaa"]).unwrap();
    spawn_agent(&ws, "20260101-r3", "config/aaa");
    let gov3 = governing_config(&ws, &agent_ref("20260101-r3"), &git()).unwrap();
    assert_eq!(gov3, head(&ws, "config/aaa"));
}

#[test]
fn config_lineage_names_the_ref_the_merge_base_is_taken_against() {
    let (_h, ws) = workspace();
    let g = git();
    spawn_agent(&ws, "20260101-r1", &default_ref());
    // Advance config/default past the fork (a later user config edit,
    // §2.3): its head stops being an ancestor of the agent, but it is
    // still the ref the governing commit is derived *against* — so it
    // is the ref an archive must carry (§9.2).
    let later = ws.join("later");
    let later_str = later.to_string_lossy().to_string();
    g.run(
        &repo_git(&ws),
        &["worktree", "add", later_str.as_str(), &default_ref()],
    )
    .unwrap();
    std::fs::write(later.join("later.md"), "x").unwrap();
    g.run(&later, &["add", "later.md"]).unwrap();
    g.run(&later, &["commit", "-m", "config: later"]).unwrap();

    let lineage = config_lineage(&ws, &agent_ref("20260101-r1"), &g).unwrap();
    assert_eq!(lineage.len(), 1);
    assert_eq!(lineage[0].0, "refs/heads/config/default");
    assert_eq!(
        lineage[0].1,
        governing_config(&ws, &agent_ref("20260101-r1"), &g).unwrap()
    );
    assert_ne!(
        lineage[0].1,
        head(&ws, &default_ref()),
        "the head advanced past the governing commit"
    );
}

#[test]
fn governing_config_skips_unrelated_config_lineages() {
    let (_h, ws) = workspace();
    let g = git();
    // An orphan config lineage sharing no ancestor with the agent
    // contributes no candidate (merge-base fails) and is skipped.
    let orphan_wt = ws.join("orphan");
    let orphan_str = orphan_wt.to_string_lossy().to_string();
    g.run(
        &repo_git(&ws),
        &[
            "worktree",
            "add",
            "--orphan",
            "-b",
            "config/island",
            orphan_str.as_str(),
        ],
    )
    .unwrap();
    std::fs::write(orphan_wt.join("v"), "1").unwrap();
    g.run(&orphan_wt, &["add", "v"]).unwrap();
    g.run(&orphan_wt, &["commit", "-m", "config: island"])
        .unwrap();

    spawn_agent(&ws, "20260101-r1", &default_ref());
    let gov = governing_config(&ws, &agent_ref("20260101-r1"), &git()).unwrap();
    assert_eq!(gov, head(&ws, &default_ref()));
}

#[test]
fn governing_config_declines_an_agent_with_no_config_ancestor() {
    let (_h, ws) = workspace();
    let g = git();
    // An orphan agent branch (nothing forks it off a config commit).
    let wt = agent_worktree(&ws, "20260101-x1");
    let wt_str = wt.to_string_lossy().to_string();
    g.run(
        &repo_git(&ws),
        &[
            "worktree",
            "add",
            "--orphan",
            "-b",
            "agents/20260101-x1",
            wt_str.as_str(),
        ],
    )
    .unwrap();
    std::fs::write(wt.join("goal.md"), "g").unwrap();
    g.run(&wt, &["add", "goal.md"]).unwrap();
    g.run(&wt, &["commit", "-m", "orphan"]).unwrap();
    let err = governing_config(&ws, &agent_ref("20260101-x1"), &git()).unwrap_err();
    assert!(err.to_string().contains("no config/* ancestor"));
}

#[test]
fn governing_config_declines_incomparable_candidates() {
    let (_h, ws) = workspace();
    let g = git();
    // A second, unrelated (orphan) config lineage merged into the
    // agent's branch makes both config heads incomparable ancestors of
    // the tip — ambiguous, declined loudly (§2.2, PRINCIPLES).
    let orphan_wt = ws.join("orphan");
    let orphan_str = orphan_wt.to_string_lossy().to_string();
    g.run(
        &repo_git(&ws),
        &[
            "worktree",
            "add",
            "--orphan",
            "-b",
            "config/island",
            orphan_str.as_str(),
        ],
    )
    .unwrap();
    std::fs::write(orphan_wt.join("island.md"), "1").unwrap();
    g.run(&orphan_wt, &["add", "island.md"]).unwrap();
    g.run(&orphan_wt, &["commit", "-m", "config: island"])
        .unwrap();

    spawn_agent(&ws, "20260101-r1", &default_ref());
    let wt = agent_worktree(&ws, "20260101-r1");
    g.run(
        &wt,
        &[
            "merge",
            "--allow-unrelated-histories",
            "-m",
            "cross",
            "config/island",
        ],
    )
    .unwrap();
    let err = governing_config(&ws, &agent_ref("20260101-r1"), &git()).unwrap_err();
    assert!(err.to_string().contains("incomparable"), "{err}");
}

#[test]
fn show_control_reads_from_the_config_commit_tree() {
    let (_h, ws) = workspace();
    let sha = head(&ws, &default_ref());
    let raw = show_control(&ws, &sha, "providers.yaml", &git()).unwrap();
    assert!(raw.contains("roles:"), "{raw}");
    assert!(show_control(&ws, &sha, "no-such-file", &git()).is_err());
}

#[test]
fn control_exists_answers_presence_in_the_tree() {
    let (_h, ws) = workspace();
    let sha = head(&ws, &default_ref());
    assert!(control_exists(&ws, &sha, "souls/worker.md", &git()));
    assert!(!control_exists(&ws, &sha, "souls/nope.md", &git()));
}
