# Agent Harness Spec

**Status:** Draft v0.4
**Scope:** Design specification for a git-backed agent harness with branch-per-dispatch context management. v0.4 folds in the `harness` repo design exploration (2026-07): brazen becomes the provider layer (§4), budgets and sandboxed tools join the milestone chain (§12).

---

## 1. Overview

This document specifies an agent harness in which conversational context is managed as a git repository. A conversation repo is one root conversation plus all its subagent descendants, materialized as worktrees within a single git repo. Every dispatch creates a branch. Subagent branches merge back on termination; root-conversation branches persist as refs and await a reprompt. Any sender — the user or another conversation — may deposit a message into a live conversation's inbox; the message is delivered at the recipient's next step boundary (§2.11). Within a branch, steps land as linear commits; a step that itself dispatches spawns a sub-branch off the commit where the dispatch landed. State flows between components through the filesystem (§3.1); commands between procedures flow through the `lernie` CLI (§3.4). There is no third channel — no library API, no in-process sidechannel, no resident broker — and no process holds state across its own termination.

The architecture optimizes for four properties:

1. **Inspectability.** The complete state of any conversation, at any point in its history, is a git ref. Replay, debugging, and counterfactual forking are first-class operations.
2. **Uniformity.** User-to-agent dispatch, agent-to-subagent dispatch, and agent self-reflection all use the same primitive: fork a branch, do work, merge back (or terminate to user, for root conversations). There is no special path for user input. There is likewise no special sender: a message into a running conversation (§2.11) is the same deposit whether the user or another conversation makes it.
3. **Testability.** Workflow (prompts, sequencing, context assembly rules) is configuration, not code. Experiments are config diffs, measurable against a task suite.
4. **Regenerability.** Any process can die at any time without losing state. Disk is durable; processes are disposable. Components — harness, tool subprocesses, provider adapters, frontends — restart independently because none hold state across their own termination; `lernie advance <repo>` (§6) is the operation that drives the workflow chain forward, and crash recovery is the same `lernie advance` invocation that runs the chain in normal operation. No process is load-bearing; disk is.

### 1.1 Non-goals for v1

The following are explicitly out of scope for v1:

