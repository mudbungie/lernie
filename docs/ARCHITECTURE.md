# Agent Harness Spec

**Status:** Draft v0.4
**Scope:** Design specification for a git-backed agent harness with branch-per-dispatch context management. v0.4 folds in the `harness` repo design exploration (2026-07): brazen becomes the provider layer (§4), budgets and sandboxed tools join the milestone chain (§12). The 2026-07 workspace-substrate rewrite (§2.2–§2.6, §7, §9.2) lands one repo per workspace, config branches, message-based child returns, and merge reserved for compaction.

---

## 1. Overview

This document specifies an agent harness in which agent context is managed as a git repository. A **workspace** is one git repository holding a config lineage and the agent tree forked from it (§2.1, §2.2); every agent is a branch plus a linked worktree, forked off a ref. Every dispatch creates a branch. A finished child returns its result as a message into its parent's inbox (§2.6); a root agent's branch persists, and the next user message resumes it in place (§2.4). Any sender — the user or another agent — may deposit a message into any agent's inbox; the message is delivered at the recipient's next step boundary, and a deposit into a quiescent agent starts a driver (§2.11). Within a branch, steps land as linear commits; a step that itself dispatches forks a child branch off the commit where the dispatch landed. State flows between components through the filesystem (§3.1); commands between procedures flow through the `lernie` CLI (§3.4). There is no third channel — no library API, no in-process sidechannel, no resident broker — and no process holds state across its own termination.

The architecture optimizes for four properties:

1. **Inspectability.** The complete state of any agent, at any point in its history, is a git ref. Replay, debugging, and counterfactual forking are first-class operations.
2. **Uniformity.** User-to-agent dispatch, agent-to-child dispatch, and agent self-reflection all use the same primitive: fork a branch off a ref, do work, return a result message (or answer the user, for a root agent). There is no special path for user input. There is likewise no special sender: a message into a running agent (§2.11) is the same deposit whether the user or another agent makes it.
3. **Testability.** Workflow (prompts, sequencing, context assembly rules) is configuration, not code. Experiments are config diffs, measurable against a task suite.
4. **Regenerability.** Any process can die at any time without losing state. Disk is durable; processes are disposable. Components — harness, tool subprocesses, provider adapters, frontends — restart independently because none hold state across their own termination; `lernie advance <repo>` (§6) is the operation that drives the workflow chain forward, and crash recovery is the same `lernie advance` invocation that runs the chain in normal operation. No process is load-bearing; disk is.

### 1.1 Non-goals for v1

The following are explicitly out of scope for v1:

- Multi-tenancy or multi-user isolation. Single user assumed. (Workspace isolation, §2.2, separates *concerns*, not *principals*.)
- Distributed execution. Single machine, single harness process.
- Tool sandboxing. Tools run with user privileges; no capability restriction. (A v1 non-goal only: specced as milestone v1.1, §12.)
- Remote git operations. Workspace repositories are never pushed anywhere.
- Secrets management infrastructure. Env vars injected at tool execution, referenced by name in config. No vault, no rotation.

---

## 2. Core Concepts

### 2.1 Terms

All terminology below is load-bearing and is used exclusively in the senses defined here. Terms in `docs/TAXONOMY.md` are the ambient field context; where the taxonomy flags a term as contested, this spec picks one sense and stays in it.

The terms form a containment ladder: a **workspace** (one git repository) holds a **config** lineage and the **agent** tree forked from it; an agent runs a sequence of **steps**; a step is one **model call**, which comprises one or more **attempts**. Each term is defined once here and referenced, not redefined, elsewhere.

> **Ladder transition (nearly complete).** This table is terminology-ladder v2: the structural primitive is the **agent** (not "conversation"), its container is the **workspace** (not "conversation repo"), and the versioned configuration it forks from is a **config**. The workspace-substrate rewrite has landed: §§2.2–2.6, §7, and §9.2 are rewritten in the ladder, §§2.7–2.11 and `docs/PRINCIPLES.md` amended onto it, and the pre-ladder *mechanics* — the `main`-rooted history, merge-back, the frozen-copy bootstrap — are gone everywhere, not merely re-worded. What remains is re-voicing: §3, §5, §6, §8, and the historical notes of §12 still say "conversation" / "conversation repo" / "subagent" in passing prose; read those as *agent* / *workspace* / *child agent*. Where a term's full mechanics land in a later section, this table defines the term and defers the mechanics.

| Term | Meaning | Relationship |
|---|---|---|
| **Workspace** | The isolation boundary: one git repository holding a config lineage and the agent tree forked from it. No mechanism reaches across workspaces — isolation is structural (there is no cross-workspace channel), not a permission check. | The outermost container. One workspace per repository. |
| **Config** | The contents of one configuration commit — souls, tool enablements, capability grants (§4.3), workflow, and manifest. A descriptively-named **config branch** is a config's version lineage; its head commit is the current config. "Multiple configs" means multiple config branches; there is no `main`. | Versioned by branch. An agent's substrate forks off a config branch's head by default. |
| **Agent** | One living instantiation: a goal, a growing context, a step loop, and a termination — the field-consensus sense of an LLM running tools in a loop toward a goal (`docs/TAXONOMY.md` §1). Its **substrate** is a branch plus a worktree, forked off a ref (default: the head of a config branch). | The primitive. A *root* agent is forked by a user message; a *child* agent by a dispatch tool call from a parent (§2.5). Parent/child is recorded provenance, not a category — there is no separate "subagent" kind. |
| **Exchange** | The span between a user message and the terminal response that answers it — a UX label over one stretch of a root agent's linear history. Owns no branch, no merge, no lifecycle of its own. | Non-structural. A view over an agent's steps, surfaced because users think in exchanges. |
| **Step** | One model call and the tool calls it emits. | Child of an agent; structurally bounded by the model call, not by tool completion. Lands as linear commits within the agent's branch. |
| **Model call** | One execution of a model to produce output. | Atomic. The defining event of a step; each step has exactly one. |
| **Tool call** | The model's structured request to invoke a named tool. | Emitted by a model call; structural child of its emitting step (even if it resolves temporally during a later step). |
| **API call** | One HTTP request to a provider endpoint. | 1:1 with attempts by construction (§4.4); retries make a model call span several. |
| **Attempt** | One invocation of the provider adapter for a step's model call — exactly one API call by construction (§4.4). | A model call comprises one or more attempts (§2.10); each attempt lands as one segment in the step's `response.json`. |
| **brazen** | The provider adapter project (binary `bz`, crate `brazen`): one stateless binary adapting every provider and wire protocol behind a canonical request/event pipe contract. | The only component that knows provider wire protocols (§4.4); its specs define the canonical vocabulary. |
| **Canonical request / canonical event** | brazen's typed request shape and its `v=1` streaming event vocabulary (brazen `architecture.md` §3). | The adapter wire contract; `response.json` is JSONL of canonical events (§2.3, §4.4). |
| **Dispatch** | The event that spawns a child agent with a goal. Two forms: a user message (spawns a root agent) and a tool call targeting a child (spawns a child agent off the commit where the dispatch landed on the parent branch). | Creates a branch. |
| **Branch** | The git container for an agent. | Every agent has exactly one branch; every dispatch creates one. |
| **Goal** | The stated objective handed to an agent at dispatch. | One per agent; not rewritten during execution; pinned at the head of context for every model call on the branch (§2.8). |
| **Soul** | The system prompt handed to an agent at dispatch, drawn from the governing config commit's `souls/<role>.md` (§2.2) and overwritten on the branch's dispatch commit. | One per agent; composed into every model call on the branch as the system message. |
| **Message** | Content addressed to an *existing* agent by a sender — the user or any agent. Deposited into the recipient's inbox; delivered at a step boundary as a committed worktree file (§2.11). | The steering primitive. Sender is recorded provenance; the recipient treats every sender uniformly. Unqualified "message" means this primitive; wire-level messages are always written qualified (user message, assistant message). |
| **Inbox** | The per-agent queue directory holding deposited-but-undelivered messages; outside every worktree, namespaced by agent id (§2.11). | Deposit target for messages (§2.11). |
| **Executor** | The process currently driving a branch's step loop — `lernie prompt`, `lernie dispatch`, or a `lernie advance` re-entry (§6); the same process the §2.9 writer scan discovers. | At most one per branch, guaranteed by the executor lock (§2.11). Distinct from the *tool executor* (§3.2), a role inside it. |
| **Transcript** | The branch-scoped sequence of committed worktree files under `messages/` recording, in order, everything that entered the agent's context: delivered messages (§2.11), each step's assistant output, each tool call's result (§2.3). | The agent's model-facing history. Entries are immutable — change is append or delete, never edit-in-place. Never process memory, never a step record. |
| **Work product** | An in-worktree, non-transcript file — the artifacts an agent edits or writes (code, docs, data) as distinct from its transcript. | Resolves the workspace/worktree-contents collision: the *workspace* is the repo-level boundary; what fills a worktree and is not transcript is a work product. |

**Retired structural terms** (v0.1–v0.3 vocabulary, superseded by the ladder above; may linger in passing prose per the transition note):

- **"conversation" / "conversation repo"** — the former primitive and its repo. The living instantiation is an **agent**; its container is a **workspace**. Where a later section still says "conversation" / "conversation repo," read *agent* / *workspace*.
- **"invocation"** — retired since v0.3 (general-field "model invocation" ≈ model call is acceptable informally, never as a structural unit).
- **"subagent"** — deleted as a *category*. A child agent is an agent; parent/child is provenance (§2.5), not a kind. Never a structural noun of its own.

**Banned usage:**

- **"call"** without a qualifier — use model call, tool call, or API call.
- **"turn"** — incompatible vendor meanings (`docs/TAXONOMY.md` §1).
- **"session"** — underdefined, per-framework overloaded, and colliding with the transport/connection sense (`docs/TAXONOMY.md` §3). Stays banned.
- **"compression"** — reserved for model-weights quantization; the harness operation is compaction.
- **"exchange"** as a *structural* term — it is the UX span defined in the table (no branch, no merge, no lifecycle); structural claims belong to the underlying agent.
- **"thread"** as an agent-instance term — considered and rejected: the running instance is an **agent** (agent-as-instance already names it), and "thread" is spoken for by §3.1 ("Threads, not processes," the OS-thread sense) and the field's server-side message-container sense (`docs/TAXONOMY.md` §3). Its only sanctioned use here is the §3.1 OS-concurrency sense.
- **"agent"** in the *config/profile* sense — "agent" is pinned to the running-instance sense above; the versioned configuration it forks from is a **config**, never an "agent" (contrast the OpenAI-SDK "agent = config object" reading, `docs/TAXONOMY.md` §1).

**Other terms of art not defined in this document or in `docs/TAXONOMY.md` require explicit definition or user approval before use.**

### 2.2 The Workspace

A **workspace** (§2.1) is one git repository: the isolation boundary, holding a config lineage and the agent tree forked from it. It lives at `<data-root>/workspaces/<workspace>/`.

**One repo per workspace; worktrees are the cloning mechanism.** Every agent in a workspace is a branch plus a linked worktree of the *same* repository (§2.3). The alternative — a repository per agent — was considered and rejected: it buys per-agent blast radius at the price of the system's point. Interoperability is the point: inter-agent messaging (§2.11) addresses any agent in the tree, and starting an agent off *any* ref — another agent's historical commit, a parallel fan of N forks off one commit (§2.3) — requires the refs to share one object store. A git worktree already provides all the isolation an agent needs (its own checkout, its own branch); a second repository would add isolation nobody asked for and destroy the forkability everything else relies on.

