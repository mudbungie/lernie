//! The child's `tools:` grant at dispatch: read from the governing
//! config commit, and the descriptor prune it drives (ARCH §3.3, §4.3,
//! §5.1, §2.3 step 2).
//!
//! Split from `tests.rs` so both stay under the 300-line repo cap; the
//! launcher stub and the request builder are shared from there.

use super::tests::{RecordingLauncher, req, test_rng};
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
        &test_rng(),
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
        &test_rng(),
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

/// A config declaring two roles with disjoint grants, plus descriptors
/// for both tools and one standalone skill no tool claims. `sensor`'s
/// grant is deliberately *not* a subset of `worker`'s.
fn disjoint_roles(ws: &Path) {
    fixture::amend_config(
        ws,
        &[
            ("souls/worker.md", "worker soul body\n"),
            ("souls/sensor.md", "sensor soul body\n"),
            (
                "providers.yaml",
                "roles:\n  worker:\n    provider: anthropic\n    \
                 model: claude-sonnet-5\n    tools: [bash]\n  sensor:\n    \
                 provider: anthropic\n    model: claude-sonnet-5\n    \
                 tools: [message]\n",
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
}

/// Dispatch `role` off `parent`'s tip, returning the child's id.
pub(super) fn dispatch_role(ws: &Path, parent: &str, role: &str) -> String {
    let parent_wt = workspace::agent_worktree(ws, parent);
    run(
        &ChildDispatchRequest {
            repo: ws,
            parent_branch: parent,
            parent_worktree: &parent_wt,
            role,
            goal: "g",
            name: None,
            fork_point: None,
            cwd: None,
            pins: crate::prompt::PinnedDocs::none(),
        },
        &crate::template::RealGit::new(),
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &RecordingLauncher::ok(),
        &test_rng(),
    )
    .unwrap()
}

/// The paths an agent branch's tree carries.
pub(super) fn tree(ws: &Path, agent: &str) -> String {
    crate::template::RealGit::new()
        .run_capture(
            &workspace::repo_git(ws),
            &["ls-tree", "-r", "--name-only", &workspace::agent_ref(agent)],
        )
        .unwrap()
}

#[test]
fn a_child_grant_its_dispatcher_lacks_is_not_capped_by_the_dispatch_chain() {
    // bl-a900, end to end over real git: the descriptor tree is derived
    // from the *governing config commit* filtered to the child's own
    // grant, so a `sensor` dispatched by a `worker` whose grant lacks
    // `message` still carries `message`'s descriptors. Pruning the
    // parent's already-pruned tree instead made every child's descriptors
    // the dispatch chain's intersection, and the tools-list assembly
    // (§3.3) then dropped the granted-but-undescribed tool silently.
    let (_h, ws) = fixture::workspace();
    disjoint_roles(&ws);
    fixture::spawn_root(&ws, "20260101-p9");
    let worker = dispatch_role(&ws, "20260101-p9", "worker");
    // The dispatcher's own tree is cut to *its* grant …
    let dispatcher = tree(&ws, &worker);
    assert!(
        dispatcher
            .lines()
            .any(|l| l == "descriptions/tools/bash.json")
    );
    assert!(
        !dispatcher
            .lines()
            .any(|l| l == "descriptions/tools/message.json")
    );

    // … and the sensor it dispatches carries its own, whole.
    let sensor = dispatch_role(&ws, &worker, "sensor");
    let listing = tree(&ws, &sensor);
    let has = |p: &str| listing.lines().any(|l| l == p);
    assert!(has("descriptions/tools/message.json"), "{listing}");
    assert!(has("descriptions/skills/message.md"), "{listing}");
    // Its dispatcher's tool is not inherited: the derivation is a
    // function of the child's grant alone, in both directions.
    assert!(!has("descriptions/tools/bash.json"), "{listing}");
    assert!(!has("descriptions/skills/bash.md"), "{listing}");
    // A skill no tool claims is granted by being present (§3.3).
    assert!(has("descriptions/skills/notes.md"), "{listing}");
}

#[test]
fn a_granted_tool_the_config_describes_nowhere_declines_before_the_fork() {
    // The diagnostic half: `providers.yaml` and `descriptions/**` live in
    // one commit and disagree, so the dispatch is refused naming the tool
    // and the described pool — never composed into a quietly smaller
    // toolset. Refused before the fork, so no branch, worktree or inbox
    // is left behind (the shape of role validation and the §6 budget gate).
    let (_h, ws) = fixture::workspace();
    fixture::amend_config(
        &ws,
        &[
            ("souls/sensor.md", "sensor soul body\n"),
            (
                "providers.yaml",
                "roles:\n  sensor:\n    provider: anthropic\n    \
                 model: claude-sonnet-5\n    tools: [message, slack_read]\n",
            ),
            ("descriptions/tools/message.json", "{}\n"),
        ],
    );
    let parent_wt = fixture::spawn_root(&ws, "20260101-pa");
    let g = crate::template::RealGit::new();
    let launcher = RecordingLauncher::ok();
    let err = run(
        &ChildDispatchRequest {
            repo: &ws,
            parent_branch: "20260101-pa",
            parent_worktree: &parent_wt,
            role: "sensor",
            goal: "g",
            name: None,
            fork_point: None,
            cwd: None,
            pins: crate::prompt::PinnedDocs::none(),
        },
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &launcher,
        &test_rng(),
    )
    .unwrap_err();

    match &err {
        Error::GrantUndescribed(u) => {
            assert_eq!((u.role.as_str(), u.tool.as_str()), ("sensor", "slack_read"));
            assert!(u.described.contains("message"), "{}", u.described);
        }
        other => panic!("got {other:?}"),
    }
    // No branch debris: the root is still the only agent, and nothing
    // was launched.
    assert_eq!(workspace::agent_ids(&ws, &g).unwrap(), ["20260101-pa"]);
    assert!(launcher.invocations.borrow().is_empty());
}
