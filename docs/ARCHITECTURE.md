# Agent Harness Spec

**Status:** Draft v0.3
**Scope:** Design specification for a git-backed agent harness with branch-per-dispatch context management.

---

## 1. Overview

This document specifies an agent harness in which conversational context is managed as a git repository. Each conversation is a standalone repo. Every dispatch is a branch. Branch completion is a merge. Within a branch, steps land as linear commits; a step that itself dispatches spawns a sub-branch off the commit where the dispatch landed. All state lives on disk; all inter-component communication is mediated through the filesystem.

The architecture optimizes for three properties:

1. **Inspectability.** The complete state of any conversation, at any point in its history, is a git ref. Replay, debugging, and counterfactual forking are first-class operations.
2. **Uniformity.** User-to-agent dispatch, agent-to-subagent dispatch, and agent self-reflection all use the same primitive: fork a branch, do work, merge back. There is no special path for user input.
3. **Testability.** Workflow (prompts, sequencing, context assembly rules) is configuration, not code. Experiments are config diffs, measurable against a task suite.

### 1.1 Non-goals for v1

The following are explicitly out of scope for v1:

- Multi-tenancy or multi-user isolation. Single user assumed.
- Distributed execution. Single machine, single harness process.
- Tool sandboxing. Tools run with user privileges; no capability restriction.
- Remote git operations. Conversation repos are never pushed anywhere.
- Secrets management infrastructure. Env vars injected at tool execution, referenced by name in config. No vault, no rotation.

---

## 2. Core Concepts

### 2.1 Terms

All terminology below is load-bearing and is used exclusively in the senses defined here. Terms in `docs/TAXONOMY.md` are the ambient field context; where the taxonomy flags a term as contested, this spec picks one sense and stays in it.

| Term | Meaning | Relationship |
|---|---|---|
| **Conversation** | The full lifecycle of exchanges between a user and the system, persisting across UI and process lifetimes. | Contains one or more exchanges; the outermost scope. One conversation = one repo. |
| **Exchange** | One user message in, one final assistant response out. | Child of a conversation; root of a trace. |
| **Invocation** | One entry into a named agent or subagent, containing its full internal execution. | Child of a step (when a tool call targets a subagent); contains its own nested steps and further invocations. |
| **Step** | One model call and the tool calls it emits. | Child of an exchange or an invocation; structurally bounded by the model call, not by tool completion. |
| **Model call** | One execution of a model to produce output. | Atomic. The defining event of a step; each step has exactly one. |
| **Tool call** | The model's structured request to invoke a named tool. | Emitted by a model call; structural child of its emitting step (even if it resolves temporally during a later step). |
| **API call** | One HTTP request to a provider endpoint. | Implementation detail of a model call (usually 1:1; streaming or retries may make it 1:many). |
| **Dispatch** | The event that spawns a branch with a goal. Two cases: a user message (spawns an exchange off `main`) and a tool call targeting a subagent (spawns an invocation off the commit where the dispatch landed on the parent branch). | Creates a branch. |
| **Branch** | The git/worktree container for an exchange or an invocation. | Every dispatch creates a branch; the branch is merged back on termination. |
| **Goal** | The stated objective handed to an agent at dispatch. | One per invocation (or exchange); not generally rewritten during execution. |

**Banned usage:** "call" without a qualifier (use model call, tool call, or API call); "turn" (its vendor meanings are incompatible — see `docs/TAXONOMY.md` §1); "session" (underdefined — `docs/TAXONOMY.md` §3); "compression" (reserved for the model-weights sense — the harness operation is compaction).

**Other terms of art not defined in this document or in `docs/TAXONOMY.md` require explicit definition or user approval before use.**

### 2.2 The Conversation Repo

Each conversation is a self-contained git repository on local disk. Repos are created by copying a versioned template. Repos are never pushed to a remote. The complete state — exchanges, tool outputs, artifacts, agent notes, configuration — lives in the repo.

Directory layout:

```
<conversation-repo>/
├── .git/
├── .agent/
│   ├── version                  # schema version, integer
│   ├── manifest.yaml            # context assembly rules
│   ├── workflow.yaml            # event-to-action bindings
│   ├── agents.yaml              # agent role definitions
│   ├── providers.yaml           # endpoint/auth/model config
│   ├── goal.md                  # branch goal, written at dispatch
│   ├── compactions/             # per-branch intermediate+terminal summaries (§2.7)
│   ├── system/                  # system prompts, tool and skill definitions
│   │   ├── prompts/
│   │   ├── skills/
│   │   └── tools/
│   └── state/                   # harness runtime state (gitignored where noted)
│       ├── branches.json        # unmerged branch tracking — runtime state, .gitignored
│       └── events.log           # append-only event log
├── exchanges/                   # compacted exchange history (post-merge)
├── artifacts/                   # agent-produced files (code, docs, outputs)
├── invocations/                 # invocation working areas (ephemeral)
└── tools/                       # tool call records for current and recent branches
```

`goal.md` is branch-scoped: each branch's worktree has its own copy, set at dispatch time. The root conversation has no goal file; goals apply to exchanges and invocations.

Read-only from the user's perspective under normal operation. The harness is the only writer. The user interacts via the UI, which produces events that the harness translates into commits.

### 2.3 Branches and the branch invariant

A **branch** is the git/worktree container for a unit of work. The harness has one structural invariant:

