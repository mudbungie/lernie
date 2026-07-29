//! The child's `tools:` grant at dispatch: read from the governing
//! config commit, and the descriptor prune it drives (ARCH §3.3, §4.3,
//! §5.1, §2.3 step 2).
//!
//! Split from `tests.rs` so both stay under the 300-line repo cap; the
//! launcher stub and the request builder are shared from there.

use super::tests::{RecordingLauncher, req};
use super::*;
use crate::workspace::fixture;

#[test]
fn missing_providers_is_surfaced_as_control_read_before_any_spawn() {
    // The grant is read from the same governing config commit as the
    // soul (§4.3), so a config carrying no `providers.yaml` fails the
    // same way and just as early — before any fork side effect.
    let (_h, ws) = fixture::workspace();
    let g = crate::template::RealGit::new();
    let author = ws.join(".strip");
    let author_str = author.to_string_lossy().to_string();
    g.run(
        &workspace::repo_git(&ws),
        &["worktree", "add", author_str.as_str(), "config/default"],
    )
    .unwrap();
    g.run(&author, &["rm", "-q", "providers.yaml"]).unwrap();
    g.run(&author, &["commit", "-m", "config: no providers"])
        .unwrap();
    g.run(
        &workspace::repo_git(&ws),
        &["worktree", "remove", "--force", author_str.as_str()],
    )
    .unwrap();
    let parent_wt = fixture::spawn_root(&ws, "20260101-p7");
    let launcher = RecordingLauncher::ok();
    let err = run(
        &req(&ws, "20260101-p7", &parent_wt, "g"),
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &launcher,
    )
    .unwrap_err();
    assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
    assert_eq!(workspace::agent_ids(&ws, &g).unwrap().len(), 1);
    assert!(launcher.invocations.borrow().is_empty());
}

#[test]
fn the_childs_tree_carries_only_the_descriptors_its_role_grants() {
    // The whole of yog bl-55b1, end to end over real git: a config
    // commit carrying the full descriptions snapshot, a `worker` grant
    // naming one of its two tools, and a child whose own tree is left
    // agreeing with the wire array it will send (§3.3, §5.1).
    let (_h, ws) = fixture::workspace();
    fixture::amend_config(
        &ws,
        &[
            ("souls/worker.md", "worker soul body\n"),
            (
                "providers.yaml",
                "roles:\n  worker:\n    provider: anthropic\n    \
                 model: claude-sonnet-5\n    tools: [bash]\n",
            ),
            ("descriptions/tools/bash.json", "{}\n"),
            (
                "descriptions/skills/bash.md",
                "name: bash\ndescription: d\n",
            ),
            ("descriptions/tools/message.json", "{}\n"),
            (
                "descriptions/skills/message.md",
                "name: message\ndescription: d\n",
            ),
            (
                "descriptions/skills/notes.md",
                "name: notes\ndescription: d\n",
            ),
        ],
    );
    let g = crate::template::RealGit::new();
    let parent_wt = fixture::spawn_root(&ws, "20260101-p8");
    let launcher = RecordingLauncher::ok();
    let child = run(
        &req(&ws, "20260101-p8", &parent_wt, "g"),
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &launcher,
    )
    .unwrap();

    let listing = g
        .run_capture(
            &workspace::repo_git(&ws),
            &[
                "ls-tree",
                "-r",
                "--name-only",
                &workspace::agent_ref(&child),
            ],
        )
        .unwrap();
    let has = |p: &str| listing.lines().any(|l| l == p);

    // Granted: schema and its claimed skill both survive.
    assert!(has("descriptions/tools/bash.json"), "{listing}");
    assert!(has("descriptions/skills/bash.md"), "{listing}");
    // Ungranted: the pair is gone — nothing on this branch documents a
    // tool the child cannot call.
    assert!(!has("descriptions/tools/message.json"), "{listing}");
    assert!(!has("descriptions/skills/message.md"), "{listing}");
    // A skill no tool claims is granted by being present (§3.3).
    assert!(has("descriptions/skills/notes.md"), "{listing}");
    // And the grant's own home is still not on the branch (§2.2).
    assert!(!has("providers.yaml"), "{listing}");
}