**Isolation lives at the workspace boundary.** Separate workspaces are separate repositories, and no mechanism reaches across: no cross-workspace messaging, no cross-workspace fork, no shared ref. The no-cross-talk guarantee is structural — there is no channel to misuse — so two workspaces that must not mix (work for two different employers, say) cannot, by construction rather than by permission check. (§1.1's single-user assumption is unchanged: workspaces separate *concerns*, not *principals*.)

**Harness root, split by XDG lifetime.** Installation-global state outside any workspace splits along XDG lifetimes into two homes:

- **config root** — `$XDG_CONFIG_HOME/lernie` (default `~/.config/lernie`). Hand-edited declarations: the global `models.yaml` (§4.2) and the `workflows/` templates from which config commits are authored (below, §6).
- **data root** — `$XDG_DATA_HOME/lernie` (default `~/.local/share/lernie`). Machine-populated state: the `workspaces/` tree and the `skills/` and `tools/` pools (§3.3).

> **Two senses of "config."** Bare **config** is always the ladder primitive (§2.1): the contents of one configuration commit inside a workspace. **Config root** — always written with the "root" suffix — is the XDG directory above: install-global, outside every workspace. The suffix is the disambiguator; this document never uses bare "config" for the directory, and never "config root" for the primitive.

`LERNIE_HOME`, when set and non-empty, is the single override that **collapses both roots to that one directory** — one env var keeps parallel testing, alternate installs, and sandboxed replay isolated. This aligns lernie's resolution with brazen's own XDG-style config resolution (§4.4). Where a statement below does not depend on the split, `<harness-root>` names whichever root holds the artifact in question.

Directory layout:

```
<data-root>/workspaces/<workspace>/
├── repo.git/                     # the workspace repository (bare): config/* and agents/* refs
├── steps/<agent-id>/NNN/         # diagnostic step records; outside every worktree (§2.3)
│   ├── meta.json                 # {commit, started_at, …} — branch tip at step-start
│   ├── request.json              # diagnostic; replay rebuilds the wire input from `commit`
│   ├── response.json             # JSONL of canonical events, attempt segments (§4.4)
│   └── tools/<tool-id>/          # input.json, output.json — diagnostic (§2.3, §3.3)
├── inbox/<agent-id>/             # deposited, undelivered messages; outside every worktree (§2.11)
└── agents/<agent-id>/            # one linked worktree per materialized agent (§2.3)
    ├── goal.md                   # branch-scoped — this agent's goal
    ├── soul.md                   # branch-scoped — this agent's system prompt
    ├── summary/NNN.md            # branch-scoped — this agent's compactions
    ├── messages/NNN-<origin>.*   # branch-scoped — the transcript (§2.3): delivered messages
    │                             #   (NNN-<sender>.md, §2.11), assistant output
    │                             #   (NNN-assistant.json), tool results (NNN-tool.json)
    ├── descriptions/             # tool + skill descriptions (inherited from the config commit)
    ├── skills/                   # loaded skill content (branch-scoped)
    └── …                         # work products (§2.1)
```

**Config branches.** A config branch (`config/<name>`, §2.3) is the version lineage of one config: souls (`souls/<role>.md`), workflow (`workflow.yaml`), context-assembly manifest (`manifest.yaml`), role → model mapping (`providers.yaml`), schema version (`version`), and the `descriptions/**` snapshot (§3.3). Config branches are descriptively named — `config/default`, `config/strict-verifier` — and there is no `main`: no branch in the workspace is a trunk, and none is privileged beyond what its name says. "Multiple configs" means multiple config branches. A workspace's first config commit is an orphan root; further config branches fork from existing ones or start fresh. Authoring a config commit is a user act (the branch advancement invariant, §2.3), harness-assisted: `lernie` materializes a checkout, writes the files — from the `<config-root>/workflows/` templates and the data-root pools, including the `descriptions/**` snapshot (§3.3) — and commits.

**Fork is the freeze.** An agent forks off a config commit (default: its chosen config branch's head, §2.3), so the config governing it is immutable by construction — a commit, permanently reachable in the agent branch's ancestry. Later user edits advance the config branch and affect only agents forked after them; reproducibility and portability win over live update, as before, but the freeze is now free. This deletes the v0.3 frozen-copy bootstrap (`cp -r` from a data-root `agents/<profile>/` pool at repo creation): the copy existed to freeze, the fork *is* the freeze, and the profile pool dissolves into config branches — versioned, diffable, and forkable where the pool was none of those. (Endpoints and auth remain brazen's alone (§4.4); the config's `providers.yaml` carries only role → (provider row name, model id) pointers, and the global `models.yaml` under the config root holds model capabilities and context windows (§4.2).)

**Control is read from the config commit; worktrees hold only context.** The harness reads an agent's control files — `workflow.yaml`, `manifest.yaml`, `providers.yaml`, `souls/`, `version` — from its **governing config commit**: the nearest ancestor of the agent's branch reachable from any `config/*` ref. Derived from ancestry, never stored (`docs/PRINCIPLES.md` Single source of truth); an agent started by fork-back-in (§2.3) inherits its source's config the same way. Reading control from the immutable commit rather than from the live worktree means an agent cannot edit its own policy — tool side effects land in the worktree, which control resolution never consults — and replay resolves the identical policy at any later date. The dispatch commit (§2.3 step 2) removes these harness-facing files from the agent's tree once `soul.md` is specialized, so the worktree invariant (§5.1) stays total: everything in a worktree is context, with no exclusion list. `descriptions/**` stays in the tree because it *is* context (§3.3); `steps/` and `inbox/` sit at the workspace root, outside every worktree, namespaced by agent id — physically outside context assembly (§2.3, §2.11).

**Step records are not context.** A **step record** is the per-step on-disk directory at `<workspace>/steps/<agent-id>/<NNN>/`: `meta.json`, `request.json`, `response.json`, and any `tools/<tool-id>/` subdirectories the step emitted (full layout in §2.3). The whole `steps/` tree at the workspace root holds these diagnostic / audit artifacts. It sits outside every worktree by construction, which means context assembly (§3.5, §5) is *physically incapable* of including it. The structural placement enforces the rule the worktree-as-context invariant (§5.1) implies: an agent's context — running, retried, or replayed — is assembled from the branch's read-state commit and nothing else. The model-facing history itself is committed worktree data, the **transcript** (§2.1, §2.3); no runtime code path reads step-record *content* at all. `response.json`'s event *framing* — the terminal `end`, `Finish`/`Error` kind, `Usage` — is read for classification and metering (§2.3 Diagnostic-only contract); everything else in this tree is write-only diagnostic.

**Sibling worktrees, not nested.** Agent worktrees are named by their full hyphenated descent (`<a>-<b>-<c>-…`, §2.3) and live as *siblings* under `agents/`, never as subdirectories of their parent's worktree. Git does not permit nested working trees; the flat layout is how the primitive's uniformity survives contact with git mechanics.

**Branch-scoped vs inherited.** `goal.md`, `soul.md`, `summary/`, and `messages/` (the transcript, §2.3) are written per-branch, and they never enter any other branch's tree — structurally: no merge-back exists (§2.6), and the work-product transfer filter excludes them (§2.6). `descriptions/` is committed in the config (§3.3) and inherited by every branch via git. `skills/` is branch-scoped (added as skills are loaded; prunable by the compactor). `steps/` and `inbox/` are *not* branch-scoped — they live at the workspace root, shared across the whole tree and namespaced by agent id, so nothing about them ever needs to cross between branches (§2.3).

**Two writers, disjoint by namespace.** The user advances config branches; each agent's executor (§2.11) advances that agent's branch — the branch advancement invariant (§2.3). The user's interactive surface is the UI, which reads the filesystem and issues `lernie` verbs (§3.5); agent worktrees are read-only to the user under normal operation. Workspace repositories are never pushed to a remote (§1.1).

### 2.3 Branches and the branch invariants

A **branch** is the git container for a single agent (§2.1). Two invariants govern every ref in the workspace:

> **Every dispatch creates a branch.** Starting an agent *is* creating a branch plus a linked worktree off a ref — there is no other way an agent comes to exist, whether the dispatch is a user message or a child-targeting tool call (§2.5).

> **Branch advancement.** A config branch advances only by user config edits; an agent branch advances only by its executor (§2.11). No branch has a second writer, and nothing merges into either kind except the compaction merge (§2.6), which the receiving agent's own executor lands.

The second invariant replaces v0.3's "nothing writes to `main` directly": there is no `main` (§2.2), so trunk protection generalizes into a per-lineage single-writer rule that covers every branch equally.

**Ref namespace.** Config branches live at `config/<name>` (descriptive names, §2.2); agent branches at `agents/<agent-id>`, where the **agent id** is the full hyphenated descent from its root agent (`<a>`, `<a>-<b>`, `<a>-<b>-<c>`, …) — hierarchy encoded in the name, not the filesystem. The prefix is the kind: which advancement rule a ref lives under is derived from its path (`git branch --list 'config/*'` / `'agents/*'`), never recorded anywhere else. Root ids are unique per workspace; an agent's id doubles as its address (§2.11), its worktree directory name (§2.2), and its `steps/` / `inbox/` namespace.

**Any ref is a legal fork point.** An agent's substrate is a branch plus a worktree **off a ref** (§2.1). Every start is the same operation with a different argument:

- *Fresh start* — fork off a config branch's head (the default). The agent begins with the config's tree and an empty transcript.
- *Child dispatch* — fork off the commit where the dispatch landed on the parent's branch (§2.5). The child inherits the parent's transcript and work products in its tree (*Fork and inheritance*, below).
- *Fork-back-in* — fork off any historical commit of any agent: counterfactual exploration, continuing from a stopped tip (§2.9), re-running with a different goal. Provenance is the ancestry; no special branch prefix marks a fork (§7.2).
- *Parallel fan* — N forks off one ref are N sibling agents (§2.5).

An agent's governing config rides its ancestry (§2.2). Re-configuring an existing lineage is out of scope: policy changes are config commits plus fresh forks, not grafts onto running agents.

Within a branch, **steps land as linear commits**: when a step's model call completes, the executor commits its assistant output into the transcript (below); each tool call the step emitted commits its result — and any worktree side effects — as it lands (§3.3). Steps are not their own branches. New branches appear only at dispatch boundaries.

> **Historical.** v0.1 committed exchanges directly on `main` without branching; v0.2 introduced `ex/*`/`inv/*` branch prefixes; v0.3 unified both under one primitive, with root history merging to `main` and children merging back to their parents. The workspace substrate (this rewrite) deletes `main` and merge-back together: root agents live on their own branches indefinitely (§2.4), children return results as messages (§2.6), and the only merge left in the system is compaction (§2.6).

**Branch lifecycle** (identical for root and child agents, up to step 5's addressee):

1. **Fork.** The dispatch creates `agents/<agent-id>` off the chosen ref and materializes a linked worktree at `<workspace>/agents/<agent-id>/`.
2. **Dispatch commit.** The harness writes `goal.md` and `soul.md` (the chosen role's soul, read from the governing config commit's `souls/<role>.md`, §2.2) and — when the fork point is a config commit — removes the harness-facing control files from the tree (§2.2). This is the first commit on the new branch. Goal and soul are overwritten in place here and frozen thereafter (§2.3 *Goal and soul are pinned files*).
3. **Work.** The agent runs its step loop. At each step boundary the executor drains the agent's inbox — pending messages land as delivery commits before the model call (§2.11). Assistant output and tool results commit as transcript entries as they land; worktree-modifying tool calls carry their modifications in the same serialized commit discipline (§3.3). Each step's diagnostic record lands at `<workspace>/steps/<agent-id>/NNN/`, outside the worktree. A step that emits a dispatch tool call forks a child branch off the branch tip.
4. **Completion.** A terminal event: final response, stop (§2.9), budget exhaustion (§6).
5. **Return.** Every terminal event deposits — this step is total. A child deposits its **result message** into its parent's inbox (§2.6), carrying its epitaph (the pinned manner of ending), its terminal ref, and — iff it spoke — its terminal response, from which delivery applies the work-product transfer. Final response, stop (§2.9), and budget exhaustion (§6) each deposit through the child's own executor with the matching epitaph; a child that crashed too hard to run its handler (SIGKILL, OOM, panic) has a `died`-epitaph deposit made on its behalf by the §8 sweep. A root agent has no parent inbox: its terminal response answers the user (§2.4, §7.1), and its non-response terminations surface in the exchange (§2.4). Nothing merges anywhere; the branch simply stops advancing.
6. **Quiescence, not teardown.** The branch persists as a ref. A quiescent agent's worktree may be torn down and is rematerialized by the next driver (`git worktree add` off the ref) — the worktree is disposable materialization, never state; the inbox directory and the lock it carries (§2.11) persist at the workspace root regardless. A later deposit revives the agent: the deposit starts a driver, and the step loop resumes on the same branch (§2.4, §2.11). Retention and GC are branch deletion plus object pruning (§9.2).

Branching is cheap — local git operations on disk — but it is not per-step. A long-running agent produces many commits on one branch and only forks children when it actually dispatches.

**Executor exclusivity is two existing rules.** Git refuses to check one branch out into two worktrees, so a branch has at most one materialization; the executor lock (§2.11) admits at most one driver per agent, so a materialized worktree has at most one process stepping it. The residual case the git rule cannot see — two processes re-entering one *existing* worktree — is exactly the lock's job. Jointly the two rules are total, and nothing new is needed.

Compaction may run *during* a branch's execution, not only near termination; see §2.6 (the compaction merge) and §2.7.

**The transcript: context has one home.** Everything that enters an agent's context is a committed worktree file. The **transcript** (§2.1) is the branch-scoped sequence under `messages/` that carries the model-facing history itself: each delivered message (§2.11), each step's assistant output, and each tool call's result is one immutable **transcript entry**, committed by the executor as it lands. Nothing context-bearing sits in process memory: the executor holds no message history across steps, and a model call's context — running, retried, or replayed — is assembled the same way, from the read-state commit's tree (§2.10, §5). This closes the gap the diagnostic-only contract (below) used to leave open: assistant text and thinking blocks previously had no readable home outside the diagnostic `response.json`, so a crashed chain could not rebuild its history without either violating the contract or having held that history in RAM — and both arms fail `docs/PRINCIPLES.md` Disk-first. Now the history has exactly one home, and it is a commit.

> **Shipped-state note.** The **transcript writer** shipped with bl-4798: a settled model call now seals its staging entry and commits `messages/NNN-assistant.json`, and each resolved tool commits `messages/NNN-tool.json` — each file a JSON array of brazen's canonical `Content` blocks, composing verbatim as one wire message. It runs **alongside** the pre-transcript path this amendment retires: the step loop still folds assistant events into an in-memory accumulator and still assembles the next request's `tool_result` blocks from that accumulator's `Content` (the `tools/<tool-id>/output.json` diagnostic capture remains). Both are written; the old one is still *read*. Re-pointing the assembler at the committed transcript — and deleting the accumulator and the `output.json` read path — is bl-26cb (blocked on this ball). The writer's mechanics are specified below (*The transcript writer*).

*Sequence and immutability.* Transcript entries are named `messages/NNN-<origin>.<ext>`; **order lives in the filename**. `NNN` is one zero-padded per-branch counter shared across every origin, assigned at commit time by the executor — the branch's single writer (§2.11 executor lock) — and derived, never stored: the next number is max-present-plus-one from a directory listing. The shared counter is race-free by the same token: every transcript commit, whatever its origin — a delivery commit (§2.11), a step's assistant entry, a tool entry (§3.3, sibling commits serialized) — lands inside the executor lock's critical section, and max-present-plus-one is evaluated there; no writer ever assigns a number outside the lock, so the counter needs no mechanism of its own. Assembly is readdir + sort — no git-log walk, no index, no sidecar order file (`docs/PRINCIPLES.md` Single source of truth: order has one home, the name). Entries are immutable: change is append or delete, never edit-in-place. Deletion is the ordinary curation primitive (§5.4), and a deleted entry stays recoverable from git history until compaction squashes it.

*Origins and wire framing.* The origin token in the name tells the assembler what wire shape an entry takes — role framing is derived from the path, not from frontmatter and not by flattening history into one assembled user block:

- `messages/NNN-<sender>.md` — a delivered message (§2.11); composes as user-role content, path as provenance hint (§5.3).
- `messages/NNN-assistant.json` — one step's assistant output: the canonical content blocks (text, thinking, `tool_use`) exactly as the model call's authoritative content emitted them (§4.4 segment authority), streamed by the executor from the adapter pass it is already consuming — one pass, two sinks, no read-back and no in-RAM history (*The transcript writer*, below) — and committed when the model call settles complete; a retried or paused model call commits only the authoritative content (§2.10, §4.4). Composes verbatim as an assistant-role wire message.
- `messages/NNN-tool.json` — one tool call's canonical `tool_result` block (content, `is_error`, and the `tool_use_id` pairing it to its emitting step), committed by the tool executor when the tool resolves (§3.3). Composes as `tool_result` blocks in the following user-role wire message.

The assembler walks the sequence and groups consecutive same-side entries into alternating wire messages; because tool results commit immediately after their emitting step's assistant entry — the boundary drain (§2.11) is ordered *after* the step's tool entries, so a delivered message can never wedge between a `tool_use` and its `tool_result` — the provider pairing rule — every `tool_use` matched by a `tool_result` in the immediately following user message (§2.5) — holds by construction. Path-derived framing is forced, not chosen: the wire itself requires role structure (a single flattened user block cannot carry `tool_use`/`tool_result` pairing, and replayed thinking blocks must re-enter under their original role with their original signatures for the request to be bit-identical), and the role's single home is the filename — frontmatter would be a second copy of a fact the path already states. The `.json` entries carry brazen's canonical vocabulary verbatim (§4.4), which is what makes replay bit-for-bit rather than a lossy re-rendering. (`assistant` and `tool` are reserved origin tokens; senders are agent ids or `user`, §2.11.)

*Self-output commits directly; the inbox is the on-ramp for everyone else.* The inbox (§2.11) exists because external senders cannot commit into a branch they do not drive — the queue plus the step-boundary drain is what serializes many writers onto the single executor. The executor needs no such ramp for its own step output: it already holds the lock and already owns the counter, so it delivery-commits assistant output and tool results straight into the same sequence namespace. Routing self-output through its own inbox would add a hop that its own next drain immediately undoes — mechanism without a difference. A *note to self* is a different thing and stays legal: that is a message (content addressed to an agent, §2.1), deposited through the front door like any other (§2.11 "Self-messages"); step output is not addressed to anyone — it is the history itself.

*The transcript writer.* Both sinks are streams off the one adapter pass; nothing is buffered in RAM beyond the block in progress, and nothing is ever read back. Every event appends to the diagnostic `response.json` as it arrives (§4.4, fd held open across attempts). Content additionally streams into the transcript entry *under construction*: a **staging file** at `<workspace>/steps/<agent-id>/<NNN>/assistant.staging.json`, appended block-by-block as each content block completes — a block is the smallest unit that exists before it is whole, so the in-progress block's deltas are the only buffering anywhere. Segment authority (§4.4) drives the staging file mechanically: an `Error`-terminated segment truncates it (the discarded attempt's audit home is `response.json`); a `Pause`-terminated segment leaves it accumulating — the pause continuation (§2.10) re-sends the staging file's blocks, the writer reading its own sink, not a diagnostic record; the final `Finish` seals it, and the executor renames it into the worktree as `messages/NNN-assistant.json` (`NNN` assigned at commit time, above) and commits. A model call that never settles complete — retries exhausted, a non-retryable error, a stop — commits nothing: the transcript gains no entry, the branch tip does not move (§2.10), and the stale staging file is debris the step's re-run overwrites (*Crash and recovery*, below). The staging file is the one path under `steps/` that is not a record — it is the entry being authored, homed outside the worktree until it is authoritative, and it *leaves* by rename at settle; the diagnostic-only contract (below) is untouched, because no content is ever read back out of `response.json` or any other step record.

*Goal and soul are pinned files, not sequence item zero.* The pinned head — `goal.md`, `soul.md`, `descriptions/**`, the manifest's `pinned` paths (§5.2) — stays outside the sequence. The two regimes are opposites on every axis: the pinned head is overwritten in place at the dispatch commit (step 2 above) and frozen thereafter — exactly the edit-in-place the sequence bans; the soul composes into the system slot, which is not a wire message at all; and the goal's contract is position-independence (§2.8, pinned at the head of context regardless of sequence position) — the antithesis of a sequence coordinate. Making them "item zero" would trade a clean two-part rule (frozen head + append-only tail, §5.5) for a sequence with one mutable, role-anomalous member.

*Fork and inheritance.* A child branch forks with its parent's transcript in its tree, and the worktree invariant (§5.1) composes it — this is what gives a compactor (§2.7) its view of the dispatching branch's history with no special channel, and what gives any child its inherited context. The child's executor appends from the inherited maximum. A dispatching branch that wants a lean child compacts first, or the child curates by `rm` — the general primitives, with no dispatch-time special case.

*Return, not merge-back.* Nothing a child appends ever enters its parent's tree, because nothing merges back (§2.6). The child's transcript, summaries, goal, and soul stay on its branch — provenance until GC (§9.2) — and its distilled result travels as the result message plus the work-product transfer (§2.6). Parent and child transcripts are separate lineages sharing a prefix; they cannot collide because they never meet.

*Crash and recovery.* An entry that never committed never happened: a chain that dies mid-model-call re-enters (§6), finds the read state unchanged, and re-runs the step — the identical shape as a retry (§2.10), with nothing in memory to lose. Running assembly and replay assembly are one code path against one input; "replay" stops being a mode.

**Step on-disk layout.** Each step lives in its own directory at `<workspace>/steps/<agent-id>/<NNN>/` — at the workspace root, *outside every worktree* (§2.2). `<NNN>` is zero-padded 3-digit and 1-indexed, so step dirs sort lexically. `<agent-id>` is the owning agent's id — namespacing this way is what lets every agent in the tree write into a single shared `steps/` tree without filename collision, and why a child's step records never need to cross anywhere: they were never below a worktree to begin with.

Per-step files:

- `meta.json` — `{commit, started_at, ended_at, …}`. The `commit` field is the sha of the branch tip at step-start; it is the **read state** for the step's model call. Replay reproduces the wire input by re-running the context assembler (§5) against this commit's tree — `request.json` is not the source of truth. Step 1's `commit` is the dispatch commit (§2.3 step 2). Step ≥2's `commit` is the prior step's tip, advanced by the commits between them — the prior step's transcript entries (assistant output, tool results), any worktree side effects, and any delivery commits from the boundary drain (§2.11); the harness writes no separate pre-call commit for step ≥2 (§2.10).
- `request.json` — diagnostic snapshot of the wire request the model saw. Written for audit and human inspection only (see Diagnostic-only contract below).
- `response.json` — JSONL of canonical events (one event per line), appended across one or more attempt segments, each segment terminated by its `{"type":"end"}` line; the wire-side streaming/non-streaming distinction collapses inside the adapter before bytes reach disk. See §4.4 for the segment rules and §3.5 for the live-streaming completion signal.
- `tools/<tool-id>/` — per-tool-call records (`input.json`, `output.json`); `<tool-id>` is the `tool_use.id` from the wire (e.g. `toolu_01abc…`). Full contract in §3.3.

**Diagnostic-only contract (request.json, response.json, tools/).** Everything under `steps/` is diagnostic; there are **zero runtime content reads** from this tree. The rule is framing-yes / content-no, with no exemptions:

- **request.json: never read at runtime.** Replay (§3.1, §2.10) re-runs the context assembler against `meta.json`'s `commit` tree and re-invokes the adapter; it does not read `request.json`. The file exists for human inspection only.
- **response.json content: never read, by anyone, for anything.** The assistant text, thinking, and tool-call arguments a step produced have their readable home in the transcript (`messages/NNN-assistant.json`, above): the executor writes the adapter stream to both sinks in one pass and never reads it back. Context during execution and context at replay are the same assembly from the read-state commit; no code site reconstructs anything by re-reading a prior step's `response.json`.
- **response.json framing: sanctioned reads.** Classification (§3.5), the harness retry loop (§2.10), and metering (§6, §8) read only the framing tail — the terminal `end`, the last segment's `Finish`/`Error` kind, and `Usage` events — never the content blocks. Workflow advance (§1 #4, §6, "No resident interpreter") stays scoped to workflow-event boundaries: it never picks up mid-history by reading response content.
- **tools/<tool-id>/ (input.json, output.json): never read at runtime.** A tool result's runtime home is its transcript entry (`messages/NNN-tool.json`, committed when the tool resolves, §3.3); the emitting `tool_use` block's home is the step's assistant entry. The per-tool-call records remain as the raw capture — full stdout/stderr/exit code with timestamps — for audit and human inspection. (Historical: through v0.7 `output.json` was the runtime source for `tool_result` assembly and `input.json` the replay source for tool framing; the transcript supersedes both reads.)
- **The frontend (§3.5)** may read all of these files in full for user inspection; that is a read-only consumer, not harness state.

This is structural, not advisory. The placement of these files at `<workspace>/steps/`, outside every worktree, makes context assembly (§3.5, §5) physically incapable of including them as model context; the worktree-as-context invariant (§5.1) is untouched, because a framing read (a terminal line, an error kind, a token count) never flows into a prompt. Harness implementations honor the split directly: no code site reads step-record content at runtime — only `response.json`'s framing, and only for classification and metering.

Step records are not committed to git. The workspace repository is bare (§2.2), and `steps/` sits beside it — outside every worktree, untracked by git. Their durability is filesystem durability (atomic write via temp + rename, fsync as needed); their authority for replay is the `commit` sha each `meta.json` records, which *is* a real git commit.

**Silent death is derived.** Git's ref database plus the executor lock is the tracking. An agent branch with no live executor (the §2.11 lock probe) whose latest step's `response.json` closed without a terminal `end` died mid-work (§2.9); a child with no live executor and no result deposit (§2.6) stalled without returning. Both classifications derive from refs, the lock probe, inbox listings, and step-record framing — no status field anywhere (`docs/PRINCIPLES.md` Single source of truth). The §8 silent-death health metric reads exactly these observations.

### 2.4 Exchanges and reprompt

An **exchange** (§2.1) is a UX span, not a structure: the stretch of a root agent's linear history between a user message and the terminal response that answers it. It owns no branch, no merge, no lifecycle. Users think in exchanges, so the UI surfaces the grouping (§7.1); the architecture does not.

**Reprompt is a message.** A root agent's branch outlives its exchanges. When the user speaks again, the deposit lands in the agent's inbox, a driver starts if none is live (§2.11), the delivery commit lands, and the step loop resumes on the same branch — the same primitive as any other message from any other sender, at any other time. There is no branch-per-exchange, no re-dispatch, no state handoff: the context is already the branch's tree. This dissolves a self-contradiction in the v0.3 shape, where each user message spawned a fresh branch off `main` that "consumed the prior branch's state as context" while root branches never merged — so `main` never actually held the history the next branch was supposed to consume. Neither arm survives: there is no `main` and no branch-per-exchange to root on it.

An exchange interrupted before a terminal response is just a stopped agent (§2.9): the branch stays, and the user moves forward with the same two primitives as ever — a message to resume the same agent, or a fork off any ref for a fresh one (§2.3).

Multiple concurrent exchanges are multiple root agents (§7.3): a *new question* forks a new root agent off a config branch's head; a *course correction* messages the running one. No mechanism beyond the branch invariants and the inbox is required.

### 2.5 Dispatch

A **dispatch** (§2.1) is the event that starts an agent with a goal. Two forms:

- **User-message dispatch.** The user starts a root agent — default fork point: a config branch's head (§2.3).
- **Tool-call dispatch.** A running agent emits a tool call targeting a child. The child forks off the commit where the dispatch landed on the parent's branch, inheriting the parent's context (§2.3 *Fork and inheritance*). From the child's perspective, its parent is indistinguishable from a user; the two forms are one primitive.

This symmetry is load-bearing:

- The same code path handles user-initiated and agent-initiated dispatches.
- Verifier agents, compactor agents, and adversarial critics are child agents with different goals.
- Parallel exploration (N workers on the same task) is N parallel dispatches from the same step — N sibling forks of one ref (§2.3).

The unification is not ergonomic sugar; it falls out of the substrate. Because starting work is forking a branch and results return through the one message channel (§2.6), any operation shaped as "spawn work with a goal, run, return a result" collapses to the same primitive. The harness does not ship a separate framework for compaction, verification, or auto-parsing of oversized tool output — each is a child agent with a different goal and toolset, dispatched the same way (§3.4's `lernie dispatch …`). A new procedure earns its place by collapsing onto this primitive or by introducing one that is genuinely new; it does not sit in parallel with something it almost is. This is the concrete instantiation of the "One obvious path" principle in `docs/PRINCIPLES.md`.

**Every tool returns synchronously.** Provider APIs (Anthropic Messages, OpenAI Responses, Gemini) require each `tool_use` block emitted by a step to be matched by a `tool_result` block in the immediately following user message. The harness honors that invariant: a step's next model call is issued only when every tool call it emitted has produced a `tool_result`. Partial-result reprompts are not attempted — they are rejected at the wire.

**Dispatch returns the child's address.** Long-running tools — including dispatch — return immediately. A dispatch's `tool_result` is the **child's address** (its agent id, §2.11), never its result and never a handle to poll: there is nothing to await. The result arrives later, on its own channel, as a **deposit into this agent's inbox** (§2.6, §2.11) — revival-on-deposit (§2.11) wakes the parent whether it is still stepping or has gone quiescent. Parallelism is expressed by issuing several dispatches in one step; each child deposits its result message as it terminates, and the parent reacts to each at a step boundary as it lands. The per-step "one result per tool_use" shape stays intact; the asynchrony rides on the inbox, not on a handle. Dispatch is a tool like any other: this is how the symmetry between inline tool calls and child agents is preserved without inventing a second control path. The v0.4 `dispatch` built-in — `lernie tool dispatch`, input `{role, goal}` — realizes this contract: it spawns the child through `lernie dispatch <role>` (§3.4) and emits `<this-agent>-<child>` — the child's agent id, which is also its branch name and its address (§2.3, §2.11).

Tool calls targeting non-dispatch tools commit their records on the emitting branch as part of the step. Dispatch tool calls fork child branches off the commit where the dispatch landed; the `tool_result` the parent step observes is the child's address. When the child terminates, its result message delivers at a step boundary like any deposit (§2.6, §2.11), or revives the parent if it had gone quiescent (§2.4, §2.11) — the parent never blocks, so there is no separate poll or wait to perform. (The merge-era `{status: conflicted}` is retired with the merge protocol; its survivor is the declined work-product transfer, marked git-natively at delivery, §2.6.)

A child must reach a terminal state before its *parent* terminates — parent termination cascades a stop to still-running children (§2.9) — but it need not resolve before the parent's next step. A long-lived child that outlives many parent steps, messaging its parent as it goes (§2.11) — a reminder agent, a watchdog, an adversarial critic running alongside — is a legitimate shape, not a stall; a still-running child is live (its executor holds the branch's lock, §2.11), and the §8 silent-death metric counts agents with *no live executor*, never branches still working.

The parent sees those upward messages at its step boundaries (§2.11): if it is still stepping, the deposit drains at the next boundary; if it has gone quiescent after dispatching, the deposit *revives* it (§2.4, §2.11). Because the parent never blocks on a child — it dispatches, terminates its step, and is parked exactly as a root agent is parked awaiting the user — every deposit is seen the moment the parent next steps, whether that step is its own continuation or one the deposit itself starts. A reminder-shaped child needs no polling apparatus to be seen mid-flight: its message either lands at the parent's next boundary or wakes a quiescent parent. Delivery is deferred to a step boundary, never dropped.

**Write paths are assigned.** The harness assigns write paths per tool call and per child agent, so two sibling branches never target the same file. This is a structural guarantee (enforced by the tool executor and the dispatch primitive), not a convention — and it is what makes the work-product transfer (§2.6) conflict-free by construction: sibling children's diffs are disjoint, and a diff that fails to apply indicates a harness defect, not an expected state.

### 2.6 Results and the compaction merge

Merge-back is eliminated. Through v0.3, a finished child was rebased onto the parent's tip, aligned by a `merge=ours` discipline over its branch-scoped context files (`goal.md`, `soul.md`, `summary/**`, `messages/**` — `.gitattributes` drivers, a pre-merge alignment commit, and a `refs/lernie/conflicted/*` escape hatch as enforcement), and merged `--no-ff` into the parent. That machinery deletes wholesale, for cause: it was complicated — rebase plus alignment plus driver registration, all to move one summary; it delivered nothing useful — everything the parent should see already traveled through the result-message channel (§2.6), and everything else was precisely what the discipline existed to strip back out; and it broke caching badly — a merge rewrote the parent's tree mid-history, flushing the provider's cached prefix that the append-only transcript exists to keep warm (§5.5). What replaces it is the channel that already existed:

**A child returns by message.** At completion (§2.3 step 5), the child deposits a **result message** into its parent's inbox — an ordinary §2.11 deposit: sender-namespaced, create-only, lock-free. **Every result message carries three fields:**

1. **terminal ref** — always. The sha of the child's branch tip at return, from which delivery applies the work-product transfer.
2. **epitaph** — always. The pinned manner of ending: *final response*, *stopped*, *budget-exhausted*, or *died*.
3. **terminal response** — present iff the agent spoke.

On disk this is the ordinary deposit shape (§2.11): `terminal_ref:` and `epitaph:` frontmatter, the terminal response as the body, the body absent exactly when the agent never spoke.

Both the terminal ref and the epitaph are **pinned** in the message so later revival of the child (§2.4) cannot move what the parent was offered: a revived child's live classification flips from `stopped` to `quiescent` (§3.5), but the parent must still see what it was handed — the epitaph is a frozen fact, not the live query §3.5 answers. The epitaph is the **union** over every terminal event, never the exception-set: were it to name only the no-terminal-response cases, it would be a *kind* of message and code would branch on the message's shape, re-opening the special case this whole channel dissolves. As a total field it does the opposite — code branches on the epitaph's **value** (a parent reasons differently about a budget-killed child than a finished one), never on shape. One primitive, one delivery path, one work-product transfer, three fields, one of them sometimes empty. Delivery at the parent's next step boundary is the ordinary delivery commit. The child's transcript, summaries, goal, and soul never appear anywhere in this: they stay on the child's branch, structurally, because nothing merges (§2.3 *Return, not merge-back*).

**The work-product transfer.** Results that are *files* — code written, docs edited — travel as a diff, not a merge. At delivery, the executor applies the diff from the child's fork point (its dispatch commit's parent, derived from ancestry) to the terminal sha the message names, **filtered to work products** (§2.1): the child's branch-scoped context paths — `goal.md`, `soul.md`, `messages/**`, `summary/**`, `skills/**` — are excluded from the application. The filtered diff lands as one commit at the parent's sequence tail, immediately before the result message's own delivery commit. Its properties are by construction:

- **Append-only and ordered.** The transfer is an ordinary commit landed by the parent's own executor at a step boundary — the same discipline as any delivery (§2.11), serialized by the single executor; two siblings' transfers cannot interleave, and the parent's branch never advances by any hand but its executor's (§2.3).
- **Conflict-free.** Sibling write paths are disjoint and children edit work products from the parent's own fork point (§2.5), so the application is clean unless the write-path guarantee was violated — a harness defect. A diff that fails to apply is **declined loudly**: the result message still delivers (the terminal ref it names preserves every byte of the work on the child's branch), and the failure is marked git-natively at `refs/lernie/conflicted/<agent-id>` for operator attention — the same marking pattern as budget exhaustion (§6), surfaced by the UI (§7.1).
- **Declinable.** The transfer is one commit; a parent that decides against the work reverts it — ordinary curation (§5.4), no protocol.

