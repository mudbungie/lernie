---
name: cd
description: "Move your working directory. Every tool call of yours starts in one directory — your own worktree to begin with — and `cd` changes which one, for good: the next `bash` command, the next relative path handed to `read_file`, and every later tool call all start where you last moved to. This is the only way to move; a `cd` inside a `bash` command dies with that shell. Give it a directory that exists (relative to where you are now, or absolute) and it returns the absolute path you are now in; give it a path that names nothing, or names a file, and it declines and you stay put. You may move anywhere on the machine, but only files under your worktree are committed onto your branch — work you do elsewhere is real and is not recorded, so move out for reading and building, and come back to write your product."
---

# cd

Changes the working directory every subsequent tool call of yours runs
in (ARCH §3.3 *Working directory*).

## Input

```json
{ "path": "<directory>" }
```

- **Relative** paths resolve against where you are *now* — `cd src`
  after `cd /home/you/project` lands in `/home/you/project/src`.
- **Absolute** paths are taken as given.
- `..` and symlinks are resolved; the answer is a real, absolute path.

## Output

```json
{ "cwd": "/absolute/path/you/are/now/in" }
```

That is where the next tool call starts, and the one after it, until you
`cd` again. Nothing else changes — no environment, no shell state, no
files. The object arrives in the result envelope, after its
`Exit code: N` line (below).

## Why this and not `cd` inside bash

A `bash` tool call is one `sh -c` process that exits when the command
does, so a `cd` inside it moves nothing but that one shell. Chaining
(`cd sub && make`) works for a single command and is fine when that is
all you need. Reach for this tool when you are about to do *several*
things somewhere else — then move once, and stop prefixing.

## Where you start, and what gets recorded

You start in your own worktree, the branch checkout the harness
materialized for you. Files you write **there** are committed onto your
branch alongside the tool result, which is how your work leaves the
conversation as a product (ARCH §3.3, §2.6).

You may `cd` anywhere on the machine — nothing is fenced off, and `bash`
could already reach outside with an absolute path. But the commit only
ever stages your worktree, so **anything you write outside it is off the
record**: not on your branch, not visible to whoever dispatched you, not
in a replay. That is the trade. Read, search, build and test wherever
the work is; write your product in your worktree.

## When to use

- The task is about a directory that is not your worktree — a repository
  under the operator's home, a build tree — and you have more than one
  command to run there.
- You are working deep inside a tree and want relative paths to be short
  and stable across calls.

## When not to use

- One command in one place: `cd sub && cmd` inside `bash` is cheaper.
- To "reset": you do not need to. If you want your worktree back, `cd`
  to it by absolute path — `pwd` in `bash` tells you where you are, and
  your worktree is `<workspace>/agents/<your-id>`.

## Failure modes

- A path that names nothing → non-zero exit, `is_error: true`, the path
  named. You have not moved.
- A path that names a file rather than a directory → the same, said
  differently. You have not moved.
- If the directory you moved to is later deleted from under you, your
  tool calls resume in your worktree rather than failing — you are never
  stranded somewhere that no longer exists.

Every result is a §3.3 **result envelope**: an `Exit code: N` line
first, then the output described above, then — whenever the tool wrote
any, on success as well as failure — its stderr under a
`--- stderr ---` marker. So the exact reason for a decline reaches you
in the next step's request, and the stated code tells you which decline
it was.
