---
name: bash
description: "Run one shell command on this machine and read back what it printed. The shell is local: lernie is a command-line program the user runs, and your command executes on that same machine — their filesystem, their network, their user account. There is no server, container, or remote sandbox between you and it, so a question about this host (its IP, its disk, its toolchain) is answerable by just running the command. It is not an interactive terminal and there is no prompt waiting for you: each tool call runs exactly one `sh -c '<command>'` with stdin closed and no TTY, waits for it to exit, and hands back its output. Shell state does not carry over — a `cd`, an `export`, or a shell function is gone by the next tool call — but files the command writes stay, in your worktree, which is where every command starts. You get stdout; if the command exits non-zero its stderr is appended and the result is flagged as an error. Use it for filesystem inspection (`ls`, `find`, `head`), text processing (`grep`, `sed`, `awk`), version-control queries (`git log`), builds and tests, and anything else a dedicated tool does not cover."
---

# bash

Executes one shell command and hands its output back. The de-facto
escape hatch when the conversation needs to do something a dedicated
tool does not cover.

## Input

```json
{ "command": "<shell command>" }
```

`command` is a string passed verbatim to `sh -c`. Quoting, expansion,
pipes, and redirection are the shell's responsibility.

## Where it runs

**On this machine.** lernie is a command-line program someone runs; the
shell it spawns is a child of that process — same host, same user
account, same filesystem, same network route. It is not a hosted
service, not a container image, not a sandbox you were handed. `curl
ifconfig.me` reports the operator's own public IP because it *is* the
operator's network. Answer from it plainly rather than hedging about
"the tool's execution environment"; there is only one environment.

**In your own worktree.** The shell starts in the branch checkout the
harness materialized for you, so relative paths resolve there and `pwd`
reports it. Anything you write, edit, or delete under it is committed
onto your branch alongside the tool result (ARCH §3.3), which is how
your work leaves the conversation as a product. There is no need to
`cd` anywhere first, and paths outside the worktree are not yours to
act on.

## It is not an interactive shell

There is no live prompt on the other end and no terminal session to
drive. One tool call is one `sh -c` process, spawned and reaped:

- **Stdin is `/dev/null`.** A command that reads stdin gets EOF
  immediately. There is no way to answer a prompt, feed input from a
  later tool call, or drive a REPL.
- **There is no TTY.** Programs that require one (`vim`, `less`,
  `top`, an `ssh` password prompt) fail or degrade; reach for their
  non-interactive flags instead (`git --no-pager`, `apt-get -y`).
- **The command runs to completion.** No wall-clock limit is imposed,
  so a slow build finishes — but nothing interrupts it from your side,
  and a command that blocks forever waiting on input blocks the step
  until the operator stops the agent.

## State between tool calls

The shell process does not survive the tool call, so **shell state does
not either**. `cd /tmp` in one tool call does not move the next one;
`export FOO=1`, `set -x`, shell functions, and aliases are all gone.
Chain them inside a single `command` string (`cd sub && make`) when you
need them together. What *does* persist is the filesystem: files the
command creates or edits in the worktree are still there next time, and
are committed onto your branch.

## Output

Stdout from the command is surfaced as the `content` of the matching
`tool_result` block. When the command exits non-zero, stderr is
concatenated after stdout and `is_error` is set, so the model sees
the failure message in the next step's request. On a zero exit stderr
is *not* surfaced — redirect it (`2>&1`) when you need to read it.

## When to use

- Listing or inspecting files and directories: `ls`, `find`, `tree`.
- Searching contents: `grep -r`, `rg`, `sed -n`.
- Slicing oversize files: `head -n 200 BIG_FILE` — the alternative
  for paths `read_file` rejects with `TooLarge`.
- Querying git: `git log --oneline -n 20`, `git status`.
- Running build / test commands when the conversation needs their
  output to reason about the next step.
- Answering a question about *this* machine — its IP, its disk, its
  installed toolchain. The answer is not hypothetical; go get it.

## When not to use

- Reading a single file in full: `read_file` is the cheaper, more
  predictable surface.
- Anything interactive — there is no stdin and no TTY, so a command
  that expects either will hang or fail rather than prompt.

## Failure modes

- Non-zero exit from the command — `is_error: true`; stderr appended
  to stdout in `tool_result.content`.
- Malformed input JSON (missing `command`, wrong type, extra fields)
  → `is_error: true` with the message on stderr.
- SIGTERM cascade — when the *harness* is cancelled (`lernie stop`,
  ARCH §2.9), SIGTERM is forwarded to the entire process group spawned
  by `sh`, then SIGKILL after the grace period. That is a
  cancellation, not a timeout: nothing kills a command for merely
  taking a long time. The reaped child's signal is reported as
  `128 + signo` (POSIX convention), so a clean cancel surfaces as exit
  code 143.

The tool's stdio contract follows ARCH §3.3 verbatim — stdout is the
result on success, stderr concatenates on failure, and `is_error`
mirrors the exit code.
