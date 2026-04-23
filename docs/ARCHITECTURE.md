# Agent Harness Spec

**Status:** Draft v0.3
**Scope:** Design specification for a git-backed agent harness with branch-per-dispatch context management.

---

## 1. Overview

This document specifies an agent harness in which conversational context is managed as a git repository. A conversation repo is one root conversation plus all its subagent descendants, materialized as worktrees within a single git repo. Every dispatch creates a branch. Subagent branches merge back on termination; root-conversation branches persist as refs and await a reprompt. Within a branch, steps land as linear commits; a step that itself dispatches spawns a sub-branch off the commit where the dispatch landed. State flows between components through the filesystem (§3.1); commands between procedures flow through the `lernie` CLI (§3.4). There is no third channel — no library API, no in-process sidechannel, no resident broker — and no process holds state across its own termination.

The architecture optimizes for four properties:

1. **Inspectability.** The complete state of any conversation, at any point in its history, is a git ref. Replay, debugging, and counterfactual forking are first-class operations.
2. **Uniformity.** User-to-agent dispatch, agent-to-subagent dispatch, and agent self-reflection all use the same primitive: fork a branch, do work, merge back (or terminate to user, for root conversations). There is no special path for user input.
3. **Testability.** Workflow (prompts, sequencing, context assembly rules) is configuration, not code. Experiments are config diffs, measurable against a task suite.
4. **Regenerability.** Any process can die at any time without losing state. Disk is durable; processes are disposable. Components — harness, tool subprocesses, provider adapters, frontends — restart independently because none hold state across their own termination; `lernie resume <repo>` re-enters the workflow state machine from what the repo records. No process is load-bearing; disk is.

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
| **Conversation** | A single dispatched execution — a harness child that consumes a goal, runs a cycle of steps, and terminates. The *root* conversation is spawned by a user message and terminates to the user's next prompt; *subagent* conversations are spawned by tool-call dispatches from a parent branch and merge back to their parent. | The primitive. All conversations share structural shape (goal, soul, steps, summaries). |
| **Conversation repo** | The git repository containing one root conversation and all its subagent descendants, materialized as sibling worktrees. | One user-facing unit. One repo per root conversation. |
| **Exchange** | UX label for a root conversation — one initiated by a user message and terminating to user re-prompt. | Non-structural alias for the root case; the underlying primitive is a conversation. |
| **Step** | One model call and the tool calls it emits. | Child of a conversation; structurally bounded by the model call, not by tool completion. Lands as linear commits within the conversation's branch. |
| **Model call** | One execution of a model to produce output. | Atomic. The defining event of a step; each step has exactly one. |
| **Tool call** | The model's structured request to invoke a named tool. | Emitted by a model call; structural child of its emitting step (even if it resolves temporally during a later step). |
| **API call** | One HTTP request to a provider endpoint. | Implementation detail of a model call (usually 1:1; streaming or retries may make it 1:many). |
| **Dispatch** | The event that spawns a child conversation with a goal. Two forms: a user message (spawns a root conversation) and a tool call targeting a subagent (spawns a subagent conversation off the commit where the dispatch landed on the parent branch). | Creates a branch. |
| **Branch** | The git container for a conversation. | Every conversation has exactly one branch; every dispatch creates one. |
| **Goal** | The stated objective handed to a conversation at dispatch. | One per conversation; not rewritten during execution; pinned at the head of context for every model call on the branch (§2.8). |
| **Soul** | The system prompt handed to a conversation at dispatch, drawn from the conversation repo's `souls/<role>.md` and overwritten on the branch's dispatch commit. | One per conversation; composed into every model call on the branch as the system message. |

**Banned usage:** "call" without a qualifier (use model call, tool call, or API call); "turn" (incompatible vendor meanings — see `docs/TAXONOMY.md` §1); "session" (underdefined — `docs/TAXONOMY.md` §3); "compression" (reserved for model-weights quantization — the harness operation is compaction); **"invocation"** as a structural unit (retired in favor of "conversation"; general-field usage such as "model invocation" ≈ model call is acceptable informally, but not as a structural term of art in this spec); **"exchange"** as a structural term (demoted to UX label per the table above — structurally, an exchange is a root conversation).

**Other terms of art not defined in this document or in `docs/TAXONOMY.md` require explicit definition or user approval before use.**

### 2.2 The Conversation Repo

A **conversation repo** is the on-disk materialization of one root conversation plus all its subagent descendants. It lives at `<harness-root>/conversations/<root-id>/`. The harness root defaults to `~/.lernie/` and is overridable via `LERNIE_HOME` (used for parallel testing, alternate installs, and sandboxed replay).

The whole tree is one git repository. The root conversation occupies the primary checkout at `root/`; every subagent conversation occupies a linked worktree at a sibling directory whose name encodes its descent from the root. Repos are created by copying a versioned agent profile from `<harness-root>/agents/<profile>/`. Repos are never pushed to a remote.

Directory layout:

```
<harness-root>/conversations/<root-id>/
├── manifest.yaml                 # context assembly rules (role-keyed)
├── workflow.yaml                 # event → action bindings (frozen copy)
├── providers.yaml                # role → (provider, model) mapping (frozen copy)
├── version                       # schema version, integer
├── souls/                        # system prompts by role (copied from the agent profile)
│   ├── worker.md
│   └── compactor.md
├── root/                         # primary worktree; .git lives here
│   ├── .git/
│   ├── .gitattributes            # merge=ours rules for goal.md, soul.md, summary/**
│   ├── goal.md                   # branch-scoped — this conversation's goal
│   ├── soul.md                   # branch-scoped — this conversation's system prompt
│   ├── summary/NNN.md            # branch-scoped — this conversation's compactions
│   ├── descriptions/             # tool + skill descriptions (committed on main; inherited)
│   ├── skills/                   # loaded skill content (branch-scoped; compactor may prune)
│   └── steps/<conv-id>/NNN/
│       ├── request.json
│       ├── response.json
│       └── tools/                # tool-call artifacts for this step
├── <a>-<b>/                      # subagent conversation (linked worktree; .git is a pointer file)
│   └── … same shape as root/ …
├── <a>-<b>-<c>/                  # sub-subagent; hierarchy encoded in name, NOT in filesystem
│   └── …
└── …
```