> **Every dispatch creates a branch.** Spawn the branch, do the work, compact, merge back. This pattern is uniform whether the dispatch is a user message (root case, off `main`) or a tool call targeting a subagent (nested case, off the commit on the parent branch where the dispatch landed).

Two classes of branch, distinguished by the dispatch that spawned them:

- **Exchange branch.** Off `main`. Spawned by a user message. Contains one exchange — the user message down to the terminal assistant response. Named `ex/<ts>-<id>`.
- **Invocation branch.** Off the commit on the parent branch where a dispatch tool call landed. Spawned by that dispatch. Contains one invocation — the subagent's full internal execution. Named `inv/<parent-id>/<id>`.

Within a branch, **steps are linear commits** — one commit per step, carrying that step's model call and the tool calls it emitted. Steps are not their own branches. New branches appear only at dispatch boundaries.

The trunk is always `main`. Nothing ever commits directly to `main`. The only way data reaches `main` is via merge from a completed exchange branch.

> **v0.1 exception.** The v0.1 milestone (§12) predates branching and explicitly commits one exchange directly on `main` per `lernie prompt` invocation. The invariant above is the v0.2 steady state.

**Branch lifecycle** (identical for both branch classes):

1. **Spawn.** The dispatch creates a new branch off the parent's current commit. A worktree is allocated. The goal is written. Branch name encodes provenance.
2. **Work.** The agent runs its loop. Each step lands as a commit on the branch. A step that emits a subagent-targeting tool call spawns a sub-branch off that commit.
3. **Completion.** A terminal event is emitted — the agent returns a final response or is stopped.
4. **Merge.** The compactor produces a signal-preserving summary of the branch's work and applies it to the parent with `--no-ff`. Merge topology is preserved.
5. **Cleanup.** The source branch is kept as a ref for a retention window, then GC'd. The worktree is removed on merge.

Branching is cheap — local git operations on disk — but it is no longer per-step. A long exchange produces many commits on one branch and only spawns sub-branches when it actually dispatches.

Compaction may also run *during* a branch's execution, not only at termination; see §2.7.

**Step on-disk layout.** Each step lives in its own directory under `exchanges/<exchange-id>/steps/<NNN>/` (for an exchange branch) or `invocations/<invocation-id>/steps/<NNN>/` (for an invocation branch, when v0.4 lands). `<NNN>` is zero-padded 3-digit and 1-indexed, so step dirs sort lexically. Two core files land per step:

- `request.json` — the model call's input. Committed **before** the model call (§2.10), so the commit's tree is the exact state the model read from; retry replays this snapshot without drift.
- `response.json` — the normalized model-call output (assistant text, `model_id`, `provider`, `usage`, `stop_reason`, `started_at`, `ended_at`). Landed as a *follow-up commit* on the same branch — not an amend of the snapshot — so the snapshot's tree continues to reflect pre-model-call state.

Tool calls emitted by a step (v0.3+) extend the step's dir with `tool_calls/<tool-id>/…` rather than creating new step dirs, preserving "one step = one model call" (§2.1). The branch's worktree is allocated at `<repo>/.lernie/worktrees/<branch-path>/`; `.lernie/` is gitignored so worktrees never land on `main`.

The rich per-step tree is branch-life state. On terminal compaction (§2.7) the compactor writes a signal-preserving summary and marks the raw step dirs for deletion, leaving `main` with only the compacted history §2.2 describes.

**Unmerged-branch tracking.** `.agent/state/branches.json` is a harness-managed JSON map keyed by branch name, written atomically on spawn and updated on merge/stop. Each entry carries `kind` (`exchange` | `invocation`), `spawned_at`, `base_sha` (the parent commit the branch was forked from), `status` (`open` | `stopped`), and an optional `stopped_at`. Merge removes the entry; the map's length is the unmerged-branch-count health metric (§8). The file is runtime state, not tracked in git — the template's `.gitignore` keeps per-branch trees from carrying stale snapshots. A missing file is treated as an empty map so the first spawn bootstraps it.

> **v0.2 scope.** The branch/snapshot/response flow plus spawn-side `branches.json` tracking land in v0.2; the merge-back protocol (§2.6) and terminal compaction are separate v0.2 steps. Until merge-back ships, `lernie prompt` spawns branches that stay open, and each open branch has a live entry in `branches.json`.

### 2.4 Exchanges

An exchange is initiated by a user message and ends with a terminal assistant response. Its branch is the root of a trace in the observability sense. An exchange that is interrupted before a terminal response is terminal by virtue of the stop itself: it does not merge back to `main`; it may be resumed by a new dispatch using the stopped branch as context (see §2.9).

Multiple concurrent exchanges are supported: a user may send a second message before the first resolves. Each is its own exchange branch off `main`, merging in completion order. No special mechanism beyond the branch invariant is required.

### 2.5 Invocations and dispatch

A **dispatch** (§2.1) is the event that spawns a branch with a goal. The tool-call form — a tool call targeting a subagent — spawns an **invocation**: the subagent's full internal execution, contained in its own branch off the commit where the dispatch landed on the parent branch. From the subagent's perspective, its parent is indistinguishable from a user; the user-message and tool-call forms of dispatch are the same primitive.

This symmetry is load-bearing:

- The same code path handles user-initiated exchanges and agent-initiated invocations.
- Verifier agents, compactor agents, and adversarial critics are invocations with different goals.
- Parallel exploration (N workers on the same task) is N parallel invocations dispatched from the same step.

