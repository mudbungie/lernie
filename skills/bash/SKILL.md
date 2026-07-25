---
name: bash
description: Run a shell command via `sh -c` and return its stdout. Use for filesystem inspection (`ls`, `find`, `head`), text processing (`grep`, `sed`, `awk`), version-control queries (`git log`), and anything else that fits a one-liner. The command's stderr is surfaced only when it exits non-zero.
---

# bash

Executes a shell command and hands its output back. The de-facto
escape hatch when the conversation needs to do something a dedicated
tool does not cover.

## Input

```json
{ "command": "<shell command>" }
```

`command` is a string passed verbatim to `sh -c`. Quoting, expansion,
pipes, and redirection are the shell's responsibility.

## Output

Stdout from the command is surfaced as the `content` of the matching
`tool_result` block. When the command exits non-zero, stderr is
concatenated after stdout and `is_error` is set, so the model sees
the failure message in the next step's request.

## Working directory

The shell starts in **your own worktree** — the branch checkout the
harness materialized for you — so relative paths resolve there and
`pwd` reports it. Anything you write, edit, or delete under it is
committed onto your branch alongside the tool result (ARCH §3.3), which
is how your work leaves the conversation as a product. There is no need
to `cd` anywhere first, and paths outside the worktree are not yours to
act on.

## When to use

- Listing or inspecting files and directories: `ls`, `find`, `tree`.
- Searching contents: `grep -r`, `rg`, `sed -n`.
- Slicing oversize files: `head -n 200 BIG_FILE` — the alternative
  for paths `read_file` rejects with `TooLarge`.
- Querying git: `git log --oneline -n 20`, `git status`.
- Running build / test commands when the conversation needs their
  output to reason about the next step.

## When not to use

- Reading a single file in full: `read_file` is the cheaper, more
  predictable surface.
- Long-running interactive processes: there is no stdin passthrough,
  and a SIGTERM-then-SIGKILL cascade fires after the harness deadline
  (ARCH §3.3). Use it for one-shot commands.

## Failure modes

- Non-zero exit from the command — `is_error: true`; stderr appended
  to stdout in `tool_result.content`.
- Malformed input JSON (missing `command`, wrong type, extra fields)
  → `is_error: true` with the message on stderr.
- SIGTERM cascade — when the harness cancels, SIGTERM is forwarded to
  the entire process group spawned by `sh`, then SIGKILL after the
  deadline. The reaped child's signal is reported as `128 + signo`
  (POSIX convention), so a clean cancel surfaces as exit code 143.

The tool's stdio contract follows ARCH §3.3 verbatim — stdout is the
result on success, stderr concatenates on failure, and `is_error`
mirrors the exit code.