**Control plane vs data plane.** The files at the conversation-repo root (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `souls/`, `version`) are **control** — the harness reads them to decide what to do — and live outside every worktree. Everything inside a worktree is **data** — it is composed into that conversation's prompt at model-call time. This is load-bearing: context assembly has no exclusion list, because nothing that isn't context lives in a worktree.

**Frozen-copy bootstrap.** Creating a repo is `cp -r <harness-root>/agents/<profile>/* <harness-root>/conversations/<root-id>/` plus `git init` plus the initial commit. All control files are frozen snapshots at that point. Subsequent changes to the global agent profile, to `<harness-root>/workflows/`, `<harness-root>/skills/`, or `<harness-root>/tools/` do not propagate into existing repos. Reproducibility and portability win over live update. (Auth credentials and endpoint URLs in `<harness-root>/providers.yaml` are *not* copied — those rotate, and the per-repo `providers.yaml` carries only the role → (provider name, model id) pointer; the harness resolves endpoint and auth against its global `providers.yaml` at call time.)

**Sibling worktrees, not nested.** Subagent worktrees are named by their full hyphenated descent from the root (`<a>-<b>-<c>-…`) and live as *siblings* of `root/`, never as subdirectories of their parent's worktree. Git does not permit nested working trees; the sibling layout is how the primitive's uniformity survives contact with git mechanics.

**Branch-scoped vs main-committed.** `goal.md`, `soul.md`, and `summary/` are written per-branch and pinned by `.gitattributes merge=ours` — on merge-back from a subagent, the parent's versions are retained verbatim, so a subagent's private goal can never clobber its parent's goal. `descriptions/` is committed once on `main` at conversation creation and inherited by every branch via git. `skills/` is branch-scoped (added as skills are loaded; removed by the compactor when no longer needed). `steps/` is branch-scoped but **namespaced by conversation id** — every conversation commits its steps under `steps/<conv-id>/NNN/`, so on merge-back the subagent's `steps/<sub-id>/` tree lands alongside the parent's `steps/<parent-id>/` tree with no filename collision.

Read-only from the user's perspective under normal operation. The harness is the only writer. The user interacts via the UI, which produces events that the harness translates into commits.

### 2.3 Branches and the branch invariant

A **branch** is the git container for a single conversation. One invariant:

> **Every dispatch creates a branch.** Spawn the branch, do the work, compact, merge back (or, in the root case, terminate to the user). This pattern is uniform whether the dispatch is a user message or a subagent-targeting tool call.

Branch naming tracks the worktree directory. The root conversation runs on `main`; every subagent conversation's branch is its full hyphenated descent (`<a>-<b>`, `<a>-<b>-<c>`, etc.). There is no `ex/` or `inv/` prefix — the hierarchy in the name is self-describing.

Within a branch, **steps are linear commits** — one commit per step, carrying that step's model call and the tool calls it emitted. Steps are not their own branches. New branches appear only at dispatch boundaries.

Nothing commits directly to `main` except the conversation-repo's initial snapshot (the frozen bootstrap, §2.2). All subsequent data on `main` arrives via merge from a completed root-conversation dispatch — i.e., a user-message exchange.

> **Historical.** v0.1 committed exchanges directly on `main` without branching; v0.2 introduced branching with separate `ex/*` (exchange) and `inv/*` (invocation) branch prefixes under an `.agent/`-rooted layout. v0.3 unifies both under the conversation primitive (§2.1): directory splits collapse, branch names drop their prefixes, and "invocation" retires as a structural term in favor of "conversation."

**Branch lifecycle** (identical across root conversations and subagent conversations, up to the merge/terminate distinction at step 5):

1. **Spawn.** The dispatch creates a new branch off the parent's current commit. A linked worktree is allocated at `<conv-repo>/<full-hyphenated-descent>/`.
2. **Dispatch commit.** The harness overwrites `goal.md` and `soul.md` in the worktree with this conversation's goal and the chosen role's soul from `<conv-repo>/souls/<role>.md`, then commits. This is the first commit on the new branch. The overwritten files are excluded from future merge-back via `.gitattributes` (§2.6).
3. **Work.** The agent runs its loop. Each step commits under `steps/<this-conv-id>/NNN/`. A step that emits a subagent-targeting tool call spawns a sub-branch off that commit.
4. **Completion.** A terminal event: final response, stop, timeout.
5. **Merge (subagent) or terminate (root).**
   - *Subagent:* the compactor produces a summary; the branch is rebased onto the current parent tip and merged `--no-ff`. `.gitattributes` retains the parent's `goal.md`, `soul.md`, and `summary/` tree; the subagent's `steps/<sub-id>/` and any explicit exports cross up.
   - *Root:* no merge. The conversation's branch persists as a ref; the UI shows it. The user reprompts by issuing a new dispatch that consumes the branch's state as context.
6. **Cleanup.** Completed branches are retained as refs for a retention window (default 30 days) and GC'd thereafter. Worktrees are torn down on completion.

Branching is cheap — local git operations on disk — but it is not per-step. A long conversation produces many commits on one branch and only spawns sub-branches when it actually dispatches.

Compaction may also run *during* a branch's execution, not only at termination; see §2.7.

**Step on-disk layout.** Each step lives in its own directory under `steps/<conv-id>/<NNN>/`. `<NNN>` is zero-padded 3-digit and 1-indexed, so step dirs sort lexically. `<conv-id>` is the owning conversation's id — namespacing steps this way is what lets a subagent's step tree merge into its parent's worktree without filename collision (§2.2).