**Every tool returns synchronously.** Provider APIs (Anthropic Messages, OpenAI Responses, Gemini) require each `tool_use` block emitted by a step to be matched by a `tool_result` block in the immediately following user message. The harness honors that invariant: a step's next model call is issued only when every tool call it emitted has produced a `tool_result`. Partial-result reprompts are not attempted — they are rejected at the wire.

**Async work uses handles.** Long-running tools — including dispatch — return immediately with a handle (`{status: in_progress, handle: <id>}`) as their `tool_result`. The agent retrieves the actual outcome later via a separate `await(handle)` or `check(handle)` tool, whose return value is a `tool_result` on a later step. Parallelism is expressed by issuing several dispatches in one step and awaiting them in subsequent steps as results come in. The per-step "one result per tool_use" shape stays intact; the asynchrony rides on the handle. Dispatch is a tool like any other: this is how the symmetry between inline tool calls and invocations is preserved without inventing a second control path.

Tool calls targeting non-subagent tools commit their records on the emitting branch as part of the step. Dispatch tool calls spawn invocation branches off the commit where the dispatch landed; the `tool_result` the parent step observes is the handle. When an invocation terminates and merges back, its compacted output is what `await(handle)` returns on the next step. Parallel dispatches from a single step spawn N sibling invocation branches off that same commit.

Invocations are expected to terminate and merge before their parent branch does.

The harness assigns write paths per tool call and per invocation, so two sibling branches never target the same file. This is a structural guarantee (enforced by the tool executor and the dispatch primitive), not a convention, which is what makes the merge protocol (§2.6) conflict-free by construction.

### 2.6 Merges

Merges are always no-fast-forward and conflict-free by construction. The merge protocol:

1. Child branch completes.
2. Terminal compaction runs (if warranted — see §2.7): a compactor subagent is dispatched off the child's tip and merges back into the child, leaving the child's tree in compacted form.
3. Harness rebases the child's compacted diff onto the current parent tip (which may have advanced).
4. If rebase succeeds: merge with `--no-ff`.
5. If rebase conflicts: the child branch is marked conflicted, left unmerged, flagged for operator attention. This indicates a harness defect — two branches were given overlapping write paths, violating the guarantee in §2.5 — and is tracked accordingly.

### 2.7 Compaction

Compaction (not compression — the term is reserved for model-weights quantization in the taxonomy) is the process of producing a signal-preserving, minimal version of a branch's work for consumption by the parent. A **compactor** is a subagent that performs compaction. It has no privileged position in the architecture: compactors are dispatched like any other invocation, run on their own branches, and merge back through the normal protocol (§2.6). What distinguishes a compactor is its goal (produce a summary of the dispatching branch's work) and its toolset.

Compactor toolset (v1):

- **`write_summary(content)`** — writes the compacted summary file on the compactor branch.
- **`mark_for_deletion(path)`** — nominates a file on the compactor branch for removal. The harness applies the deletions at commit time.

Giving the compactor no general filesystem write surface makes "deletion-only" structural rather than disciplinary: the worst case is lost information, never corrupted information. The compactor has access to the dispatching branch's goal, passed through as `parent_goal`, and decides relevance against it.

**Terminal compaction.** When a branch is ready to merge back to its parent, the harness dispatches a compactor off the branch's tip with a goal that instructs it to summarize the branch's work. The compactor writes the summary and marks superseded files for deletion, then merges back into the dispatching branch. The dispatching branch then merges to its parent (§2.6); the parent sees only the compacted tree.

**Intermediate compaction.** Compaction may also run at checkpoints during a branch's execution — not only at termination. Each checkpoint is another compactor dispatch with the same shape: it writes a new numbered summary (`.agent/compactions/<seq>.md`), marks the previous summary for deletion, and merges back. The branch continues running afterward against its updated tree. Terminal compaction is the last checkpoint in the sequence.

The discipline that parent branches see only compacted results — never raw internal state — is preserved. Intermediate compaction just makes the view progressively richer rather than all-or-nothing.

Intermediate compaction triggers are declared in `workflow.yaml` (see §6): by commit count, by elapsed time, or by an explicit `flush` action the agent may call. A branch with no configured trigger compacts only at termination.

While a compactor invocation is in flight, the branch that dispatched it is quiescent — it is awaiting the compactor's `tool_result`, same as any other dispatch. No special interlock is required.

Compaction failures (compactor produces garbage, times out, etc.) fall back to unmerged state and user review, as with any invocation failure.

### 2.8 Goals

Each branch (other than the root conversation) has a **goal**: the stated objective it was dispatched with. The goal is written to `.agent/goal.md` in the branch's worktree at dispatch time and is pinned at the head of the context assembled for every model call on that branch, regardless of position in the message sequence.

A goal is set at dispatch and is not rewritten during execution. If an agent determines its goal is wrong, the expected workflow is:

1. Terminate the current branch (stopped or with an explanatory terminal response).
2. Analyze the failure.
3. Dispatch a new branch with a corrected goal and/or different prompts or workflow.

Rewriting a goal in place is not structurally forbidden but is not a core path — the expected unit of iteration is the branch, not the goal.

The pinned goal resolves the recency-decay problem in deep agent trees where sequence-as-authority (last user message = current order) fails.

### 2.9 Stopped branches

Stops are aggressive. When a stop is issued (by user, by timeout, by cascade from a parent):

1. A cancel marker is written to the branch's state directory.
2. In-flight HTTP requests to provider endpoints are dropped.
3. SIGTERM is sent to all tool subprocesses in the branch and its descendants.
4. Descendants' cancel markers are written, cascading.
5. Branches are left unmerged and flagged `stopped`.

