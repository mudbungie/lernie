---
name: await
description: Resolve a `dispatch` handle. Blocks until the named subagent reaches a terminal state (merged, stopped, or conflicted) and returns the outcome as `tool_result`. Pair with `dispatch` to retrieve a subagent's terminal compacted summary on a later step.
---

# await

Block until the subagent identified by `handle` reaches a terminal
state, then return the outcome as the `tool_result.content` payload.
The handle is the value `dispatch` returned in its earlier
`tool_result` — i.e. the subagent's branch name (ARCH §2.5 "Async work
uses handles").

## Input

```json
{ "handle": "<sub-conv-id>" }
```

- `handle` — a value previously returned by `dispatch` on this
  conversation. Must be a descendant of this conversation
  (`<this-conv>-<sub-id>`, ARCH §2.3). Awaiting an unrelated branch is
  rejected.

## Output

A JSON object on `tool_result.content`. Exactly one of:

```json
{ "status": "merged",     "summary": "<terminal compacted text>" }
{ "status": "stopped" }
{ "status": "conflicted" }
```

- `merged` — the subagent ran its loop and merged back into this
  conversation (ARCH §2.6). `summary` is the latest
  `summary/<NNN>.md` on the subagent's branch (its terminal
  compactor's output, ARCH §2.7).
- `stopped` — the subagent terminated without merging — its latest
  step's `response.json` ended in a §4.4 `error` event. The chain is
  not advancing.
- `conflicted` — the subagent's merge-back attempt hit a rebase
  conflict (ARCH §2.6 step 6 — harness defect; the structural
  single-author guarantee was violated and operator attention is
  required).

The exit code is 0 in all three terminal cases; the in-band `status`
field is what communicates outcome (same shape as the adapter's
in-band errors, ARCH §4.4).

## When to use

- After `dispatch` returned a handle, on a later step, to retrieve the
  subagent's compacted result.
- To synchronize with a parallel subagent before continuing — issuing
  several `dispatch` calls in one step and `await`-ing them in
  subsequent steps as their work lands.

## When not to use

- For a short read or computation the parent agent could do inline.
- To check liveness — `await` blocks. A non-blocking `check(handle)`
  is filed for v0.5+ and not part of v0.4.

## Notes

- `await` blocks until the subagent is terminal. It does not time
  out — if the subagent's harness has wedged, the parent harness must
  be killed at the user's discretion.
- v0.4 detects `stopped` from the §4.4 `error` event in the latest
  step's `response.json`. A subagent killed mid-stream (no `error`,
  no `message_stop`) is not yet detected as `stopped` — that path
  needs filesystem-close detection (inotify) and is filed for a
  follow-on.