> **Shipped-state note.** The shipped harness still implements the v0.3 protocol this section retires — `merge=ours` attributes at scaffold, the alignment step, rebase-then-merge (see `tests/merge_ours.rs`). This section is the target contract; deleting the merge machinery and building the result-message return is implementation work tracked separately.

**Merge is reserved for compaction.** With merge-back gone, exactly one merge remains in the system: the **compaction merge**. A compactor (§2.7) forks off the dispatching branch at a **checkpoint commit** `C` and rewrites only what existed at `C` — deleting superseded transcript entries, landing a new summary — while the live agent keeps stepping. The live branch's commits since `C` only *append* new sequence filenames (transcript immutability, §2.3) — so the two write sets are disjoint by construction and the merge is conflict-free: the agent's executor lands it `--no-ff` at a step boundary, on the workflow's binding for the compactor's return (§6). The merge commit is the context rebuild point (§5.5). The one theoretical overlap — the compactor nominating a *work product* the agent has rewritten since `C` — resolves live-branch-wins: the executor drops that deletion at merge time. A dropped deletion is lost compaction, never lost work — the same worst case the compactor's deletion-only toolset already guarantees (§2.7). No quiescence is required and none is imposed: v0.3's rule that the dispatching branch idle while its compactor runs is deleted; compaction is surgical, and the agent steps straight through it.

The compaction merge needs no exemption from a pinning discipline, because no pinning discipline remains to be exempt from; the polarity that motivated the old exemption survives as the shape of the two channels. A *child's* context is contamination the parent must never inherit — so the return path is a message plus a filtered diff that structurally cannot carry it. A *compactor's* payload is the dispatching branch's own context, rewritten on purpose — so its path is the one merge. Two channels, each admitting exactly what the other excludes.

