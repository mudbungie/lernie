# Design Principles

Quick-reference catalog of the principles shaping this architecture. Canonical source is `docs/ARCHITECTURE.md`; this document is a distillation.

## Disk first
System state — context, messages while in flight, responses, tool calls — lives on disk; processes hold none of it across restart. This is what makes git the substrate at all: history, branching, rollback, replay, and audit collapse to git operations on data that's already there. It is also what eliminates entire classes of bug — lost updates, stale caches, ghost state surviving a crash, in-memory views drifting from on-disk truth — because there is no in-memory truth to drift from. Every other principle below depends on this one.

## Inspectability first
Every point in a conversation's life is a git ref. Replay, counterfactual forks, and debugging are first-class because the state is on disk in an append-only, inspectable tree.

## Symmetry of dispatch
User-to-agent, agent-to-subagent, verifier, and compactor all flow through one primitive: fork a branch, do work, merge back. No special code paths for user input, reviews, or summarization — everything is an invocation.

## Testability via configuration
Workflow, prompts, and assembly rules are configuration, not code. Experiments become config diffs measured against a task suite, so improvements can be validated without shipping new software.

## Everything through git
Conversations are repos, dispatches are branches, steps are commits, terminations are merges. Using git as the substrate gets history, branching, rollback, and replay for free.

## Nothing writes to main directly
The trunk only advances via no-ff merges from completed exchange branches. Every commit on main is provenanced to a goal, keeping the user-facing view clean and traceable.

## Single author per file
The harness assigns write paths per tool call and per invocation so sibling branches never target the same file. This turns "conflict-free merges" from a convention into a structural guarantee and keeps parallelism safe at scale.

## Commit before model call
A step's snapshot commit is written before the model call is issued. Retry becomes trivial, and every model call is bit-for-bit replayable from its commit.

## Tools return synchronously; async rides on handles
Provider APIs require paired `tool_use`/`tool_result`, so every tool returns immediately. Long-running work surfaces as a handle and is retrieved later via `await(handle)`, preserving parallelism without breaking the wire protocol.

## Integrations are external binaries
Tools (§3.3) and provider adapters (§4.4) are separate executables the harness invokes as subprocesses, each with a narrow stdio contract — a handful of subcommands, JSON on stdin/stdout, env-var auth, SIGTERM cancellation. The harness owns orchestration and on-disk state; the binaries own wire protocols, vendor quirks, and credential handling. That is what lets external contributors extend the harness — a corporate SSO flow or a new model provider is a standalone binary, not a patch to the core — and is what keeps the core small enough to reason about.

## Compaction, never compression
Summarizing a branch's work uses a subagent with a constrained toolset (`write_summary`, `mark_for_deletion`). "Deletion-only" is structural — no general filesystem write surface — so the worst case is lost information, never corrupted information.

## Goals are pinned
Each branch's goal sits at the head of context for every model call on that branch, regardless of position in the sequence. This defeats the recency-decay failure mode in deep agent trees, where the last message drowns out the original intent.

## Stops are aggressive and cascading
When a stop fires, in-flight HTTP is dropped, tool subprocesses receive SIGTERM, and descendants are cancel-marked. Better to waste a step than let runaway work continue after a user or parent has pulled the plug.

## Context assembly is deterministic
What the model sees is a pure function of the repo state via `manifest.yaml`. Reproducibility becomes a property of the system, and "why is this in context?" always has a concrete answer.

## Agents manage their own context by `rm`
An agent reduces its context by deleting files; the next assembly excludes them. Context hygiene is an agent-level decision expressed in the native language of the filesystem, not a bespoke API.

## File path as hint
Paths like `invocations/a1b2/result.md` ride into context verbatim. Structure in the name is cheaper and usually sufficient compared to bolt-on metadata.

## Decline illegal operations
Capabilities are code-backed; a config selecting a capability the harness does not implement is rejected at load time. Silent degradation is never preferable to a loud refusal.

## Measure reliability separately from capability
`pass@1` is the primary metric; `pass@5` is tracked alongside it. A workflow that lifts pass@1 without pass@5 is noise reduction; lifting both is a real capability gain — conflating the two hides which is happening.

## Terminology is load-bearing
Contested field terms ("turn", "session", bare "call", "compression") are banned in code, docs, and commit messages. Every term of art has a single definition in `docs/TAXONOMY.md` or the document introducing it, and coinage without approval is not allowed.