A stopped exchange is terminal: it does not merge back to `main`. The user may purge it, ignore it, or *resume* it — resumption is a new dispatch using the stopped branch's state as context. The new exchange is a distinct branch with its own goal; the stopped branch remains as a ref for retention.

Default retention: 30 days, then tarballed and GC'd.

### 2.10 Retries and failures

A step's commit is written *before* its model call is issued (§2.3). That commit is the exact snapshot the model call was derived from, which makes retry tractable: a failed model call can be reissued against the same state without drift.

- **Retryable provider errors** (transient network failure, 429 rate limit, 5xx) are retried inline with backoff, bounded by a configurable attempt cap. Retries do not produce additional commits — the commit frames the model call, not the individual API call.
- **Non-retryable errors** (400 validation failure, auth failure, impossible-to-satisfy schema) abort the step. The branch is left in the state it held before the model call and flagged for operator attention.
- **Unknown or ambiguous failures** trigger a diagnostic dispatch: a subagent is dispatched off the branch's current commit with a goal describing the failure, access to the branch state and the raw error, and instructions to produce a recommended next action (retry, abort, modify config, escalate).

Tool failures follow the same shape at the tool-executor level, surfaced to the emitting agent as the tool's `tool_result` content. The agent decides whether to retry, ignore, or escalate — this is an ordinary agent decision, not a harness one.

---

## 3. Component Architecture

### 3.1 Disk as Bus

All components communicate through the filesystem. No shared memory, no direct function calls between components. This applies to:

- Harness → provider adapter (request mirrored to disk, adapter reads from stdin; response events mirrored back to disk from adapter stdout — see §4.4).
- Harness → tool execution (tool call record written to disk, executor reads, output streamed to disk).
- Harness → UI: the filesystem is the event stream. The UI watches paths in the conversation repo (git tree, `branches.json`, tool outputs, invocation dirs, `goal.md`, compactions) and re-renders on change. Notification is inotify where available, polling otherwise. `.agent/state/events.log` is retained as an append-only harness-internal bookkeeping log — useful for replay and observability — but it is not the UI's primary read surface.
- UI → Harness: user actions are issued as `lernie <subcommand>` invocations per §3.4. There is no input directory.

**Threads, not processes.** "Worker" and "executor" name roles that run as threads inside the single harness process; the disk contract is not an inter-process bus. Tool subprocesses invoked by the tool executor are genuinely separate processes. Routing inter-role communication through disk — even between threads in the same process — is load-bearing rather than ceremonial: it is what buys inspectability, audit trail, and the single-author-per-file discipline that keeps many concurrent workers from corrupting each other's state.

Notification mechanism: inotify where available, with a polling fallback. Every event write updates a `last_event_ts` sentinel file as a sanity check for consumers to detect missed events.

Consequences:

- Every component is independently restartable (threads respawn; subprocesses are relaunched from disk state).
- Replay from any point is `git checkout <ref>` + re-tail the event log.
- Latency is higher than in-memory IPC. Accepted and compensated by streaming-first UI design.

### 3.2 Components

- **Harness** (permitted synonym: **daemon**). The single program that drives execution: watches for events, spawns branches, runs model calls via the provider adapter layer (§4.4), invokes the tool executor, triggers merges and compactions, updates state. It owns all external↔filesystem interaction on the repo — provider endpoints (via adapter subprocesses), tool subprocesses, git operations. Stateless across restarts — resumes from disk. Any place this document says "the harness does X", it is this component. "Daemon" is allowed as a shorthand; both refer to the same role.
- **Tool executor.** Runs tool subprocesses on behalf of the harness. Streams output to disk atomically (temp path + rename).
- **Provider adapter.** External binary, one per named provider, that owns HTTP, auth, and transient-error retry. Invoked per model call over stdio; non-resident (process per model call, no long-lived state). Contract in §4.4.
- **UI** (permitted synonym: **frontend**). A stateless renderer over the conversation repo. Reads and watches filesystem paths in the repo; issues user actions exclusively as `lernie <subcommand>` invocations per §3.4. Holds no persistent state — every render is a pure function of filesystem state at the current git ref. The UI is pluggable: multiple frontends (desktop GUI, webclient, TUI) may run concurrently against one repo without coordination, because they share nothing but the filesystem and the CLI. Contract in §3.5. "Frontend" is allowed as a shorthand; both refer to the same role.

The harness, the tool executor, and provider adapters share the same disk contract. None share memory. The UI participates in the same disk contract as a read-only consumer.

### 3.3 Tools and skills

Tools and skills are separate primitives. Every tool has a skill; skills can exist without tools.

**Skill.** A directory containing a `SKILL.md` with YAML frontmatter (`name`, `description`) plus markdown instructions, optionally with bundled scripts and reference files. The `description` is always present in agent context; the full `SKILL.md` body is loaded only when the agent elects to use the skill. This is the Anthropic progressive-disclosure convention (`docs/TAXONOMY.md` §4).

A standalone skill (no associated tool) exists to give an agent capability via prompt — recipes, conventions, workflows — without a callable binary.

**Tool.** Composed of three required artifacts:

1. **Binary.** An executable invoked by the harness. In-process tools (dispatch, git ops) are implemented as built-in subcommands invoked the same way as external tools.
2. **JSON schema.** Declares tool call parameters, types, required fields. Required by provider APIs. Either generated from the binary's metadata or hand-authored.
3. **Skill.** A `SKILL.md` describing when and how the tool should be used. Required for every tool.