Two core files land per step:

- `request.json` — the model call's input. Committed **before** the model call (§2.10), so the commit's tree is the exact state the model read from; retry replays this snapshot without drift.
- `response.json` — the normalized model-call output (assistant text, `model_id`, `provider`, `usage`, `stop_reason`, `started_at`, `ended_at`). Landed as a *follow-up commit* on the same branch — not an amend of the snapshot — so the snapshot's tree continues to reflect pre-model-call state.

Tool calls emitted by a step extend the step's dir with `tools/<tool-id>/…` rather than creating new step dirs, preserving "one step = one model call" (§2.1).

The rich per-step tree is branch-life state. Terminal compaction (§2.7) writes a signal-preserving summary and marks raw step dirs for deletion, leaving the parent's view (post-merge) minimal.

**Unmerged-branch tracking.** Git's ref database is the tracking. Subagent conversations that should have merged back but didn't are readily enumerable — any non-`main` ref that is not merged into its parent indicates a stalled or failed pipeline. The §8 unmerged-branch-count health metric is read directly from `git branch` (PRINCIPLES.md "Single source of truth"); see §8 for the specific form.

### 2.4 Exchanges

An **exchange** is the UX label for a root conversation — one initiated by a user message. It is not a separate structural primitive; structurally, it is a conversation whose parent is the user rather than another conversation. The label survives because users think in exchanges ("the conversation I just had with the agent starts with my message and ends with the final response"), and the UI surfaces that framing. Every architectural property of a conversation applies equally to an exchange.

An exchange that is interrupted before a terminal response is terminal by virtue of the stop itself: like any root conversation, it does not merge back to `main`; it may be resumed by a new dispatch using the stopped branch's state as context (see §2.9).

Multiple concurrent exchanges are supported: a user may send a second message before the first resolves. Each is its own root conversation, its own branch. No special mechanism beyond the branch invariant is required.

### 2.5 Dispatch

A **dispatch** (§2.1) is the event that spawns a child conversation with a goal. Two forms:

- **User-message dispatch.** A user sends a message to the root of a conversation repo. This spawns a root conversation — an exchange (§2.4) in UX terms.
- **Tool-call dispatch.** A running conversation emits a tool call targeting a subagent. This spawns a subagent conversation off the commit where the dispatch landed on the parent branch. From the subagent's perspective, its parent is indistinguishable from a user; the user-message and tool-call forms are the same primitive.

This symmetry is load-bearing:

- The same code path handles user-initiated and agent-initiated dispatches.
- Verifier agents, compactor agents, and adversarial critics are subagent conversations with different goals.
- Parallel exploration (N workers on the same task) is N parallel dispatches from the same step.

