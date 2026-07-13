//! Unit tests for the workspace physical model (ARCH §2.2–§2.3):
//! path/ref arithmetic, the clean-break layout guard, and
//! governing-config resolution over real git ancestry.

use super::fixture::{spawn_agent, workspace};
use super::*;
use crate::template::{GitRunner, RealGit};
use tempfile::TempDir;

fn git() -> RealGit {
    RealGit::new()
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
fn require_accepts_a_current_workspace() {
    let (_h, ws) = workspace();
    require(&ws).unwrap();
}

#[test]
fn require_refuses_the_retired_layout_with_an_actionable_error() {
    let holder = TempDir::new().unwrap();
    let old = holder.path().join("conv");
    std::fs::create_dir_all(old.join("root/.git")).unwrap();
    std::fs::write(old.join("providers.yaml"), "roles: {}\n").unwrap();
    let err = require(&old).unwrap_err();
    let msg = err.to_string();
    // The refusal names what was found and what the current layout is
    // (pre-v1 clean break, §10) — actionable, not just "no".
    assert!(matches!(err, LayoutError::OldLayout(_)), "{msg}");
    assert!(msg.contains("retired per-conversation layout"), "{msg}");
    assert!(msg.contains("repo.git"), "{msg}");
    assert!(msg.contains("lernie new"), "{msg}");
}

#[test]
fn require_refuses_a_non_workspace() {
    let holder = TempDir::new().unwrap();
    let err = require(holder.path()).unwrap_err();
    assert!(matches!(err, LayoutError::NotAWorkspace(_)));
    assert!(err.to_string().contains("lernie new"));
}

#[test]
fn config_head_resolves_and_is_loud_when_absent() {
    let (_h, ws) = workspace();
    let sha = config_head(&ws, DEFAULT_CONFIG_REF, &git()).unwrap();
    assert_eq!(sha.len(), 40);
    assert!(config_head(&ws, "config/nope", &git()).is_err());
}

#[test]
fn agent_ids_enumerates_the_agents_namespace_only() {
    let (_h, ws) = workspace();
    // No agents yet: empty, and config/default is never a candidate.
    assert!(agent_ids(&ws, &git()).unwrap().is_empty());
    spawn_agent(&ws, "20260101-r1", DEFAULT_CONFIG_REF);
    spawn_agent(&ws, "20260101-r1-20260102-c1", "agents/20260101-r1");
    let ids = agent_ids(&ws, &git()).unwrap();
    assert_eq!(ids, vec!["20260101-r1", "20260101-r1-20260102-c1"]);
}

#[test]
fn governing_config_is_the_fork_point_for_a_fresh_agent() {
    let (_h, ws) = workspace();
    spawn_agent(&ws, "20260101-r1", DEFAULT_CONFIG_REF);
    let gov = governing_config(&ws, "20260101-r1", &git()).unwrap();
    assert_eq!(gov, config_head(&ws, DEFAULT_CONFIG_REF, &git()).unwrap());
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
            DEFAULT_CONFIG_REF,
        ],
    )
    .unwrap();
    std::fs::write(strict_wt.join("extra.md"), "x").unwrap();
    g.run(&strict_wt, &["add", "extra.md"]).unwrap();
    g.run(&strict_wt, &["commit", "-m", "config: strict"])
        .unwrap();

    spawn_agent(&ws, "20260101-r1", "config/strict");
    let gov = governing_config(&ws, "20260101-r1", &git()).unwrap();
    assert_eq!(gov, config_head(&ws, "config/strict", &g).unwrap());
    // Symmetric enumeration order: an agent still on config/default
    // resolves to config/default's head even with config/strict extant
    // (exercises the keep-the-nearer arm in both directions).
    spawn_agent(&ws, "20260101-r2", DEFAULT_CONFIG_REF);
    let gov2 = governing_config(&ws, "20260101-r2", &git()).unwrap();
    assert_eq!(gov2, config_head(&ws, DEFAULT_CONFIG_REF, &g).unwrap());
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
            DEFAULT_CONFIG_REF,
        ],
    )
    .unwrap();
    std::fs::write(aaa_wt.join("aaa.md"), "x").unwrap();
    g.run(&aaa_wt, &["add", "aaa.md"]).unwrap();
    g.run(&aaa_wt, &["commit", "-m", "config: aaa"]).unwrap();
    spawn_agent(&ws, "20260101-r3", "config/aaa");
    let gov3 = governing_config(&ws, "20260101-r3", &git()).unwrap();
    assert_eq!(gov3, config_head(&ws, "config/aaa", &g).unwrap());
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

    spawn_agent(&ws, "20260101-r1", DEFAULT_CONFIG_REF);
    let gov = governing_config(&ws, "20260101-r1", &git()).unwrap();
    assert_eq!(gov, config_head(&ws, DEFAULT_CONFIG_REF, &g).unwrap());
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
    let err = governing_config(&ws, "20260101-x1", &git()).unwrap_err();
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

    spawn_agent(&ws, "20260101-r1", DEFAULT_CONFIG_REF);
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
    let err = governing_config(&ws, "20260101-r1", &git()).unwrap_err();
    assert!(err.to_string().contains("incomparable"), "{err}");
}

#[test]
fn show_control_reads_from_the_config_commit_tree() {
    let (_h, ws) = workspace();
    let sha = config_head(&ws, DEFAULT_CONFIG_REF, &git()).unwrap();
    let raw = show_control(&ws, &sha, "providers.yaml", &git()).unwrap();
    assert!(raw.contains("roles:"), "{raw}");
    assert!(show_control(&ws, &sha, "no-such-file", &git()).is_err());
}

#[test]
fn control_exists_answers_presence_in_the_tree() {
    let (_h, ws) = workspace();
    let sha = config_head(&ws, DEFAULT_CONFIG_REF, &git()).unwrap();
    assert!(control_exists(&ws, &sha, "souls/worker.md", &git()));
    assert!(!control_exists(&ws, &sha, "souls/nope.md", &git()));
}