Tool output contract:

- Tools write output to a temp path and atomic-rename on completion.
- Tools must handle SIGTERM cleanly; partial output is the harness's responsibility to clean up post-kill.
- Tools exceeding a configurable output size threshold trigger automatic dispatch: the raw output is handed to an invocation for parsing, and only that invocation's compacted result reaches the parent step.

### 3.4 CLI as control plane

`lernie` is a single binary with subcommands. Every procedure the harness can start — subagent dispatch (§2.5), compaction (§2.7), verification, auto-dispatch on oversized tool output (§3.3), and any other workflow-invoked procedure (§6) — is reachable through a subcommand of that binary. The CLI is the sole entry point: a procedure invoking another procedure does so by going through the CLI dispatcher, never through in-process function calls, shared memory, or ad-hoc sockets. Subagent dispatch, the canonical case, is `lernie dispatch …`.

This is the invocation counterpart to §3.1. **Disk-as-bus carries state; CLI-as-control-plane carries commands.** Between any two procedures, state flows through the filesystem (§3.1) and invocations flow through the CLI. There is no third channel — no library API surface, no sidechannel. An external caller embedding lernie in another tool uses exactly the same CLI surface the harness uses internally; that symmetry is what lets lernie be a component in another tool rather than a standalone monolith.

Whether a given CLI invocation is dispatched as a subprocess `exec` or as an in-process re-entry into the same argument parser is an implementation detail chosen per-procedure (isolation needs, latency, resource cost). What is invariant is that the procedure's *interface* is the CLI: the same arguments, the same on-disk effects, whether exec'd or in-process. Internal procedures are not permitted to shortcut past the CLI dispatcher via a private function call.

Three consequences fall out:

- **Integration testability.** Every inter-procedure edge is an observable CLI invocation. A test captures the arguments and on-disk effects, asserts on their shape, and replays outputs from fixtures without needing to mock in-process interfaces.
- **Embeddability.** Embedding lernie in another tool is `exec("lernie", args)` with env-var auth. No library port, no shared runtime, no plugin loader.
- **No back-channels.** With disk for state and CLI for commands — and nothing else — operational atomicity is structural: a procedure has no surface with which to back-channel into another. The single-author-per-file discipline (§2.6) and the per-procedure commit boundaries that make replay work are consequences of this, not separate protections layered on top.

"Procedure" here is the term from §6 (subordinate routines invoked by workflow), extended to cover the dispatch and compactor invocations already described in §2.5 and §2.7 — every named operation the harness can start.

### 3.5 UI contract

A **UI** (or **frontend** — same role, §3.2) is any program that presents the conversation repo to a user. The architecture admits multiple frontends concurrently against one repo — a desktop GUI and a webclient rendering the same conversation simultaneously is the default case, not a special one.

The frontend surface is exactly two things, and nothing else:

1. **Filesystem reads.** The frontend reads and watches paths under the conversation repo. The load-bearing paths follow the repo layout (§2.2): the git tree itself, `.agent/state/branches.json`, `.agent/goal.md` per branch, `exchanges/`, `invocations/`, `artifacts/`, `tools/`, `.agent/compactions/`. Notification is inotify where available, polling otherwise (§3.1).
2. **CLI invocations.** The frontend issues user actions by `exec`'ing `lernie <subcommand>`. New prompt, stop, resume, fork-from-history — all are ordinary CLI subcommands per §3.4. There is no separate API surface, no socket, no shared input directory, no library port.

Frontends hold no persistent state. Everything a frontend renders is derived from the filesystem at the current git ref; ephemeral UI state (cursor position, scroll offset, selection) lives in memory only and is discarded on exit. Restart is equivalent to re-reading the repo.

This discipline is what makes pluggability structural rather than aspirational. Two frontends running against one repo cannot corrupt each other because neither writes repo state; both observe the same on-disk ground truth, and both issue commands through the same CLI surface the harness itself uses. Swapping a frontend out is unplugging one reader; adding a second is adding another reader.

---

## 4. Providers, Auth, and Models

### 4.1 Provider abstraction

The taxonomy (`docs/TAXONOMY.md` §2) flags "provider" as one of the field's most overloaded terms, naming three distinct roles: **model creator** (trains the weights), **inference provider** (serves the weights over an API), and **gateway** (unifies multiple inference providers behind one surface). This document uses **provider** in the *inference provider* sense throughout: an (endpoint, auth) pair.

The harness does not speak provider wire protocols directly. Each provider is served by a **provider adapter** (§4.4) — a separate binary invoked as a subprocess per model call. That keeps the harness free of per-vendor HTTP quirks and lets external contributors (corporate users with bespoke SSO, custom retry logic, private model routers) ship adapters without modifying core code. In the vocabulary of this spec, "provider" names the `(endpoint, auth)` pair in config; "provider adapter" names the binary that implements one provider's protocol. They are distinct terms of art and not interchangeable.

Provider config (`providers.yaml`):

```yaml
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth:
      type: api_key
      env: ANTHROPIC_API_KEY
  bedrock:
    endpoint: https://bedrock-runtime.us-east-1.amazonaws.com
    adapter: /opt/corp/lernie-provider-corp-bedrock
    auth:
      type: aws_sigv4
      profile: default
```

The optional `adapter:` key names the binary to invoke; when absent, the harness looks up `lernie-provider-<name>` on `PATH`. The harness itself does not read `endpoint:` or `auth:` — those are the adapter's to interpret. The harness only needs the adapter binary and the env-var names returned by `describe` (§4.4).

