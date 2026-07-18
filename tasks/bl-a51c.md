+++
title = "epic: workspace substrate — config branches, agents/* refs, no main (§2.2–§2.3 physical)"
created = 1783917121
updated = 1784337562
claimant = "Sorehead-a51c"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]

[[blockers]]
id = "bl-6572"
on = "close"

[[blockers]]
id = "bl-eaed"
on = "close"

[[blockers]]
id = "bl-a3c1"
on = "close"
+++
## Status: DELIVERED (squash 0d5aa2d, 2026-07-13) + verified 2026-07-17
The workspace-substrate reshape landed on main four days ago; the store's archive step was lost to a concurrent-write race, stranding this task open under a stale claim. This close-out re-verified every must-pin against the SHIPPED code (not docs/commit messages) and filled one consistency gap.

## Verified must-pins (code + test)
1. One repo per workspace at <data-root>/workspaces/<workspace>/repo.git (bare): workspace::REPO_DIR + repo_git(); template::scaffold does 'git init --bare -b config/default'; Makefile DATA_DIRS has workspaces/.
2. Config branches config/<name>, NO main: workspace::CONFIG_REF_PREFIX; scaffold inits -b config/default; ARCH §2.2 'there is no main'.
3. Agent refs agents/<agent-id>, worktrees siblings under agents/: AGENT_REF_PREFIX, AGENTS_DIR, agent_worktree().
4. Control read from governing config commit via ancestry, never stored: workspace::governing_config() (merge-base against config/* heads, nearest descendant kept; incomparable candidates declined); show_control()/control_exists() read the commit tree, never the worktree.
5. Dispatch commit removes control files from agent tree: dispatch/step_commit.rs::remove_control_files ('git rm -r -q --ignore-unmatch' over workspace::CONTROL_PATHS), called by commit_dispatch.
6. lernie new = workspace creation + first config-commit (orphan root), descriptions/** snapshot: template::scaffold (orphan config/default root, TEMPLATE.extract, descriptions::snapshot); later commits via template::authoring::author.
7. Ref-prefix seam flipped: workspace::agent_ids() enumerates 'agents/*' via for-each-ref; consumed by inbox/scan/derive.rs and stop/inspector.rs — no 'every branch except main'.
8. LERNIE_CONV_REPO/BRANCH semantics unchanged: still the env names across tool/builtin/* and inbox.
9. Root id unique per workspace: dispatch/mod.rs run_exchange conv_id = '{ts}-{short_id}' (compact timestamp + nano id).
10. Old layout refused not migrated: workspace::require() (OldLayout / NotAWorkspace, actionable errors naming what was found + 'lernie new'); guarded in prompt::run, stop, inbox scan, dispatch_cli, advance/cli, template/authoring, and (gap filled this pass) archive::bundle.

## Gap filled this close-out
archive::bundle lacked the §10 layout guard that every other verb has; added workspace::require(ws)? + ArchiveError::Layout(#[from] LayoutError) with two tests (retired-layout + non-workspace). Split src/archive/tests.rs -> tests/{mod,bundle,replay}.rs to stay under the 300-line cap. Stabilized the cascade drain test (generous deadline, matching siblings) against a coverage-runner race.

## Docs verified truthful
ARCH §2.2-§2.3 describe the shipped mechanism (repo.git bare, config/*, agents/*, no main, governing config commit, dispatch commit removes control, fork-is-freeze). §9.2 shipped-state note credits bl-a51c for re-basing archive. TAXONOMY defines 'workspace' and 'governing config commit'. README shows 'lernie new ... (bare repo.git + config/default)'.

## make check: green — fmt-check, clippy -D warnings, 100% coverage (3978/3978).