- Multi-tenancy or multi-user isolation. Single user assumed.
- Distributed execution. Single machine, single harness process.
- Tool sandboxing. Tools run with user privileges; no capability restriction. (A v1 non-goal only: specced as milestone v1.1, §12.)
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
| **API call** | One HTTP request to a provider endpoint. | 1:1 with attempts by construction (§4.4); retries make a model call span several. |
| **Attempt** | One invocation of the provider adapter for a step's model call — exactly one API call by construction (§4.4). | A model call comprises one or more attempts (§2.10); each attempt lands as one segment in the step's `response.json`. |
| **brazen** | The provider adapter project (binary `bz`, crate `brazen`): one stateless binary adapting every provider and wire protocol behind a canonical request/event pipe contract. | The only component that knows provider wire protocols (§4.4); its specs define the canonical vocabulary. |
| **Canonical request / canonical event** | brazen's typed request shape and its `v=1` streaming event vocabulary (brazen `architecture.md` §3). | The adapter wire contract; `response.json` is JSONL of canonical events (§2.3, §4.4). |
| **Dispatch** | The event that spawns a child conversation with a goal. Two forms: a user message (spawns a root conversation) and a tool call targeting a subagent (spawns a subagent conversation off the commit where the dispatch landed on the parent branch). | Creates a branch. |
| **Branch** | The git container for a conversation. | Every conversation has exactly one branch; every dispatch creates one. |
| **Goal** | The stated objective handed to a conversation at dispatch. | One per conversation; not rewritten during execution; pinned at the head of context for every model call on the branch (§2.8). |
| **Soul** | The system prompt handed to a conversation at dispatch, drawn from the conversation repo's `souls/<role>.md` and overwritten on the branch's dispatch commit. | One per conversation; composed into every model call on the branch as the system message. |
| **Message** | Content addressed to an *existing* conversation by a sender — the user or any conversation. Deposited into the recipient's inbox; delivered at a step boundary as a committed worktree file (§2.11). | The steering primitive. Sender is recorded provenance; the recipient treats every sender uniformly. Unqualified "message" means this primitive; wire-level messages are always written qualified (user message, assistant message). |
| **Inbox** | The per-conversation queue directory `<conv-repo>/inbox/<conv-id>/` holding deposited-but-undelivered messages. Outside every worktree, namespaced by conversation id like `steps/` (§2.2). | Deposit target for messages (§2.11). |
| **Executor** | The process currently driving a branch's step loop — `lernie prompt`, `lernie dispatch`, or a `lernie advance` re-entry (§6); the same process the §2.9 writer scan discovers. | At most one per branch, guaranteed by the executor lock (§2.11). Distinct from the *tool executor* (§3.2), a role inside it. |

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
├── steps/<conv-id>/NNN/          # diagnostic step records; outside every worktree (§2.3)
│   ├── meta.json                 # {commit, started_at, …} — branch tip at step-start
│   ├── request.json              # diagnostic; replay rebuilds the wire input from `commit`
│   ├── response.json             # JSONL of canonical events, attempt segments (§4.4)
│   └── tools/<tool-id>/          # input.json, output.json — runtime-read by harness
├── inbox/<conv-id>/              # deposited, undelivered messages; outside every worktree (§2.11)
├── root/                         # primary worktree; .git lives here
│   ├── .git/
│   ├── .gitattributes            # merge=ours rules for goal.md, soul.md, summary/**, messages/**
│   ├── goal.md                   # branch-scoped — this conversation's goal
│   ├── soul.md                   # branch-scoped — this conversation's system prompt
│   ├── summary/NNN.md            # branch-scoped — this conversation's compactions
│   ├── messages/NNN-<sender>.md  # branch-scoped — delivered messages (§2.11)
│   ├── descriptions/             # tool + skill descriptions (committed on main; inherited)
│   └── skills/                   # loaded skill content (branch-scoped; compactor may prune)
├── <a>-<b>/                      # subagent conversation (linked worktree; .git is a pointer file)
│   └── … same shape as root/ …
├── <a>-<b>-<c>/                  # sub-subagent; hierarchy encoded in name, NOT in filesystem
│   └── …
└── …
```

**Control plane vs data plane.** The files at the conversation-repo root (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `souls/`, `version`) are **control** — the harness reads them to decide what to do — and live outside every worktree. Everything inside a worktree is **data** — it is composed into that conversation's prompt at model-call time. This is load-bearing: context assembly has no exclusion list, because nothing that isn't context lives in a worktree.

**Step records are not context.** A **step record** is the per-step on-disk directory at `<conv-repo>/steps/<conv-id>/<NNN>/`: `meta.json`, `request.json`, `response.json`, and any `tools/<tool-id>/` subdirectories the step emitted (full layout in §2.3). The whole `steps/` tree at the conversation-repo root holds these diagnostic / audit artifacts. It sits outside every worktree by construction, which means context assembly (§3.5, §5) is *physically incapable* of including it. The structural placement enforces the rule the worktree-as-context invariant (§5.1) implies: model conversations replay from in-memory message history while running, and from `commit` + context assembler at replay time — never from re-reading `request.json` / `response.json`. Per-tool `output.json` is read at runtime when assembling the next step's `tool_result` blocks (§3.3), and per-tool `input.json` is read at replay time when reconstructing tool framing (§2.3) — these are the runtime *content* reads in this tree; `response.json`'s event *framing* is read for classification and metering (§2.3 Diagnostic-only contract), and everything else is write-only diagnostic. See the diagnostic-only contract in §2.3 for the full split.

**Frozen-copy bootstrap.** Creating a repo is `cp -r <harness-root>/agents/<profile>/* <harness-root>/conversations/<root-id>/` plus `git init` plus the initial commit. All control files are frozen snapshots at that point. Subsequent changes to the global agent profile, to `<harness-root>/workflows/`, `<harness-root>/skills/`, or `<harness-root>/tools/` do not propagate into existing repos. Reproducibility and portability win over live update. (Endpoints and auth are not lernie's to copy — since v0.6 they live entirely in brazen's own config and credstore (§4.4), never a harness file; the per-repo `providers.yaml` carries only the role → (provider row name, model id) pointer, and the global `<harness-root>/models.yaml` holds model capabilities and context windows (§4.2). brazen resolves endpoint and auth from the row name at call time (§4.1).)

**Sibling worktrees, not nested.** Subagent worktrees are named by their full hyphenated descent from the root (`<a>-<b>-<c>-…`) and live as *siblings* of `root/`, never as subdirectories of their parent's worktree. Git does not permit nested working trees; the sibling layout is how the primitive's uniformity survives contact with git mechanics.

**Branch-scoped vs main-committed.** `goal.md`, `soul.md`, `summary/`, and `messages/` are written per-branch and pinned by `.gitattributes merge=ours` — on merge-back from a subagent, the parent's versions are retained verbatim, so a subagent's private goal — or the messages it was steered by (§2.11) — can never clobber its parent's. `descriptions/` is committed once on `main` at conversation creation and inherited by every branch via git. `skills/` is branch-scoped (added as skills are loaded; removed by the compactor when no longer needed). `steps/` is *not* branch-scoped — it lives at the conversation-repo root (above), shared across the whole conversation tree and namespaced by conversation id (`steps/<conv-id>/NNN/`). Subagent step records do not cross up via merge; they share the conv-repo's `steps/` tree from the moment they're written. `inbox/` follows the same construction: conversation-repo root, namespaced by conversation id, outside every worktree (§2.11). See §2.3 (Step on-disk layout) for the rationale.

Read-only from the user's perspective under normal operation. The harness is the only writer. The user interacts via the UI, which produces events that the harness translates into commits.

### 2.3 Branches and the branch invariant

A **branch** is the git container for a single conversation. One invariant:

> **Every dispatch creates a branch.** Spawn the branch, do the work, compact, merge back (or, in the root case, terminate to the user). This pattern is uniform whether the dispatch is a user message or a subagent-targeting tool call.

Branch naming tracks the worktree directory. The conversation-repo's `main` branch is the durable home of root-conversation history: each user-message dispatch spawns a `<conv-id>` branch off the current `main` tip, work happens in a sibling worktree at `<conv-repo>/<conv-id>/`, and the branch merges back to `main` on completion (paragraph 3 below is the load-bearing rule). Every subagent conversation's branch is its full hyphenated descent (`<a>-<b>`, `<a>-<b>-<c>`, etc.). There is no `ex/` or `inv/` prefix — the hierarchy in the name is self-describing.

Within a branch, **steps are linear commits** — one commit per step, carrying that step's model call and the tool calls it emitted. Steps are not their own branches. New branches appear only at dispatch boundaries.

Nothing commits directly to `main` except the conversation-repo's initial snapshot (the frozen bootstrap, §2.2). All subsequent data on `main` arrives via merge from a completed root-conversation dispatch — i.e., a user-message exchange.

> **Historical.** v0.1 committed exchanges directly on `main` without branching; v0.2 introduced branching with separate `ex/*` (exchange) and `inv/*` (invocation) branch prefixes under an `.agent/`-rooted layout. v0.3 unifies both under the conversation primitive (§2.1): directory splits collapse, branch names drop their prefixes, and "invocation" retires as a structural term in favor of "conversation."

**Branch lifecycle** (identical across root conversations and subagent conversations, up to the merge/terminate distinction at step 5):

1. **Spawn.** The dispatch creates a new branch off the parent's current commit. A linked worktree is allocated at `<conv-repo>/<full-hyphenated-descent>/`.
2. **Dispatch commit.** The harness overwrites `goal.md` and `soul.md` in the worktree with this conversation's goal and the chosen role's soul from `<conv-repo>/souls/<role>.md`, then commits. This is the first commit on the new branch. The overwritten files are excluded from future merge-back by the merge=ours discipline (§2.6).
3. **Work.** The agent runs its loop. At each step boundary the executor drains the conversation's inbox — pending messages land as a delivery commit before the model call (§2.11). Each step's diagnostic record (§2.3 Step on-disk layout) lands at `<conv-repo>/steps/<this-conv-id>/NNN/`, outside the worktree. Worktree-modifying tool calls (e.g. `bash` editing files) commit those modifications on the branch; tool calls without worktree side effects produce no commit. A step that emits a subagent-targeting tool call spawns a sub-branch off the branch tip.
4. **Completion.** A terminal event: final response, stop, timeout.
5. **Merge (subagent) or terminate (root).**
   - *Subagent:* the compactor produces a summary; the branch is rebased onto the current parent tip, the merge=ours-disciplined paths are aligned to the parent's pre-merge state on the subagent's tip, and the result is merged `--no-ff`. The parent's `goal.md`, `soul.md`, and `summary/` tree carry through verbatim; any explicit exports cross up. The subagent's step records are *not* part of the merge — they live at `<conv-repo>/steps/<sub-id>/` (§2.2), already shared with the parent before the merge starts.
   - *Root:* no merge. The conversation's branch persists as a ref; the UI shows it. The user reprompts by issuing a new dispatch that consumes the branch's state as context.
6. **Cleanup.** Completed branches are retained as refs for a retention window (default 30 days) and GC'd thereafter. Worktrees are torn down on completion.

Branching is cheap — local git operations on disk — but it is not per-step. A long conversation produces many commits on one branch and only spawns sub-branches when it actually dispatches.

Compaction may also run *during* a branch's execution, not only at termination; see §2.7.

**Step on-disk layout.** Each step lives in its own directory at `<conv-repo>/steps/<conv-id>/<NNN>/` — at the conversation-repo root, *outside every worktree* (§2.2). `<NNN>` is zero-padded 3-digit and 1-indexed, so step dirs sort lexically. `<conv-id>` is the owning conversation's id — namespacing this way is what lets every conversation in the tree (root + every subagent) write into a single shared `steps/` tree without filename collision; subagent step records do not need to cross up via merge because they were never below a worktree to begin with.

Per-step files:

- `meta.json` — `{commit, started_at, ended_at, …}`. The `commit` field is the sha of the branch tip at step-start; it is the **read state** for the step's model call. Replay reproduces the wire input by re-running the context assembler (§5) against this commit's tree — `request.json` is not the source of truth. Step 1's `commit` is the dispatch commit (§2.3 step 2). Step ≥2's `commit` is the prior step's tip, advanced by any worktree-modifying tool-call commits between them; the harness writes no pre-call commit for step ≥2 (§2.10).
- `request.json` — diagnostic snapshot of the wire request the model saw. Written for audit and human inspection only (see Diagnostic-only contract below).
- `response.json` — JSONL of canonical events (one event per line), appended across one or more attempt segments, each segment terminated by its `{"type":"end"}` line; the wire-side streaming/non-streaming distinction collapses inside the adapter before bytes reach disk. See §4.4 for the segment rules and §3.5 for the live-streaming completion signal.
- `tools/<tool-id>/` — per-tool-call records (`input.json`, `output.json`); `<tool-id>` is the `tool_use.id` from the wire (e.g. `toolu_01abc…`). Full contract in §3.3.

**Diagnostic-only contract (request.json, response.json).** `request.json` is a pure diagnostic / audit artifact — never read at runtime. `response.json` is read at runtime, but only along one sanctioned seam: its event **framing** — the terminal `end` line, the last segment's `Finish`/`Error` kind, and `Usage` events — is read for state classification (§3.5, §4.4) and metering (§6 budgets, §8 metrics). Its event **content** — the assistant text, thinking, and tool-call arguments a step produced — is **never** read for context assembly. The rule is framing-yes / content-no:

- **request.json: never read at runtime.** Replay (§3.1, §2.10) re-runs the context assembler against `meta.json`'s `commit` tree and re-invokes the adapter; it does not read `request.json`. The file exists for human inspection only.
- **response.json content: never read for context.** Messages history during execution is assembled in-memory within a single process — the running step holds prior steps' messages in RAM — and at replay time from `commit` + the context assembler (§2.10). No code site reconstructs context by re-reading a prior step's `response.json`.
- **response.json framing: sanctioned reads.** Classification (§3.5), `await` (§2.5), the harness retry loop (§2.10), and metering (§6, §8) read only the framing tail — the terminal `end`, the last segment's `Finish`/`Error` kind, and `Usage` events — never the content blocks. Workflow advance (§1 #4, §6, "No resident interpreter") stays scoped to workflow-event boundaries: it never picks up mid-conversation by reading response content.
- **The frontend (§3.5)** may read both files in full for user inspection; that is a read-only consumer, not harness state.

This is structural, not advisory. The placement of these files at `<conv-repo>/steps/`, outside every worktree, makes context assembly (§3.5, §5) physically incapable of including them as model context; the worktree-as-context invariant (§5.1) is untouched, because a framing read (a terminal line, an error kind, a token count) never flows into a prompt. Harness implementations honor the split directly: no code site reads `request.json` at runtime, and no code site reads event *content* from `response.json` for context assembly — only its framing, and only for classification and metering.

**Tool records are exempt from the diagnostic-only rule.** `tools/<tool-id>/output.json` is read at runtime by the harness when assembling the next step's `tool_result` blocks (§3.3), because tool outputs are nondeterministic and cannot be reconstructed by replay. Tool records share the `<conv-repo>/steps/` location with `request.json` / `response.json`, but they are **runtime state**, not diagnostic. `input.json` is the `tool_use` block the model emitted, recorded for parity with `output.json`; the harness reads it only when reconstructing tool framing during replay.

Step records are not committed to git. The conversation-repo's `.git` lives inside `root/`, and `<conv-repo>/steps/` sits above that — outside every worktree, untracked by git. Their durability is filesystem durability (atomic write via temp + rename, fsync as needed); their authority for replay is the `commit` sha each `meta.json` records, which *is* a real git commit.

**Unmerged-branch tracking.** Git's ref database is the tracking. Subagent conversations that should have merged back but didn't are readily enumerable — any non-`main` ref that is not merged into its parent indicates a stalled or failed pipeline. The §8 unmerged-branch-count health metric is read directly from `git branch` (PRINCIPLES.md "Single source of truth"); see §8 for the specific form.

### 2.4 Exchanges

An **exchange** is the UX label for a root conversation — one initiated by a user message. It is not a separate structural primitive; structurally, it is a conversation whose parent is the user rather than another conversation. The label survives because users think in exchanges ("the conversation I just had with the agent starts with my message and ends with the final response"), and the UI surfaces that framing. Every architectural property of a conversation applies equally to an exchange.

An exchange that is interrupted before a terminal response is terminal by virtue of the stop itself: like any root conversation, it does not merge back to `main`; the user moves forward by issuing a new dispatch — `lernie prompt` for a fresh conversation, or fork-from-history for a new branch off the stopped tip (see §2.9).

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

**Async work uses handles.** Long-running tools — including dispatch — return immediately with a handle (`{status: in_progress, handle: <id>}`) as their `tool_result`. The agent retrieves the actual outcome later via a separate `await(handle)` or `check(handle)` tool, whose return value is a `tool_result` on a later step. Parallelism is expressed by issuing several dispatches in one step and awaiting them in subsequent steps as results come in. The per-step "one result per tool_use" shape stays intact; the asynchrony rides on the handle. Dispatch is a tool like any other: this is how the symmetry between inline tool calls and subagent conversations is preserved without inventing a second control path. The v0.4 `dispatch` built-in — `lernie tool dispatch`, input `{role, goal}` — realizes this contract: it spawns the subagent through `lernie dispatch <role>` (§3.4) and emits the handle as `<this-conv>-<sub-id>` (the new subagent's full hyphenated descent / branch name).

Tool calls targeting non-subagent tools commit their records on the emitting branch as part of the step. Dispatch tool calls spawn subagent-conversation branches off the commit where the dispatch landed; the `tool_result` the parent step observes is the handle. When the subagent terminates and merges back, its compacted output is what `await(handle)` returns on the next step. Parallel dispatches from a single step spawn N sibling subagent-conversation branches off that same commit.

A subagent conversation must reach a terminal state before its *parent* terminates — parent termination cascades a stop to still-running children (§2.9) — but it need not resolve before the parent's next step. A long-lived subagent that outlives many parent steps, messaging its parent as it goes (§2.11) — a reminder agent, a watchdog, an adversarial critic running alongside — is a legitimate shape, not a stall; a still-running child is live (its executor holds the branch's lock, §2.11), and the §8 unmerged-branch health metric counts unmerged branches with *no live executor*, never branches still working.

The parent sees those upward messages only while it is *stepping*: delivery lands at step boundaries (§2.11), and a parent parked in a blocking `await(handle)` has no step boundary until that await resolves. A reminder-shaped child therefore pairs with the parent's non-blocking `check(handle)` form (above), which returns immediately instead of blocking — the parent's step resolves, the next step boundary drains the inbox, and the polling cadence keeps step boundaries recurring (§6) so the child is seen while it runs. Blocking `await` is reserved for a result the parent cannot proceed without; a message deposited while the parent is blocked in one is still accepted — the parent's executor is live (§2.11) — but its delivery defers to the first step boundary after the await returns. Delivery is deferred, never dropped, and nothing new is introduced: `check` and `await` are the two forms already defined above.

The harness assigns write paths per tool call and per subagent conversation, so two sibling branches never target the same file. This is a structural guarantee (enforced by the tool executor and the dispatch primitive), not a convention, which is what makes the merge protocol (§2.6) conflict-free by construction.

### 2.6 Merges

Merges are always no-fast-forward and conflict-free by construction. Root conversations do not merge (§2.3 step 5); subagent conversations do. The merge protocol for a subagent:

1. Subagent conversation completes.
2. Terminal compaction runs (if warranted — see §2.7): a compactor subagent is dispatched off the subagent's tip and merges back into the subagent, leaving its tree in compacted form.
3. Harness rebases the compacted diff onto the current parent tip (which may have advanced).
4. **Alignment step.** While still on the subagent branch's tip, the harness pins the merge=ours-disciplined paths (`goal.md`, `soul.md`, `summary/`, `messages/`) to the parent's pre-merge versions: any version the subagent has at those paths is replaced with the parent's, or removed if the parent has nothing there. If anything changed, the harness commits the result as a "merge=ours alignment" commit on the subagent branch. This is the load-bearing enforcement of the discipline below; the `.gitattributes` driver alone is not sufficient (see "What `merge=ours` alone cannot do").
5. Merge with `--no-ff` into the parent.
6. If the rebase in step 3 conflicts: the subagent branch is marked conflicted, left unmerged, flagged for operator attention. The mark is a git ref `refs/lernie/conflicted/<sub-branch>` written by the harness at the sub-branch's pre-rebase tip — single source of truth (`docs/PRINCIPLES.md`), no sidecar file. `await(handle)` (§2.5) reads this ref to surface `{status: conflicted}` to the parent. This case indicates a harness defect — two branches were given overlapping write paths, violating the guarantee in §2.5 — and is tracked accordingly.

**The merge=ours discipline.** Four categories of file are pinned to the parent on every merge-back. The discipline is enforced by the alignment step above, with `.gitattributes` (committed on `main` inside `root/` at scaffold) as a backstop for any merge that bypasses the harness:

```
goal.md      merge=ours
soul.md      merge=ours
summary/**   merge=ours
messages/**  merge=ours
```

These files are all branch-scoped: each conversation writes its own `goal.md` and `soul.md` on the dispatch commit (§2.3 step 2), its own `summary/NNN.md` as compactions happen (§2.7), and its own `messages/` files as deliveries land (§2.11). Without the discipline, every subagent merge-back would carry the subagent's versions of these files into the parent's tree — which is precisely wrong, since the parent resumes with *its* goal, not the subagent's. With it, the subagent's versions stay on the subagent branch (visible in history for provenance, not reflected in the parent's post-merge state).

**What `merge=ours` alone cannot do.** A `.gitattributes` `merge=ours` attribute only resolves *conflicting* paths during a 3-way merge — both branches modified the same file (or both added it). Git's merge logic does not invoke per-file drivers when:

- the path is added on theirs only and absent on ours (the file is just added — no conflict, no driver call);
- the path is modified on theirs and unchanged on ours (the parent ref's tree is replaced with theirs — fast-forward direction, no driver call);
- the custom `merge.ours.driver` is not registered in the repo's git config (the attribute is silently ignored).

The first two are the common case for subagent merge-backs (the parent typically has *no* in-flight `summary/`, and after the alignment step the subagent's overrides match the parent's exactly), so a vanilla `.gitattributes` setup would let everything cross up regardless of intent. The harness sidesteps this by replacing-or-removing the disciplined paths *before* the merge sees them. Scaffold registers `merge.ours.driver true` so the attribute is at least active for any human-driven merge that bypasses the alignment step.

Step records do not pass through merge. They live at `<conv-repo>/steps/<conv-id>/NNN/` — outside every worktree, shared across the whole conversation tree from the moment they're written (§2.2, §2.3). Namespacing by conversation id is what keeps the parent's and subagent's records collision-free without any merge-time alignment.

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

Rewriting a goal in place is not structurally forbidden but is not a core path — the expected unit of iteration is the conversation, not the goal. Mid-flight steering does not rewrite the goal either: a message (§2.11) adds context *beside* the pinned goal; the goal stays what the conversation was dispatched with.

The pinned goal resolves the recency-decay problem in deep agent trees where sequence-as-authority (last user message = current order) fails.

### 2.9 Stopped branches

Stops are aggressive. When a stop is issued (by user, by timeout, by cascade from a parent — the user-driven case is the CLI subcommand `lernie stop <repo> <branch>` per §3.4, idempotent against an already-terminal branch):

1. SIGTERM is sent to the harness process working on the branch (and the kernel cascades through its process group, covering tool subprocesses and any subagent harnesses spawned per §3.4).
2. In-flight HTTP requests to provider endpoints are dropped as a side effect of the adapter receiving SIGTERM — `bz` installs no signal handlers and dies at once (§4.4 Cancellation); already-flushed event lines stay valid.
3. The kernel closes the harness's open fds. The latest step's `response.json` (§4.4 "On-disk response shape: appended attempt segments") receives `IN_CLOSE_WRITE` (§3.5) without a terminal `end` event having been emitted — this missing terminal event *is* the on-disk signature of a stopped step. No separate cancel marker is written; the absence of a trailing `end` on a closed file is sufficient.
4. The branch is left unmerged. Its `stopped` status is derived state, not a written flag — consistent with `docs/PRINCIPLES.md` "Single source of truth": an unmerged branch whose latest step's `response.json` is closed without a trailing `end` is `stopped`. (Crashes, kills, and explicit user stops are indistinguishable on disk and treated identically.)

The user-action stop (`lernie stop <repo> <branch>`) discovers the harness pid the same way: it scans `/proc/<pid>/fd/*` for the writer holding the latest step's `response.json` open and signals that pid's process group. There is no sidecar pid file — the open fd is already the source the §3.5 `in_flight` classification reads, so the same observation drives both. The harness sets its own process group at startup (`setpgid(0, 0)` in `lernie prompt`) so the `kill(-pgid, SIGTERM)` cascade reaches its provider adapter and any subagent harnesses re-entered via `lernie dispatch` (which deliberately do *not* setpgid, inheriting the parent's pgid) without touching the invoking shell or UI process.

A stopped root conversation is terminal: like any root conversation, it does not merge back to `main`. A stopped subagent conversation is also terminal: it does not merge back to its parent, and the parent's `await(handle)` resolves to a `stopped` status. The stopped branch remains as a ref for retention. User paths forward are the ordinary dispatch primitives: `lernie prompt` to start a new conversation, or fork-from-history to spawn a new branch off the stopped tip. There is no distinct "resume" operation — both fork-from-history and new-prompt subsume what resume would have done, and a third name would only obscure that.

Between-step lulls (the brief window when one `lernie advance` subprocess has emitted its terminal `end` and exited but the next `lernie advance` subprocess has not yet exec'd) are not stops: the latest step's `response.json` ends with `end`, distinguishing it from a terminated chain. The §6 stateless re-entrance pattern guarantees that during normal operation a chain has either an emitted terminal event or a live process — never neither.

Default retention: 30 days, then tarballed and GC'd.

### 2.10 Retries and failures

**Read state per step.** Step 1 commits the dispatch artifacts (`goal.md`, `soul.md` per §2.3 step 2) on the new branch *before* its model call; that commit is step 1's read state. Step ≥2 takes no pre-call commit — the prior step's tip (advanced by any worktree-modifying tool-call commits, §2.3) is its read state. The branch tip at step-start is recorded in `meta.json`'s `commit` field (§2.3) so retry and replay are tractable: a failed model call is reissued by re-running the context assembler (§5) against the recorded sha and re-invoking the adapter, with no drift from the original wire input. The on-disk `request.json` is diagnostic, not authoritative.

- **Retryable provider errors** (transient network failure, 429 rate limit, 5xx) are retried inline with backoff, bounded by the attempt cap in `workflow.yaml` (§6). Retryability is classified by `CanonicalError::retryable()` from the linked brazen crate (§4.4) — computed from the in-band `Error` event, never re-derived by the harness. Each retry is a fresh attempt: a new `bz` invocation appending a new segment to `response.json` (§4.4). Retries do not produce additional commits — the commit frames the model call, not the individual attempt.
- **Non-retryable errors** (400 validation failure, auth failure, impossible-to-satisfy schema) abort the step. The branch is left in the state it held before the model call and flagged for operator attention.
- **Unknown or ambiguous failures** trigger a diagnostic dispatch: a subagent conversation is dispatched off the branch's current commit with a goal describing the failure, access to the branch state and the raw error, and instructions to produce a recommended next action (retry, abort, modify config, escalate).

**Resumable pauses.** A `Finish{Pause}` (the linked brazen crate's `FinishReason::Pause`, Anthropic `pause_turn`) is neither a completion nor a failure: the provider stopped mid-work and expects the partial assistant content replayed back to continue (brazen `anthropic-messages.md` §3.5, "resume by re-sending the assistant content as-is"). The harness absorbs it *within the same step*, not as a new step: on the `Pause` segment it folds that segment's partial assistant content and re-invokes `bz` with that content appended, landing a further attempt segment (§2.1) on the same `response.json` — the identical multi-segment shape as retry, differing only in the request delta (a retry re-sends the identical request; a Pause continuation appends the paused content). A *new* step is structurally impossible here: a step's read state is a commit (§2.3), but the paused partial content is never committed worktree data — it lives only in the diagnostic `response.json`, which context assembly may not read (§2.3, §5.1) — so it has no read-state home, and only within-step continuation preserves the replay-from-recorded-commit invariant above. Each continuation is a billed attempt (§6, §8), and the loop is bounded by the step's attempt cap (§6) so a pathological pause loop terminates. brazen emits `Pause` *only* with provider server tools (brazen `anthropic-messages.md` §3.5), which lernie's stdio tool contract (§3.3) does not enable — so no v1 model call can pause; the absorption is specified here so classification (§3.5, §4.4) is total, and it activates whenever server tools are wired (a future milestone).

Tool failures follow the same shape at the tool-executor level, surfaced to the emitting agent as the tool's `tool_result` content. The agent decides whether to retry, ignore, or escalate — this is an ordinary agent decision, not a harness one.

### 2.11 Messages

A **message** is content addressed to an *existing* conversation. Any sender may deposit one: the user, the conversation's parent, one of its children, a sibling — or the conversation itself. This generalizes the dispatch symmetry (§2.5): where dispatch makes the parent indistinguishable from a user *at spawn*, messaging makes **every sender indistinguishable from a user for the rest of the conversation's life**. One primitive dissolves what would otherwise be four features — the user steering a running exchange, a parent steering a dispatched subagent, a child reporting upward to its still-running parent (the reminder/watchdog shape), and the user dropping into any descendant of the tree. There is no "attach" operation to define: watching any branch is already the frontend contract (§3.5), and speaking to it is a message.

A conversation is exactly two things — recoverable on-disk state, and at most one in-flight execution. A message is a write to the first that the second picks up at its own pace.

**Deposit.** `lernie message <repo> <conversation> <content>` (§3.4) writes the message to the recipient's **inbox** at `<conv-repo>/inbox/<conv-id>/<sender>-<NNN>.md` — at the conversation-repo root, outside every worktree, namespaced by conversation id exactly like `steps/` (§2.2). `<sender>` is the depositing conversation's id, or `user`; `<NNN>` is the sender's own sequence. Sender-namespacing is what makes deposit collision-free without any lock — two senders never target the same file (the same construction as §2.5's write-path guarantee), and a single sender is sequential with itself. Writes are temp-path + atomic rename. The file carries `from:` and `deposited_at:` frontmatter: provenance is recorded even though the recipient treats every sender uniformly. That uniformity is by design, not an oversight — every sender is influence-equivalent for the recipient's whole life, so a hostile message is a lateral prompt-injection surface the primitive does not itself fence; the single-user assumption (§1.1) covers authorization but not influence, and defense is deferred and layered (§11), not a change to the deposit.

The model-facing side is the `message` built-in tool (`lernie tool message`, input `{conversation, content}`), which goes through the CLI per §3.4. The sender identity is taken from `LERNIE_CONV_BRANCH` (§3.3) — harness-derived, never model-supplied, so a conversation cannot forge provenance. Deposit is synchronous and returns `{status: deposited}`: it is not a dispatch, creates no branch, and returns no handle. Addressing needs no registry, because a conversation's address *is* its branch name (§2.3): the children's addresses are the handles the conversation already holds (§2.5), the parent's address is the conversation's own name minus its last descent segment, and the user reads addresses off the UI's branch view (§7.1).

**A message to a conversation with no live executor is refused.** Without an executor, no step boundary will ever come, so nothing could ever deliver the message. The liveness probe is the executor lock itself (below): `lernie message` try-acquires it non-blocking, and *succeeding* is the failure — no one is driving the branch — so it releases, declines loudly (`docs/PRINCIPLES.md` "Decline illegal operations"), and names the remedy: the ordinary dispatch primitives — `lernie prompt` consuming the branch's state, or fork-from-history off its tip (§2.9). That remedy is not a consolation; it is what "messaging" a quiescent conversation already meant. The quiescent case needs nothing new — the reprompt shape (§2.3 step 5) covers it — so the only genuinely new mechanism in this section is delivery into a *live* execution.

**Delivery.** At each step boundary — after the prior step resolves, before the next model call's context is assembled — the executor drains the inbox: each pending message file moves into the branch's worktree at `messages/<NNN>-<sender>.md` and the move is committed (the **delivery commit**). Delivery is where ordering becomes authoritative: arrival order across senders in the inbox is advisory (mtimes), and the committed sequence is the record — the order question has one home, and it is the commit. From there, no new mechanism exists or is needed:

- **Context inclusion is the worktree invariant.** Delivered messages are worktree files, so §5.1 composes them into every subsequent model call; the manifest orders and budgets them like anything else (§5.2). The path is the hint (§5.3): `messages/007-user.md` reads as provenance at a glance.
- **Replay is the read-state rule.** A step's read state is a commit (§2.3, §2.10). Because delivery is a commit landing ahead of the model call, replay from `meta.json`'s `commit` reproduces exactly what the step saw — a message cannot appear in a prompt without a commit that carries it.
- **Curation is deletion.** The agent may `rm` a stale message (§5.4); the compactor may `mark_for_deletion` delivered messages like any other worktree file (§2.7).
- **Merge-back is the merge=ours discipline.** `messages/**` joins `goal.md`, `soul.md`, `summary/**` (§2.6): a subagent's delivered messages are its own context, never part of the parent's post-merge state.

Step-boundary delivery is not a compromise; it is the only possible seam. Mid-step injection is structurally impossible twice over: the provider wire requires every `tool_use` matched by a `tool_result` in the immediately following user message (§2.5), and a mid-step message would have no read-state home (§2.3). A deposit therefore never interrupts in-flight work — interruption remains `lernie stop` (§2.9), a different verb because it is a different act.

A parent parked in a blocking `await(handle)` (§2.5) is this same rule at longer range: its next step boundary is merely distant — deferred until the await resolves — so a message deposited meanwhile waits in the inbox and delivers at that boundary, exactly as any deposit does. A blocking await widens the gap between boundaries; it does not move the seam. Reminder-shaped children that must be seen mid-flight therefore pair with the parent's non-blocking `check(handle)` polling (§2.5), never a blocking await — the blocking form defers delivery, it never weakens the only-possible-seam invariant.

**The executor lock: at most one step loop per branch.** Messaging makes concurrent invocations against one branch an ordinary event — a user and a subagent acting on the same conversation at once — so the exclusivity that was previously implicit becomes a stated invariant: **at most one executor per branch**. The mechanism is `flock(2)` on the conversation's inbox directory fd, acquired non-blocking when an executor starts and held for the whole step loop. The lock is kernel state bound to process lifetime: released by the kernel on any death, observable but never written — the same family as the fd-open `in_flight` signal (§2.9, §3.5, §4.4). Liveness is observed, not stored, and there is no stale-lock cleanup because there is nothing on disk to go stale (`docs/PRINCIPLES.md` "Single source of truth"). The lock fd is deliberately inherited across the §6 exec baton (`lernie advance` exec'ing the next `lernie advance`), so the lease spans the chain exactly as long as the chain lives.

**Writer/driver totality.** Every invocation against a branch is exactly one of two things, never both. A **writer** (`lernie message`) deposits and exits; its lock try-acquire is the liveness probe above — released immediately, never held to drive — and a writer never contends for execution at all. A **driver** (`lernie prompt`, `lernie dispatch`, `lernie advance`) acquires-or-exits: winning the acquire, it *is* the executor and runs the ordinary synchronous loop; losing it, it has nothing to deposit and exits as a clean no-op — not an error, because the branch being already driven is the condition it exists to establish. No abandonment, no takeover, no watcher left behind: the loser terminates, and any process that wants to *observe* the branch does so through the read-only frontend contract (§3.5), which is not a role the lock assigns. Because no verb combines the arms, the losing path is the same code as the uncontended one — a deposit is a deposit, a no-op is an exit — and there is no second-class branch of a combined flow to drift out of step. This closes a gap that predates messaging: §6's idempotence rule covers *sequential* replay of `lernie advance`, and the executor lock is its concurrency half — two simultaneous drivers against one branch resolve to one executor and one no-op, never two step loops.

**Undelivered is derived.** A deposit can land in the gap between an executor's final drain and its termination; the no-live-executor refusal above is best-effort against observed state, so this race is acknowledged rather than fenced. The truth stays on disk: a message file still under `inbox/<conv-id>/` whose conversation has no live executor is *undelivered* — a derived classification (inbox listing + the lock probe), never a flag. The sender observes its outcome the way anything is observed here: its file left the inbox in a delivery commit, or it didn't. Undelivered count is a §8 health metric alongside unmerged branches.

**Self-messages are legal and unremarkable.** A conversation may deposit into its own inbox — a note its own next step boundary delivers. No special case arises; the sender happens to equal the recipient.

---

## 3. Component Architecture

### 3.1 Disk as Bus

All components communicate through the filesystem. No shared memory, no direct function calls between components. This applies to:

- Harness → provider adapter (request mirrored to disk, adapter reads from stdin; response events mirrored back to disk from adapter stdout — see §4.4).
- Harness → tool execution (tool call record written to disk, executor reads, output streamed to disk).
- Harness → UI: the filesystem is the event stream. The UI watches paths in the conversation repo (git refs; worktree contents `goal.md`, `soul.md`, `summary/`, `descriptions/`, `skills/`; the conv-repo-root step records under `steps/<conv-id>/NNN/` per §2.2; the conv-repo-root inboxes under `inbox/<conv-id>/` per §2.11; and the conv-repo-root control files) and re-renders on change. Notification is inotify where available, polling otherwise.
- UI → Harness: user actions are issued as `lernie <subcommand>` invocations per §3.4. There is no input directory.

**Threads, not processes.** "Worker" and "executor" name roles that run as threads inside the single harness process; the disk contract is not an inter-process bus. Tool subprocesses invoked by the tool executor are genuinely separate processes. Routing inter-role communication through disk — even between threads in the same process — is load-bearing rather than ceremonial: it is what buys inspectability, audit trail, and the single-author-per-file discipline that keeps many concurrent workers from corrupting each other's state.

Notification mechanism: inotify where available, with a polling fallback. Every event write updates a `last_event_ts` sentinel file as a sanity check for consumers to detect missed events.

Consequences:

- Every component is independently restartable (threads respawn; subprocesses are relaunched from disk state).
- Replay from any point is `git checkout <ref>` + re-tail the event log.
- Latency is higher than in-memory IPC. Accepted and compensated by streaming-first UI design.

### 3.2 Components

- **Harness** (permitted synonym: **daemon**). The single program that drives execution: watches for events, spawns branches, runs model calls via the provider adapter layer (§4.4), invokes the tool executor, triggers merges and compactions, updates state. It owns all external↔filesystem interaction on the repo — provider endpoints (via adapter subprocesses), tool subprocesses, git operations. Stateless across restarts — resumes from disk. Any place this document says "the harness does X", it is this component. "Daemon" is allowed as a shorthand; both refer to the same role.
- **Tool executor.** Runs tool subprocesses on behalf of the harness; contract in §3.3. Per tool call: assembles stdin from the `tool_use.input` the model emitted; invokes the tool binary (`lernie tool <name>` in-process or `lernie-tool-<name>` external); captures stdout and stderr atomically (temp path + rename) into `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/output.json` (out of every worktree, §2.2, §2.3); maps the exit code to the `is_error` flag on the `tool_result` block the harness builds when assembling the next step's request payload. Cascades SIGTERM on cancel (§2.9) with a 5s deadline before SIGKILL (§3.3). Termination by a signal other than the harness's own SIGTERM (SIGSEGV, SIGABRT, etc.) is a harness-level fault per §2.10. Does not auto-dispatch on oversized tool output in v0.3 (§11).
- **Provider adapter.** External binary — brazen's `bz`, one binary for every provider — that owns HTTP, wire dialects, and auth. Invoked per attempt over stdio; non-resident (process per attempt, no long-lived state). Transient-error retry belongs to the harness (§2.10), not the adapter. Contract in §4.4.
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

1. **Binary.** An executable invoked by the harness. **In-process** tools are subcommands of the `lernie` binary, addressed as `lernie tool <name>` — the default for tools shipped with the harness (v0.3 ships `bash` and `read_file` here). **External** tools are standalone binaries named `lernie-tool-<name>` — the externalization pattern that lets contributors ship tools without patching the core (the provider layer used the same per-name pattern until v0.6 retired it for brazen, §4.4). The harness looks up `lernie-tool-<name>` at `<harness-root>/tools/` (installed by `make install`) before falling back to `PATH`. The choice of flavor is per-tool; the stdio contract below is identical.
2. **JSON schema.** Declares the tool's `input` parameters at `<harness-root>/tools/<name>.json`. Required by provider APIs. Either generated from the binary's metadata or hand-authored. Sent verbatim as the schema of the tool's entry in the canonical request's `tools: [...]` array (a brazen `Tool::Custom` — each protocol projects it into its own spelling, §4.4). The harness commits a copy under `descriptions/tools/` at conversation creation time, inherited by every branch via git, and composes it into the context.
3. **Skill.** A `SKILL.md` describing when and how the tool should be used. Required for every tool; follows the skill lifecycle above. The frontmatter `description` becomes the `description` field of the tool's entry in the `tools: [...]` array.

**Tools-list assembly.** A role's enabled tools are declared in the per-repo `<conv-repo>/providers.yaml` `roles:` section under a `tools: [...]` field (see §4.3). Any on-disk triple (binary + schema + skill) is generally discoverable; role configs select from that pool. At request-assembly time the harness composes the declared names against the schemas committed in the branch's read-state tree (§2.10): each declared tool whose `descriptions/tools/<name>.json` schema is present becomes one entry in the canonical request's `tools: [...]` array, with that file sent verbatim as `input_schema`; a declared name with no committed schema is dropped (the intersection of declaration and availability), and a present-but-malformed schema is declined rather than dropped. **Shipped-state note:** the entry's `description` (point 3, from `SKILL.md` frontmatter) is wired with the descriptions-always population; the `input_schema` composition above is independent of it and lands first, so an entry may currently carry a schema with no top-level `description`.

**Stdio contract.** Identical for in-process and external tools:

- **Stdin.** The `tool_use.input` JSON object the model emitted, passed verbatim. The tool owns its own input-schema validation; the harness does not interpret the payload beyond extracting it from the `tool_use` block.
- **Stdout.** Raw bytes. The harness wraps them as the `content` of the `tool_result` block returned to the agent on the next step. No JSON envelope around tool output — tools stay simple and the canonical `tool_result` shape is the harness's concern.
- **Stderr.** Raw bytes. Captured to the on-disk record regardless of exit status; concatenated into `tool_result.content` after stdout when the tool exits non-zero so the agent sees the failure message.
- **Exit code.** 0 → `tool_result.is_error = false`; non-zero → `tool_result.is_error = true`. Termination by a signal other than the harness's own SIGTERM (SIGSEGV, SIGABRT, etc.) is a harness-level fault per §2.10 — the step aborts and the branch is flagged, not delivered to the model as a semantic error.
- **Environment.** The harness sets two env vars on every tool subprocess so tools that depend on conversation context can read them without the model having to thread context through the input schema: `LERNIE_CONV_REPO` (absolute path to the conv-repo root, §2.2) and `LERNIE_CONV_BRANCH` (the calling conversation's branch == its full hyphenated descent / conv-id, §2.2 / §2.3). Both are derived from the executor's `step_dir` so they are guaranteed-correct for the call. Tools that do not need them ignore them; the v0.4 `dispatch` and `await` built-ins are the canonical readers. The direction and discipline are fixed: harness → subprocess, and the harness owns the names.
- **SIGTERM and deadline.** The harness sends SIGTERM on cancel (§2.9); the tool has 5 seconds to flush and exit cleanly, after which SIGKILL follows. The flush deadline is tools-only: the provider adapter needs none — `bz` dies on SIGTERM at once, and the missing trailing `end` is the signature (§4.4 Cancellation).

**Disk record.** The tool executor (§3.2) lands two files per tool call under `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/` — at the conv-repo root, outside every worktree (§2.2, §2.3):

- `input.json` — the `tool_use` block from the model verbatim (`id`, `name`, `input`).
- `output.json` — `{stdout, stderr, exit_code, started_at, ended_at}`.

`<tool-id>` is the `tool_use.id` from the wire (e.g. `toolu_01abc…`). Writes use temp-path + atomic rename. These records are not git-tracked (the location is outside every worktree).

**Commit-per-side-effect, serialized.** A tool call's *diagnostic* record (above) is not a commit — it is an out-of-worktree plain file. A tool call's *worktree side effects* (e.g. `bash` editing files in the branch's worktree) are committed on the emitting branch by the harness before the next tool runs. Sibling tool calls running in parallel serialize their worktree commits, since a dirty worktree between siblings would violate "Single author per file" (§2.5, PRINCIPLES). Tools without worktree side effects (e.g. `read_file`) produce no commit; the per-call `output.json` outside the worktree is sufficient for downstream framing (next paragraph).

**Wire `tool_result` framing is application-layer.** The wire-level `tool_result` blocks the agent reads on its next step are *not* a structural commit. The harness assembles them at request-assembly time by reading step N's per-call `output.json` files when constructing step N+1's request payload; the resulting payload is captured in step N+1's diagnostic `request.json` (§2.3) — that file records what the model saw, but is itself diagnostic-only. The per-call `output.json` files are the single source of truth for tool output during execution and for assembler-driven replay (§5, §2.10); the next step's `tool_result` blocks are derived. "Tool in progress" is derived state too — step N's `response.json` carries a `tool_use` block with no matching `output.json` yet — not a separate file.

**Deferred to v0.4+ (see §11).** Oversized-output auto-dispatch — raw output handed to a parsing subagent, only the compacted result reaching the parent step — is not in v0.3. Oversized output reaches the agent unchanged.

**Sandbox (v1.1, §3.6).** The v1.1 milestone wraps this contract in a capability sandbox derived from the artifact kind — a `wasm32-wasip2` component runs WASI-clamped, a native binary runs only under an `exec` grant. The stdin/stdout/exit protocol above is unchanged; the sandbox bounds *authority*, not *interface*. See §3.6.

### 3.4 CLI as control plane

`lernie` is a single binary with subcommands. Every procedure the harness can start — subagent dispatch (§2.5), compaction (§2.7), verification, and any other workflow-invoked procedure (§6) — is reachable through a subcommand of that binary. The CLI is the sole entry point: a procedure invoking another procedure does so by going through the CLI dispatcher, never through in-process function calls, shared memory, or ad-hoc sockets. Subagent dispatch, the canonical case, is `lernie dispatch …`; messaging an existing conversation is `lernie message …` (§2.11), the same front door whether the invoker is the user, a frontend, or another conversation's `message` tool.

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

1. **Filesystem reads.** The frontend reads and watches paths under the conversation repo. The load-bearing paths follow the repo layout (§2.2): the git tree itself (refs, commits, objects — branch state is read from `refs/heads/` per §2.3); the conv-repo-root control files (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `souls/`); the conv-repo-root step records under `steps/<conv-id>/NNN/` (§2.3); the conv-repo-root inboxes under `inbox/<conv-id>/` (§2.11); and each branch's worktree contents (`goal.md`, `soul.md`, `summary/`, `messages/`, `descriptions/`, `skills/`). Notification is inotify where available, polling otherwise (§3.1).

   **Streaming text watch path.** Live model-call output is tailed at `<conv-repo>/steps/<conv-id>/<NNN>/response.json` — the JSONL stream of canonical events (§4.4) is appended event-by-event as the adapter writes it. **Completion signal: writer closes the fd; inotify `IN_CLOSE_WRITE` marks the response complete.** The frontend tails the file (offset-tracking line read), and on `IN_CLOSE_WRITE` flips the response from "in flight" to "done" — no separate sentinel file, no out-of-band marker. This is the same fd-close convention used elsewhere in the architecture (e.g. atomic-rename writes, where `IN_CLOSE_WRITE` on the temp path precedes the rename); here the streaming append *is* the writer, so close-of-fd directly signals end-of-stream.

   **Branch-state classification.** The four branch states the live view renders (§7.1) are derived from refs and the JSONL terminal event, not from any sidecar marker. `merged`: the branch is reachable from `main`'s HEAD (`merge-base(branch, main) == HEAD(branch)`). `in_flight`: branch is unmerged and the latest step's `response.json` is still open (no `IN_CLOSE_WRITE` yet). The harness holds this fd open across *all* of a model call's attempts and the backoff sleeps between them (§4.4 "Fd held open for the whole model call"), so `in_flight` covers the entire retry loop — an intermediate failed attempt's trailing `end`, or a mid-retry `Error` segment, is still `in_flight`, never `stopped`, because the file is still open. The terminal classifications are evaluated only once the fd closes. `stopped`: branch is unmerged, the latest step's `response.json` is closed, and its last JSONL line is not `end` (§2.9 — kill, crash, and explicit stop are indistinguishable on disk; a closed file whose last segment carries an `Error` event is a *failed* step per §2.10, rendered as stopped with the error surfaced). `conflicted`: derived from the integration attempt per §2.6 step 6, not from a property of the branch itself; relevant once subagent merges ship (v0.4+). Precedence at render time is `merged > stopped > in_flight`; `conflicted` is orthogonal and rendered alongside.
2. **CLI invocations.** The frontend issues user actions by `exec`'ing `lernie <subcommand>`. New prompt, stop, fork-from-history — all are ordinary CLI subcommands per §3.4. There is no separate API surface, no socket, no shared input directory, no library port. There is no user-facing "resume": continuing from a stopped branch is `lernie prompt` (new conversation, stopped branch as context) or fork-from-history (new branch off the stopped tip), per §2.9.

Frontends hold no persistent state. Everything a frontend renders is derived from the filesystem at the current git ref; ephemeral UI state (cursor position, scroll offset, selection) lives in memory only and is discarded on exit. Restart is equivalent to re-reading the repo.

This discipline is what makes pluggability structural rather than aspirational. Two frontends running against one repo cannot corrupt each other because neither writes repo state; both observe the same on-disk ground truth, and both issue commands through the same CLI surface the harness itself uses. Swapping a frontend out is unplugging one reader; adding a second is adding another reader.

### 3.6 Sandboxed tools (v1.1)

Forward spec for the v1.1 milestone (§12). The sandbox is a **hosting wrapper** around the §3.3 tool contract, not a change to it: an external tool artifact runs inside a bounded authority envelope while still speaking the identical stdin/stdout/exit protocol. Everything §3.3 says about how a tool is invoked and how its output is framed holds verbatim; §3.6 adds only *with what authority* the artifact runs. In-process built-ins (`lernie tool <name>`, §3.3) are subcommands of the trusted `lernie` binary — they *are* the harness, not guests, and run with its authority; the sandbox governs only **external** artifacts (the `lernie-tool-<name>` slot at `<harness-root>/tools/`). Shipping a tool in-process is the decision to place it in the trusted computing base.

**Flavor is derived from the artifact, never configured.** The host reads the artifact's bytes and dispatches on format — there is no `sandbox:` field, no `native:` flag, nothing in `providers.yaml` selecting *how* a tool runs:

- A **`wasm32-wasip2` component** (WebAssembly component-model preamble) runs under a wasmtime host with WASI (WebAssembly System Interface) clamped to the effective authority below.
- A **native executable** (ELF or any host binary) is not a WASI guest and cannot be clamped; the only thing between it and the host is the `exec` capability. It runs *iff* the effective grant includes `exec` (below), with full host-process authority.

The flavor being a property of the artifact — not of config — is the severability line (`docs/PRINCIPLES.md`): deleting the sandbox deletes no config field, because the choice was never a field. Removing a default deletes config, not code.

**Manifest, grant, effective authority.** Two inputs meet on five axes — **fs** (filesystem scopes), **net** (egress hosts), **exec** (spawn a host process), **clock** (read wall-clock time), **env** (host environment variables):

- The **capability manifest** is what a tool *declares it needs*. For a WASM component this is not a sidecar file — it is the component's own **WASI imports**, read from the artifact (a component that never imports `wasi:sockets` cannot open a socket; the manifest cannot be under-declared, because a non-imported interface has no host function to reach). Deriving the manifest from the imports is the single-source rule (`docs/PRINCIPLES.md` "Single source of truth"): a sidecar manifest could drift from the actual imports; the imports cannot drift from themselves.
- The **capability grant** is what the **role** *permits* — a `grant:` block in the role's `providers.yaml` entry (§4.3), the ceiling for every tool the role may invoke. **Default: empty.** A role with no `grant:` runs tools with no authority at all, so only a pure-compute tool (empty manifest) loads.
- The **effective authority** a tool runs with is the intersection **manifest ∩ grant** — never a third stored field, always computed at load. Because the load gate below guarantees manifest ⊆ grant, the intersection equals the manifest: a tool gets exactly what it imports, and the grant is the ceiling that must contain it, never a floor that dilates it.

**Native is the `exec` grant.** A native artifact carries no WASI imports to clamp, so running it *is* spawning a host process with host authority — exactly the `exec` capability. A native tool therefore loads only under a role granting `exec`, and then runs unclamped. `exec` is the same axis a WASM component needs to spawn a subprocess of its own: one axis, one meaning — "may hold host-process authority." With the empty default, no native tool runs until a role explicitly grants `exec`; the wide-authority escape hatch is one legible, opt-in word.

**A tool asking beyond its grant fails at load, loudly.** The load gate is **manifest ⊆ grant**, checked when the harness resolves a role's toolset — before any model call. A tool whose manifest names an axis, host, scope, or mode the grant does not cover is rejected with an error naming the tool and the offending axis; it is never silently clamped to a crippled subset that would fail mysteriously at runtime (`docs/PRINCIPLES.md` "Decline illegal operations": silent degradation is never preferable to a loud refusal). The check is total and static: a role's grant either contains a tool's needs, or the repo does not load.

**The grant grammar.** The grant attaches to the §4.3 role block as one `grant:` map — five keys, no nesting beyond a flat list per axis, no per-tool escape hatch, no freeform field to validate:

```yaml
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-4-7
    tools: [bash, read_file]
    grant:                        # v1.1 — omitted means empty (no authority)
      fs:  [rw:.]                 # <mode>:<subtree>, mode in {r, rw}, worktree-relative
      net: [api.anthropic.com]    # exact egress hosts; no wildcards, no ports
      exec: true                  # may spawn / be a host process
      clock: true                 # may read wall-clock time
      env:  [PATH, HOME]          # host env-var passthrough allowlist
```

Each axis is exactly as wide as its audit unit and no wider: **fs** scopes are `<mode>:<path>` with mode `r` or `rw` over disjoint worktree-relative subtrees (§5.1), so `rw:.` reads at a glance as "read-write the worktree, nothing above it" (granting a parent subsumes its children — do not list both); **net** is a list of exact hostnames — wildcards and port syntax are rejected, because a wildcard host defeats audit-at-a-glance; **exec** and **clock** are booleans, since a host process and a clock are each all-or-nothing; **env** is an allowlist of variable names (the harness-owned `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH`, §3.3, are the contract and always present — never part of the grant). The booleans-and-lists shape is not imposed; it is the natural granularity of each axis. If a grant needs a comment to be understood, the grammar has failed — the inline comments above annotate the *grammar* for this spec, not any real grant. The `grant:` block is the one new config knob the sandbox adds; it is the severability home for capability policy, and everything else the sandbox needs — the flavor, the manifest — is derived from the artifact.

**Boundaries — what the clamp does not solve.** The clamp bounds a tool's *authority*, not the trustworthiness of its code:

- **Supply-chain trust.** A malicious component granted `net: [evil.example]` will exfiltrate to `evil.example` — the host permits exactly the grant. The clamp is an authority bound, not a provenance check. The mitigation is the empty default plus the legible grant: authoring a grant is the operator vouching for the artifact, a decision made once and auditable at a glance. Artifact signing/provenance is out of scope (deferred, §11).
- **Escalation via chained tools.** A tool with `exec` already holds host-process authority; anything it spawns is that grant working as designed, not an escalation — `exec` *is* the ceiling, which is why it is one explicit opt-in axis defaulting off. A tool without `exec` can spawn nothing. Cross-tool data-passing through the worktree is bounded by fs scopes and by single-author-per-file (§2.6): sibling tools cannot target the same file.
- **Clock and env covert channels.** A granted `clock` permits timing side channels and a granted `env` var can carry data; both default off/empty, so the channel is closed unless a role opens it deliberately. The harness holds no provider credentials to leak (brazen owns them, §4.4), so an empty `env` grant exposes nothing sensitive by construction. The residual — that granting an axis grants its misuse — is the accepted cost of the grant, kept visible by the grammar.
- **Resource exhaustion.** The grant bounds *where* a tool acts, not how much it spends. Wall-time is bounded by the SIGTERM deadline (§3.3) and by budgets (§6, `max_wall_seconds`); a disk/CPU quota beyond that is deferred (§11).

---

## 4. Providers, Auth, and Models

### 4.1 Provider abstraction

The taxonomy (`docs/TAXONOMY.md` §2) flags "provider" as one of the field's most overloaded terms, naming three distinct roles: **model creator** (trains the weights), **inference provider** (serves the weights over an API), and **gateway** (unifies multiple inference providers behind one surface). This document uses **provider** in the *inference provider* sense throughout: an (endpoint, auth) pair.

The harness does not speak provider wire protocols directly. Every model call goes through the **provider adapter** (§4.4) — brazen's `bz`, one external binary invoked as a subprocess per attempt. One binary serves every provider: which providers exist, their endpoints, their auth, and their wire dialects are **brazen's** facts, declared in brazen's own config (`~/.config/brazen/config.toml`, overridable via `--config`/`BRAZEN_CONFIG`; brazen's `config.md` is authoritative) as named provider rows. lernie references a provider row by name and never reads endpoint or credential material — the row name is the entire provider surface lernie sees. In the vocabulary of this spec, "provider" names the `(endpoint, auth)` pair (a brazen row); "provider adapter" names the binary. They are distinct terms of art and not interchangeable.

> **Historical.** v0.1–v0.5 shipped a bespoke per-provider adapter contract: one `lernie-provider-<name>` binary per provider, `describe`/`complete` subcommands, an Anthropic-Messages-shaped canonical request, a home-grown event vocabulary, and env-var forwarding for auth and endpoints (`auth_env`/`endpoint_env`). v0.6 retires all of it in favor of brazen (§4.4). The externalization *principle* survives — the provider integration is still a separate binary behind a narrow stdio contract — but the contract is now brazen's canonical protocol, and there is one adapter binary rather than one per provider.

**Config split by lifetime and owner:**

- **brazen's config** (`config.toml`; brazen's tooling — `bz --dump-config`, `bz --login` — edits and inspects it). Provider rows: protocol, endpoint, auth mode, model aliases, per-row body defaults. Rotates with key rollover and infrastructure changes.
- **Global lernie** (`<harness-root>/models.yaml`, replacing the retired global `providers.yaml`). Model capabilities and context windows (§4.2) — facts lernie's behavior relies on and brazen does not own — plus the optional `adapter:` binary override (§4.4 Extensibility).
- **Per-repo** (`<conv-repo>/providers.yaml`). Role → (provider row, model id) mapping and role toolsets only. Frozen at conversation creation (§2.2); governs which model this conversation's roles dispatch to for the rest of its life.

The frozen-bootstrap property is preserved: a conversation repo pins *which row and model* its roles use; endpoint and auth resolve at call time inside brazen, so rotating a key or endpoint immediately affects in-flight conversations (correctly), and the per-repo file never carries machine-local or secret material. Retry policy is not provider config at all — the attempt cap and backoff are workflow policy and live in `workflow.yaml` (§6).

**Row names are the portability contract.** Because a conversation repo pins provider *row names* — not endpoints or credentials — replaying or moving a repo to another machine requires that machine's brazen config to define rows of the *same names*. This is the same expectation v0.3's per-provider adapter binary names carried (a repo naming provider `anthropic` needed a `lernie-provider-anthropic` there, a row named `anthropic` here), now stated rather than left implicit. A row absent on the target machine is a load-time failure — brazen cannot resolve it and lernie declines the model call (`docs/PRINCIPLES.md` "Decline illegal operations") — never a silent fallback to a different provider. The row *name* travels with the repo; its endpoint, auth, and model aliases stay machine-local by design (they rotate; the config split above).

### 4.2 Model abstraction

A **model** is (provider row, model_id, capabilities). Capabilities is an extensible mapping declaring features the harness can rely on. The `models:` block lives in the global `<harness-root>/models.yaml`:

```yaml
adapter: /usr/local/bin/bz        # optional override; default is `bz` on PATH (§4.4)
models:
  claude-sonnet-4-7:
    provider: anthropic           # a brazen provider-row name
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native, prompt_caching, streaming, stop_sequences]
    context_window: 200000
```

Capabilities are code-backed (each capability has a behavior implementation in the harness). Capabilities are extend-only: once declared, they are never removed from the registry, but the set on a given model may shrink if the provider removes support. The loader seeds a known-name registry from the names that appear in this spec; an unknown name on load produces a warning, never an error, so a new provider may declare new capabilities without blocking parsing.

Model-id validity is brazen's concern, not lernie's: `bz` resolves the id against its per-provider model cache (refreshed by `bz --list-models`) and otherwise attempts it verbatim on the wire. lernie performs no model-list calls of its own; `models.yaml` carries only the facts lernie *acts on* (capabilities, context window), not a mirror of what the provider serves.

### 4.3 Role-based model assignment

Agent roles specify which model to use. This allows cheap models for compaction and expensive models for the worker. The role → model mapping lives in the conversation repo's `providers.yaml` (frozen at creation, §2.2):

```yaml
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-4-7
    tools: [bash, read_file]
  compactor:
    provider: anthropic
    model: claude-haiku-4-5
```

Each role's system prompt is read from `<conv-repo>/souls/<role>.md` by convention — there is no per-role path override, and no freeform path field to validate. At dispatch time the harness copies the appropriate soul to the new branch's `soul.md` (§2.3 step 2). `provider:` names a brazen row; endpoint and auth resolve at call time inside brazen (§4.1) — the per-repo file carries only the (row-name, model-id) pointer, cross-validated against `<harness-root>/models.yaml` at load.

The optional `tools:` field selects which tools the role's agent can call (see §3.3). Omitted or empty means none. The compactor's toolset (`write_summary`, `mark_for_deletion`) is built into the compactor primitive (§2.7), not declared here.

**(v1.1)** A `grant:` block may sit beside `tools:` in the role entry, declaring the capability ceiling — fs/net/exec/clock/env — for every tool the role invokes. Omitted means empty (no authority). The grammar and its manifest-intersect-grant semantics are §3.6; the grant is the one new config knob the sandbox adds, and it is the severability home for capability policy (removing it deletes config, not code).

### 4.4 The provider adapter: brazen

The **provider adapter** is [brazen](https://github.com/mudbungie/brazen) — one small, stateless binary (`bz`) that adapts every provider and wire protocol behind a single pipe contract:

```
stdin (canonical request, JSON) → bz → stdout (canonical event stream, NDJSON, one terminal `end`)
```

brazen's own specs are the authoritative contract — its `architecture.md` (canonical request §3.1, `v=1` event vocabulary §3.2–3.4, exit codes §8, CLI surface §5.10), `config.md`, and `providers.md`. This section binds lernie to that contract and records the division of labor; it deliberately re-specifies nothing brazen already owns. brazen is our own project: pieces the harness needs that brazen lacks are built in brazen, not worked around here.

**Invocation.** One `bz` process per **attempt** (§2.1): `bz --json --provider <row>`, canonical request on stdin. The model id rides the request; config resolution is brazen's (`--config` / `BRAZEN_CONFIG` / XDG — the harness sets `BRAZEN_CONFIG` only under test isolation). Streaming is brazen's default and lernie never overrides it; a provider whose wire response is non-streaming is normalized by brazen into the same event stream. The v0.3 "synthesize JSONL from a non-streaming response" path is deleted — every attempt's stdout is already the on-disk shape.

**The vocabulary is linked; the data plane is exec'd.** lernie links the `brazen` crate at an exact pinned version for the canonical *types* — `CanonicalRequest` and its content vocabulary, `Event`, `CanonicalError` — and builds requests as typed structs, which makes brazen's fail-open `extra` map unreachable (the typo hazard of hand-built JSON does not exist). This is a shared vocabulary, not a channel: no lernie code calls brazen's `generate()`; every model call crosses the subprocess boundary, preserving §3.4's discipline. Two consequences:

- **`retryable` has one home.** Retry classification is `CanonicalError::retryable()` from the linked crate — a computed query lernie never re-implements (§2.10).
- **Version skew is guarded.** At load, `bz --version` must equal the linked crate version; a mismatch is rejected — no silent downgrade (PRINCIPLES "Decline illegal operations"). `make install` installs the pinned binary (`cargo install brazen --version =<pin>`). Under an `adapter:` override (§4.2) the version guard is skipped and the in-band event-schema handshake (`MessageStart.v = 1`) governs instead.

**Attempts and retry — the harness's job.** brazen never retries (a stated brazen non-goal); each `bz` process performs exactly one HTTP round-trip. The harness owns the retry loop (§2.10): on an in-band `Error` event whose kind is retryable, it re-invokes `bz` with the identical request — the assembler is deterministic from the step's recorded `commit`, so no request drift is possible — up to the attempt cap and backoff declared in `workflow.yaml` (§6). Backoff is purely config-driven — the exponential schedule in `workflow.yaml`, keyed off the attempt number — and consumes no provider pacing hint: the pinned brazen `CanonicalError` is `{kind, message, provider_detail}`, carrying no `Retry-After` / `retry_after_seconds`, so there is no hint to honor. (A pin-version fact, not a permanent stance: were a later brazen to surface a pacing hint in-band, the harness could clamp its backoff to it.) One attempt ≡ one API call *by construction*, so "API calls per model call" (§8) is derived, never stored: it is the number of attempt segments in `response.json`.

**On-disk response shape: appended attempt segments.** The harness appends each attempt's stdout verbatim to the step's `response.json`. brazen guarantees every stream — success, refusal, or failure — ends with exactly one `{"type":"end"}` line, so the file is a sequence of self-delimiting **segments**, one per attempt, and the last segment is authoritative; earlier segments are the audit trail of failed attempts. Reading rules (shared by §3.5 classification, `await`, and replay tooling):

- *in flight* — the file is open (no `IN_CLOSE_WRITE` yet).
- *complete* — closed, last line `end`, last segment carries a `Finish` (any reason, including refusal) and no `Error`.
- *failed* — closed, last line `end`, last segment carries an `Error` event (retry budget exhausted or non-retryable); the branch is flagged per §2.10.
- *stopped/killed* — closed **without** a trailing `end` line: the writer died mid-stream (§2.9). Kill, crash, and explicit stop remain indistinguishable on disk and are treated identically.

**Fd held open for the whole model call.** The harness opens `response.json` once, at the model call's first attempt, and holds the fd open across *every* attempt and *every* backoff sleep between them (§2.10) — closing it only at step resolution, when the loop settles into `complete`, `failed`, or `stopped`. This makes fd-open the single `in_flight` signal and the four reading rules above **terminal-only**: they are evaluated once the fd is closed. While it is open the step is `in_flight` no matter how many `end`-terminated segments it already carries, so an intermediate failed attempt's trailing `end` never reads as `complete`, and a mid-retry `Error` segment never reads as `failed` — the retry is still pending and the file is still open. This is the same fd-open observation the `/proc/<pid>/fd` writer scan reads in §2.9 and `lernie stop` (§3.4), and the same signal §3.5 classifies on: one open fd, three readers.

**Refusal is a completed model call, not a failure.** brazen surfaces a provider refusal as `Finish{Refusal}` on HTTP 200, exit 0 (brazen `architecture.md` §3.2) — its own truth, never an `Error` — so a refusal segment classifies *complete* (above), with no retry and no operator flag. A root conversation surfaces the refusal as its terminal response, exactly like any other terminal `Finish`. A subagent's refusal reaches its parent through the ordinary path: terminal compaction (§2.7) summarizes it and `await(handle)` (§2.5) returns that summary — no distinct status, because a refusal is a normal completion the parent agent reasons about, not a harness fault. The workflow layer needs no refusal-specific event; the ordinary completion bindings (`worker_return` etc., §6) fire, and any refusal policy is a binding on that existing signal, not a new one.

**A `Finish{Pause}` segment is never terminal.** A paused model call (brazen `FinishReason::Pause`, Anthropic `pause_turn`) is continued *within the same step* (§2.10 Resumable pauses), so the fd stays open across the continuation and the segment always reads *in flight* by the rule above — the classifier never sees `Pause` as a settled state. Pause arises only with provider server tools, which lernie's stdio tool contract (§3.3) does not wire, so it cannot occur in v1; the rule is stated so the classification is total.

**Errors.** brazen surfaces every failure in-band as an `Error` event on stdout and also sets a sysexits exit code computed from the same fact. The event is authoritative: the harness classifies from the parsed `CanonicalError` and treats the exit code as diagnostic. A `bz` process that dies without emitting a trailing `end` is the kill signature above — handled by §2.9/§2.10, never delivered to the model.

**Cancellation.** brazen installs no signal handlers: SIGTERM kills `bz` at once (default disposition, exit 143), dropping the in-flight HTTP request; already-flushed NDJSON lines stay valid. The missing trailing `end` on the closed fd *is* the stop signature — no flush deadline, no cancel marker, determinism via absence of mechanism. (The 5-second SIGTERM→SIGKILL deadline remains the contract for *tools*, §3.3, which may need to flush real work.)

**Auth and endpoints.** Entirely brazen's: provider rows carry auth mode and endpoint; credentials live in brazen's 0600 credstore; interactive flows are `bz --login`, operator-run, never harness-run — the harness never prompts and never sees credential material. The v0.3 `auth_env`/`endpoint_env` forwarding machinery is retired with the rest of the bespoke contract.

**Fit with disk-as-bus (§3.1).** Unchanged in shape: the adapter's stdin and stdout are pipes; the harness mirrors the assembled canonical request to the step's diagnostic `request.json` (one per step — attempts share the same assembled request) and appends events to `response.json`. Both remain outside every worktree and outside context assembly (§2.3); the terminal-line classification above reads only event framing from `response.json`'s tail — the same observation the frontend makes (§3.5). Because the pinned `CanonicalRequest` sets no `skip_serializing_if`, unset options serialize as explicit `null`, so the mirrored `request.json` shows `"stream":null`, `"temperature":null`, and the like — this is consistent with "lernie never overrides `stream`" (Invocation): `null` *is* the non-override, and brazen's `fill_absent` resolves it to the configured default. The pipes are the wire; the disk is the record.

**Extensibility.** A new provider on a supported protocol is a brazen config row — no code anywhere. A new wire protocol or auth mode is a contribution to brazen. The escape hatch for a deployment that cannot ship through brazen is the `adapter:` override in `models.yaml` (§4.2): any binary honoring the same pipe contract — canonical request in, `v=1` events out, one `end` — slots in verbatim, with the in-band `MessageStart.v` handshake as its compatibility gate.

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
      - messages/**
      - skills/**
    budget_tokens: 150000
    overflow: drop_oldest_summaries
  compactor:
    pinned:
      - goal.md
      - soul.md
    # The compactor's view onto the parent's work is pinned in §2.7
    # (and refined in v0.3.1+ as step records move out of every
    # worktree, §2.3). The compactor's manifest sees only its own
    # worktree, same as any other role.
    budget_tokens: 50000
    overflow: truncate
```

Paths are interpreted relative to the branch's worktree. The manifest sees only worktree contents by construction (§5.1). Pinned paths are always included regardless of budget; `order` entries fill the remaining budget in declared order until overflow policy kicks in.

**Step records are not context.** `worker.order` (and any other role's manifest entries) MUST NOT reference `steps/**` — step records live at `<conv-repo>/steps/<conv-id>/NNN/`, outside every worktree (§2.2, §2.3), and are diagnostic-only (§2.3 Diagnostic-only contract). They are physically excluded from context assembly by their location: a worktree-relative path cannot resolve to them. Examples in this doc and the shipped template manifest reflect that constraint.

This corresponds to the LangChain write/select/compress/isolate taxonomy (`docs/TAXONOMY.md` §3): **write** = commits; **select** = manifest inclusion; **compress** (here: compact) = compactor; **isolate** = subagent-conversation branches.

### 5.3 File path as hint

File paths are preserved in the assembled context as structural hints to the model. The path itself carries information (`summary/003.md`, `skills/git-ops/SKILL.md`, `descriptions/tools/bash.json`) that is cheaper than explicit metadata and often sufficient.

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

retry:
  max_attempts: 3              # attempt cap per model call (§2.10, §4.4)
  backoff: exponential

budgets:                       # v0.7 — spend limits (see "Budgets" below)
  max_total_tokens: 2000000
  max_wall_seconds: 3600
  max_depth: 4
```

Action strings that contain `: ` (named arguments) must be quoted, since YAML otherwise parses them as map entries. Bare actions (no named args) need no quotes.

Actions are implemented in the harness; the workflow declares which run when. The `flush` action emitted by a running agent triggers an intermediate compaction without terminating the branch (§2.7). This is the primary surface for experimentation.

Per-step hooks (`pre_step`, `post_step`, `on_tool_return`) fire on every branch and are the primary extension points for cross-cutting behavior — observability, budget enforcement, cache maintenance, scheduled intermediate compaction triggers. Their handlers typically dispatch subagents or emit log entries rather than modifying the in-flight branch's tree directly; any write still goes through the harness-assigned write-path machinery (§2.5).

**Budgets (v0.7).** `budgets:` declares per-conversation spend limits: `max_total_tokens`, `max_wall_seconds`, `max_depth`. The harness checks them at every model-call boundary, before invoking the adapter. Spend is derived at check time from the `Usage` events already on disk in `response.json` across the conversation tree — no running counter is stored (`docs/PRINCIPLES.md` "Single source of truth"). **Every attempt segment counts:** the derivation sums `Usage` across *all* segments of every step, not only the authoritative last one, because a failed or superseded attempt still consumed provider tokens and real money — the last-segment-authoritative rule (§4.4) governs which segment supplies *context*, never which segments are *billed* (§8). `max_wall_seconds` is likewise wall-clock: it counts the backoff sleeps between attempts (§2.10), since wall time elapses whether the harness is streaming or sleeping — derived by summing each step's `started_at`→`ended_at` span (one span already covers that step's attempts and backoff, §4.4 "Fd held open for the whole model call"). All three axes derive over the branch and its entire descent (`steps/<branch>/` plus every `steps/<branch>-*/`, the §2.9 stop-cascade walk); `max_depth` is the branch's hyphenated dispatch depth (§2.2 — root = 0, each dispatch one deeper). Tokens and wall are consumables and exhaust at `derived ≥ limit` (stopping before the next model call overspends); `max_depth` is the deepest *allowed* depth, so a conversation exhausts only when its depth *exceeds* it — the root (depth 0) is never depth-exhausted. A dispatch hands the child the minimum of the parent's remaining budget and the child's own declaration (per axis; tokens/wall deplete, depth being a shared absolute ceiling). Exhaustion is a harness-issued stop (§2.9) plus a `refs/lernie/budget-exhausted/<branch>` ref — the same git-native marking pattern as the conflicted ref (§2.6 step 6) — which `await(handle)` surfaces as `{status: budget_exhausted}`. Exhaustion is an ordinary terminal state, never a special transcript shape.

**No resident interpreter.** Nothing parses `workflow.yaml` resident-style and drives the state machine from memory. The chain is driven by a single CLI subcommand, **`lernie advance <repo>`**: a fresh subprocess that reads the repo, evaluates workflow rules against on-disk state to determine the next workflow action, runs that action's procedures by exec'ing them per §3.4 (`lernie dispatch <role>`, `lernie merge`, etc.), and before exiting exec's another `lernie advance` to continue the chain. The currently-executing `lernie advance` subprocess *is* the interpreter while it runs — a baton passed forward by exec, not a daemon. The chain ends at a terminal action (final merge, stop, error); there is no watcher noticing completions, each step hands off by exec. Combined with disk-as-bus (§3.1), this keeps the system stateless across process boundaries: a crashed subprocess leaves nothing in memory to reconstruct, and the same `lernie advance` invocation re-enters the chain by reading the repo and running the next action — recovery is not a separate operation, it is the same operation as advancing. This is the concrete mechanism behind §1's Regenerability property.

**Workflow position is derivable from disk.** The workflow's current position is a function of repo state — refs, step records, merge history, sentinel files explicitly named in `workflow.yaml` — never of a `workflow_state.json` or analogous sidecar that mirrors the position. Implementations may not introduce such a mirror. This is the invariant that makes the unification of "advance" and "resume" load-bearing rather than aspirational: if position were stored separately from disk state, then re-entry after crash would require reconciling the mirror against disk, and "advance" and "resume" would need to be different operations to handle the divergence. Holding the line on derivability collapses them.

**Procedures are idempotent under replay.** §2.10 commits to replay-from-recorded-sha for model calls. The corresponding requirement at the workflow level is that every procedure invoked by `lernie advance` must be idempotent under replay from its recorded read-state, or detect already-completed work and no-op. Without this, two `lernie advance` invocations against the same disk state could produce divergent effects, and the position-from-disk invariant above would not be sufficient to make recovery safe. With it, any number of `lernie advance` invocations against the same state converge. Idempotence covers *sequential* re-entry; *simultaneous* drivers against one branch are resolved by the executor lock (§2.11 "Writer/driver totality") — one becomes the executor, the other exits as a clean no-op.

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
- Pending-message indicators for conversations with a non-empty inbox (§2.11); delivered messages appear as ordinary delivery commits in the tree.

### 7.2 History view

Clicking an old commit is read-only by default. Forking from history (creating a new branch from an old commit to explore a counterfactual) is a v1 feature but distinguished in the UI and in branch naming (`fork/<source-ref>/<id>`) so accounting and replay can tell the difference.

### 7.3 Concurrent exchanges

Because every exchange (root conversation) is its own branch and nothing else touches `main` directly, the user may send a second message before the first resolves. Two distinct intents map to two existing primitives: a *new question* is a new exchange — `lernie prompt`, its own root-conversation branch off `main` (or the first's tip, for strict sequencing), running independently; a *course correction to the running exchange* is a message — `lernie message` into the live conversation's inbox, delivered at its next step boundary (§2.11). No special mechanism is required beyond the branch invariant and the inbox.

---

## 8. Metrics and Observability

First-class metrics, written to commit trailers and event log. All counts are reported along three scope axes, which are not interchangeable:

- Tokens per step, per conversation, per conversation repo — summed across *every* attempt segment of each step's `response.json`, not only the authoritative last segment; failed and superseded attempts are billed (§4.4, §6 budgets).
- Model calls per step (always 1), tool calls per step, attempts per model call (≡ API calls, §2.1; derived as the segment count of the step's `response.json`, §4.4 — never stored).
- Cost per step, per conversation, per conversation repo (derived from the all-segments token sum above).
- Unmerged-subagent-branch count per conversation repo. This is a critical health metric: a ballooning count indicates silent failure somewhere in the merge pipeline. Subagent branches are those whose name contains a hyphen (root conversations live on `main`, §2.3); candidates are enumerated by `git branch --list '*-*' --no-merged main`, then filtered to branches with *no live executor* (the §2.11 lock probe) — a long-lived child still working (legitimate per §2.5, §2.11) is not a pipeline failure. Root conversations are intentionally unmerged (§2.3 step 5) and are not counted here.
- Undelivered-message count per conversation repo: files under `inbox/<conv-id>/` (§2.11) whose conversation has no live executor (the §2.11 lock probe). Derived from the inbox listing plus the liveness observation — no flag, no sidecar (§2.11 "Undelivered is derived").
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

- Tool sandboxing and capability restriction — now a named milestone (v1.1, §12) with its spec at §3.6: a WASM/WASI capability clamp over the unchanged §3.3 stdio contract.
- Tool artifact provenance and signing — the v1.1 sandbox (§3.6) bounds a tool's *authority*, not the trustworthiness of its bytes; verifying artifact origin is deferred.
- Multi-user and multi-tenant isolation.
- Distributed execution across machines.
- Secrets management beyond env var injection.
- Sophisticated compaction (rewriting compactors, semantic merging).
- Adversarial compactor defense.
- Adversarial sender defense — front-door messaging (§2.11) lets any sender deposit context into any live branch of the tree, including the root, so a prompt-injected child steering its parent is now one `message` deposit rather than a compaction escape. Provenance is unforgeable and the single-user assumption (§1.1) covers authorization, not influence; the recipient's uniform treatment of every sender (§2.11) is deliberate — uniformity is the feature — so defense is deferred and layered (sender-aware manifest budgets per §5.2, role-scoped send grants) rather than a change to the primitive.
- Cross-conversation memory / shared context.
- Oversized tool-output auto-dispatch (§3.3) — raw output reaches the agent unchanged in v0.3; the parsing-subagent handoff lands alongside v0.4's subagent dispatch machinery.

---

## 12. Milestones

### v0.1 — One model call

**Success criterion:** A single prompt is sent to a provider endpoint, the response is written to disk, and is visible in the conversation repo as a commit. No git branching, no tools, no subagent dispatches.

**Shipped shape.** `lernie new <path>` scaffolds a conversation repo from the embedded template; `lernie prompt <repo> <message>` loads `providers.yaml` + `agents.yaml`, resolves the `worker` role, invokes `lernie-provider-<name>` as a subprocess (§4.1 Historical), writes `exchanges/<ts>-<short-id>.json` with `user_message` / `assistant_response` / `model_id` / `provider` / `usage` / `stop_reason` / `started_at` / `ended_at`, and commits the file to `main`.

**Exceptions to later invariants, historical.** v0.1 committed directly on `main` rather than via an exchange branch merge (§2.3); retired in v0.2, where `lernie prompt` spawns an `ex/<ts>-<id>` branch, commits the step snapshot before the model call (§2.10), lands the response as a follow-up commit, dispatches the terminal compactor off the tip (§2.7, stub in v0.2), and `--no-ff` merges the compacted branch back into `main` (§2.6). The earlier `--endpoint` argv pragma was also retired in v0.2: endpoint forwarding now goes through `describe.endpoint_env` per §4.1 Historical.

### v0.2 — Git tree

**Success criterion:** An exchange is a branch, completion is a no-ff merge back to `main`, the repo layout matches §2.2. User message → exchange branch → steps as linear commits → compactor (deletion-only, stub is fine) → merge. Unmerged branch count metric available.

**Shipped shape.** `lernie prompt <repo> <message>` spawns `ex/<ts>-<short-id>` off `main` in a dedicated worktree under `.lernie/worktrees/`, writes `.agent/goal.md` (§2.8) and `exchanges/<ts>-<short-id>/steps/001/request.json`, commits that snapshot before the model call (§2.10), invokes `lernie-provider-<name> complete`, lands the normalized response at `steps/001/response.json` as a follow-up commit, then dispatches the terminal compactor by re-entering itself as `lernie dispatch compactor <repo> <exchange-branch>` (subprocess per §3.4). The compactor spawns `inv/<ex-id>/<cmp-id>` off the exchange tip, writes its boilerplate `.agent/goal.md` and commits it as the dispatch snapshot (§2.8), then writes `.agent/compactions/001.md` and commits it as the terminal-summary follow-up (stub — no model call; `mark_for_deletion` is a no-op), and `--no-ff` merges back into the exchange branch. Control returns to `lernie prompt`, which rebases the exchange onto the current `main` tip and `--no-ff` merges it into `main` (§2.6). The unmerged branch count metric is read directly from `git branch --list 'ex/*' 'inv/*' --no-merged main` — no sidecar file.

### v0.3 — Tools

**Success criterion:** Agent can invoke at least two tools (`bash`, `read_file`). Worktree-modifying tool calls land as commits on the emitting branch (§2.3, §3.3); read-only tool calls produce no commit. Tool contract — binary (`lernie tool <name>` in-process or `lernie-tool-<name>` external, mirroring §4.4 adapter discovery), JSON schema, `SKILL.md`, the stdio/exit-code shape, and the per-call disk record (`input.json`, `output.json` under `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/`, outside every worktree) — pinned in §3.3. Oversized-output auto-dispatch deferred to v0.4+ (§11). Conversation-repo layout migrated from the v0.2 `.agent/`-rooted shape to the v0.3 layout described in §2.2 (control + step records at the conversation-repo root, worktrees as siblings, steps namespaced by conversation id, `merge=ours` on goal/soul/summary, "invocation" retired as a structural term). v0.3.1 follow-on tightens this with the diagnostic-only contract on `request.json` / `response.json` and relocates the step records out of every worktree (§2.3); it also drives `complete` with `stream: true` end-to-end and writes `response.json` as JSONL of §4.4 stream events tail-appended event-by-event (§4.4 "On-disk response shape: JSONL of stream events, always."), with writer-closes-fd as the §3.5 completion signal.

### v0.4 — Subagent dispatch

**Success criterion:** Agent can dispatch a subagent conversation. The subagent runs in its own worktree and branch. Merge-back flow works end-to-end (including the `merge=ours` discipline on `goal.md`/`soul.md`/`summary/**`, §2.6). Parallel subagent conversations do not corrupt each other's state. Handle-based async works: a dispatch returns immediately with a handle, siblings can be in flight concurrently, and `await(handle)` retrieves the compacted result on a later step (§2.5).

Phase 2 lands the `dispatch` built-in tool — `lernie tool dispatch`, input `{role, goal}` — that realizes the §2.5 dispatch primitive on the model-facing side: it reads conversation context from the `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` env vars (§3.3), spawns the subagent through Phase 1's `lernie dispatch <role>` CLI (§3.4), and returns `{status: in_progress, handle}` synchronously. The subagent's own step loop is a subsequent phase.

Phase 3 lands the `await` built-in tool — `lernie tool await`, input `{handle}` — the resolution half of the §2.5 dispatch/await pair. It blocks until the named subagent reaches a terminal state and emits exactly one of `{status: merged, summary}`, `{status: stopped}`, or `{status: conflicted}`. Every state read is git-or-fs (`docs/PRINCIPLES.md` "Single source of truth"): merged via `merge-base(handle, parent) == HEAD(handle)`, conflicted via `refs/lernie/conflicted/<handle>` (the merge protocol writes this on rebase failure, §2.6 step 6), stopped via either of two on-disk signatures — the latest step's `response.json` ending in a §4.4 `error` event (clean failure), or `response.json` having bytes but no terminal `message_stop`/`error` line AND no process holding the fd open (kill-mid-stream §2.9). The kill-mid-stream check reuses the `/proc/<pid>/fd/*` writer scan from §2.9 / `lernie stop` (line 267 — same observation that drives §3.5 `in_flight` classification, single source of truth). Linux-only on the kill arm; the `error` arm is portable. The v0.4 P3 ball shipped only the `error` signature; the kill arm landed in the follow-on (bl-c9ec).

### v0.5 — UI

**Success criterion:** Git tree view, live streaming, pulsing tool indicators, branch-state indicators. Watching the repo filesystem (§3.5) is the only read mechanism; user actions go out as `lernie <subcommand>` invocations.

### v0.6 — brazen provider layer

**Success criterion:** Every model call flows through `bz` (§4.4): the context assembler emits a typed brazen canonical request; `response.json` is JSONL of canonical `v=1` events in attempt segments; the harness owns retry (a forced-retry case produces two segments and correct classification); `await` and the UI classify from the `end`-terminal vocabulary; at least two providers (anthropic plus one other) work via brazen config rows with zero lernie code difference; `crates/lernie-provider-anthropic`, the `describe`/`complete` contract, and `auth_env`/`endpoint_env` forwarding are deleted; the load-time version-skew guard works. The compactor is still the v0.3 no-model-call stub (§2.7), so in v0.6 only the **worker's** model call exercises the `bz`/retry path end-to-end; the compactor's own model call is pending a later milestone and will traverse the identical §4.4 contract when it lands.

This milestone folds in the `harness` repo design exploration (2026-07). Its keeper is the adapter boundary — brazen owns the one thing the harness must never know: which provider or model is on the wire. Its other keepers land as budgets (v0.7) and sandboxed tools (v1.1). Its transcript-as-JSONL reframe was rejected: lernie's git substrate is the single source of truth and strictly richer (refs, worktrees, merges) than an append-only transcript file.

### v0.7 — Workflow config and budgets

**Success criterion:** The `workflow.yaml` surface works. At least one non-baseline workflow variant (e.g., a verifier step) runs end-to-end without code changes. Budgets (§6) enforce at the model-call boundary: a conversation tree that exhausts `max_total_tokens` stops with a `refs/lernie/budget-exhausted/<branch>` ref and `await` surfaces `{status: budget_exhausted}`; a child dispatch inherits the clamped remainder.

### v0.8 — Front-door messaging

**Success criterion:** Any sender can message any live conversation (§2.11). A message deposited by the user into a running root conversation's inbox lands as a delivery commit at the next step boundary and visibly steers the exchange; a subagent messages its still-running parent (the reminder shape) and the parent's next step sees it; a message to a conversation with no live executor is refused loudly, naming the dispatch primitives as the remedy; two simultaneous drivers on one branch resolve through the executor lock — the loser exits as a clean no-op (§2.11 "Writer/driver totality"), and killing the winner releases the lock with no cleanup step. `messages/**` rides the merge=ours discipline end-to-end: a subagent merge-back carries none of its delivered messages into the parent tree. The undelivered-message health metric (§8) reads from `inbox/` listings and branch classification only — no sidecar state anywhere in the milestone.

### v0.9 — Task suite

**Success criterion:** 50 tasks with machine-checkable success criteria. Baseline harness achieves 40% ± 5% pass@1 on the suite (Wilson CI). Per-category failure tagging works.

### v0.10 — Experiments and replay

**Success criterion:** `agent-eval --config <experiment> --suite <suite> --runs N` produces per-task pass@1 and pass@5 with confidence intervals. Any run can be tarballed and replayed. Config changes (prompt edits) deployable without code changes in under 60 seconds end-to-end.

### v1.0

**Success criterion:** All of the above, plus at least one demonstrated workflow variant that beats baseline on at least one failure category by a statistically significant margin on pass@1. This is the proof that the architecture's experimentation surface is actually useful.

### v1.1 — Sandboxed tools

**Success criterion:** A tool compiled to `wasm32-wasip2` runs under a wasmtime host with WASI clamped to the intersection of the tool's declared capability manifest and the role's grant (fs scopes, net hosts, exec, clock, env). The §3.3 stdio contract is unchanged — the sandbox is a hosting decision derived from the artifact kind (a WASM component vs a native executable), never a config field. A native binary requires the `exec` grant; the default grant set is empty; a tool asking beyond its grant fails at load, loudly, before any model call. The capability grammar must be small enough to audit at a glance — if a grant needs a comment, the grammar failed. Full spec: §3.6 (grammar, artifact-kind derivation, manifest-intersect-grant, boundaries). (From the `harness` repo's spec §6 — its strongest differentiator, preserved.)