### 4.2 Model abstraction

A **model** is (provider, model_id, capabilities). Model lists come from the provider's API where available. Capabilities is an extensible mapping declaring features the harness can rely on. The `models:` block lives alongside `providers:` in `providers.yaml`, since each model is one key away from its (endpoint, auth) pair.

```yaml
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native, prompt_caching, streaming, stop_sequences]
    context_window: 200000
```

Capabilities are code-backed (each capability has a behavior implementation in the harness). Capabilities are extend-only: once declared, they are never removed from the registry, but the set on a given model may shrink if the provider removes support. The loader seeds a known-name registry from the names that appear in this spec; an unknown name on load produces a warning, never an error, so a new provider may declare new capabilities without blocking parsing.

### 4.3 Role-based model assignment

Agent roles specify which model to use. This allows cheap models for compaction and expensive models for the worker.

```yaml
agents:
  worker:
    model: claude-sonnet-4-7
    system_prompt: prompts/worker.md
  compactor:
    model: claude-haiku-4-5
    system_prompt: prompts/compactor.md
```

`system_prompt` is a path relative to `.agent/system/`. Absolute paths and `..` traversals are rejected at load time.

### 4.4 Provider adapters

A **provider adapter** is a binary that implements one provider's wire protocol. The harness invokes the adapter per model call; the adapter owns the HTTP request, auth, and any transient-error retry loop. This makes the provider layer externally extensible — the same externalization pattern already used for tools (§3.3), now applied to the provider boundary.

**Discovery.** By default, the harness looks up `lernie-provider-<name>` on `PATH`, where `<name>` is the provider key from `providers.yaml`. A provider entry may pin a specific binary with `adapter: /abs/path/to/binary`, which is used verbatim. The adapter is located once per model call — there is no long-lived adapter process.

**Subcommands.** Every adapter supports exactly two subcommands:

- `describe` — reads nothing from stdin; writes a single JSON object to stdout:
  ```json
  {
    "name": "anthropic",
    "schema_version": 2,
    "capabilities": ["tool_use_native", "streaming", "prompt_caching"],
    "models": ["claude-sonnet-4-7", "claude-haiku-4-5"],
    "auth_env": ["ANTHROPIC_API_KEY"],
    "endpoint_env": ["LERNIE_PROVIDER_ANTHROPIC_ENDPOINT"]
  }
  ```
  `schema_version` is an integer. The harness rejects unknown major versions at load and refuses to use the adapter — no silent downgrade (see "Decline illegal operations" in PRINCIPLES.md).

- `complete` — reads a JSON request on stdin (Anthropic Messages-API shape is the canonical form; adapters for other providers translate internally) and writes one of two forms to stdout:
  - **Non-streaming:** a single JSON response object.
  - **Streaming:** a JSON Lines event stream — one JSON event per line. Event types are block-oriented to mirror the content-block structure of assistant messages: `message_start`, `content_block_start`, `text_delta`, `tool_use_delta`, `content_block_stop`, `message_stop`. The terminal `message_stop` carries `usage` and `api_calls` (the count of HTTP requests the adapter made — see §2.1 for the model call / API call distinction; retries or streaming reconnects may make this greater than one).

  Streaming vs non-streaming is chosen by a field in the stdin request; the contract is the same binary in both modes.

**Errors.** Adapters report errors in-band, not by exiting non-zero:

- Non-streaming: the top-level JSON object is `{ "type": "error", "kind": "retryable" | "fatal", "http_status": <int|null>, "message": "…", "retry_after_seconds": <int|null> }`.
- Streaming: the terminal event has the same shape with `"type": "error"`.

Exit code 0 means the adapter produced a valid output (including a `type: error` object). Non-zero exit means the adapter itself crashed — the harness treats that as a harness-level fault, not a provider failure, and flags it for operator attention per §2.10. The adapter owns transient-error retry; the harness treats one `complete` invocation as one model call regardless of how many API calls the adapter makes under the hood.

**Cancellation.** The harness sends SIGTERM on stop (§2.9). The adapter must drop any in-flight HTTP request, flush partial state — for streaming, emit a final `message_stop` or `error` event — and exit within 5 seconds. SIGKILL follows if it does not.

**Auth.** Auth lives entirely inside the adapter. The harness forwards the env vars named in `describe.auth_env` into the subprocess environment; everything else — refresh tokens, keychain access, `aws-sso login`, Okta CLI flows — is the adapter's concern. The harness never handles credentials directly and never prompts.

**Endpoint.** Endpoint URLs are opaque to the harness. The adapter declares one or more env var names in `describe.endpoint_env`; the harness sets each to the value of `providers.<name>.endpoint` (verbatim, no parsing) before invoking `complete`. An adapter that omits `endpoint_env` opts out of harness-set endpoints and uses its built-in default. Symmetric in shape with `auth_env`, but `auth_env` propagates values from the harness's environment whereas `endpoint_env` carries values from `providers.yaml` — neither requires the harness to interpret the URL.

**Fit with disk-as-bus (§3.1).** The adapter's stdin and stdout are pipes, but the harness mirrors both to disk under the framing step's commit (`invocations/<id>/request.json`, `invocations/<id>/events.jsonl` or `response.json`). Replay (§3.1, §2.10) works against those files, not against a live adapter process. The pipes are the wire; the disk is the record.

