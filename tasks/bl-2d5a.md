+++
title = "wire compaction bindings live: dispatch(compactor) on checkpoint + compaction_merge on return, role-aware advance"
created = 1784077357
updated = 1784077357
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]
+++
## Scope
bl-9dbd landed the compaction MECHANISM (src/prompt/compactor/: merge, checkpoint, tools) + the compactor built-in tools + deleted the terminal-compaction stage, but the LIVE step-boundary activation is deferred, coordinated with bl-6a3b (workflow interpreter). Wire it:

1. **compaction_merge action** — bl-6a3b's workflow_actions interpreter parses `compactor_return: compaction_merge` but declines CompactionMerge as ActionUnsupported. Wire it to `compactor::merge(parent_worktree, compactor_id, git)` at the compactor's return. Needs a `compactor_return` event + the returning compactor's id passed to the action (the current run_terminal_bindings only handles the agent's OWN epitaph/branch).

2. **checkpoint dispatch** — at each step boundary the executor should call `compactor::checkpoint::state()` + `due(compaction_config, state)` and, on fire, dispatch a compactor child off the tip C (child_dispatch role=compactor). This is the `worker_flush: dispatch(compactor)` binding + the config-clock `compaction:` block. bl-6a3b's interpreter declines Dispatch as ActionUnsupported.

3. **role-aware advance resolution** — resolve::resolve_worker hardcodes WORKER_ROLE. A dispatched compactor child, when driven by `lernie advance`, must resolve its compactor soul (souls/compactor.md) + the INJECTED built-in toolset (write_summary/mark_for_deletion), not the worker default. Derive the child's role on-disk (single source of truth — the pinned soul.md, or a recorded per-agent role) and inject the compactor toolset schemas into the model request for the compactor role only.

## Already shipped (bl-9dbd), do not redo
- compactor::merge (live-branch-wins + overlap test), compactor::checkpoint (due + state), compactor::tools + tool/builtin/compaction (write_summary/mark_for_deletion built-ins), child_dispatch role param, dispatch_cli compactor->child, terminal stage + Dispatcher deleted. Template workflow.yaml has the bindings + compaction: block.

## Blocked on
Coordinate with bl-6a3b's interpreter surface (Action executors, event model).