The unification is not ergonomic sugar; it falls out of the git-oriented architecture. Because state flows through branches and results land through merges (§2.6), any operation shaped as "spawn work with a goal, run, return a result" collapses to the same primitive. The harness does not ship a separate framework for compaction, verification, or auto-parsing of oversized tool output — each is a subagent conversation with a different goal and toolset, dispatched the same way (§3.4's `lernie dispatch …`). A new procedure earns its place by collapsing onto this primitive or by introducing one that is genuinely new; it does not sit in parallel with something it almost is. This is the concrete instantiation of the "One obvious path" principle in `docs/PRINCIPLES.md`.

**Every tool returns synchronously.** Provider APIs (Anthropic Messages, OpenAI Responses, Gemini) require each `tool_use` block emitted by a step to be matched by a `tool_result` block in the immediately following user message. The harness honors that invariant: a step's next model call is issued only when every tool call it emitted has produced a `tool_result`. Partial-result reprompts are not attempted — they are rejected at the wire.

**Async work uses handles.** Long-running tools — including dispatch — return immediately with a handle (`{status: in_progress, handle: <id>}`) as their `tool_result`. The agent retrieves the actual outcome later via a separate `await(handle)` or `check(handle)` tool, whose return value is a `tool_result` on a later step. Parallelism is expressed by issuing several dispatches in one step and awaiting them in subsequent steps as results come in. The per-step "one result per tool_use" shape stays intact; the asynchrony rides on the handle. Dispatch is a tool like any other: this is how the symmetry between inline tool calls and subagent conversations is preserved without inventing a second control path.

Tool calls targeting non-subagent tools commit their records on the emitting branch as part of the step. Dispatch tool calls spawn subagent-conversation branches off the commit where the dispatch landed; the `tool_result` the parent step observes is the handle. When the subagent terminates and merges back, its compacted output is what `await(handle)` returns on the next step. Parallel dispatches from a single step spawn N sibling subagent-conversation branches off that same commit.

Subagent conversations are expected to terminate and merge before their parent branch does.

The harness assigns write paths per tool call and per subagent conversation, so two sibling branches never target the same file. This is a structural guarantee (enforced by the tool executor and the dispatch primitive), not a convention, which is what makes the merge protocol (§2.6) conflict-free by construction.

### 2.6 Merges

Merges are always no-fast-forward and conflict-free by construction. Root conversations do not merge (§2.3 step 5); subagent conversations do. The merge protocol for a subagent:

1. Subagent conversation completes.
2. Terminal compaction runs (if warranted — see §2.7): a compactor subagent is dispatched off the subagent's tip and merges back into the subagent, leaving its tree in compacted form.
3. Harness rebases the compacted diff onto the current parent tip (which may have advanced).
4. If rebase succeeds: merge with `--no-ff`.
5. If rebase conflicts: the subagent branch is marked conflicted, left unmerged, flagged for operator attention. This indicates a harness defect — two branches were given overlapping write paths, violating the guarantee in §2.5 — and is tracked accordingly.

**The `merge=ours` discipline.** Three categories of file are pinned to the parent on every merge-back via a `.gitattributes` file committed on `main`:

```
goal.md     merge=ours
soul.md     merge=ours
summary/**  merge=ours
```

These files are all branch-scoped: each conversation writes its own `goal.md` and `soul.md` on the dispatch commit (§2.3 step 2) and its own `summary/NNN.md` as compactions happen (§2.7). Without the `merge=ours` driver, every subagent merge-back would clobber the parent's goal, soul, and in-flight summaries with the subagent's — which is precisely wrong, since the parent resumes with *its* goal, not the subagent's. With it, the subagent's versions stay on the subagent branch (visible in history for provenance, not reflected in the parent's post-merge state).

`steps/<sub-id>/` is *not* merge=ours — the subagent's step records do cross up into the parent, landing alongside the parent's `steps/<parent-id>/` tree. Namespacing by conversation id (§2.3) is what keeps this collision-free.

### 2.7 Compaction

Compaction (not compression — the term is reserved for model-weights quantization in the taxonomy) is the process of producing a signal-preserving, minimal version of a branch's work for consumption by the parent. A **compactor** is a subagent that performs compaction. It has no privileged position in the architecture: compactors are dispatched like any other subagent conversation, run on their own branches, and merge back through the normal protocol (§2.6). What distinguishes a compactor is its goal (produce a summary of the dispatching branch's work) and its toolset.

Compactor toolset (v1):

- **`write_summary(content)`** — writes the compacted summary file on the compactor branch.
- **`mark_for_deletion(path)`** — nominates a file on the compactor branch for removal. The harness applies the deletions at commit time.

Giving the compactor no general filesystem write surface makes "deletion-only" structural rather than disciplinary: the worst case is lost information, never corrupted information. The compactor has access to the dispatching branch's goal, passed through as `parent_goal`, and decides relevance against it.

**Terminal compaction.** When a branch is ready to merge back to its parent, the harness dispatches a compactor off the branch's tip with a goal that instructs it to summarize the branch's work. The compactor writes a final summary and marks superseded files for deletion, then merges back into the dispatching branch. The dispatching branch then merges to its parent (§2.6); the parent sees only the compacted tree.

**Intermediate compaction.** Compaction may also run at checkpoints during a branch's execution — not only at termination. Each checkpoint is another compactor dispatch with the same shape: it writes a new numbered summary (`summary/<seq>.md` on the running branch) and the previous summary may be marked for deletion. The branch continues running afterward against its updated tree. Terminal compaction is the last checkpoint in the sequence.

Intermediate summaries are explicitly *not* propagated to the parent on merge-back — they exist to help the branch manage its own context window, not to produce parent-visible history. This is enforced by the `merge=ours` rule on `summary/**` (§2.6): a subagent's `summary/NNN.md` files stay on the subagent branch. What the parent receives from the subagent is whatever the subagent's terminal compactor wrote into the `tool_result` channel (via the handle-resolution path in §2.5), not the intermediate in-branch summaries. The discipline "parent branches see only compacted results, never raw internal state" (and now: never intermediate compacted state either) is preserved end to end.

Intermediate compaction triggers are declared in `workflow.yaml` (§6): by commit count, by elapsed time, or by an explicit `flush` action the agent may call. A branch with no configured trigger compacts only at termination.

While a compactor is in flight, the branch that dispatched it is quiescent — it is awaiting the compactor's `tool_result`, same as any other dispatch. No special interlock is required.

Compaction failures (compactor produces garbage, times out, etc.) fall back to unmerged state and user review, as with any subagent failure.

### 2.8 Goals

Every conversation has a **goal**: the stated objective it was dispatched with. The goal is written to `goal.md` at the root of the branch's worktree (alongside `soul.md`) on the dispatch commit (§2.3 step 2) and is pinned at the head of the context assembled for every model call on that branch, regardless of position in the message sequence. On merge-back, the `merge=ours` rule (§2.6) retains the parent's goal — a subagent's goal never overwrites its parent's.

A goal is set at dispatch and is not rewritten during execution. If an agent determines its goal is wrong, the expected workflow is:

1. Terminate the current branch (stopped or with an explanatory terminal response).
2. Analyze the failure.
3. Dispatch a new conversation with a corrected goal and/or different prompts or workflow.

Rewriting a goal in place is not structurally forbidden but is not a core path — the expected unit of iteration is the conversation, not the goal.

The pinned goal resolves the recency-decay problem in deep agent trees where sequence-as-authority (last user message = current order) fails.

### 2.9 Stopped branches

Stops are aggressive. When a stop is issued (by user, by timeout, by cascade from a parent):

1. A cancel marker is written to the branch's state directory.
2. In-flight HTTP requests to provider endpoints are dropped.
3. SIGTERM is sent to all tool subprocesses in the branch and its descendants.
4. Descendants' cancel markers are written, cascading.
5. Branches are left unmerged and flagged `stopped`.

A stopped root conversation is terminal: like any root conversation, it does not merge back to `main`. A stopped subagent conversation is also terminal: it does not merge back to its parent, and the parent's `await(handle)` resolves to a `stopped` status. The user may purge a stopped branch, ignore it, or *resume* it — resumption is a new dispatch using the stopped branch's state as context. The new conversation is a distinct branch with its own goal; the stopped branch remains as a ref for retention.

Default retention: 30 days, then tarballed and GC'd.

### 2.10 Retries and failures

A step's commit is written *before* its model call is issued (§2.3). That commit is the exact snapshot the model call was derived from, which makes retry tractable: a failed model call can be reissued against the same state without drift.

- **Retryable provider errors** (transient network failure, 429 rate limit, 5xx) are retried inline with backoff, bounded by a configurable attempt cap. Retries do not produce additional commits — the commit frames the model call, not the individual API call.
- **Non-retryable errors** (400 validation failure, auth failure, impossible-to-satisfy schema) abort the step. The branch is left in the state it held before the model call and flagged for operator attention.
- **Unknown or ambiguous failures** trigger a diagnostic dispatch: a subagent conversation is dispatched off the branch's current commit with a goal describing the failure, access to the branch state and the raw error, and instructions to produce a recommended next action (retry, abort, modify config, escalate).

Tool failures follow the same shape at the tool-executor level, surfaced to the emitting agent as the tool's `tool_result` content. The agent decides whether to retry, ignore, or escalate — this is an ordinary agent decision, not a harness one.

---

## 3. Component Architecture

### 3.1 Disk as Bus

All components communicate through the filesystem. No shared memory, no direct function calls between components. This applies to:

- Harness → provider adapter (request mirrored to disk, adapter reads from stdin; response events mirrored back to disk from adapter stdout — see §4.4).
- Harness → tool execution (tool call record written to disk, executor reads, output streamed to disk).
- Harness → UI: the filesystem is the event stream. The UI watches paths in the conversation repo (git refs, worktree contents including `goal.md`, `soul.md`, `summary/`, `steps/`, and the conversation-repo root's control files) and re-renders on change. Notification is inotify where available, polling otherwise.
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

**Skill.** A directory containing a `SKILL.md` with YAML frontmatter (`name`, `description`) plus markdown instructions, optionally with bundled scripts and reference files. Skill source directories live globally at `<harness-root>/skills/<name>/` and are referenced from the conversation repo by two mechanisms:

- **Description-always.** Every available skill's `SKILL.md` frontmatter (name + description) is committed to the conversation repo's `main` at creation time under `descriptions/skills/`, where it is inherited by every branch via git and composed into the context on every model call. This is the Anthropic progressive-disclosure convention (`docs/TAXONOMY.md` §4).
- **Body-on-demand.** When an agent elects to use a skill, the harness copies the skill directory into the current branch's worktree at `skills/<name>/`. From that point on, the skill body is part of the conversation's data and is composed into the context (§5.1). The compactor may `mark_for_deletion` a skill directory when it is no longer needed — next context assembly sees the branch without it.

Copying (not symlinking) is deliberate. It is the same portability discipline as the rest of the repo (§2.2): a skill that lives in the worktree is self-contained and survives the global skill directory changing or disappearing. Disk cost is trivial.

A standalone skill (no associated tool) exists to give an agent capability via prompt — recipes, conventions, workflows — without a callable binary.

**Tool.** Composed of three required artifacts:

1. **Binary.** An executable invoked by the harness. In-process tools (dispatch, git ops) are implemented as built-in subcommands invoked the same way as external tools. Tool binaries live globally (at `<harness-root>/tools/` or on `PATH`); the harness executes them — they do not move into the conversation repo.
2. **JSON schema.** Declares tool call parameters, types, required fields. Required by provider APIs. Either generated from the binary's metadata or hand-authored. Schemas are committed to the conversation repo under `descriptions/tools/` at creation time, inherited via git, and composed into the context.
3. **Skill.** A `SKILL.md` describing when and how the tool should be used. Required for every tool; follows the skill lifecycle above.

Tool output contract:

- Tools write output to a temp path and atomic-rename on completion.
- Tools must handle SIGTERM cleanly; partial output is the harness's responsibility to clean up post-kill.
- Tools exceeding a configurable output size threshold trigger automatic dispatch: the raw output is handed to a subagent conversation for parsing, and only that subagent's compacted result reaches the parent step.

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

1. **Filesystem reads.** The frontend reads and watches paths under the conversation repo. The load-bearing paths follow the repo layout (§2.2): the git tree itself (refs, commits, objects — branch state is read from `refs/heads/` per §2.3), the conversation-repo root's control files (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `souls/`), and each branch's worktree contents (`goal.md`, `soul.md`, `summary/`, `steps/`, `descriptions/`, `skills/`). Notification is inotify where available, polling otherwise (§3.1).
2. **CLI invocations.** The frontend issues user actions by `exec`'ing `lernie <subcommand>`. New prompt, stop, resume, fork-from-history — all are ordinary CLI subcommands per §3.4. There is no separate API surface, no socket, no shared input directory, no library port.

Frontends hold no persistent state. Everything a frontend renders is derived from the filesystem at the current git ref; ephemeral UI state (cursor position, scroll offset, selection) lives in memory only and is discarded on exit. Restart is equivalent to re-reading the repo.

This discipline is what makes pluggability structural rather than aspirational. Two frontends running against one repo cannot corrupt each other because neither writes repo state; both observe the same on-disk ground truth, and both issue commands through the same CLI surface the harness itself uses. Swapping a frontend out is unplugging one reader; adding a second is adding another reader.

---

## 4. Providers, Auth, and Models

### 4.1 Provider abstraction

The taxonomy (`docs/TAXONOMY.md` §2) flags "provider" as one of the field's most overloaded terms, naming three distinct roles: **model creator** (trains the weights), **inference provider** (serves the weights over an API), and **gateway** (unifies multiple inference providers behind one surface). This document uses **provider** in the *inference provider* sense throughout: an (endpoint, auth) pair.

The harness does not speak provider wire protocols directly. Each provider is served by a **provider adapter** (§4.4) — a separate binary invoked as a subprocess per model call. That keeps the harness free of per-vendor HTTP quirks and lets external contributors (corporate users with bespoke SSO, custom retry logic, private model routers) ship adapters without modifying core code. In the vocabulary of this spec, "provider" names the `(endpoint, auth)` pair in config; "provider adapter" names the binary that implements one provider's protocol. They are distinct terms of art and not interchangeable.

**Two-file config split.** Provider configuration is split by lifetime and scope:

- **Global** (`<harness-root>/providers.yaml`). Endpoint URLs, auth env var names, adapter binary overrides, retry knobs. Shared across all conversation repos; rotates with key rollover and infrastructure changes.
- **Per-repo** (`<conv-repo>/providers.yaml`). Role → (provider-name, model-id) mapping only. Frozen at conversation creation (§2.2); governs which model this conversation's roles dispatch to for the rest of its life.

This split is what makes frozen-bootstrap repos portable without also freezing credentials. A conversation repo references providers by name and picks up the current endpoint/auth at call time; rotating an API key in the global file immediately affects in-flight conversations (correctly).

Global provider config shape:

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

The optional `adapter:` key names the binary to invoke; when absent, the harness looks up `lernie-provider-<name>` at `<harness-root>/adapters/` (installed by `make install`) before falling back to `PATH`. The harness itself does not read `endpoint:` or `auth:` — those are the adapter's to interpret. The harness only needs the adapter binary and the env-var names returned by `describe` (§4.4).

### 4.2 Model abstraction

A **model** is (provider, model_id, capabilities). Model lists come from the provider's API where available. Capabilities is an extensible mapping declaring features the harness can rely on. The `models:` block lives alongside `providers:` in the global `<harness-root>/providers.yaml`, since each model is one key away from its (endpoint, auth) pair.

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

Agent roles specify which model to use. This allows cheap models for compaction and expensive models for the worker. The role → model mapping lives in the conversation repo's `providers.yaml` (frozen at creation, §2.2):

```yaml
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-4-7
  compactor:
    provider: anthropic
    model: claude-haiku-4-5
```

Each role's system prompt is read from `<conv-repo>/souls/<role>.md` by convention — there is no per-role path override, and no freeform path field to validate. At dispatch time the harness copies the appropriate soul to the new branch's `soul.md` (§2.3 step 2). Provider endpoint and auth are resolved at call time against the global `<harness-root>/providers.yaml` — the per-repo file carries only the (provider-name, model-id) pointer.

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
  - **Non-streaming:** a single JSON response object. See **Response shape (non-streaming)** below.
  - **Streaming:** a JSON Lines event stream — one JSON event per line. Event types are block-oriented to mirror the content-block structure of assistant messages: `message_start`, `content_block_start`, `text_delta`, `tool_use_delta`, `content_block_stop`, `message_stop`. The terminal `message_stop` carries `usage` and `api_calls` (the count of HTTP requests the adapter made — see §2.1 for the model call / API call distinction; retries or streaming reconnects may make this greater than one). See **Response shape (streaming)** below.

  Streaming vs non-streaming is chosen by a field in the stdin request; the contract is the same binary in both modes.

  Adapters MAY accept an optional `--request <path>` argv flag as an alternative to stdin; when set, the adapter reads the request JSON from that file and invocation semantics are identical to the stdin path. The flag is additive — adapters are not required to implement it, and the harness currently always uses stdin. It exists so deterministic replay against the on-disk `steps/<conv-id>/<NNN>/request.json` (§2.3, §2.10, §3.1) needs no shell redirect. A file-open failure on `--request` is an adapter-side fault (non-zero exit, per **Errors** below), not an in-band provider error.

**Response shape (non-streaming).** The response object is the Anthropic Messages-API wire shape (the body of a `POST /v1/messages` response at <https://docs.anthropic.com/en/api/messages>). Non-Anthropic adapters translate their provider's native response into this shape before writing it. Required top-level fields:

- `id` (string) — opaque message id. Adapters backing providers that do not mint one MUST synthesize a stable value.
- `model` (string) — the model that actually produced the response (may differ from the requested model if the provider routed).
- `stop_reason` (string) — the Anthropic wire vocabulary for this field, round-tripped verbatim. Unknown values are accepted (forward-compat).
- `content` (array of content blocks) — each block is `{"type": <str>, ...}`. Consumers at v0.1 handle the `text` block (`{"type":"text","text":<str>}`); other block types parse without error.
- `usage` (object) — MUST include `input_tokens` (integer) and `output_tokens` (integer). Prompt-caching fields, when present, MUST use Anthropic's native names `cache_creation_input_tokens` and `cache_read_input_tokens`; adapters MUST NOT rename them (no `cache_write_tokens` / `cache_read_tokens`).

`api_calls` is NOT a field of the non-streaming response object: the harness treats one `complete` invocation as one model call regardless of how many HTTP calls the adapter made internally. (The streaming `message_stop` does carry `api_calls` — see below — because the harness cannot otherwise observe retry fan-out mid-stream.)

Unknown top-level fields are accepted and ignored (forward-compat, mirroring `describe.schema_version`). The top-level `type` field is reserved: `"error"` signals the in-band error object (see **Errors** below); any other value is undefined and MUST NOT be produced. In particular, there is no `"type": "message"` wrapper — the response object itself is the message.

**Response shape (streaming).** Event names (`text_delta`, `tool_use_delta`, etc.) are an adapter-contract shape, not Anthropic's wire streaming shape — adapters translate. The terminal `message_stop`:

- `usage` — same object shape as non-streaming (Anthropic-native field names, including cache fields when present).
- `api_calls` (integer, ≥1) — REQUIRED. Count of HTTP requests the adapter made under this `complete` invocation; accounts for retries and streaming reconnects.

**Errors.** Adapters report errors in-band, not by exiting non-zero:

- Non-streaming: the top-level JSON object is `{ "type": "error", "kind": "retryable" | "fatal", "http_status": <int|null>, "message": "…", "retry_after_seconds": <int|null> }`.
- Streaming: the terminal event has the same shape with `"type": "error"`.

Exit code 0 means the adapter produced a valid output (including a `type: error` object). Non-zero exit means the adapter itself crashed — the harness treats that as a harness-level fault, not a provider failure, and flags it for operator attention per §2.10. The adapter owns transient-error retry; the harness treats one `complete` invocation as one model call regardless of how many API calls the adapter makes under the hood.

**Cancellation.** The harness sends SIGTERM on stop (§2.9). The adapter must drop any in-flight HTTP request, flush partial state — for streaming, emit a final `message_stop` or `error` event — and exit within 5 seconds. SIGKILL follows if it does not.

**Auth.** Auth lives entirely inside the adapter. The harness forwards the env vars named in `describe.auth_env` into the subprocess environment; everything else — refresh tokens, keychain access, `aws-sso login`, Okta CLI flows — is the adapter's concern. The harness never handles credentials directly and never prompts.

**Endpoint.** Endpoint URLs are opaque to the harness. The adapter declares one or more env var names in `describe.endpoint_env`; the harness sets each to the value of `providers.<name>.endpoint` (verbatim, no parsing) before invoking `complete`. An adapter that omits `endpoint_env` opts out of harness-set endpoints and uses its built-in default. Symmetric in shape with `auth_env`, but `auth_env` propagates values from the harness's environment whereas `endpoint_env` carries values from `providers.yaml` — neither requires the harness to interpret the URL.

**Fit with disk-as-bus (§3.1).** The adapter's stdin and stdout are pipes, but the harness mirrors both to disk under the framing step's commit (`steps/<conv-id>/<NNN>/request.json`, `steps/<conv-id>/<NNN>/events.jsonl` or `response.json`). Replay (§3.1, §2.10) works against those files, not against a live adapter process. The pipes are the wire; the disk is the record.

**Schema versioning.** `describe.schema_version` is the adapter's self-declared contract version. The harness keeps a minimum-supported-version constant. A `schema_version` below that is rejected at load. A `schema_version` above that is accepted optimistically — the harness ignores unknown fields. This is the same forward-compatibility discipline used for `providers.yaml` capabilities (§4.2).

---

## 5. Context Assembly

### 5.1 The worktree invariant

**Everything inside a branch's worktree is composed into that conversation's prompt.** This is the load-bearing invariant of context assembly. There is no exclusion list, no filter, no "this path is for the harness only." Data lives in the worktree; control lives at the conversation-repo root (§2.2). The invariant is what lets `manifest.yaml` be a *sequencing and budget* file rather than an *inclusion* file — the question manifest answers is "in what order and under what budget," not "which things."

Consequences:

- Agents curate their own context by `rm` (§5.3). The primitive they need already exists in the filesystem.
- The compactor's `mark_for_deletion` operates on worktree paths and takes effect on the next assembly.
- Control-plane files (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `souls/`, etc.) sit at the conversation-repo root — *outside* every worktree — and are never composed. This is structural, not disciplinary: the path is not under the worktree root, so it cannot be included.

### 5.2 Assembly rules

The manifest (`<conv-repo>/manifest.yaml`, role-keyed) declares ordering, pinning, budget, and overflow policy:

```yaml
roles:
  worker:
    pinned:
      - goal.md
      - soul.md
      - descriptions/**
    order:
      - summary/**
      - steps/**/request.json
      - steps/**/response.json
      - skills/**
    budget_tokens: 150000
    overflow: drop_oldest_steps
  compactor:
    pinned:
      - goal.md
      - soul.md
    order:
      - steps/**
    budget_tokens: 50000
    overflow: truncate
```

Paths are interpreted relative to the branch's worktree. The manifest sees only worktree contents by construction (§5.1). Pinned paths are always included regardless of budget; `order` entries fill the remaining budget in declared order until overflow policy kicks in.

This corresponds to the LangChain write/select/compress/isolate taxonomy (`docs/TAXONOMY.md` §3): **write** = commits; **select** = manifest inclusion; **compress** (here: compact) = compactor; **isolate** = subagent-conversation branches.

### 5.3 File path as hint

File paths are preserved in the assembled context as structural hints to the model. The path itself carries information (`steps/<conv-id>/0042/request.json`, `summary/003.md`, `skills/git-ops/SKILL.md`) that is cheaper than explicit metadata and often sufficient.

### 5.4 Removal by deletion

Agents reduce their own context by deleting files from the worktree. An agent that no longer needs a 5k-token file `rm`s it; the next context assembly excludes it naturally (per §5.1 — if it's not in the worktree, it's not in the prompt). Deleted files remain recoverable from git history until compaction squashes them.

---

## 6. Workflow as Configuration

Workflows are declared as event-to-action bindings in `<conv-repo>/workflow.yaml` (a frozen copy at conversation creation, §2.2, from a named workflow template at `<harness-root>/workflows/<name>.yaml`). Changing the workflow for an in-flight conversation is a direct edit of the per-repo file, since global changes do not propagate. "Workflow" here is the Anthropic sense (`docs/TAXONOMY.md` §1): predetermined code paths, as contrasted with LLM-driven agent control flow. Subordinate routines invoked by the workflow (compaction, verification, auto-dispatch on large tool output) are **procedures**.

```yaml
events:
  user_message:
    - spawn_root_conversation
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

**No resident interpreter.** Nothing parses `workflow.yaml` resident-style and drives the state machine from memory. Each event is a CLI subcommand (`lernie event <name> <repo> …`); the currently-executing subprocess *is* the interpreter while it runs. It reads the workflow, runs the event's action list by exec'ing the relevant procedures per §3.4 (`lernie dispatch <role>`, `lernie merge`, etc.), and before exiting emits the next event — itself another `lernie event` invocation. Procedures terminate by emitting their completion event. The chain ends at a terminal action (final merge, stop, error); there is no watcher noticing completions, each step hands off by exec. Combined with disk-as-bus (§3.1), this keeps the system stateless across process boundaries: a crashed subprocess leaves nothing in memory to reconstruct, and `lernie resume <repo>` re-enters the chain by reading the repo and exec'ing the event the state machine is waiting on. This is the concrete mechanism behind §1's Regenerability property.

---

## 7. UI

### 7.1 Live view

The UI watches the conversation repo filesystem (§3.5) and re-renders from what it observes. Top of the interface shows a git tree view of the current conversation. Clicking a commit navigates to that point. Clicking a branch navigates to that agent.

Live indicators:

- Line-by-line streaming text for model responses in flight.
- Pulsing indicators for tool calls.
- Arrows to sub-branches for active subagent conversations.
- Distinct patterns for model call states: queued, in flight, streaming, terminal.
- Branch termination markers (merged, stopped, conflicted).

### 7.2 History view

Clicking an old commit is read-only by default. Forking from history (creating a new branch from an old commit to explore a counterfactual) is a v1 feature but distinguished in the UI and in branch naming (`fork/<source-ref>/<id>`) so accounting and replay can tell the difference.

### 7.3 Concurrent exchanges

Because every exchange (root conversation) is its own branch and nothing else touches `main` directly, the user may send a second message before the first resolves. Both are root-conversation branches off `main` (or the first's tip, if the user wants strict sequencing). They run independently. No special mechanism is required beyond the branch invariant.

---

## 8. Metrics and Observability

First-class metrics, written to commit trailers and event log. All counts are reported along three scope axes, which are not interchangeable:

- Tokens per step, per conversation, per conversation repo.
- Model calls per step (always 1), tool calls per step, API calls per model call.
- Cost per step, per conversation, per conversation repo.
- Unmerged-subagent-branch count per conversation repo. This is a critical health metric: a ballooning count indicates silent failure somewhere in the merge pipeline. Subagent branches are those whose name contains a hyphen (root conversations live on `main`, §2.3); unmerged ones are enumerated by `git branch --list '*-*' --no-merged main | wc -l`. Root conversations are intentionally unmerged (§2.3 step 5) and are not counted here.
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

Every conversation repo declares its schema version in `<conv-repo>/version` (at the conversation-repo root, alongside the other control files — §2.2). Old versions remain readable by the harness; migration code is written when a version bumps. Tarballed runs from any prior version must remain inspectable.

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

**Success criterion:** A single prompt is sent to a provider endpoint, the response is written to disk, and is visible in the conversation repo as a commit. No git branching, no tools, no subagent dispatches.

**Shipped shape.** `lernie new <path>` scaffolds a conversation repo from the embedded template; `lernie prompt <repo> <message>` loads `providers.yaml` + `agents.yaml`, resolves the `worker` role, invokes `lernie-provider-<name>` as a subprocess (§4.4), writes `exchanges/<ts>-<short-id>.json` with `user_message` / `assistant_response` / `model_id` / `provider` / `usage` / `stop_reason` / `started_at` / `ended_at`, and commits the file to `main`.

**Exceptions to later invariants, historical.** v0.1 committed directly on `main` rather than via an exchange branch merge (§2.3); retired in v0.2, where `lernie prompt` spawns an `ex/<ts>-<id>` branch, commits the step snapshot before the model call (§2.10), lands the response as a follow-up commit, dispatches the terminal compactor off the tip (§2.7, stub in v0.2), and `--no-ff` merges the compacted branch back into `main` (§2.6). The earlier `--endpoint` argv pragma was also retired in v0.2: endpoint forwarding now goes through `describe.endpoint_env` per §4.4.

### v0.2 — Git tree

**Success criterion:** An exchange is a branch, completion is a no-ff merge back to `main`, the repo layout matches §2.2. User message → exchange branch → steps as linear commits → compactor (deletion-only, stub is fine) → merge. Unmerged branch count metric available.

**Shipped shape.** `lernie prompt <repo> <message>` spawns `ex/<ts>-<short-id>` off `main` in a dedicated worktree under `.lernie/worktrees/`, writes `.agent/goal.md` (§2.8) and `exchanges/<ts>-<short-id>/steps/001/request.json`, commits that snapshot before the model call (§2.10), invokes `lernie-provider-<name> complete`, lands the normalized response at `steps/001/response.json` as a follow-up commit, then dispatches the terminal compactor by re-entering itself as `lernie dispatch compactor <repo> <exchange-branch>` (subprocess per §3.4). The compactor spawns `inv/<ex-id>/<cmp-id>` off the exchange tip, writes its boilerplate `.agent/goal.md` and commits it as the dispatch snapshot (§2.8), then writes `.agent/compactions/001.md` and commits it as the terminal-summary follow-up (stub — no model call; `mark_for_deletion` is a no-op), and `--no-ff` merges back into the exchange branch. Control returns to `lernie prompt`, which rebases the exchange onto the current `main` tip and `--no-ff` merges it into `main` (§2.6). The unmerged branch count metric is read directly from `git branch --list 'ex/*' 'inv/*' --no-merged main` — no sidecar file.

### v0.3 — Tools

**Success criterion:** Agent can invoke at least two tools (bash, read_file). Tool calls are commits. Large tool outputs auto-dispatch to a parsing subagent conversation. Tool contract (binary + schema + skill) documented. Conversation-repo layout migrated from the v0.2 `.agent/`-rooted shape to the v0.3 layout described in §2.2 (control at conversation-repo root, worktrees as siblings, steps namespaced by conversation id, `merge=ours` on goal/soul/summary, "invocation" retired as a structural term).

### v0.4 — Subagent dispatch

**Success criterion:** Agent can dispatch a subagent conversation. The subagent runs in its own worktree and branch. Merge-back flow works end-to-end (including the `merge=ours` discipline on `goal.md`/`soul.md`/`summary/**`, §2.6). Parallel subagent conversations do not corrupt each other's state. Handle-based async works: a dispatch returns immediately with a handle, siblings can be in flight concurrently, and `await(handle)` retrieves the compacted result on a later step (§2.5).

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
