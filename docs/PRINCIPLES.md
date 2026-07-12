# Design Principles

Quick-reference catalog of the principles shaping this architecture. Canonical source is `docs/ARCHITECTURE.md`; this document is a distillation.

## Disk first
System state — context, messages while in flight, responses, tool calls — lives on disk; processes hold none of it across restart. That includes the model-facing history itself: assistant output and tool results are committed transcript entries (ARCHITECTURE §2.3), never message lists held in executor memory. This is what makes git the substrate at all: history, branching, rollback, replay, and audit collapse to git operations on data that's already there. It is also what eliminates entire classes of bug — lost updates, stale caches, ghost state surviving a crash, in-memory views drifting from on-disk truth — because there is no in-memory truth to drift from. Every other principle below depends on this one.

## Single source of truth
Every piece of data is recorded in at most one place. If git already carries it — branch existence (`git branch --list`), commit history (`git log`), tree contents (`git ls-tree`), the fork point of a branch — we don't also write it into a sidecar state file. If the filesystem already carries it — a file's presence, a directory's contents — we don't also record that in a manifest. Copy is the bug: two sources means a consistency protocol, and a consistency protocol means drift. When in doubt, derive don't mirror. The unmerged-subagent-branch health metric (§8), for instance, is enumerated by `git branch --list '*-*' --no-merged main`, filtered by the executor-lock liveness probe (§2.11) — the refs and the kernel are authoritative and free.

## Context has one home
Every context-entering artifact is a committed worktree file — delivered messages, assistant step output, tool results — sequenced as the branch's transcript (`messages/NNN-<origin>`, ARCHITECTURE §2.3). Order lives in the filename and assembly is readdir+sort; step records under `steps/` are diagnostic, with zero runtime content reads. Between compactions the assembled prompt only grows at the tail, so provider prompt caches stay warm; compaction is the sole sanctioned reorganization point (ARCHITECTURE §5.5).

## Inspectability first
Every point in a conversation's life is a git ref. Replay, counterfactual forks, and debugging are first-class because the state is on disk in an append-only, inspectable tree.

## Symmetry of dispatch
User-to-agent, agent-to-subagent, verifier, and compactor all flow through one primitive: fork a branch, do work, merge back (or terminate to user, for the root case). No special code paths for user input, reviews, or summarization — everything is a conversation. The symmetry extends past spawn: any sender — user or conversation — steers a live conversation through the same message primitive (§2.11), so every sender is indistinguishable from a user for the recipient's whole life.

## One obvious path
There should be one — and ideally only one — correct way through the system for any given operation. When two procedures look similar, the discipline is to identify the core operation they share and unify them on *that* core, not to open a parallel path and not to broaden the abstraction past where the sharing actually holds. Compaction and verification are dispatches because the core — spawn a branch with a goal, do work, merge back — is identical; stopping the shared path at "dispatch" is deliberate, because the next layer up (what to do with the result) is where they genuinely diverge. Structurally identical operations share code; operations that merely rhyme stay apart. A new procedure earns its place by collapsing onto an existing primitive or by introducing one that is genuinely new, never by sitting in parallel with something it almost is.

## Testability via configuration
Workflow, prompts, and assembly rules are configuration, not code. Experiments become config diffs measured against a task suite, so improvements can be validated without shipping new software.

## Everything through git
Conversations are repos, dispatches are branches, steps are commits, terminations are merges. Using git as the substrate gets history, branching, rollback, and replay for free.

## Nothing writes to main directly
The trunk only advances via no-ff merges from completed root-conversation (exchange) branches. Every commit on main is provenanced to a goal, keeping the user-facing view clean and traceable.

## Single author per file
The harness assigns write paths per tool call and per subagent conversation so sibling branches never target the same file. This turns "conflict-free merges" from a convention into a structural guarantee and keeps parallelism safe at scale.

## Read state per step is a real git commit
Step 1 commits the dispatch artifacts (goal.md + soul.md) before its model call; step ≥2 reads from the prior step's tip, advanced by the prior step's transcript, side-effect, and delivery commits. Either way, every step's read state is a real git commit, recorded in the step's `meta.json`. Replay re-runs the context assembler against that sha and reproduces the wire input bit-for-bit; the diagnostic `request.json` on disk is not authoritative. Because the transcript is worktree data (ARCHITECTURE §2.3), running assembly and replay assembly are the same read — replay is not a separate mode.

## Tools return synchronously; async rides on handles
Provider APIs require paired `tool_use`/`tool_result`, so every tool returns immediately. Long-running work surfaces as a handle and is retrieved later via `await(handle)`, preserving parallelism without breaking the wire protocol.

