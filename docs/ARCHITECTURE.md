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
│   ├── system/                  # system prompts, tool and skill definitions
│   │   ├── prompts/
│   │   ├── skills/
│   │   └── tools/
│   └── state/                   # harness runtime state
│       ├── branches.json        # unmerged branch tracking
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

**Branch lifecycle** (identical for both branch classes):

1. **Spawn.** The dispatch creates a new branch off the parent's current commit. A worktree is allocated. The goal is written. Branch name encodes provenance.
2. **Work.** The agent runs its loop. Each step lands as a commit on the branch. A step that emits a subagent-targeting tool call spawns a sub-branch off that commit.
3. **Completion.** A terminal event is emitted — the agent returns a final response or is stopped.
4. **Merge.** The compactor produces a signal-preserving summary of the branch's work and applies it to the parent with `--no-ff`. Merge topology is preserved.
5. **Cleanup.** The source branch is kept as a ref for a retention window, then GC'd. The worktree is removed on merge.

Branching is cheap — local git operations on disk — but it is no longer per-step. A long exchange produces many commits on one branch and only spawns sub-branches when it actually dispatches.

Compaction may also run *during* a branch's execution, not only at termination; see §2.7.

### 2.4 Exchanges

An exchange is initiated by a user message and ends with a terminal assistant response. Its branch is the root of a trace in the observability sense. An exchange that is interrupted before a terminal response is terminal by virtue of the stop itself: it does not merge back to `main`; it may be resumed by a new dispatch using the stopped branch as context (see §2.9).

Multiple concurrent exchanges are supported: a user may send a second message before the first resolves. Each is its own exchange branch off `main`, merging in completion order. No special mechanism beyond the branch invariant is required.

### 2.5 Invocations and dispatch

A **dispatch** (§2.1) is the event that spawns a branch with a goal. The tool-call form — a tool call targeting a subagent — spawns an **invocation**: the subagent's full internal execution, contained in its own branch off the commit where the dispatch landed on the parent branch. From the subagent's perspective, its parent is indistinguishable from a user; the user-message and tool-call forms of dispatch are the same primitive.

This symmetry is load-bearing:

- The same code path handles user-initiated exchanges and agent-initiated invocations.
- Verifier agents, compactor agents, and adversarial critics are invocations with different goals.
- Parallel exploration (N workers on the same task) is N parallel invocations dispatched from the same step.

**Streaming reprompt.** A step may emit multiple tool calls. When each tool call returns, the emitting agent is reprompted — even while other tool calls from the same step are still outstanding. Each reprompt initiates a new step (a new commit on the parent branch). A tool call's *structural parent* is the step whose model call emitted it; its *temporal resolution* may overlap with later steps. The two are distinct and both tracked.

Tool calls targeting non-subagent tools run inline: their records are committed on the branch as part of the emitting step. Dispatch tool calls spawn invocation branches off the commit where the dispatch landed. Parallel dispatches from a single step spawn N sibling invocation branches off that same commit.

Invocations are expected to terminate and merge before their parent branch does.

By convention, invocations never write outside their own `invocations/<id>/` namespace or `artifacts/` (append-mostly). This makes merge conflicts structurally unlikely.

### 2.6 Merges

Merges are always no-fast-forward and conflict-free by construction. The merge protocol:

1. Child branch completes.
2. Compactor runs on the child branch (if warranted — see §2.7), producing a compacted diff against the parent at fork time.
3. Harness rebases the compacted diff onto the current parent tip (which may have advanced).
4. If rebase succeeds: merge with `--no-ff`.
5. If rebase conflicts: the child branch is marked conflicted, left unmerged, flagged for user attention. This should be rare; it indicates two invocations modified the same path, which is a design smell.

### 2.7 Compaction

Compaction (not compression — the term is reserved for model-weights quantization in the taxonomy) is the process of producing a signal-preserving, minimal version of a branch's work for consumption by the parent. A **compactor** is a subagent that performs compaction.

Compactor constraints (v1):

- **Deletion-only.** The compactor can remove files and truncate file contents, but cannot rewrite content. Worst case is lost information, never corrupted information.
- **Scoped to the branch's diff.** It does not modify pre-existing parent state.
- **Has access to the branch's goal.** The compactor decides relevance against the stated goal.

**Terminal compaction.** Every exchange and invocation is compacted at termination before the merge to its parent. The compacted summary is what the parent sees.

**Intermediate compaction.** Compaction may also run at checkpoints during a branch's execution — not only at termination. At each checkpoint the compactor produces a current-state summary and merges only that summary to the parent, *replacing* the previous checkpoint's summary. The branch continues running afterward; the next checkpoint supersedes this one; the terminal compaction is the final checkpoint in the sequence.

The discipline that parent branches see only compacted results — never raw internal state — is preserved. Intermediate compaction just makes the view progressively richer rather than all-or-nothing.

Intermediate compaction triggers are declared in `workflow.yaml` (see §6): by commit count, by elapsed time, or by an explicit `flush` action the agent may call. A branch with no configured trigger compacts only at termination.

