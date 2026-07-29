+++
title = "runaway recursive compaction dispatch: compactor branches re-trip every_n_commits, max_depth is unenforced at dispatch, and a conflicted compaction merge commits its markers"
created = 1785287936
updated = 1785287936
priority = 5
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["bug"]
+++
## Origin

Filed from **yog `bl-ebbd`** (yog's store: `bl -C ~/dev/yog show bl-ebbd`), an
operator-witnessed incident in a live yog workspace running lernie `=0.0.2`.
The layer at fault is lernie's automatic intermediate compaction (ARCH §2.6
compaction merge, §2.7 compaction, §6 `compaction.intermediate` trigger and
`budgets.max_depth`), not yog and not the model.

## Evidence (from bl-ebbd)

Workspace `~/.local/share/yog/workspaces/deceit-muscular/repo.git`, root
conversation `agents/20260728T080619Z-3b14deaf`. In ~90 seconds one root
conversation produced **226 `dispatch: compactor` commits and 227 branches**,
descending at least five generations:

    agents/20260728T080619Z-3b14deaf
      -20260728T080702Z-23b44ea9
        -20260728T080706Z-035fcd29
          -20260728T080712Z-07ee7ffc
            -20260728T080719Z-1102f2eb
              -20260728T080724Z-251df4d0

None was model-elected: the worker role has no `dispatch` tool. Every one is
the harness's own `worker_flush -> dispatch(compactor)` binding (§6).

The workspace's `config/default:workflow.yaml` declared
`compaction.intermediate: {trigger: every_n_commits, n: 20}` and
`budgets: {max_total_tokens: 2000000, max_wall_seconds: 3600, max_depth: 4}`.

The root branch's tip is a `compaction merge [...]` commit whose
`summary/001.md` contains **three fully unresolved 3-way git conflicts** (9
marker lines). `summary/**` is composed into every model call on that branch
(§5.2 manifest), so the next step feeds conflict markup to the model as if it
were a clean summary.

## Three defects

1. **Recursive eligibility.** `checkpoint::state` measures
   `commits_since_checkpoint` from the last compaction merge, else the branch
   *root* commit. A freshly forked child inherits the whole parent history, so
   a seconds-old compactor already reads >= 20 commits and trips
   `every_n_commits` immediately — dispatching a compactor off a compactor,
   generation after generation. Two things are wrong at once: the checkpoint
   clock is not scoped to the branch's own life, and a compactor is treated as
   a member of the compaction-eligible set at all.

2. **`max_depth` enforced at one call site only.** `budget::check` runs at the
   model-call boundary (`dispatch::run_exchange`, `advance::hop`). Nothing
   checks it at *dispatch* time, so the branches are created regardless and
   only decline once they try to step. Workflow-triggered dispatch
   (`worker_flush`, verifier gate/reject) and model-called dispatch must share
   one enforcement point.

3. **A conflicted compaction merge commits its markers.** `merge::merge` runs
   `git add -A` after `--no-ff --no-commit` to realize live-branch-wins for the
   one *expected* conflict class (work-product modify/delete). A genuine
   content collision — two concurrent compactors both writing `summary/001.md`
   — is also staged by that `add -A`, and `filter_to_product` excludes
   `summary/` from its restore pass, so the markered file commits. §2.6 already
   has the escape hatch (`refs/lernie/conflicted/<agent-id>`); compaction never
   used it.

## Fix

1. Scope the checkpoint clock to the branch's own founding commit, and state
   the invariant that a compactor is never compaction-eligible.
2. One dispatch-time budget gate inside `child_dispatch::run`, shared by every
   call site.
3. A compaction merge with a content conflict aborts, marks
   `refs/lernie/conflicted/<compactor-id>`, and lands nothing.

Tests for all three. yog pins `lernie = "=0.0.2"`, so consuming this needs a
release bump.