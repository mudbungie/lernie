---
name: dispatch
description: Spawn a subagent in the background to work toward a goal you supply. Returns a handle immediately; the subagent runs on its own branch and its terminal compacted result arrives on a later step as a message in your inbox. Use for delegating a discrete piece of work — a focused investigation, a parallel exploration, a parsing pass — when the parent agent shouldn't (or doesn't need to) do the work inline.
---

# dispatch

Dispatches a subagent conversation off the current step, on a fresh
branch off this conversation's tip. The subagent receives the goal
verbatim and runs against the role's soul; the parent agent gets back
a handle and continues. Parallelism is expressed by issuing several
dispatches in one step; each subagent's result lands as its own
message on a later step as it completes (ARCH §2.5, §2.11).

## Input

```json
{ "role": "<role-name>", "goal": "<goal text>" }
```

- `role` — the subagent role. v0.4 Phase 2 supports `worker`; the role
  must be defined in this repo's `providers.yaml` (`roles:` block) and
  have a soul at `souls/<role>.md`.
- `goal` — the per-call goal text. Pinned at the head of every model
  call on the subagent's branch and not rewritten during execution.

## Output

A JSON object on `tool_result.content`:

```json
{ "status": "in_progress", "handle": "<sub-branch>" }
```

`handle` is the subagent's full hyphenated descent branch
(`<this-conv>-<sub-id>`) — the subagent's address, which also shows up
in the git tree on the UI. The dispatch is fire-and-forget from this
step's perspective; the terminal compacted result arrives on a later
step as a message from the subagent in your inbox (ARCH §2.11).

## When to use

- A subtask is large enough that finishing it inline would crowd this
  conversation's context — push it to a subagent, get back a focused
  summary later.
- Several independent investigations can run in parallel — issue N
  dispatches in one step; each result lands as its own message as it
  completes.
- A goal needs a different toolset or model than this role has — pick
  the role that fits and dispatch there.

## When not to use

- A short read or computation that fits inline — `read_file` and
  `bash` are cheaper than a whole subagent branch.
- A goal that depends on this conversation's in-flight reasoning —
  the subagent only sees the goal text and its branch's tree, not
  this branch's prior steps.

## Notes

- The handle is the subagent's branch name and its address; its
  result returns to you as a message on a later step (ARCH §2.11).
- The harness writes the goal to `goal.md` and the role's soul to
  `soul.md` on the subagent's first commit; the subagent's worktree
  is at `<conv-repo>/<handle>/`.
- Stopping a subagent is a separate user action through the UI;
  there is no `stop_dispatch` tool in v0.4.