## Integrations are external binaries
Tools (§3.3) and the provider adapter (§4.4 — brazen's `bz`, one binary for every provider) are separate executables the harness invokes as subprocesses, each with a narrow stdio contract: JSON on stdin/stdout, SIGTERM cancellation, exit codes with defined meaning. The harness owns orchestration, retry, and on-disk state; the binaries own wire protocols, vendor quirks, and credential handling — lernie never sees a credential. That keeps the core small enough to reason about, and it is what lets deployments extend the system without patching it: a new tool is a standalone binary, a new provider is a brazen config row, and a bespoke wire dialect is either a brazen contribution or a contract-compatible alternate adapter binary (§4.4 Extensibility).

## Everyone uses the front door
The `lernie` CLI is the sole control plane: every procedure-to-procedure invocation — subagent dispatch, compaction, verification, any workflow-invoked procedure (§6) — goes through it. The harness is a first-class consumer of its own CLI, using the same command surface an external caller would. Disk-as-bus (§3.1) carries state between procedures; the CLI carries commands. Nothing else — no library API, no in-process sidechannel, no ad-hoc socket. Three payoffs: every inter-procedure edge is a capturable CLI call (integration testability); embedding lernie in another tool is `exec("lernie", ...)` rather than a library port (embeddability); and with only those two channels, back-channel communication has no place to go (operational atomicity as structure, not convention).

## Frontends are stateless and pluggable
The UI holds no persistent state; every render is a pure function of filesystem state at the current git ref. Its only interfaces to the harness are reading repo paths and issuing `lernie <subcommand>` invocations — no third channel. Two or more frontends (desktop GUI, webclient, TUI) can run against one repo simultaneously because none of them write repo state or share memory; they all observe the same on-disk truth and drive changes through the same CLI the harness itself uses. Pluggability is structural: a new frontend is a new consumer, not a new integration. Direct consequence of "Disk first" + "Everyone uses the front door".

## Regenerability
Any process can die at any time without losing state. Components (harness, tool subprocesses, provider adapters, frontends) restart independently because none hold state across their own termination. The workflow interpreter itself is the currently-executing CLI subprocess (§6) — a baton passed forward by exec, not a daemon — and the operation that drives the chain is `lernie advance <repo>`. Crash recovery and normal operation are the same operation: `lernie advance` reads the repo, evaluates workflow rules against on-disk state, runs the next action, and exec's another `lernie advance` to continue. There is no distinct "resume"; recovery is just running advance from where the chain happens to be. No process is load-bearing; disk is. Direct consequence of "Disk first" + "Everyone uses the front door": with no state off-disk and no commands off the CLI, a crashed process leaves nothing orphaned to reconstruct.

## Compaction, never compression
Summarizing a branch's work uses a subagent with a constrained toolset (`write_summary`, `mark_for_deletion`). "Deletion-only" is structural — no general filesystem write surface — so the worst case is lost information, never corrupted information.

## Goals are pinned
Each branch's goal sits at the head of context for every model call on that branch, regardless of position in the sequence. This defeats the recency-decay failure mode in deep agent trees, where the last message drowns out the original intent.

## Stops are aggressive and cascading
When a stop fires, in-flight HTTP is dropped, tool subprocesses receive SIGTERM, and descendants are cancel-marked. Better to waste a step than let runaway work continue after a user or parent has pulled the plug.

## Context assembly is deterministic
What the model sees is a pure function of the repo state via `manifest.yaml`. Reproducibility becomes a property of the system, and "why is this in context?" always has a concrete answer.

## Agents manage their own context by `rm`
An agent reduces its context by deleting files — work products and transcript entries alike; the next assembly excludes them. Context hygiene is an agent-level decision expressed in the native language of the filesystem, not a bespoke API.

## File path as hint
Paths like `summary/003.md` or `skills/git-ops/SKILL.md` ride into context verbatim. Structure in the name is cheaper and usually sufficient compared to bolt-on metadata. (Step records — `steps/<conv-id>/NNN/request.json` etc. — are not context; they live at the conv-repo root, outside every worktree, and are diagnostic-only — see ARCHITECTURE.md §2.3.)

## Decline illegal operations
Capabilities are code-backed; a config selecting a capability the harness does not implement is rejected at load time. Silent degradation is never preferable to a loud refusal.

## Measure reliability separately from capability
`pass@1` is the primary metric; `pass@5` is tracked alongside it. A workflow that lifts pass@1 without pass@5 is noise reduction; lifting both is a real capability gain — conflating the two hides which is happening.

## Terminology is load-bearing
Contested field terms ("turn", "session", bare "call", "compression") are banned in code, docs, and commit messages. Every term of art has a single definition in `docs/TAXONOMY.md` or the document introducing it, and coinage without approval is not allowed.
