+++
title = "caller-supplied pinned documents reach prompt and dispatch"
created = 1785649842
updated = 1785649842
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Source: yog bl-e249's Claude Code comparison. Verify the current architecture and code before editing.

## Existing deferred capability

Lernie ARCH already describes dispatch as pinning `goal.md`, `soul.md`, and "whatever documents the dispatcher chose to pin." The shipped `prompt`/`dispatch` path exposes only goal/soul. A caller such as yog therefore cannot freeze target-project instructions into agent context without rewriting the user's goal or mutating a shared config branch.

## Deliverable: generic mechanism, no filename policy

Let root prompt and child dispatch accept zero or more caller-supplied pinned documents:

- one deep, narrow CLI/library/tool shape; preserve exact parity;
- each input names its destination and source bytes/path without allowing traversal, reserved harness names, collisions, or control-file injection;
- validate and snapshot exact bytes into the dispatch commit before the first model request;
- ordinary Git fork inheritance carries them to descendants unless the project-work contract later rules an explicit replacement;
- absent inputs preserve today's tree and request byte-for-byte;
- provenance is inspectable without a second content copy;
- failures happen before a branch/ref or inference exists;
- amend ARCH/PRINCIPLES/TAXONOMY only where the actual contract changes.

Lernie owns only pinning. Yog owns which files count as project instructions, precedence, trust, and size policy (yog bl-aa8b). Do not hardcode `AGENTS.md`, `CLAUDE.md`, or a memory system here.