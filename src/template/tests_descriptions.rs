//! Snapshot-time validation failures (bl-e3f5): a malformed pooled
//! `SKILL.md` frontmatter or tool schema must decline `scaffold` before
//! any commit lands or checkout is left behind — split out of `tests.rs`
//! to keep it under the 300-line cap.

use super::*;
use tempfile::TempDir;

#[test]
fn scaffold_surfaces_malformed_skill_frontmatter_before_any_commit() {
    // The YAML plain-scalar trap (bl-e3f5): a fenced-but-malformed
    // frontmatter body used to pass unparsed and only surface at the
    // agent's first prompt step. It must now decline here, before `new`
    // authors any commit or leaves a checkout behind.
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("ws");
    let data_root = holder.path().join("data");
    fs::create_dir_all(data_root.join("skills/trap")).unwrap();
    fs::write(
        data_root.join("skills/trap/SKILL.md"),
        "---\nname: trap\ndescription: posts to slack: general\n---\n",
    )
    .unwrap();
    let roots = crate::harness_root::Roots {
        config: holder.path().join("no-conf"),
        data: data_root,
    };
    let err = scaffold(&dest, &roots, &RealGit::new()).unwrap_err();
    match &err {
        ScaffoldError::Descriptions(descriptions::Error::SkillFrontmatter { name, .. }) => {
            assert_eq!(name, "trap");
        }
        other => panic!("expected Descriptions(SkillFrontmatter), got {other:?}"),
    }
    assert!(err.to_string().contains("SKILL.md"), "{err}");
    assert!(
        !dest.join(".config-author").exists(),
        "the transient checkout must not survive a decline"
    );
    // An orphan branch gets a ref only once it has a commit — none landed.
    let refs = RealGit::new()
        .run_capture(
            &dest.join("repo.git"),
            &["for-each-ref", "--format=%(refname)"],
        )
        .unwrap();
    assert_eq!(refs, "", "no ref should exist without a commit");
}

#[test]
fn scaffold_surfaces_malformed_tool_schema_before_any_commit() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("ws");
    let data_root = holder.path().join("data");
    fs::create_dir_all(data_root.join("tools")).unwrap();
    fs::write(data_root.join("tools/broken.json"), "{ not json").unwrap();
    let roots = crate::harness_root::Roots {
        config: holder.path().join("no-conf"),
        data: data_root,
    };
    let err = scaffold(&dest, &roots, &RealGit::new()).unwrap_err();
    assert!(
        matches!(
            err,
            ScaffoldError::Descriptions(descriptions::Error::ToolSchema { .. })
        ),
        "got {err:?}"
    );
    assert!(!dest.join(".config-author").exists());
}