**Schema versioning.** `describe.schema_version` is the adapter's self-declared contract version. The harness keeps a minimum-supported-version constant. A `schema_version` below that is rejected at load. A `schema_version` above that is accepted optimistically — the harness ignores unknown fields. This is the same forward-compatibility discipline used for `providers.yaml` capabilities (§4.2).

---

## 5. Context Assembly

### 5.1 Assembly rules

The context sent to the model is built deterministically from the repo state according to `manifest.yaml`. The manifest declares:

- Inclusion globs and their order.
- Pinned paths (always included, regardless of budget).
- Token budget.
- Overflow policy (truncate oldest, summarize, drop).

```yaml
context:
  pinned:
    - .agent/goal.md
    - .agent/system/prompts/base.md
  include:
    - exchanges/**
    - artifacts/**
    - invocations/*/result.md
  budget_tokens: 150000
  overflow: drop_oldest_exchanges
```

This corresponds to the LangChain write/select/compress/isolate taxonomy (`docs/TAXONOMY.md` §3): **write** = commits; **select** = manifest inclusion; **compress** (here: compact) = compactor; **isolate** = invocation branches.

### 5.2 File path as hint

File paths are preserved in the assembled context as structural hints to the model. The path itself carries information (`exchanges/0042-user.json`, `invocations/a1b2/result.md`) that is cheaper than explicit metadata and often sufficient.

### 5.3 Removal by deletion

Agents reduce their own context by deleting files from the working directory. An agent that no longer needs a 5k-token file `rm`s it; the next context assembly excludes it naturally. Deleted files remain recoverable from git history until compaction squashes them.

---

## 6. Workflow as Configuration

Workflows are declared as event-to-action bindings in `workflow.yaml`. Changing workflow is a config edit, not a code change. "Workflow" here is the Anthropic sense (`docs/TAXONOMY.md` §1): predetermined code paths, as contrasted with LLM-driven agent control flow. Subordinate routines invoked by the workflow (compaction, verification, auto-dispatch on large tool output) are **procedures**.

```yaml
events:
  user_message:
    - spawn_exchange
    - dispatch(worker)
  worker_return:
    - dispatch(verifier)
    - gate_merge_on(verifier.approve)
  verifier_approve:
    - dispatch(compactor)
    - merge
  verifier_reject:
    - "dispatch(worker, with: verifier.feedback)"
  worker_flush:
    - "dispatch(compactor, mode: intermediate)"
  branch_stopped:
    - mark_abandoned
    - notify_ui

  # Per-step hooks, fire on every branch.
  pre_step:
    - <actions run before a step's model call is issued>
  post_step:
    - <actions run after the step's model call returns, before tool execution>
  on_tool_return:
    - <actions run each time a tool call resolves>

compaction:
  intermediate:
    trigger: every_n_commits   # or: every_t_seconds, on_flush
    n: 10
```

Action strings that contain `: ` (named arguments) must be quoted, since YAML otherwise parses them as map entries. Bare actions (no named args) need no quotes.

Actions are implemented in the harness; the workflow declares which run when. The `flush` action emitted by a running agent triggers an intermediate compaction without terminating the branch (§2.7). This is the primary surface for experimentation.

Per-step hooks (`pre_step`, `post_step`, `on_tool_return`) fire on every branch and are the primary extension points for cross-cutting behavior — observability, budget enforcement, cache maintenance, scheduled intermediate compaction triggers. Their handlers typically dispatch subagents or emit log entries rather than modifying the in-flight branch's tree directly; any write still goes through the harness-assigned write-path machinery (§2.5).

---

## 7. UI

### 7.1 Live view

The UI watches the conversation repo filesystem (§3.5) and re-renders from what it observes. Top of the interface shows a git tree view of the current conversation. Clicking a commit navigates to that point. Clicking a branch navigates to that agent.

Live indicators:

- Line-by-line streaming text for model responses in flight.
- Pulsing indicators for tool calls.
- Arrows to sub-branches for active invocations.
- Distinct patterns for model call states: queued, in flight, streaming, terminal.
- Branch termination markers (merged, stopped, conflicted).

### 7.2 History view

Clicking an old commit is read-only by default. Forking from history (creating a new branch from an old commit to explore a counterfactual) is a v1 feature but distinguished in the UI and in branch naming (`fork/<source-ref>/<id>`) so accounting and replay can tell the difference.

### 7.3 Concurrent exchanges