Step records pass through none of this. They live at `<workspace>/steps/<agent-id>/NNN/` — outside every worktree, shared across the whole tree from the moment they're written (§2.2, §2.3), namespaced by agent id, collision-free with no alignment of any kind.

### 2.7 Compaction

Compaction (not compression — the term is reserved for model-weights quantization in the taxonomy) is the process of producing a signal-preserving, minimal version of a branch's context. A **compactor** is a child agent that performs compaction. It has almost no privileged position in the architecture: compactors are dispatched like any other child agent and run on their own branches; what is unique to them is how their output lands — the compaction merge (§2.6), the one merge in the system. What distinguishes a compactor otherwise is its goal (produce a summary of the dispatching branch's history) and its toolset.

Compactor toolset (v1):

- **`write_summary(content)`** — writes the compacted summary file on the compactor branch.
- **`mark_for_deletion(path)`** — nominates a file on the compactor branch for removal. The harness applies the deletions at commit time.

Giving the compactor no general filesystem write surface makes "deletion-only" structural rather than disciplinary: the worst case is lost information, never corrupted information. The compactor has access to the dispatching branch's goal, passed through as `parent_goal`, and decides relevance against it. Its view of the dispatching branch's work needs no special channel: forked off the checkpoint commit (§2.6), its worktree carries that branch's transcript (§2.3), summaries, and work products — the worktree invariant (§5.1) *is* the view.

**Checkpoints.** Compaction runs at checkpoints during a branch's execution: the harness dispatches a compactor off the branch's tip — the checkpoint commit `C` of §2.6 — with a goal that instructs it to compact. The compactor writes a new numbered summary (`summary/<seq>.md`, on its own branch) and marks superseded files — prior summaries, stale transcript entries, spent skill bodies — for deletion. Its return lands as the compaction merge (§2.6); the dispatching agent steps straight through the whole operation, since compaction is surgical and no quiescence is required (§2.6). Checkpoint triggers are declared in `workflow.yaml` (§6): by commit count, by elapsed time, or by an explicit `flush` action the agent may call. A branch with no configured trigger never compacts.

There is no *terminal* compaction stage anymore: a child's result message carries its own terminal response (§2.6), not a compactor product, and with merge-back gone (§2.6) there is no merge payload to slim before returning. A workflow that wants a distilled hand-off before a child returns binds a compactor dispatch to the child's completion event (§6) — policy in config, not a pipeline stage.

Summaries are never parent-visible: `summary/**` is branch-scoped (§2.2) and excluded from the work-product transfer (§2.6), so a child's summaries structurally cannot reach its parent's tree — they exist to help the branch manage its own context window. What a parent receives from a child is the result message and the filtered diff (§2.6); "parents see only returned results, never raw internal state" is enforced by the shape of the channel, not by a merge discipline.

**Compaction is the sole sanctioned reorganization point — and it is a merge operation.** Between compactions, assembly is append-only (§5.5): the assembled context only grows at the tail, keeping the provider's cached prefix valid. The compaction merge — the compactor's branch merging into the dispatching branch (§2.6) — is where the context is rebuilt: superseded transcript entries deleted, a new summary landed, the manifest's category order reapplied at the next assembly (§5.2), and the cache prefix deliberately flushed. The merge commit *is* the rebuild point (§5.5); no other event reorganizes a running branch's context. Why this one merge is legitimate where merge-back was not is the two-channels polarity of §2.6: a compactor's payload is the dispatching branch's own context, rewritten on purpose — its payload, not its contamination.

In every other respect the compactor is an ordinary child: it returns by depositing its result message like any other (§2.5, §2.6), and the dispatching branch does not idle while it runs (§2.6).

Compaction failures (compactor produces garbage, times out, etc.) land no merge: the branch continues uncompacted and retries at its next checkpoint trigger; the failed compactor is an ordinary stopped child (§2.5, §2.9), surfaced for user review like any other child failure.

### 2.8 Goals

Every agent has a **goal**: the stated objective it was dispatched with. The goal is written to `goal.md` at the root of the branch's worktree (alongside `soul.md`) on the dispatch commit (§2.3 step 2) and is pinned at the head of the context assembled for every model call on that branch, regardless of position in the message sequence. A child's goal lives on the child's branch and nowhere else: no merge-back exists and the work-product transfer excludes it (§2.6), so a child's goal structurally cannot touch its parent's.

A goal is set at dispatch and is not rewritten during execution. If an agent determines its goal is wrong, the expected workflow is:

1. Terminate the current branch (stopped or with an explanatory terminal response).
2. Analyze the failure.
3. Dispatch a new agent with a corrected goal and/or a different config (§2.2).

Rewriting a goal in place is not structurally forbidden but is not a core path — the expected unit of iteration is the agent, not the goal. Mid-flight steering does not rewrite the goal either: a message (§2.11) adds context *beside* the pinned goal; the goal stays what the agent was dispatched with.

The pinned goal resolves the recency-decay problem in deep agent trees where sequence-as-authority (last user message = current order) fails.

### 2.9 Stopped branches

Stops are aggressive. When a stop is issued (by user, by timeout, by cascade from a parent — the user-driven case is the CLI subcommand `lernie stop <workspace> <agent>` per §3.4, idempotent against an already-terminal branch):

1. SIGTERM is sent to the harness process working on the branch (and the kernel cascades through its process group, covering tool subprocesses and any child-agent harnesses spawned per §3.4).
2. In-flight HTTP requests to provider endpoints are dropped as a side effect of the adapter receiving SIGTERM — `bz` installs no signal handlers and dies at once (§4.4 Cancellation); already-flushed event lines stay valid.
3. The adapter (`bz`) is already dead (step 2), so the latest step's `response.json` (§4.4 "On-disk response shape: appended attempt segments") will close without a terminal `end` event — this missing terminal event *is* the on-disk signature of a stopped step. No separate cancel marker is written; the absence of a trailing `end` on a closed file is sufficient (§3.5). The executor itself does **not** die on the spot: SIGTERM is catchable (unlike the uncatchable crashes of §8), and the executor's handler first deposits the branch's **result message** with a `stopped` epitaph into its parent's inbox (§2.6, and "deposits a result" below) — if the branch is a child; a root has no parent inbox — then exits, at which point the kernel closes its fds and the `response.json` receives `IN_CLOSE_WRITE` (§3.5). The deposit lands in the parent's inbox tree; the stop signature lives on the stopped branch's own `response.json` — independent writes, neither disturbing the other, so the missing-`end` signature is untouched by the return.
4. The branch is left where it was. Its `stopped` status is derived state, not a written flag — consistent with `docs/PRINCIPLES.md` "Single source of truth": a branch with no live executor whose latest step's `response.json` is closed without a trailing `end` is `stopped` (§3.5). (Crashes, kills, and explicit user stops are indistinguishable on disk and treated identically.)