Compaction failures (compactor produces garbage, times out, etc.) fall back to unmerged state and user review.

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

---

## 3. Component Architecture

### 3.1 Disk as Bus

All components communicate through the filesystem. No shared memory, no direct function calls between components. This applies to:

- Harness → provider endpoint (request written to disk, worker reads, response streamed to disk).
- Harness → tool execution (tool call record written to disk, executor reads, output streamed to disk).
- Harness → UI (events appended to `.agent/state/events.log`, UI tails).
- UI → Harness (user actions written to an input directory, harness picks up).

Notification mechanism: inotify where available, with a polling fallback. Every event write updates a `last_event_ts` sentinel file as a sanity check for consumers to detect missed events.

Consequences:

- Every component is independently restartable.
- Replay from any point is `git checkout <ref>` + re-tail the event log.
- Latency is higher than in-memory IPC. Accepted and compensated by streaming-first UI design.

### 3.2 Components

- **Harness.** The single program that drives execution: watches for events, spawns branches, runs model calls via the provider layer, invokes the tool executor, triggers merges and compactions, updates state. Stateless across restarts — resumes from disk. Any place this document says "the harness does X", it is this component.
- **Tool executor.** Runs tool subprocesses on behalf of the harness. Streams output to disk atomically (temp path + rename).
- **UI.** Tails event log and git history. Read-only view of live state plus write access to a well-defined input directory.

The harness and the tool executor share the same disk contract. None share memory.

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

---

## 4. Providers, Auth, and Models

### 4.1 Provider abstraction

The taxonomy (`docs/TAXONOMY.md` §2) flags "provider" as one of the field's most overloaded terms, naming three distinct roles: **model creator** (trains the weights), **inference provider** (serves the weights over an API), and **gateway** (unifies multiple inference providers behind one surface). This document uses **provider** in the *inference provider* sense throughout: an (endpoint, auth) pair.

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
    auth:
      type: aws_sigv4
      profile: default
```

### 4.2 Model abstraction

A **model** is (provider, model_id, capabilities). Model lists come from the provider's API where available. Capabilities is an extensible mapping declaring features the harness can rely on.

```yaml
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native, prompt_caching, streaming, stop_sequences]
    context_window: 200000
```

Capabilities are code-backed (each capability has a behavior implementation in the harness). Capabilities are extend-only: once declared, they are never removed from the registry, but the set on a given model may shrink if the provider removes support.

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
    - dispatch(worker, with: verifier.feedback)
  worker_flush:
    - dispatch(compactor, mode: intermediate)
  branch_stopped:
    - mark_abandoned
    - notify_ui

compaction:
  intermediate:
    trigger: every_n_commits   # or: every_t_seconds, on_flush
    n: 10
```

Actions are implemented in the harness; the workflow declares which run when. The `flush` action emitted by a running agent triggers an intermediate compaction without terminating the branch (§2.7). This is the primary surface for experimentation.

---

## 7. UI

### 7.1 Live view

The UI tails `.agent/state/events.log` and renders live state. Top of the interface shows a git tree view of the current conversation. Clicking a commit navigates to that point. Clicking a branch navigates to that agent.

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
- Unmerged branch count per conversation. This is a critical health metric: a ballooning count indicates silent failure somewhere in the merge pipeline.
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

### v0.2 — Git tree

**Success criterion:** An exchange is a branch, completion is a no-ff merge back to `main`, the repo layout matches §2.2. User message → exchange branch → steps as linear commits → compactor (deletion-only, stub is fine) → merge. Unmerged branch count metric available.

### v0.3 — Tools

**Success criterion:** Agent can invoke at least two tools (bash, read_file). Tool calls are commits. Large tool outputs auto-dispatch to a parsing invocation. Tool contract (binary + schema + skill) documented.

### v0.4 — Invocations

**Success criterion:** Agent can dispatch a subagent. Invocation runs in its own worktree and branch. Merge-back flow works end-to-end. Parallel invocations do not corrupt each other's state. Streaming reprompt works: each tool return initiates a new step while siblings may still be in flight.

### v0.5 — UI

**Success criterion:** Git tree view, live streaming, pulsing tool indicators, branch-state indicators. Tailing the event log is the only read mechanism.

### v0.6 — Workflow config

**Success criterion:** The `workflow.yaml` surface works. At least one non-baseline workflow variant (e.g., a verifier step) runs end-to-end without code changes.

### v0.7 — Task suite

**Success criterion:** 50 tasks with machine-checkable success criteria. Baseline harness achieves 40% ± 5% pass@1 on the suite (Wilson CI). Per-category failure tagging works.

### v0.8 — Experiments and replay

**Success criterion:** `agent-eval --config <experiment> --suite <suite> --runs N` produces per-task pass@1 and pass@5 with confidence intervals. Any run can be tarballed and replayed. Config changes (prompt edits) deployable without code changes in under 60 seconds end-to-end.

### v1.0

**Success criterion:** All of the above, plus at least one demonstrated workflow variant that beats baseline on at least one failure category by a statistically significant margin on pass@1. This is the proof that the architecture's experimentation surface is actually useful.