Because every exchange is a fork-merge cycle and nothing touches `main` directly, the user may send a second message before the first resolves. Both are branches off the same base (or the first's parent if the user wants strict sequencing). Merges happen in completion order. This requires no special mechanism beyond the branch invariant.

---

## 8. Metrics and Observability

First-class metrics, written to commit trailers and event log. All counts are reported along four scope axes, which are not interchangeable:

- Tokens per step, per invocation, per exchange, per conversation.
- Model calls per step (always 1), tool calls per step, API calls per model call.
- Cost per step, per invocation, per exchange, per conversation.
- Unmerged branch count per conversation. This is a critical health metric: a ballooning count indicates silent failure somewhere in the merge pipeline. Directly computable as the length of `.agent/state/branches.json` (§2.3).
- Step duration, tool call duration, compaction duration.
- Compaction ratio (tokens before / tokens after).

---

## 9. Testing and Replay

### 9.1 Task suite

A task suite of 50–200 manually constructed tasks with machine-checkable success criteria. Target baseline pass rate: approximately 40%, with failure modes distributed across categories:

- Early termination.
- Scope reduction.
- Skipped tests.
- Hallucinated APIs or facts.
- Error recovery failure.
- Fabricated success claims.
- Context hygiene failures (needle-in-haystack, compaction ratio, prompt injection resistance).

Each category has sufficient tasks (≥10) for statistical movement to be detectable. Each task runs N≥5 times per evaluation.

**Primary metric: pass@1** — the mean per-task pass rate. For each task, compute `#passes / N`; then take the mean across tasks (mean-of-means, with N fixed per task). Report 95% Wilson score intervals on the mean. This captures **reliability**: can the harness solve tasks consistently?

**Secondary metric: pass@5 (any-of-5)** — for each task, did any of 5 runs pass? Report fraction of tasks. This captures **ceiling capability**: what is the harness ever capable of solving? A workflow variant that lifts pass@1 without lifting pass@5 is noise reduction; a variant that lifts both is a real capability gain.

Optimization target is pass@1; pass@5 is tracked to distinguish capability shifts from reliability shifts.

### 9.2 Replay

A run is a conversation repo plus its event log. Tarballing a run is `tar czf run.tar.gz <repo>/`. Replay from a tarball drops the user into exactly that run's state for inspection. Replay is also the long-term archival format: conversations unused for some window are tarballed in place, reducing inode pressure while remaining trivially restorable.

### 9.3 Experiments

Experiments are workflow config variants:

```
experiments/
├── baseline/
│   └── workflow.yaml
├── strict-verifier/
│   └── workflow.yaml
└── parallel-workers/
    └── workflow.yaml
```

An evaluation run is (experiment × suite × N), producing per-task pass@1 and per-category failure breakdowns. A new experiment is a config diff; no code changes.

---

## 10. Schema Versioning

Every conversation repo declares its schema version in `.agent/version`. Old versions remain readable by the harness; migration code is written when a version bumps. Tarballed runs from any prior version must remain inspectable.

---

## 11. Deferred to Future Versions

Named explicitly so they are not rediscovered later:

- Tool sandboxing and capability restriction.
- Multi-user and multi-tenant isolation.
- Distributed execution across machines.
- Secrets management beyond env var injection.
- Sophisticated compaction (rewriting compactors, semantic merging).
- Adversarial compactor defense.
- Cross-conversation memory / shared context.

---

## 12. Milestones

### v0.1 — One model call

**Success criterion:** A single prompt is sent to a provider endpoint, the response is written to disk, and is visible in the conversation repo as a commit. No git branching, no tools, no invocations.

**Shipped shape.** `lernie new <path>` scaffolds a conversation repo from the embedded template; `lernie prompt <repo> <message>` loads `providers.yaml` + `agents.yaml`, resolves the `worker` role, invokes `lernie-provider-<name>` as a subprocess (§4.4), writes `exchanges/<ts>-<short-id>.json` with `user_message` / `assistant_response` / `model_id` / `provider` / `usage` / `stop_reason` / `started_at` / `ended_at`, and commits the file to `main`.

**Exceptions to later invariants, historical.** v0.1 committed directly on `main` rather than via an exchange branch merge (§2.3); retired in v0.2, where `lernie prompt` spawns an `ex/<ts>-<id>` branch and commits the step snapshot before the model call (§2.10), with the response landing as a follow-up commit on the same branch. Merge-back (§2.6) arrives in a subsequent v0.2 step. The earlier `--endpoint` argv pragma was also retired in v0.2: endpoint forwarding now goes through `describe.endpoint_env` per §4.4.

### v0.2 — Git tree

**Success criterion:** An exchange is a branch, completion is a no-ff merge back to `main`, the repo layout matches §2.2. User message → exchange branch → steps as linear commits → compactor (deletion-only, stub is fine) → merge. Unmerged branch count metric available.

### v0.3 — Tools

**Success criterion:** Agent can invoke at least two tools (bash, read_file). Tool calls are commits. Large tool outputs auto-dispatch to a parsing invocation. Tool contract (binary + schema + skill) documented.

### v0.4 — Invocations

**Success criterion:** Agent can dispatch a subagent. Invocation runs in its own worktree and branch. Merge-back flow works end-to-end. Parallel invocations do not corrupt each other's state. Handle-based async works: a dispatch returns immediately with a handle, siblings can be in flight concurrently, and `await(handle)` retrieves the compacted result on a later step (§2.5).

### v0.5 — UI

**Success criterion:** Git tree view, live streaming, pulsing tool indicators, branch-state indicators. Watching the repo filesystem (§3.5) is the only read mechanism; user actions go out as `lernie <subcommand>` invocations.

### v0.6 — Workflow config

**Success criterion:** The `workflow.yaml` surface works. At least one non-baseline workflow variant (e.g., a verifier step) runs end-to-end without code changes.

### v0.7 — Task suite

**Success criterion:** 50 tasks with machine-checkable success criteria. Baseline harness achieves 40% ± 5% pass@1 on the suite (Wilson CI). Per-category failure tagging works.

### v0.8 — Experiments and replay

**Success criterion:** `agent-eval --config <experiment> --suite <suite> --runs N` produces per-task pass@1 and pass@5 with confidence intervals. Any run can be tarballed and replayed. Config changes (prompt edits) deployable without code changes in under 60 seconds end-to-end.

### v1.0

**Success criterion:** All of the above, plus at least one demonstrated workflow variant that beats baseline on at least one failure category by a statistically significant margin on pass@1. This is the proof that the architecture's experimentation surface is actually useful.