The user-action stop (`lernie stop <workspace> <agent>`) discovers the harness pid the same way: it scans `/proc/<pid>/fd/*` for the writer holding the latest step's `response.json` open and signals that pid's process group. There is no sidecar pid file — the open fd is already the source the §3.5 `in_flight` classification reads, so the same observation drives both. The harness sets its own process group at startup (`setpgid(0, 0)` in `lernie prompt`) so the `kill(-pgid, SIGTERM)` cascade reaches its provider adapter and any child-agent harnesses re-entered via `lernie dispatch` (which deliberately do *not* setpgid, inheriting the parent's pgid) without touching the invoking shell or UI process.

A stopped agent deposits a result. A stopped child deposits its result message with a `stopped` epitaph (§2.6), performed by its executor's SIGTERM handler (step 3) on its way out — the deposit is **executor-side**, never a model tool call (the model has no return verb; termination is the return, §2.3 step 5, "Return is not a verb" in `docs/PRINCIPLES.md`). The parent is thereby revived (§2.11) and offered the manner of ending, not left to infer it from an absent message. A stopped root agent has no parent inbox: its exchange ends without a terminal response (§2.4). A child that crashed too hard to run its handler (SIGKILL, OOM, panic) deposits nothing itself; the §8 sweep deposits on its behalf, with a `died` epitaph. The branch remains as a ref for retention (§9.2), and a stop is not a locked door: the paths forward are the ordinary primitives — a message into the stopped agent's inbox starts a driver and resumes the same branch (§2.4, §2.11), or a fork off any of its commits starts a fresh agent (§2.3). There is no distinct "resume" operation — message-and-drive and fork-from-history subsume what resume would have done, and a third name would only obscure that.

Between-step lulls (the brief window when one `lernie advance` subprocess has emitted its terminal `end` and exited but the next `lernie advance` subprocess has not yet exec'd) are not stops: the latest step's `response.json` ends with `end`, distinguishing it from a terminated chain. The §6 stateless re-entrance pattern guarantees that during normal operation a chain has either an emitted terminal event or a live process — never neither.

Default retention: 30 days, then archived or deleted per §9.2 (branch deletion plus object pruning).

### 2.10 Retries and failures

**Read state per step.** Step 1 commits the dispatch artifacts (`goal.md`, `soul.md` per §2.3 step 2) on the new branch *before* its model call; that commit is step 1's read state. Step ≥2 takes no separate pre-call commit — the prior step's tip (advanced by its transcript entries, worktree side effects, and any delivery commits, §2.3, §2.11) is its read state. The branch tip at step-start is recorded in `meta.json`'s `commit` field (§2.3) so retry and replay are tractable: a failed model call is reissued by re-running the context assembler (§5) against the recorded sha and re-invoking the adapter, with no drift from the original wire input. The on-disk `request.json` is diagnostic, not authoritative.

- **Retryable provider errors** (transient network failure, 429 rate limit, 5xx) are retried inline with backoff, bounded by the attempt cap in `workflow.yaml` (§6). Retryability is classified by `CanonicalError::retryable()` from the linked brazen crate (§4.4) — computed from the in-band `Error` event, never re-derived by the harness. Each retry is a fresh attempt: a new `bz` invocation appending a new segment to `response.json` (§4.4). Retries do not produce additional commits — the commit frames the model call, not the individual attempt.
- **Non-retryable errors** (400 validation failure, auth failure, impossible-to-satisfy schema) abort the step. The branch is left in the state it held before the model call and flagged for operator attention.
- **Unknown or ambiguous failures** trigger a diagnostic dispatch: a child agent is dispatched off the branch's current commit with a goal describing the failure, access to the branch state and the raw error, and instructions to produce a recommended next action (retry, abort, modify config, escalate).

**Resumable pauses.** A `Finish{Pause}` (the linked brazen crate's `FinishReason::Pause`, Anthropic `pause_turn`) is neither a completion nor a failure: the provider stopped mid-work and expects the partial assistant content replayed back to continue (brazen `anthropic-messages.md` §3.5, "resume by re-sending the assistant content as-is"). The harness absorbs it *within the same step*, not as a new step: on the `Pause` segment it re-invokes `bz` with the paused content appended — read from the writer's staging file (§2.3 *The transcript writer*), where a `Pause`-terminated segment's blocks accumulate; the writer reading its own sink, not a diagnostic record — landing a further attempt segment (§2.1) on the same `response.json` — the identical multi-segment shape as retry, differing only in the request delta (a retry re-sends the identical request; a Pause continuation appends the paused content). A *new* step is structurally impossible here: a step's read state is a commit (§2.3), but the paused partial content is never committed worktree data — it lives in the staging file and the diagnostic `response.json`, neither of which is a commit and neither of which context assembly may read (§2.3, §5.1) — so it has no read-state home, and only within-step continuation preserves the replay-from-recorded-commit invariant above. Each continuation is a billed attempt (§6, §8), and the loop is bounded by the step's attempt cap (§6) so a pathological pause loop terminates. brazen emits `Pause` *only* with provider server tools (brazen `anthropic-messages.md` §3.5), which lernie's stdio tool contract (§3.3) does not enable — so no v1 model call can pause; the absorption is specified here so classification (§3.5, §4.4) is total, and it activates whenever server tools are wired (a future milestone).

Tool failures follow the same shape at the tool-executor level, surfaced to the emitting agent as the tool's `tool_result` content. The agent decides whether to retry, ignore, or escalate — this is an ordinary agent decision, not a harness one.

### 2.11 Messages

A **message** is content addressed to an *existing* agent. Any sender may deposit one: the user, the agent's parent, one of its children, a sibling — or the agent itself. This generalizes the dispatch symmetry (§2.5): where dispatch makes the parent indistinguishable from a user *at spawn*, messaging makes **every sender indistinguishable from a user for the rest of the agent's life**. One primitive dissolves what would otherwise be five features — the user steering a running exchange, the user reprompting a finished one (§2.4), a parent steering a dispatched child, a child reporting upward to its still-running parent (the reminder/watchdog shape — and, as the result message, every return, §2.6), and the user dropping into any descendant of the tree. There is no "attach" operation to define: watching any branch is already the frontend contract (§3.5), and speaking to it is a message.

An agent is exactly two things — recoverable on-disk state, and at most one in-flight execution. A message is a write to the first that the second picks up at its own pace; and if there is no second, the write conjures one (below).

> **Shipped-state note.** Nothing implements this section yet — no `inbox/`, no `message` verb or tool, no executor lock, no delivery commit. This section is the target contract; implementation is tracked as bl-cb44 (lock, deposit, `lernie message`, the `message` tool), bl-1129 (the delivery drain), and bl-d148 (the startup scan); the result-message return path that rides the channel is bl-4ce8.

**Deposit.** `lernie message <workspace> <agent> <content>` (§3.4) writes the message to the recipient's **inbox** at `<workspace>/inbox/<agent-id>/<sender>-<NNN>.md` — at the workspace root, outside every worktree, namespaced by agent id exactly like `steps/` (§2.2). `<sender>` is the depositing agent's id, or `user`; `<NNN>` is the sender's own sequence. A deposit is a **create-only new file**, never an append or edit — an append is two messages — and writes are temp-path + atomic rename. This makes the inbox lock-free by construction: sender-namespacing makes cross-sender collision impossible (two senders never target the same file — the same construction as §2.5's write-path guarantee), a single sender is sequential with itself, and create-only means no reader can observe a half-written mutation. The file carries `from:` and `deposited_at:` frontmatter, and the division of labor is general: **the path carries exactly one fact — framing** (the sender in the deposit name; the wire role from the origin token after delivery, §2.3) — **every other fact a message asserts rides its frontmatter, and the body is the content.** The result message (§2.6) is the worked example: an ordinary deposit whose frontmatter adds `epitaph:` and `terminal_ref:`, whose body is the terminal response verbatim, and whose body is absent exactly when the agent never spoke — one file shape, no sidecar, no variant kinds; code branches on frontmatter values, never on shape. Frontmatter is delivered with the message and is model-visible, which is the point: the epitaph exists for the parent to reason about. Provenance is thus recorded even though the recipient treats every sender uniformly. That uniformity is by design, not an oversight — every sender is influence-equivalent for the recipient's whole life, so a hostile message is a lateral prompt-injection surface the primitive does not itself fence; the single-user assumption (§1.1) covers authorization but not influence, and defense is deferred and layered (§11), not a change to the deposit.

The model-facing side is the `message` built-in tool (`lernie tool message`, input `{agent, content}`), which goes through the CLI per §3.4. The sender identity is taken from `LERNIE_CONV_BRANCH` (§3.3) — harness-derived, never model-supplied, so an agent cannot forge provenance. Deposit is synchronous and returns `{status: deposited}`: it is not a dispatch, creates no branch, and returns no child address. Addressing needs no registry, because an agent's address *is* its id (§2.3): the children's addresses are the ids their dispatches returned (§2.5), the parent's address is the agent's own id minus its last descent segment, and the user reads addresses off the UI's branch view (§7.1).

**A deposit into a quiescent agent starts a driver.** Delivery needs a step boundary and a step boundary needs an executor, so after depositing, `lernie message` probes the executor lock (a non-blocking try-acquire; *succeeding* means no one is driving the branch). Finding the agent quiescent, it releases the probe and launches a driver — `lernie advance` (§6) — rather than refusing: the driver acquires the lock, rematerializes the worktree if it was torn down (§2.3 step 6), drains the inbox, and steps. This is the reprompt path (§2.4): "messaging a quiescent agent" *is* how a root agent receives its next user message, so it must succeed and must cause execution. (The v0.3 rule was the loud inverse — a deposit with no live executor was refused, and the remedy was a fresh dispatch consuming the old branch "as context." The refusal inverted when reprompt collapsed onto the message primitive: the branch itself continues, so nothing needs to be declined and no new agent needs to exist.) The launch is race-free without coordination: if several deposits each launch a driver, or a driver was already starting, the lock admits one and the rest exit as clean no-ops (Writer/driver totality, below). Launching is not driving — the message verb never holds the lock past the probe; the driver it spawns competes for the acquire like any other.

**Delivery.** At each step boundary — after the prior step resolves, before the next model call's context is assembled — the executor drains the inbox: each pending message file moves into the branch's worktree as a transcript entry at `messages/<NNN>-<sender>.md` — `NNN` drawn from the branch's single transcript counter (§2.3), the same sequence the executor's own assistant output and tool results commit into — and the move is committed (the **delivery commit**). The move is a literal `rename(2)` — the message file has exactly one home at every instant, so delivery can neither duplicate nor lose it — and the crash window between rename and commit is closed at the other end: a drain *begins* by committing any renamed-but-uncommitted stray a prior death left in `messages/` (the file left the inbox, so it must land — the inverse of §2.3's "an entry that never committed never happened"). A result message (§2.6) additionally applies its work-product transfer as a commit immediately before its own delivery commit. Delivery is where ordering becomes authoritative: arrival order across senders in the inbox is advisory (mtimes), and the committed sequence is the record — the order question has one home, and it is the commit. From there, no new mechanism exists or is needed:

- **Context inclusion is the worktree invariant.** Delivered messages are transcript entries, so §5.1 composes them into every subsequent model call at their sequence position (§2.3, §5.5). The path is the hint (§5.3): `messages/007-user.md` reads as provenance at a glance.
- **Replay is the read-state rule.** A step's read state is a commit (§2.3, §2.10). Because delivery is a commit landing ahead of the model call, replay from `meta.json`'s `commit` reproduces exactly what the step saw — a message cannot appear in a prompt without a commit that carries it.
- **Curation is deletion.** The agent may `rm` a stale message (§5.4); the compactor may `mark_for_deletion` delivered messages like any other worktree file (§2.7).
- **Nothing crosses to the parent.** A child's delivered messages are transcript entries on the child's branch, and they stay there: no merge-back exists, and the work-product transfer excludes `messages/**` (§2.6).

Step-boundary delivery is not a compromise; it is the only possible seam. Mid-step injection is structurally impossible twice over: the provider wire requires every `tool_use` matched by a `tool_result` in the immediately following user message (§2.5), and a mid-step message would have no read-state home (§2.3). A deposit therefore never interrupts in-flight work — interruption remains `lernie stop` (§2.9), a different verb because it is a different act.

A parent that has dispatched a child and gone quiescent (§2.4) is this same rule at its limit: with no step in flight, the child's deposit *revives* it — the deposit starts a driver, the delivery commit lands at the fresh step's boundary, and the parent reacts ("A deposit into a quiescent agent starts a driver," above). The parent never blocks on a child, so there is no distant-boundary case to reconcile: it is either stepping — draining at the next boundary — or quiescent — revived by the deposit. Waiting on a child is the same act as waiting on the user (§2.5); parking is parking.

**The executor lock: at most one step loop per branch.** Messaging makes concurrent invocations against one branch an ordinary event — a user and a child acting on the same agent at once — so the exclusivity that was previously implicit is a stated invariant: **at most one executor per branch**. The mechanism is `flock(2)` on the agent's inbox directory fd, acquired non-blocking when an executor starts and held for the whole step loop. (The inbox directory lives at the workspace root and persists across worktree teardown, §2.3 step 6, so the lock's home outlives the substrate's materialization.) The lock is kernel state bound to process lifetime: released by the kernel on any death, observable but never written — the same family as the fd-open `in_flight` signal (§2.9, §3.5, §4.4). Liveness is observed, not stored, and there is no stale-lock cleanup because there is nothing on disk to go stale (`docs/PRINCIPLES.md` "Single source of truth"). The lock fd is deliberately inherited across the §6 exec baton (`lernie advance` exec'ing the next `lernie advance`), so the lease spans the chain exactly as long as the chain lives. Together with git's one-branch-one-worktree rule, the lock makes driver exclusivity total (§2.3 "Executor exclusivity is two existing rules").

The lock and the fd-open model-call signal (§2.9, §4.4) are **two observations of two different facts, deliberately not collapsed**: the lock answers *is anyone driving this branch* — held across tool execution, inbox drains, and retry backoff sleeps alike — while the open `response.json` answers *is a model call in flight right now, and which pid signals it* (`lernie stop`'s writer discovery, §2.9; §3.5's `in_flight` sub-state). Widening either to serve the other would misclassify a tool-executing driver as absent or a between-calls driver as in a model call. What they share is the discipline, stated once here: kernel state bound to process lifetime, observed, never stored, nothing on disk to go stale.

**Writer/driver totality.** Every invocation against a branch is exactly one of two things, never both. A **writer** (`lernie message`) deposits and exits; its lock try-acquire is the liveness probe above — released immediately, never held to drive — and a writer never contends for execution at all: when the probe finds no driver, the writer *launches* one and exits, it does not become one. A **driver** (`lernie prompt`, `lernie dispatch`, `lernie advance`) acquires-or-exits: winning the acquire, it *is* the executor and runs the ordinary synchronous loop; losing it, it has nothing to deposit and exits as a clean no-op — not an error, because the branch being already driven is the condition it exists to establish. No abandonment, no takeover, no watcher left behind: the loser terminates, and any process that wants to *observe* the branch does so through the read-only frontend contract (§3.5), which is not a role the lock assigns. Because no verb combines the arms, the losing path is the same code as the uncontended one — a deposit is a deposit, a no-op is an exit — and there is no second-class branch of a combined flow to drift out of step. This closes a gap that predates messaging: §6's idempotence rule covers *sequential* replay of `lernie advance`, and the executor lock is its concurrency half — two simultaneous drivers against one branch resolve to one executor and one no-op, never two step loops.

**Undelivered is derived.** A deposit can land in the gap between an executor's final drain and its termination; the writer's post-deposit probe closes this race in the common case (it observes the death and launches a fresh driver), but the probe is best-effort against observed state, so the classification stays on disk rather than in anyone's memory: a message file still under `inbox/<agent-id>/` whose agent has no live executor is *undelivered* — a derived classification (inbox listing + the lock probe), never a flag. Under normal operation it is transient — the next driver, launched by any deposit or started by hand, delivers it. The sender observes its outcome the way anything is observed here: its file left the inbox in a delivery commit, or it didn't. Undelivered count is a §8 health metric alongside silent deaths.

**The startup scan.** Every driver invocation (`lernie prompt`, `lernie dispatch`, `lernie advance`), before it touches its own branch, runs one workspace-wide scan with two derived actions. First, the **silent-death sweep** (§8): for each hard-crashed child in the candidate set — no live executor, died mid-work or never deposited — it deposits the `died`-epitaph result message on the child's behalf (§2.6, §2.9). Second, the **inbox flush**: it lists `inbox/*/*`, and every agent with pending files and a free lock gets a driver launched for it — *launched, never drained*: the scanner moves no files and commits nothing; only an agent's own lock-holding executor delivers (Writer/driver totality, above), and an agent whose lock is held is left alone — its executor drains at its next boundary. The sweep's own deposits are picked up by the flush that follows in the same scan. The scan is a startup phase of a transient process — no resident watcher (none is permitted, above) and no schedule. A fully idle workspace therefore stays unswept until its next invocation, and that is accepted rather than patched: with nothing running, nobody is blocked on the outcome, and the first touch — a user reprompt anywhere, or a cron'd `lernie advance` if an operator wants a heartbeat (config, not schema) — flushes everything pending. This is what closes the undelivered race (above) beyond the writer's best-effort probe, and what makes the stalled-parent story self-healing with no new verb: reprompting a parked parent starts a driver, the driver's scan deposits its dead child's epitaph, and the drain that delivers the user's message delivers the epitaph beside it.

**Self-messages are legal and unremarkable.** An agent may deposit into its own inbox — a note its own next step boundary delivers. No special case arises; the sender happens to equal the recipient. (Distinct from the executor's own step output, which is not a message — not addressed to anyone — and takes no inbox hop: it delivery-commits directly into the same sequence, §2.3 "Self-output commits directly.")

---

## 3. Component Architecture

### 3.1 Disk as Bus

All components communicate through the filesystem. No shared memory, no direct function calls between components. This applies to:

- Harness → provider adapter (request mirrored to disk, adapter reads from stdin; response events mirrored back to disk from adapter stdout — see §4.4).
- Harness → tool execution (tool call record written to disk, executor reads, output streamed to disk).
- Harness → UI: the filesystem is the event stream. The UI watches paths in the workspace (git refs — config and agent branches alike; worktree contents `goal.md`, `soul.md`, `summary/`, `messages/`, `descriptions/`, `skills/`; the workspace-root step records under `steps/<agent-id>/NNN/` per §2.2; the workspace-root inboxes under `inbox/<agent-id>/` per §2.11) and re-renders on change. Notification is inotify where available, polling otherwise.
- UI → Harness: user actions are issued as `lernie <subcommand>` invocations per §3.4. There is no input directory.

**Threads, not processes.** "Worker" and "executor" name roles that run as threads inside the single harness process; the disk contract is not an inter-process bus. Tool subprocesses invoked by the tool executor are genuinely separate processes. Routing inter-role communication through disk — even between threads in the same process — is load-bearing rather than ceremonial: it is what buys inspectability, audit trail, and the single-author-per-file discipline that keeps many concurrent workers from corrupting each other's state.

Notification mechanism: inotify where available, with a polling fallback. Every event write updates a `last_event_ts` sentinel file as a sanity check for consumers to detect missed events.

Consequences:

- Every component is independently restartable (threads respawn; subprocesses are relaunched from disk state).
- Replay from any point is `git checkout <ref>` + re-tail the event log.
- Latency is higher than in-memory IPC. Accepted and compensated by streaming-first UI design.

### 3.2 Components

- **Harness** (permitted synonym: **daemon**). The single program that drives execution: watches for events, forks branches, runs model calls via the provider adapter layer (§4.4), invokes the tool executor, triggers compactions and lands their merges (§2.6), updates state. It owns all external↔filesystem interaction on the repo — provider endpoints (via adapter subprocesses), tool subprocesses, git operations. Stateless across restarts — resumes from disk. Any place this document says "the harness does X", it is this component. "Daemon" is allowed as a shorthand; both refer to the same role.
- **Tool executor.** Runs tool subprocesses on behalf of the harness; contract in §3.3. Per tool call: assembles stdin from the `tool_use.input` the model emitted; invokes the tool binary (`lernie tool <name>` in-process or `lernie-tool-<name>` external); captures stdout and stderr atomically (temp path + rename) into the diagnostic `<workspace>/steps/<agent-id>/<NNN>/tools/<tool-id>/output.json` (out of every worktree, §2.2, §2.3); derives the canonical `tool_result` block (stdout as content, stderr concatenated on non-zero exit, exit code as `is_error`) and commits it as a transcript entry `messages/NNN-tool.json` on the emitting branch (§2.3, §3.3), from which the next step's request payload composes. Cascades SIGTERM on cancel (§2.9) with a 5s deadline before SIGKILL (§3.3). Termination by a signal other than the harness's own SIGTERM (SIGSEGV, SIGABRT, etc.) is a harness-level fault per §2.10. Does not auto-dispatch on oversized tool output in v0.3 (§11).
- **Provider adapter.** External binary — brazen's `bz`, one binary for every provider — that owns HTTP, wire dialects, and auth. Invoked per attempt over stdio; non-resident (process per attempt, no long-lived state). Transient-error retry belongs to the harness (§2.10), not the adapter. Contract in §4.4.
- **UI** (permitted synonym: **frontend**). A stateless renderer over the workspace. Reads and watches filesystem paths in the repo; issues user actions exclusively as `lernie <subcommand>` invocations per §3.4. Holds no persistent state — every render is a pure function of filesystem state at the current git ref. The UI is pluggable: multiple frontends (desktop GUI, webclient, TUI) may run concurrently against one repo without coordination, because they share nothing but the filesystem and the CLI. Contract in §3.5. "Frontend" is allowed as a shorthand; both refer to the same role.

The harness, the tool executor, and provider adapters share the same disk contract. None share memory. The UI participates in the same disk contract as a read-only consumer.

### 3.3 Tools and skills

Tools and skills are separate primitives. Every tool has a skill; skills can exist without tools.

**Skill.** A directory containing a `SKILL.md` with YAML frontmatter (`name`, `description`) plus markdown instructions, optionally with bundled scripts and reference files. Skill source directories live globally at `<data-root>/skills/<name>/` and are referenced from the workspace by two mechanisms:

- **Description-always.** Every available skill's `SKILL.md` frontmatter (name + description) is committed in the config under `descriptions/skills/` (§2.2), where it is inherited by every agent branch via git and composed into the context on every model call. This is the Anthropic progressive-disclosure convention (`docs/TAXONOMY.md` §4).
- **Body-on-demand.** When an agent elects to use a skill, the harness copies the skill directory into the current branch's worktree at `skills/<name>/`. From that point on, the skill body is part of the agent's context (§5.1). The compactor may `mark_for_deletion` a skill directory when it is no longer needed — next context assembly sees the branch without it.

Copying (not symlinking) is deliberate. It is the same portability discipline as the rest of the repo (§2.2): a skill that lives in the worktree is self-contained and survives the global skill directory changing or disappearing. Disk cost is trivial.

A standalone skill (no associated tool) exists to give an agent capability via prompt — recipes, conventions, workflows — without a callable binary.

**Tool.** Composed of three required artifacts:

1. **Binary.** An executable invoked by the harness. **In-process** tools are subcommands of the `lernie` binary, addressed as `lernie tool <name>` — the default for tools shipped with the harness (v0.3 ships `bash` and `read_file` here). **External** tools are standalone binaries named `lernie-tool-<name>` — the externalization pattern that lets contributors ship tools without patching the core (the provider layer used the same per-name pattern until v0.6 retired it for brazen, §4.4). The harness looks up `lernie-tool-<name>` at `<data-root>/tools/` (installed by `make install`) before falling back to `PATH`. The choice of flavor is per-tool; the stdio contract below is identical.
2. **JSON schema.** Declares the tool's `input` parameters at `<data-root>/tools/<name>.json`. Required by provider APIs. Either generated from the binary's metadata or hand-authored. Sent verbatim as the schema of the tool's entry in the canonical request's `tools: [...]` array (a brazen `Tool::Custom` — each protocol projects it into its own spelling, §4.4). A copy is committed under `descriptions/tools/` in the config (§2.2), inherited by every agent branch via git, and composed into the context.
3. **Skill.** A `SKILL.md` describing when and how the tool should be used. Required for every tool; follows the skill lifecycle above. The frontmatter `description` becomes the `description` field of the tool's entry in the `tools: [...]` array.

**Tools-list assembly.** A role's enabled tools are declared in the config's `providers.yaml` `roles:` section under a `tools: [...]` field (§2.2, §4.3). Any on-disk triple (binary + schema + skill) is generally discoverable; role configs select from that pool. At request-assembly time the harness composes the declared names against the schemas committed in the branch's read-state tree (§2.10): each declared tool whose `descriptions/tools/<name>.json` schema is present becomes one entry in the canonical request's `tools: [...]` array, with that file sent verbatim as `input_schema`; a declared name with no committed schema is dropped (the intersection of declaration and availability), and a present-but-malformed schema is declined rather than dropped. **Shipped-state note:** both halves are now wired — the consumer (this assembly) shipped with bl-9e96 and the descriptions-always producer with bl-3092, which snapshots the data-root pools into `descriptions/**` as a step of the shipped creation routine (`lernie new`, `src/template`; the config-commit-authoring routine of §2.2 is the home this step migrates to as that routine is built out). A role declaring an available tool therefore composes a populated `tools: [...]` entry; an empty (or absent) data-root pool still yields an empty `descriptions/**` and an empty array, unchanged. An entry's `input_schema` (from `descriptions/tools/`) and its `description` (point 3, the tool's own `SKILL.md` frontmatter via `descriptions/skills/`) compose in one pass; a tool whose schema is present but whose skill frontmatter is absent composes with no top-level `description` rather than being dropped.

**Descriptions-always population.** The `descriptions/**` tree the assembly above intersects against is written by a single step of config-commit authoring (§2.2): the harness copies, from the data-root pools, every available tool's JSON schema (`<data-root>/tools/<name>.json` → `descriptions/tools/<name>.json`, point 2) and every available skill's `SKILL.md` frontmatter (`<data-root>/skills/<name>/SKILL.md` → `descriptions/skills/<name>.md`, the Description-always mechanism above). It is **one mechanism over two artifact kinds, not two producers.** Committing it in the config is what lets every agent branch inherit the descriptors via git (§2.2, §2.3) and makes context assembly intersect against the branch's own read-state tree (§2.10, §5.1) instead of re-reading mutable data-root state — the same snapshot-not-symlink discipline as skill bodies above, and the committed form of "fork is the freeze" (§2.2). The data-root pools are the single source of truth for *what this install provides*; the committed `descriptions/**` snapshot is the single source of truth for *what agents forked from this config are pinned to see* — distinct facts that must not drift together, so the copy is a snapshot, not a mirror (PRINCIPLES, single source of truth).

**Stdio contract.** Identical for in-process and external tools:

- **Stdin.** The `tool_use.input` JSON object the model emitted, passed verbatim. The tool owns its own input-schema validation; the harness does not interpret the payload beyond extracting it from the `tool_use` block.
- **Stdout.** Raw bytes. The harness wraps them as the `content` of the tool call's canonical `tool_result` block, committed as a transcript entry (§2.3) and composed into the next step's request from there. No JSON envelope around tool output — tools stay simple and the canonical `tool_result` shape is the harness's concern.
- **Stderr.** Raw bytes. Captured to the on-disk record regardless of exit status; concatenated into `tool_result.content` after stdout when the tool exits non-zero so the agent sees the failure message.
- **Exit code.** 0 → `tool_result.is_error = false`; non-zero → `tool_result.is_error = true`. Termination by a signal other than the harness's own SIGTERM (SIGSEGV, SIGABRT, etc.) is a harness-level fault per §2.10 — the step aborts and the branch is flagged, not delivered to the model as a semantic error.
- **Environment.** The harness sets two env vars on every tool subprocess so tools that depend on the calling agent's identity can read it without the model having to thread context through the input schema: `LERNIE_CONV_REPO` (absolute path to the workspace root, §2.2) and `LERNIE_CONV_BRANCH` (the calling agent's id == its full hyphenated descent, §2.3). The names predate the ladder and are kept — they are code, not prose. Both are derived from the executor's `step_dir` so they are guaranteed-correct for the tool call. Tools that do not need them ignore them; the v0.4 `dispatch` and `message` built-ins are the canonical readers. The direction and discipline are fixed: harness → subprocess, and the harness owns the names.
- **SIGTERM and deadline.** The harness sends SIGTERM on cancel (§2.9); the tool has 5 seconds to flush and exit cleanly, after which SIGKILL follows. The flush deadline is tools-only: the provider adapter needs none — `bz` dies on SIGTERM at once, and the missing trailing `end` is the signature (§4.4 Cancellation).

**Disk record.** The tool executor (§3.2) lands two files per tool call under `<workspace>/steps/<agent-id>/<NNN>/tools/<tool-id>/` — at the workspace root, outside every worktree (§2.2, §2.3):

- `input.json` — the `tool_use` block from the model verbatim (`id`, `name`, `input`).
- `output.json` — `{stdout, stderr, exit_code, started_at, ended_at}`.

`<tool-id>` is the `tool_use.id` from the wire (e.g. `toolu_01abc…`). Writes use temp-path + atomic rename. These records are not git-tracked (the location is outside every worktree).

**Commit-per-side-effect, serialized.** A tool call's *diagnostic* record (above) is not a commit — it is an out-of-worktree plain file. Every tool call commits at least its `tool_result` transcript entry on the emitting branch (§2.3); a tool call's *worktree side effects* (e.g. `bash` editing files in the branch's worktree) ride the same commits, landed before the next tool runs. Sibling tool calls running in parallel serialize their worktree commits, since a dirty worktree between siblings would violate "Single author per file" (§2.5, PRINCIPLES).

**Wire `tool_result` framing is transcript-backed.** The wire-level `tool_result` blocks the agent reads on its next step compose from the committed transcript entries (`messages/NNN-tool.json`, §2.3), never from `steps/`: the tool executor derives the canonical block — stdout as `content`, stderr concatenated on non-zero exit, `is_error` from the exit code, the pairing `tool_use_id` — once, at commit time. The per-call `output.json` remains the raw capture (full stdout/stderr/exit code with timestamps) for audit; the transcript entry is what entered context. These are two facts, not two copies — *what the subprocess emitted* versus *what the model saw* — the same snapshot discipline as `descriptions/**` (above). Tool outputs are nondeterministic and cannot be reconstructed by re-running, which is exactly why their context home must be a committed file (§2.3): replay from the read-state commit re-reads the same entry bit-for-bit. "Tool in progress" is derived state too — the emitting step's assistant transcript entry carries a `tool_use` block with no matching committed `tool_result` entry yet — not a separate file.

**Deferred to v0.4+ (see §11).** Oversized-output auto-dispatch — raw output handed to a parsing subagent, only the compacted result reaching the parent step — is not in v0.3. Oversized output reaches the agent unchanged.

**Sandbox (v1.1, §3.6).** The v1.1 milestone wraps this contract in a capability sandbox derived from the artifact kind — a `wasm32-wasip2` component runs WASI-clamped, a native binary runs only under an `exec` grant. The stdin/stdout/exit protocol above is unchanged; the sandbox bounds *authority*, not *interface*. See §3.6.

### 3.4 CLI as control plane

`lernie` is a single binary with subcommands. Every procedure the harness can start — child dispatch (§2.5), compaction (§2.7), verification, and any other workflow-invoked procedure (§6) — is reachable through a subcommand of that binary. The CLI is the sole entry point: a procedure invoking another procedure does so by going through the CLI dispatcher, never through in-process function calls, shared memory, or ad-hoc sockets. Child dispatch, the canonical case, is `lernie dispatch …`; messaging an existing agent is `lernie message …` (§2.11), the same front door whether the invoker is the user, a frontend, or another agent's `message` tool.

This is the invocation counterpart to §3.1. **Disk-as-bus carries state; CLI-as-control-plane carries commands.** Between any two procedures, state flows through the filesystem (§3.1) and invocations flow through the CLI. There is no third channel — no library API surface, no sidechannel. An external caller embedding lernie in another tool uses exactly the same CLI surface the harness uses internally; that symmetry is what lets lernie be a component in another tool rather than a standalone monolith.

Whether a given CLI invocation is dispatched as a subprocess `exec` or as an in-process re-entry into the same argument parser is an implementation detail chosen per-procedure (isolation needs, latency, resource cost). What is invariant is that the procedure's *interface* is the CLI: the same arguments, the same on-disk effects, whether exec'd or in-process. Internal procedures are not permitted to shortcut past the CLI dispatcher via a private function call.

Three consequences fall out:

- **Integration testability.** Every inter-procedure edge is an observable CLI invocation. A test captures the arguments and on-disk effects, asserts on their shape, and replays outputs from fixtures without needing to mock in-process interfaces.
- **Embeddability.** Embedding lernie in another tool is `exec("lernie", args)` with env-var auth. No library port, no shared runtime, no plugin loader.
- **No back-channels.** With disk for state and CLI for commands — and nothing else — operational atomicity is structural: a procedure has no surface with which to back-channel into another. The single-author-per-file discipline (§2.6) and the per-procedure commit boundaries that make replay work are consequences of this, not separate protections layered on top.

"Procedure" here is the term from §6 (subordinate routines invoked by workflow), extended to cover the dispatch and compactor invocations already described in §2.5 and §2.7 — every named operation the harness can start.

### 3.5 UI contract

A **UI** (or **frontend** — same role, §3.2) is any program that presents the workspace to a user. The architecture admits multiple frontends concurrently against one workspace — a desktop GUI and a webclient rendering the same agent simultaneously is the default case, not a special one.

The frontend surface is exactly two things, and nothing else:

1. **Filesystem reads.** The frontend reads and watches paths under the workspace. The load-bearing paths follow the layout (§2.2): the git tree itself (refs, commits, objects — branch state is read from `refs/heads/` per §2.3, config commits' control files among them); the workspace-root step records under `steps/<agent-id>/NNN/` (§2.3); the workspace-root inboxes under `inbox/<agent-id>/` (§2.11); and each agent's worktree contents (`goal.md`, `soul.md`, `summary/`, `messages/`, `descriptions/`, `skills/`). Notification is inotify where available, polling otherwise (§3.1).

   **Streaming text watch path.** Live model-call output is tailed at `<workspace>/steps/<agent-id>/<NNN>/response.json` — the JSONL stream of canonical events (§4.4) is appended event-by-event as the adapter writes it. **Completion signal: writer closes the fd; inotify `IN_CLOSE_WRITE` marks the response complete.** The frontend tails the file (offset-tracking line read), and on `IN_CLOSE_WRITE` flips the response from "in flight" to "done" — no separate sentinel file, no out-of-band marker. This is the same fd-close convention used elsewhere in the architecture (e.g. atomic-rename writes, where `IN_CLOSE_WRITE` on the temp path precedes the rename); here the streaming append *is* the writer, so close-of-fd directly signals end-of-stream.

   **Agent-state classification.** The agent states the live view renders (§7.1) are derived from the executor lock, refs, and the JSONL terminal event — never a sidecar marker. `live`: the agent's executor holds its lock (§2.11). `in_flight` (a sub-state of live): the latest step's `response.json` is still open (no `IN_CLOSE_WRITE` yet). The harness holds this fd open across *all* of a model call's attempts and the backoff sleeps between them (§4.4 "Fd held open for the whole model call"), so `in_flight` covers the entire retry loop — an intermediate failed attempt's trailing `end`, or a mid-retry `Error` segment, is still `in_flight`, never `stopped`, because the file is still open. The quiescent classifications are evaluated only with no lock held and the fd closed. `quiescent`: the latest step's `response.json` ends with `end` — a finished-for-now agent awaiting a message (§2.4); for a child, its return is its result deposit, visible in the parent's inbox or transcript (§2.6). `stopped`: no lock, closed, and the last JSONL line is not `end` (§2.9 — kill, crash, and explicit stop are indistinguishable on disk; a closed file whose last segment carries an `Error` event is a *failed* step per §2.10, rendered as stopped with the error surfaced). `declined-transfer` (`refs/lernie/conflicted/<agent-id>`, §2.6) and `budget-exhausted` (`refs/lernie/budget-exhausted/<branch>`, §6) are orthogonal ref-derived marks, rendered alongside.
2. **CLI invocations.** The frontend issues user actions by `exec`'ing `lernie <subcommand>`. New prompt, message, stop, fork-from-history — all are ordinary CLI subcommands per §3.4. There is no separate API surface, no socket, no shared input directory, no library port. There is no user-facing "resume": continuing a stopped or quiescent agent is `lernie message` (the deposit starts a driver, §2.11), and exploring an alternative is fork-from-history (new agent off any commit), per §2.3, §2.9.

Frontends hold no persistent state. Everything a frontend renders is derived from the filesystem at the current git ref; ephemeral UI state (cursor position, scroll offset, selection) lives in memory only and is discarded on exit. Restart is equivalent to re-reading the repo.

This discipline is what makes pluggability structural rather than aspirational. Two frontends running against one repo cannot corrupt each other because neither writes repo state; both observe the same on-disk ground truth, and both issue commands through the same CLI surface the harness itself uses. Swapping a frontend out is unplugging one reader; adding a second is adding another reader.

### 3.6 Sandboxed tools (v1.1)

Forward spec for the v1.1 milestone (§12). The sandbox is a **hosting wrapper** around the §3.3 tool contract, not a change to it: an external tool artifact runs inside a bounded authority envelope while still speaking the identical stdin/stdout/exit protocol. Everything §3.3 says about how a tool is invoked and how its output is framed holds verbatim; §3.6 adds only *with what authority* the artifact runs. In-process built-ins (`lernie tool <name>`, §3.3) are subcommands of the trusted `lernie` binary — they *are* the harness, not guests, and run with its authority; the sandbox governs only **external** artifacts (the `lernie-tool-<name>` slot at `<data-root>/tools/`). Shipping a tool in-process is the decision to place it in the trusted computing base.

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
- **Global lernie** (`<config-root>/models.yaml`, replacing the retired global `providers.yaml`; the config root is `$XDG_CONFIG_HOME/lernie`, §2.2). Model capabilities and context windows (§4.2) — facts lernie's behavior relies on and brazen does not own — plus the optional `adapter:` binary override (§4.4 Extensibility). This sits under the config root, not the data root, precisely because it is hand-edited and must survive a reinstall untouched — the same lifetime as brazen's own `config.toml`.
- **Per-config** (`providers.yaml` in the config commit, §2.2). Role → (provider row, model id) mapping and role toolsets only. Immutable for every agent forked from it — fork is the freeze (§2.2); governs which model that agent's roles dispatch to for the rest of its life.

The frozen-bootstrap property is preserved: a config pins *which row and model* its roles use; endpoint and auth resolve at call time inside brazen, so rotating a key or endpoint immediately affects in-flight agents (correctly), and the config file never carries machine-local or secret material. Retry policy is not provider config at all — the attempt cap and backoff are workflow policy and live in `workflow.yaml` (§6).

**Row names are the portability contract.** Because a config pins provider *row names* — not endpoints or credentials — replaying or moving a workspace (or an archived bundle, §9.2) to another machine requires that machine's brazen config to define rows of the *same names*. This is the same expectation v0.3's per-provider adapter binary names carried (a repo naming provider `anthropic` needed a `lernie-provider-anthropic` there, a row named `anthropic` here), now stated rather than left implicit. A row absent on the target machine is a load-time failure — brazen cannot resolve it and lernie declines the model call (`docs/PRINCIPLES.md` "Decline illegal operations") — never a silent fallback to a different provider. The row *name* travels with the workspace; its endpoint, auth, and model aliases stay machine-local by design (they rotate; the config split above).

### 4.2 Model abstraction

A **model** is (provider row, model_id, capabilities). Capabilities is an extensible mapping declaring features the harness can rely on. The `models:` block lives in the global `<config-root>/models.yaml` (the config root is `$XDG_CONFIG_HOME/lernie`, §2.2):

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

Agent roles specify which model to use. This allows cheap models for compaction and expensive models for the worker. The role → model mapping lives in the config's `providers.yaml` (immutable per agent — fork is the freeze, §2.2):

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

Each role's system prompt is read from the governing config commit's `souls/<role>.md` by convention (§2.2) — there is no per-role path override, and no freeform path field to validate. At dispatch time the harness copies the appropriate soul to the new branch's `soul.md` (§2.3 step 2). `provider:` names a brazen row; endpoint and auth resolve at call time inside brazen (§4.1) — the config file carries only the (row-name, model-id) pointer, cross-validated against `<config-root>/models.yaml` at load.

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

**On-disk response shape: appended attempt segments.** The harness appends each attempt's stdout verbatim to the step's `response.json`. brazen guarantees every stream — success, refusal, or failure — ends with exactly one `{"type":"end"}` line, so the file is a sequence of self-delimiting **segments**, one per attempt. **Segment authority:** an `Error`-terminated segment is audit-only — the record of a failed attempt, contributing nothing; a `Pause`-terminated segment *contributes* its content, which the continuation resumes rather than replaces (§2.10 Resumable pauses — the provider's own contract: paused content is re-sent and continued); the authoritative content of the model call is the accumulation across contributing segments, sealed at the final `Finish`. With no server tools wired (§3.3) no v1 segment can pause, so the rule degenerates to last-segment-authoritative — but it is stated in the general form because the transcript writer (§2.3) acts on it mechanically. Reading rules (shared by §3.5 classification, metering (§6, §8), and replay tooling):

- *in flight* — the file is open (no `IN_CLOSE_WRITE` yet).
- *complete* — closed, last line `end`, last segment carries a `Finish` (any reason, including refusal) and no `Error`.
- *failed* — closed, last line `end`, last segment carries an `Error` event (retry budget exhausted or non-retryable); the branch is flagged per §2.10.
- *stopped/killed* — closed **without** a trailing `end` line: the writer died mid-stream (§2.9). Kill, crash, and explicit stop remain indistinguishable on disk and are treated identically.

**Fd held open for the whole model call.** The harness opens `response.json` once, at the model call's first attempt, and holds the fd open across *every* attempt and *every* backoff sleep between them (§2.10) — closing it only at step resolution, when the loop settles into `complete`, `failed`, or `stopped`. This makes fd-open the single `in_flight` signal and the four reading rules above **terminal-only**: they are evaluated once the fd is closed. While it is open the step is `in_flight` no matter how many `end`-terminated segments it already carries, so an intermediate failed attempt's trailing `end` never reads as `complete`, and a mid-retry `Error` segment never reads as `failed` — the retry is still pending and the file is still open. This is the same fd-open observation the `/proc/<pid>/fd` writer scan reads in §2.9 and `lernie stop` (§3.4), and the same signal §3.5 classifies on: one open fd, three readers.

**Refusal is a completed model call, not a failure.** brazen surfaces a provider refusal as `Finish{Refusal}` on HTTP 200, exit 0 (brazen `architecture.md` §3.2) — its own truth, never an `Error` — so a refusal segment classifies *complete* (above), with no retry and no operator flag. A root agent surfaces the refusal as its terminal response, exactly like any other terminal `Finish`. A child's refusal reaches its parent through the ordinary path: it is the terminal response its result message carries (§2.6) — no distinct status, because a refusal is a normal completion the parent agent reasons about, not a harness fault. The workflow layer needs no refusal-specific event; the ordinary completion bindings (`worker_return` etc., §6) fire, and any refusal policy is a binding on that existing signal, not a new one.

**A `Finish{Pause}` segment is never terminal.** A paused model call (brazen `FinishReason::Pause`, Anthropic `pause_turn`) is continued *within the same step* (§2.10 Resumable pauses), so the fd stays open across the continuation and the segment always reads *in flight* by the rule above — the classifier never sees `Pause` as a settled state. Pause arises only with provider server tools, which lernie's stdio tool contract (§3.3) does not wire, so it cannot occur in v1; the rule is stated so the classification is total.

**Errors.** brazen surfaces every failure in-band as an `Error` event on stdout and also sets a sysexits exit code computed from the same fact. The event is authoritative: the harness classifies from the parsed `CanonicalError` and treats the exit code as diagnostic. A `bz` process that dies without emitting a trailing `end` is the kill signature above — handled by §2.9/§2.10, never delivered to the model.

**Cancellation.** brazen installs no signal handlers: SIGTERM kills `bz` at once (default disposition, exit 143), dropping the in-flight HTTP request; already-flushed NDJSON lines stay valid. The missing trailing `end` on the closed fd *is* the stop signature — no flush deadline, no cancel marker, determinism via absence of mechanism. (The 5-second SIGTERM→SIGKILL deadline remains the contract for *tools*, §3.3, which may need to flush real work.)

**Auth and endpoints.** Entirely brazen's: provider rows carry auth mode and endpoint; credentials live in brazen's 0600 credstore; interactive flows are `bz --login`, operator-run, never harness-run — the harness never prompts and never sees credential material. The v0.3 `auth_env`/`endpoint_env` forwarding machinery is retired with the rest of the bespoke contract.

**Fit with disk-as-bus (§3.1).** Unchanged in shape: the adapter's stdin and stdout are pipes; the harness mirrors the assembled canonical request to the step's diagnostic `request.json` (one per step — attempts share the same assembled request) and appends events to `response.json`. Both remain outside every worktree and outside context assembly (§2.3); the terminal-line classification above reads only event framing from `response.json`'s tail — the same observation the frontend makes (§3.5). Because the pinned `CanonicalRequest` sets no `skip_serializing_if`, unset options serialize as explicit `null`, so the mirrored `request.json` shows `"stream":null`, `"temperature":null`, and the like — this is consistent with "lernie never overrides `stream`" (Invocation): `null` *is* the non-override, and brazen's `fill_absent` resolves it to the configured default. The pipes are the wire; the disk is the record.

**Extensibility.** A new provider on a supported protocol is a brazen config row — no code anywhere. A new wire protocol or auth mode is a contribution to brazen. The escape hatch for a deployment that cannot ship through brazen is the `adapter:` override in `models.yaml` (§4.2): any binary honoring the same pipe contract — canonical request in, `v=1` events out, one `end` — slots in verbatim, with the in-band `MessageStart.v` handshake as its compatibility gate.

---

## 5. Context Assembly

### 5.1 The worktree invariant

**Everything inside a branch's worktree is composed into that agent's prompt.** This is the load-bearing invariant of context assembly. There is no exclusion list, no filter, no "this path is for the harness only." Data lives in the worktree; control lives in the governing config commit, whose harness-facing files the dispatch commit removes from the agent's tree (§2.2, §2.3 step 2). The invariant is what lets the manifest be a *sequencing and budget* file rather than an *inclusion* file — the question the manifest answers is "in what order and under what budget," not "which things."

Consequences:

- Agents curate their own context by `rm` (§5.3). The primitive they need already exists in the filesystem.
- The compactor's `mark_for_deletion` operates on worktree paths and takes effect on the next assembly.
- Control files (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `souls/`, `version`) are read from the immutable config commit (§2.2) and are absent from every agent worktree — never composed. This is structural, not disciplinary: the path is not under the worktree root, so it cannot be included. (`steps/` and `inbox/` are excluded the same way, by living at the workspace root, §2.2.)

The invariant is total now that the transcript lives in the worktree (§2.3): the agent's own model-facing history — assistant output, tool results, delivered messages — is worktree data like everything else. Nothing context-bearing lives anywhere but the branch's tree, so assembly is a pure function of the read-state commit (§2.10) with no second input — the manifest itself resolves from that commit's derived config ancestor (§2.2), so even control needs no separate input. Assembling a running step and replaying an old one are the same operation. `docs/PRINCIPLES.md` Disk-first stops having an exception to explain.

### 5.2 Assembly rules

The manifest (`manifest.yaml` in the governing config commit, §2.2; role-keyed) declares ordering, pinning, budget, and overflow policy:

```yaml
roles:
  worker:
    pinned:
      - goal.md
      - soul.md
      - descriptions/**
    order:
      - summary/**
      - skills/**
    # The transcript (messages/**) is not an order category: it always
    # assembles last, in sequence order (§2.3, §5.5).
    budget_tokens: 150000
    overflow: drop_oldest_summaries
  compactor:
    pinned:
      - goal.md
      - soul.md
    # The compactor's view onto the dispatching branch's work is its
    # inherited worktree: forked off the checkpoint commit, it carries
    # that branch's transcript, summaries, and work products (§2.3,
    # §2.6, §2.7). The compactor's manifest sees only its own worktree,
    # same as any other role.
    budget_tokens: 50000
    overflow: truncate
```

Paths are interpreted relative to the branch's worktree. The manifest sees only worktree contents by construction (§5.1). Pinned paths are always included regardless of budget; `order` entries fill the body in declared order until overflow policy kicks in. The transcript (§2.3) is not an `order` category: it always assembles after the body, in sequence order, so the raw recent history sits nearest the next model call and the assembled prompt stays append-only between compactions (§5.5).

**Step records are not context.** `worker.order` (and any other role's manifest entries) MUST NOT reference `steps/**` — step records live at `<workspace>/steps/<agent-id>/NNN/`, outside every worktree (§2.2, §2.3), and are diagnostic-only (§2.3 Diagnostic-only contract). They are physically excluded from context assembly by their location: a worktree-relative path cannot resolve to them. Examples in this doc and the shipped template manifest reflect that constraint.

This corresponds to the LangChain write/select/compress/isolate taxonomy (`docs/TAXONOMY.md` §3): **write** = commits; **select** = manifest inclusion; **compress** (here: compact) = compactor; **isolate** = child-agent branches.

### 5.3 File path as hint

File paths are preserved in the assembled context as structural hints to the model. The path itself carries information (`summary/003.md`, `skills/git-ops/SKILL.md`, `descriptions/tools/bash.json`) that is cheaper than explicit metadata and often sufficient.

### 5.4 Removal by deletion

Agents reduce their own context by deleting files from the worktree — work products and transcript entries alike (§2.3). An agent that no longer needs a 5k-token file `rm`s it; the next context assembly excludes it naturally (per §5.1 — if it's not in the worktree, it's not in the prompt). Deleted files remain recoverable from git history until compaction squashes them — the squash is also what keeps a transcript-bearing repo's history from bloating under large committed tool outputs. Deletion is priced: removing a file invalidates the provider's cached prefix from that entry's position onward (§5.5) — legal, paid for at the wire, and the reason bulk curation belongs to compaction.

### 5.5 Append-only assembly and the prompt cache

Provider prompt caching is prefix-based (`docs/TAXONOMY.md` §8): a request whose leading bytes match a previous request's re-reads the cached prefix at a fraction of the cost. The assembly rules are shaped so that, between compactions, each step's request is byte-prefixed by the last. The assembled context has three parts, in order:

1. **Pinned head, frozen at dispatch.** `goal.md`, `soul.md`, `descriptions/**` — the manifest's `pinned` paths — are written or inherited at the dispatch commit (§2.3 step 2) and do not change on the branch. The head of every request on the branch is identical.
2. **Body, in manifest category order.** The non-transcript context — `summary/**`, `skills/**` — assembles in the manifest's declared `order` (§5.2). The body changes only at **rebuild points**: dispatch (the branch's first assembly) and the compaction merge (§2.7), the sole sanctioned reorganization. Between rebuild points the body is stable. (A mid-run skill load, §3.3, inserts into the body and pays the flush from its position — rare, heavy, and priced.)
3. **Transcript tail, in sequence order.** After the body, the transcript (§2.3) in filename-sequence order — *not* manifest category order. Every ordinary step event — assistant output, tool results, delivered messages — is an append at the tail, so the previous request is a byte prefix of the next and the provider's cache hits.

Deletion (an agent's `rm`, §5.4; the compactor's `mark_for_deletion`, §2.7) truncates the cached prefix from the deleted entry's position; edit-in-place is banned outright (§2.3). Compaction is where wholesale reorganization happens and the cache is deliberately flushed — it is a merge operation (§2.7): the compactor's merge back into the dispatching branch deletes superseded transcript entries and lands the new summary, and the next assembly rebuilds from the new tree. One flush, planned, at the moment the context is worth re-paying for.

There is no history walk in any of this: the assembled order is a pure function of the read-state tree — pinned paths, category-sorted body, sequence-sorted tail — so the same readdir+sort produces the same request during execution, after a crash, and at replay (§2.3, §2.10).

---

## 6. Workflow as Configuration

Workflows are declared as event-to-action bindings in the config's `workflow.yaml` (§2.2), authored from a named workflow template at `<config-root>/workflows/<name>.yaml`. An agent's workflow is fixed for its life — it is read from the immutable governing config commit (§2.2) — so changing policy is a config commit plus a fresh fork, never an edit under a running agent. "Workflow" here is the Anthropic sense (`docs/TAXONOMY.md` §1): predetermined code paths, as contrasted with LLM-driven agent control flow. Subordinate routines invoked by the workflow (compaction, verification, auto-dispatch on large tool output) are **procedures**.

```yaml
events:
  user_message:
    - spawn_root_agent
    - dispatch(worker)
  worker_return:
    - dispatch(verifier)
    - gate_return_on(verifier.approve)
  verifier_approve:
    - deliver_result            # result message + work-product transfer (§2.6)
  verifier_reject:
    - "dispatch(worker, with: verifier.feedback)"
  worker_flush:
    - dispatch(compactor)
  compactor_return:
    - compaction_merge          # the one merge (§2.6)
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

Actions are implemented in the harness; the workflow declares which run when. The `flush` action emitted by a running agent triggers a compaction checkpoint without terminating the branch (§2.6, §2.7). This is the primary surface for experimentation.

Per-step hooks (`pre_step`, `post_step`, `on_tool_return`) fire on every branch and are the primary extension points for cross-cutting behavior — observability, budget enforcement, cache maintenance, scheduled compaction-checkpoint triggers. Their handlers typically dispatch children or emit log entries rather than modifying the in-flight branch's tree directly; any write still goes through the harness-assigned write-path machinery (§2.5).

**Budgets (v0.7).** `budgets:` declares per-agent-tree spend limits: `max_total_tokens`, `max_wall_seconds`, `max_depth`. The harness checks them at every model-call boundary, before invoking the adapter. Spend is derived at check time from the `Usage` events already on disk in `response.json` across the agent tree — no running counter is stored (`docs/PRINCIPLES.md` "Single source of truth"). **Every attempt segment counts:** the derivation sums `Usage` across *all* segments of every step, not only the authoritative last one, because a failed or superseded attempt still consumed provider tokens and real money — the last-segment-authoritative rule (§4.4) governs which segment supplies *context*, never which segments are *billed* (§8). `max_wall_seconds` is likewise wall-clock: it counts the backoff sleeps between attempts (§2.10), since wall time elapses whether the harness is streaming or sleeping — derived by summing each step's `started_at`→`ended_at` span (one span already covers that step's attempts and backoff, §4.4 "Fd held open for the whole model call"). Tokens and wall are **whole-tree** consumables: they derive over the *root* agent and its entire descent (`steps/<root>/` plus every `steps/<root>-*/`, the §2.9 stop-cascade walk over the one shared `steps/` tree, §2.2/§2.3) — so any driver, root or child, reads the same whole-tree total by deriving against the root id (the agent id's root prefix, §2.3). They exhaust at `derived ≥ limit` (stopping before the next model call overspends). `max_depth` is instead the *driving branch's own* hyphenated dispatch depth (§2.3 — root = 0, each dispatch one deeper) and is the deepest *allowed* depth, so an agent exhausts only when its depth *exceeds* it — the root (depth 0) is never depth-exhausted. There is **no per-dispatch inheritance**: because `steps/` is one shared tree written live by the whole agent tree and never merged (§2.2/§2.3), a child reads the same whole-tree total its parent would, so nothing is handed down or snapshotted at dispatch and no parent-minus-child quantity is ever computed. (A future per-subtree cap — a `--token-cap`-style knob checked against a subtree's own spend — would sit *beside* this whole-tree ceiling, not inherit through it; it is not built.) Exhaustion is a harness-issued stop (§2.9) plus a `refs/lernie/budget-exhausted/<branch>` ref — the same git-native marking pattern as the declined-transfer ref (§2.6). Because exhaustion is voluntary and harness-issued, the executor deposits the branch's result message with a `budget-exhausted` epitaph (§2.6, §2.3 step 5) before exiting — the parent is revived and told how the child ended, on the same channel as any return. Exhaustion is an ordinary terminal state, never a special transcript shape.

**No resident interpreter.** Nothing parses `workflow.yaml` resident-style and drives the state machine from memory. The chain is driven by a single CLI subcommand, **`lernie advance <repo>`**: a fresh subprocess that reads the repo, evaluates workflow rules against on-disk state to determine the next workflow action, runs that action's procedures by exec'ing them per §3.4 (`lernie dispatch <role>`, `lernie merge`, etc.), and before exiting exec's another `lernie advance` to continue the chain. The currently-executing `lernie advance` subprocess *is* the interpreter while it runs — a baton passed forward by exec, not a daemon. The chain ends at a terminal action (final merge, stop, error); there is no watcher noticing completions, each step hands off by exec. Combined with disk-as-bus (§3.1), this keeps the system stateless across process boundaries: a crashed subprocess leaves nothing in memory to reconstruct, and the same `lernie advance` invocation re-enters the chain by reading the repo and running the next action — recovery is not a separate operation, it is the same operation as advancing. This is the concrete mechanism behind §1's Regenerability property.

**Workflow position is derivable from disk.** The workflow's current position is a function of repo state — refs, step records, merge history, sentinel files explicitly named in `workflow.yaml` — never of a `workflow_state.json` or analogous sidecar that mirrors the position. Implementations may not introduce such a mirror. This is the invariant that makes the unification of "advance" and "resume" load-bearing rather than aspirational: if position were stored separately from disk state, then re-entry after crash would require reconciling the mirror against disk, and "advance" and "resume" would need to be different operations to handle the divergence. Holding the line on derivability collapses them.

**Procedures are idempotent under replay.** §2.10 commits to replay-from-recorded-sha for model calls. The corresponding requirement at the workflow level is that every procedure invoked by `lernie advance` must be idempotent under replay from its recorded read-state, or detect already-completed work and no-op. Without this, two `lernie advance` invocations against the same disk state could produce divergent effects, and the position-from-disk invariant above would not be sufficient to make recovery safe. With it, any number of `lernie advance` invocations against the same state converge. Idempotence covers *sequential* re-entry; *simultaneous* drivers against one branch are resolved by the executor lock (§2.11 "Writer/driver totality") — one becomes the executor, the other exits as a clean no-op.

---

## 7. UI

### 7.1 Live view

The UI watches the workspace filesystem (§3.5) and re-renders from what it observes. Top of the interface shows the workspace's git tree: config branches and the agent tree forked from them. Clicking a commit navigates to that point. Clicking a branch navigates to that agent.

Live indicators:

- Line-by-line streaming text for model calls in flight.
- Pulsing indicators for tool calls.
- Arrows to child branches for active children.
- Distinct patterns for model-call states: queued, in flight, streaming, terminal.
- Agent-state markers — live, quiescent, stopped (§3.5) — with declined-transfer and budget-exhausted ref marks rendered alongside (§2.6, §6).
- Pending-message indicators for agents with a non-empty inbox (§2.11); delivered messages appear as ordinary delivery commits in the tree, and a child's return appears as its result message plus, when files came back, the work-product transfer commit (§2.6).

An exchange (§2.4) is a rendering span, not a structure: the UI groups a user message with the steps and terminal response that answer it on the root agent's linear history. The grouping is presentational; nothing structural corresponds to it.

### 7.2 History view

Clicking an old commit is read-only by default. Forking from history — starting a new agent off an old commit to explore a counterfactual — is the ordinary fork with a historical ref argument (§2.3): no special branch prefix, no distinct operation. Provenance is the ancestry — the new branch's fork point is visible in the graph, which is what accounting and replay read — so the UI distinguishes forked-from-history agents visually, never structurally.

### 7.3 Concurrent exchanges

Root agents multiply freely: the user may open a second piece of work while the first runs, or speak twice to the same one. Two intents, two existing primitives (§2.4): a *new question* is a new root agent — `lernie prompt`, its own branch forked off a config branch's head (§2.3), running independently; a *course correction or follow-up* is `lernie message` into the existing agent's inbox, delivered at its next step boundary — and if that agent is quiescent, the deposit itself starts the driver (§2.11). Steering a running exchange and reprompting a finished one are the same verb. No mechanism is required beyond the branch invariants (§2.3) and the inbox.

---

## 8. Metrics and Observability

First-class metrics, written to commit trailers and event log. All counts are reported along three scope axes, which are not interchangeable:

- Tokens per step, per agent, per workspace — summed across *every* attempt segment of each step's `response.json`, not only the contributing ones (§4.4 segment authority); failed and superseded attempts are billed (§4.4, §6 budgets).
- Model calls per step (always 1), tool calls per step, attempts per model call (≡ API calls, §2.1; derived as the segment count of the step's `response.json`, §4.4 — never stored).
- Cost per step, per agent, per workspace (derived from the all-segments token sum above).
- Silent-death sweep and count per workspace. This is a critical health metric: a ballooning count indicates failure somewhere in the return pipeline. With merge-back deleted (§2.6), branch count itself is not a health axis — every agent branch persists by design until GC (§9.2) — so the signals are liveness and return: candidates are enumerated by `git branch --list 'agents/*'`, filtered to branches with *no live executor* (the §2.11 lock probe) that either died mid-work (latest `response.json` closed without a terminal `end`, §2.9) or, for a child, never deposited a result message (§2.6). A long-lived child still working (legitimate per §2.5, §2.11) holds its lock and is never counted; a quiescent root agent awaiting its next message (§2.4) has a terminal `end` and is never counted. The sweep is not only a counter: for each **child** in the candidate set that never deposited — a hard crash (SIGKILL, OOM, panic) that ran no handler, so neither the stop path (§2.9) nor the budget path (§6) deposited for it — it **deposits on the child's behalf**, a result message with a `died` epitaph (§2.6), so the return path is total even across uncatchable death and the parent is revived rather than stalled. Its trigger is the startup scan (§2.11): every driver invocation sweeps before it steps — a transient process, no watcher, no schedule. The deposit's sender is the **child**, not the harness: the parent's inbox is sender-namespaced (§2.11) and the "a message from the child exists" derivation depends on it, so the sweep writes as the child — the sweep is the scribe, not the author. No watcher process is added; §2.11 forbids one and none is needed, because stop is SIGTERM-catchable and budget exhaustion is voluntary (those deposit through the executor, §2.9, §6), so only hard crash reaches the sweep, and the sweep already enumerates exactly that candidate set. Same query, one added action, zero new mechanism.
- Undelivered-message count per workspace: files under `inbox/<agent-id>/` (§2.11) whose agent has no live executor (the §2.11 lock probe). Derived from the inbox listing plus the liveness observation — no flag, no sidecar (§2.11 "Undelivered is derived"); transient under normal operation, since a deposit into a quiescent agent starts a driver (§2.11).
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

### 9.2 Replay and archival

A workspace is long-lived; "a run" is an agent subtree within it. The archival unit follows the agent, not the workspace (the v0.3 tarball-per-run died with the repo-per-root layout — there is no per-run directory to tar):

- **Archive.** One `git bundle` of the agent's branch and its descendants' branches (`agents/<id>`, `agents/<id>-*`). A full bundle is self-contained: it carries the complete ancestry back through the dispatch commits to the governing config commit (§2.2), so the config that produced the run travels inside the archive. Alongside the bundle ride the matching diagnostic slices — `steps/<id>*/` and `inbox/<id>*/` (plain directories outside git, §2.2). One bundle plus two slices is the whole run.
- **Replay.** Clone the bundle into a scratch workspace (`LERNIE_HOME` keeps it isolated, §2.2) and restore the slices. Every §2.10 guarantee then holds verbatim: each step's read state is a commit in the bundle, assembly re-derives the wire input from it, and `response.json` framing classifies the outcome. Inspection is the ordinary frontend over the scratch workspace (§3.5); replay is not a mode (§2.3).
- **Retention and GC.** Deleting an expired or archived agent is deleting its branches; `git gc` prunes whatever objects nothing else reaches. Reachability *is* the retention policy: an ancestor shared with a config branch or a still-live sibling survives automatically, and nothing needs to compute what is safe to drop. The `steps/` and `inbox/` slices are plain directories, removed with the branches. Default retention 30 days (§2.9), then bundle-and-delete or delete outright, per workspace policy.

A whole-workspace snapshot stays trivially available — `tar` the workspace directory (repo, steps, inboxes) — and remains the right unit for machine migration. It is not the archival unit, because a workspace never ends; agents do.

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

Every config declares its schema version in its `version` file (a control file in the config commit, §2.2), and every agent inherits its config's version through ancestry. Old versions remain readable by the harness; migration code is written when a version bumps. Archived bundles and snapshots from any prior version (§9.2) must remain inspectable.

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

**Success criterion:** An exchange is a branch, completion is a no-ff merge back to `main`, the repo layout matches the then-current §2.2 (the pre-substrate layout; both the `main`-rooted history and that layout are retired, §2.2/§2.3 Historical). User message → exchange branch → steps as linear commits → compactor (deletion-only, stub is fine) → merge. Unmerged branch count metric available.

**Shipped shape.** `lernie prompt <repo> <message>` spawns `ex/<ts>-<short-id>` off `main` in a dedicated worktree under `.lernie/worktrees/`, writes `.agent/goal.md` (§2.8) and `exchanges/<ts>-<short-id>/steps/001/request.json`, commits that snapshot before the model call (§2.10), invokes `lernie-provider-<name> complete`, lands the normalized response at `steps/001/response.json` as a follow-up commit, then dispatches the terminal compactor by re-entering itself as `lernie dispatch compactor <repo> <exchange-branch>` (subprocess per §3.4). The compactor spawns `inv/<ex-id>/<cmp-id>` off the exchange tip, writes its boilerplate `.agent/goal.md` and commits it as the dispatch snapshot (§2.8), then writes `.agent/compactions/001.md` and commits it as the terminal-summary follow-up (stub — no model call; `mark_for_deletion` is a no-op), and `--no-ff` merges back into the exchange branch. Control returns to `lernie prompt`, which rebases the exchange onto the current `main` tip and `--no-ff` merges it into `main` (§2.6). The unmerged branch count metric is read directly from `git branch --list 'ex/*' 'inv/*' --no-merged main` — no sidecar file.

### v0.3 — Tools

**Success criterion:** Agent can invoke at least two tools (`bash`, `read_file`). Worktree-modifying tool calls land as commits on the emitting branch (§2.3, §3.3); read-only tool calls produce no commit. Tool contract — binary (`lernie tool <name>` in-process or `lernie-tool-<name>` external, mirroring §4.4 adapter discovery), JSON schema, `SKILL.md`, the stdio/exit-code shape, and the per-call disk record (`input.json`, `output.json` under `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/`, outside every worktree) — pinned in §3.3. Oversized-output auto-dispatch deferred to v0.4+ (§11). Conversation-repo layout migrated from the v0.2 `.agent/`-rooted shape to the v0.3 layout described in §2.2 (control + step records at the conversation-repo root, worktrees as siblings, steps namespaced by conversation id, `merge=ours` on goal/soul/summary, "invocation" retired as a structural term). v0.3.1 follow-on tightens this with the diagnostic-only contract on `request.json` / `response.json` and relocates the step records out of every worktree (§2.3); it also drives `complete` with `stream: true` end-to-end and writes `response.json` as JSONL of §4.4 stream events tail-appended event-by-event (§4.4 "On-disk response shape: JSONL of stream events, always."), with writer-closes-fd as the §3.5 completion signal.

### v0.4 — Subagent dispatch

**Success criterion:** Agent can dispatch a subagent conversation. The subagent runs in its own worktree and branch. Merge-back flow works end-to-end (including the `merge=ours` discipline on `goal.md`/`soul.md`/`summary/**`). Parallel subagent conversations do not corrupt each other's state. Handle-based async works: a dispatch returns immediately with a handle, siblings can be in flight concurrently, and `await(handle)` retrieves the compacted result on a later step (§2.5).

> **Historical.** v0.4's merge-back flow shipped as specced above and is retired by the workspace substrate: children now return results as messages plus a work-product transfer, and the `merge=ours` machinery deletes (§2.6). This criterion records what v0.4 delivered, not the current contract; unwinding the shipped merge path is implementation work tracked separately (§2.6 Shipped-state note).

Phase 2 lands the `dispatch` built-in tool — `lernie tool dispatch`, input `{role, goal}` — that realizes the §2.5 dispatch primitive on the model-facing side: it reads conversation context from the `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` env vars (§3.3), spawns the subagent through Phase 1's `lernie dispatch <role>` CLI (§3.4), and returns `{status: in_progress, handle}` synchronously. The subagent's own step loop is a subsequent phase.

Phase 3 lands the `await` built-in tool — `lernie tool await`, input `{handle}` — the resolution half of the §2.5 dispatch/await pair. As shipped it blocks until the named subagent reaches a terminal state and emits exactly one of `{status: merged, summary}`, `{status: stopped}`, or `{status: conflicted}` (the merge-era statuses; the substrate dissolves `await` entirely — the child's result arrives as an inbox deposit carrying an epitaph, not a polled status, §2.5, §2.6). Every state read is git-or-fs (`docs/PRINCIPLES.md` "Single source of truth"): merged via `merge-base(handle, parent) == HEAD(handle)`, conflicted via `refs/lernie/conflicted/<handle>` (written on rebase failure by the retired merge protocol; the ref name survives as the declined-transfer mark, §2.6), stopped via either of two on-disk signatures — the latest step's `response.json` ending in a §4.4 `error` event (clean failure), or `response.json` having bytes but no terminal `message_stop`/`error` line AND no process holding the fd open (kill-mid-stream §2.9). The kill-mid-stream check reuses the `/proc/<pid>/fd/*` writer scan from §2.9 / `lernie stop` (line 267 — same observation that drives §3.5 `in_flight` classification, single source of truth). Linux-only on the kill arm; the `error` arm is portable. The v0.4 P3 ball shipped only the `error` signature; the kill arm landed in the follow-on (bl-c9ec).

### v0.5 — UI

**Success criterion:** Git tree view, live streaming, pulsing tool indicators, branch-state indicators. Watching the repo filesystem (§3.5) is the only read mechanism; user actions go out as `lernie <subcommand>` invocations.

### v0.6 — brazen provider layer

**Success criterion:** Every model call flows through `bz` (§4.4): the context assembler emits a typed brazen canonical request; `response.json` is JSONL of canonical `v=1` events in attempt segments; the harness owns retry (a forced-retry case produces two segments and correct classification); `await` and the UI classify from the `end`-terminal vocabulary; at least two providers (anthropic plus one other) work via brazen config rows with zero lernie code difference; `crates/lernie-provider-anthropic`, the `describe`/`complete` contract, and `auth_env`/`endpoint_env` forwarding are deleted; the load-time version-skew guard works. The compactor is still the v0.3 no-model-call stub (§2.7), so in v0.6 only the **worker's** model call exercises the `bz`/retry path end-to-end; the compactor's own model call is pending a later milestone and will traverse the identical §4.4 contract when it lands.

This milestone folds in the `harness` repo design exploration (2026-07). Its keeper is the adapter boundary — brazen owns the one thing the harness must never know: which provider or model is on the wire. Its other keepers land as budgets (v0.7) and sandboxed tools (v1.1). Its transcript-as-JSONL reframe was rejected: lernie's git substrate is the single source of truth and strictly richer (refs, worktrees, merges) than an append-only transcript file.

### v0.7 — Workflow config and budgets

**Success criterion:** The `workflow.yaml` surface works. At least one non-baseline workflow variant (e.g., a verifier step) runs end-to-end without code changes. Budgets (§6) enforce at the model-call boundary: a conversation tree that exhausts `max_total_tokens` stops with a `refs/lernie/budget-exhausted/<branch>` ref and the exhausted agent deposits a result message with a `budget-exhausted` epitaph (§2.6); the limit is a whole-tree ceiling checked live by any driver, with no per-dispatch inheritance.

### v0.8 — Front-door messaging

**Success criterion:** Any sender can message any agent (§2.11). A message deposited by the user into a running root agent's inbox lands as a delivery commit at the next step boundary and visibly steers the exchange; a child messages its still-running parent (the reminder shape) and the parent's next step sees it; a deposit into a quiescent agent starts a driver and the step loop resumes on the same branch (§2.11 — the reprompt path, §2.4); two simultaneous drivers on one branch resolve through the executor lock — the loser exits as a clean no-op (§2.11 "Writer/driver totality"), and killing the winner releases the lock with no cleanup step. A child's delivered messages stay on its branch: no path carries them into the parent's tree (§2.6). The undelivered-message health metric (§8) reads from `inbox/` listings and the lock probe only — no sidecar state anywhere in the milestone.

### v0.9 — Task suite

**Success criterion:** 50 tasks with machine-checkable success criteria. Baseline harness achieves 40% ± 5% pass@1 on the suite (Wilson CI). Per-category failure tagging works.

### v0.10 — Experiments and replay

**Success criterion:** `agent-eval --config <experiment> --suite <suite> --runs N` produces per-task pass@1 and pass@5 with confidence intervals. Any run (agent subtree) can be bundled and replayed (§9.2). Config changes (prompt edits) deployable without code changes in under 60 seconds end-to-end.

### v1.0

**Success criterion:** All of the above, plus at least one demonstrated workflow variant that beats baseline on at least one failure category by a statistically significant margin on pass@1. This is the proof that the architecture's experimentation surface is actually useful.

### v1.1 — Sandboxed tools

**Success criterion:** A tool compiled to `wasm32-wasip2` runs under a wasmtime host with WASI clamped to the intersection of the tool's declared capability manifest and the role's grant (fs scopes, net hosts, exec, clock, env). The §3.3 stdio contract is unchanged — the sandbox is a hosting decision derived from the artifact kind (a WASM component vs a native executable), never a config field. A native binary requires the `exec` grant; the default grant set is empty; a tool asking beyond its grant fails at load, loudly, before any model call. The capability grammar must be small enough to audit at a glance — if a grant needs a comment, the grammar failed. Full spec: §3.6 (grammar, artifact-kind derivation, manifest-intersect-grant, boundaries). (From the `harness` repo's spec §6 — its strongest differentiator, preserved.)
