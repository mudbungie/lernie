# lernie

[![CI](https://github.com/mudbungie/lernie/actions/workflows/ci.yml/badge.svg)](https://github.com/mudbungie/lernie/actions/workflows/ci.yml)

A git-backed agent harness. Design spec: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
Principles catalog: [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md).
Vocabulary reference: [`docs/TAXONOMY.md`](docs/TAXONOMY.md).

CI runs `make ci` (`fmt-check` + `lint` + `coverage` with the 100% gate) on every push and pull request to `main`.

## Quickstart

```
make install              # lay down the XDG harness homes + binaries on PATH
lernie new ~/work/chat    # scaffold a conversation repo
ANTHROPIC_API_KEY=... lernie prompt ~/work/chat 'hello'
```

## Install

```
make install                                  # default: ~/.local/bin, XDG homes
make install INSTALL_PREFIX=/usr/local        # binaries -> /usr/local/bin/
make install LERNIE_HOME=/opt/lernie          # collapse both homes -> /opt/lernie/
```

`make install` runs a release build and then:

1. Lays down the harness root skeleton, split by XDG lifetime (ARCH
   §2.2): the **config root** (`$XDG_CONFIG_HOME/lernie`, default
   `~/.config/lernie`) gets `models.yaml` and `workflows/`; the **data
   root** (`$XDG_DATA_HOME/lernie`, default `~/.local/share/lernie`)
   gets the `tools/`, `skills/`, `agents/`, and `conversations/` pools.
   `LERNIE_HOME`, if set, collapses both roots to that one directory.
   Re-runs are idempotent: directory creation uses `mkdir -p`; existing
   config files are kept.
2. Installs `lernie` and `lernie-ui-egui` into `$INSTALL_PREFIX/bin`
   with `install -m 0755` (atomic overwrite, no symlinks). Make sure
   that directory is on your `PATH`.
3. Installs the provider adapter — brazen's `bz` — with
   `cargo install brazen --version =0.0.2 --locked` (the `BRAZEN_PIN`
   in the Makefile, kept in lockstep with the `brazen = "=0.0.2"`
   dependency in `Cargo.toml`). One binary serves every provider
   (ARCH §4.4); the harness resolves `bz` on `PATH`, and a load-time
   guard rejects any `bz` whose version differs from the pin.
4. Drops a default `models.yaml` under the config root (model
   capabilities + context windows — no endpoints or auth, which are
   brazen's) and an `agents/default/` skeleton under the data root,
   copied from [`template/`](template/) — only when those paths don't
   already exist, so hand-edited config survives a re-install.
5. Smoke-tests the freshly installed binaries with `lernie --version`
   and a throwaway `lernie new`. Failure aborts the install with a
   non-zero exit.

Provider endpoints, auth, and wire dialects live entirely in brazen's
own config (`~/.config/brazen/config.toml`; inspect with
`bz --dump-config`, authenticate with `bz --login --provider <id>`).
lernie references a provider *row* by name and never sees credential
material (ARCH §4.1).

`make uninstall` removes the `lernie`/`lernie-ui-egui` binaries; `bz`
(installed via cargo) is removed with `cargo uninstall brazen`. The
harness homes (the config and data roots, holding config and
conversations) stay put — clean them up manually if you want a true
uninstall.

## Configuration schemas

JSON Schemas for the harness-root and conversation-repo config files (per
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) §2.2, §4.1) are generated
from the Rust types under `src/config/`. `make schemas` writes them to
`schemas/` for editor integration and external validators:

| File                          | Backed by Rust type                        | On-disk file                          |
|-------------------------------|--------------------------------------------|---------------------------------------|
| `schemas/version.json`        | `config::version::Version`                 | `<conv-repo>/version`                 |
| `schemas/manifest.json`       | `config::manifest::Manifest`               | `<conv-repo>/manifest.yaml`           |
| `schemas/workflow.json`       | `config::workflow::Workflow`               | `<conv-repo>/workflow.yaml`           |
| `schemas/providers.json`      | `config::per_repo_providers::PerRepoProviders` | `<conv-repo>/providers.yaml` (`roles:`) |
| `schemas/models.json`         | `config::models::Models`                   | `<config-root>/models.yaml`           |

## Layout: harness root and conversation repos

The harness root is installation-global state, split by XDG lifetime
into two homes (ARCH §2.2). `LERNIE_HOME`, if set and non-empty,
collapses both to that one directory (test isolation, alternate
installs). Three distinct on-disk locations:

- **Config root** — hand-edited declarations, `$XDG_CONFIG_HOME/lernie`
  (default `~/.config/lernie`). Holds the global
  [`models.yaml`](docs/ARCHITECTURE.md#42-model-abstraction) (model
  capabilities + context windows, plus an optional `adapter:` binary
  override — §4.2) and the `workflows/` templates. Provider endpoints
  and auth live in brazen's config, not here (§4.1).
- **Data root** — machine-populated pools, `$XDG_DATA_HOME/lernie`
  (default `~/.local/share/lernie`). Holds `tools/`, `skills/`, and
  `agents/<profile>/` per-profile skeletons, plus the `conversations/`
  tree. Shared across every conversation.
- **Conversation repo** — one self-contained git repository per root
  conversation, at `<data-root>/conversations/<root-id>/`. Carries the
  per-repo `providers.yaml` (`roles:` only — §4.3), `manifest.yaml`,
  `workflow.yaml`, `souls/`, and a `root/` worktree where `main` is
  checked out. Subagent conversations are sibling worktrees of `root/`,
  one per `<a>-<b>-…/` directory. Conversation repos are never pushed to
  a remote.

Each conversation repo is scaffolded from [`template/`](template/) — the
versioned skeleton embedded into the `lernie` binary at build time, the
v0.3-shape default profile in the absence of a custom
`<data-root>/agents/<profile>/`. Create one:

```
lernie new                     # auto-id under <data-root>/conversations/
lernie new /path/to/my-conversation
```

Or via the Makefile wrapper:

```
make new-conversation DEST=/path/to/my-conversation
```

The binary embeds `template/` at build time (via `include_dir`), extracts
it to the destination, creates the `root/` worktree subdirectory with
`.gitattributes` pinning the §2.6 `merge=ours` rules, runs `git init -b
main` *inside* `root/`, registers the `merge.ours.driver` config so
those rules are actually honored on hand-run merges (the harness's
own merges enforce the discipline more strictly via the rebase-time
alignment step), and lands a single `init conversation repo` commit. The control-plane files (`manifest.yaml`, `workflow.yaml`,
`providers.yaml`, `version`, `souls/`) sit at the conv-repo root —
outside any worktree — and are deliberately untracked. The destination
must either not exist or be an empty directory. With no path argument,
the destination is `<data-root>/conversations/<auto-id>/`;
the scaffolded path is printed on stdout. `goal.md` and `soul.md` inside
`root/` are intentionally not in the template — they are written at
dispatch time (ARCH §2.3, §2.8).

## Sending a prompt

```
lernie prompt /path/to/my-conversation 'hello'
```

`lernie prompt` is the root-conversation path (ARCH §2.3, §2.6, §2.7,
§2.8, §2.10). Each invocation spawns its own branch off `main`, drives
the model call through brazen's `bz` (§4.4), runs the terminal compactor
off the tip, and `--no-ff` merges the compacted branch back into `main`:

1. Resolve the harness root (`LERNIE_HOME`, else XDG homes, ARCH
   §2.2). Load `<config-root>/models.yaml` (capabilities + context
   windows + optional `adapter:` override — §4.2) and
   `<repo>/providers.yaml` (`roles:` block — §4.3); cross-validate
   `roles.worker.{provider,model}` against `models.yaml`.
2. Run the load-time version guard: `bz --version` must equal the
   linked brazen crate version (§4.4). Under an `adapter:` override the
   guard is skipped and the in-band `MessageStart.v` handshake governs.
   Read the worker soul from `<repo>/souls/worker.md` (§4.3).
3. Spawn branch `<conv-id>` (the bare hyphenated id, §2.3) off `main`
   and allocate a sibling worktree at `<repo>/<conv-id>/` (§2.2). Write
   the branch goal to `goal.md` and the role soul to `soul.md`; commit
   them as the dispatch snapshot — that commit's tree is step 1's read
   state (§2.10).
4. Build a typed `brazen::CanonicalRequest` (linked crate — the
   fail-open `extra` map stays unreachable), mirror it to
   `<conv-repo>/steps/<conv-id>/001/request.json` (a diagnostic
   artifact, outside every worktree, never read at runtime, §2.3).
5. **Model call, harness-owned retry loop (§2.10, §4.4).** Exec
   `bz --json --provider <row>` once per *attempt*, canonical request
   on stdin, appending each attempt's stdout verbatim to
   `<conv-repo>/steps/<conv-id>/<NNN>/response.json` as brazen `v=1`
   NDJSON — one self-delimiting segment per attempt, each ending in a
   terminal `end`. On a retryable in-band `Error`
   (`CanonicalError::retryable()`, never re-derived) the harness
   re-invokes `bz` with the identical request, up to the `workflow.yaml`
   attempt cap with exponential backoff. brazen never retries; auth and
   endpoints are entirely its own. The `response.json` fd is held open
   across every attempt and backoff sleep — its close is the §3.5
   IN_CLOSE_WRITE completion signal. The harness folds the events into
   an in-memory accumulator (assistant text + `tool_use` blocks) for the
   next step; `meta.json` carries `{commit, started_at, ended_at}`.
   In the same pass a second sink — the **transcript writer** (§2.3) —
   streams content into a staging file
   `<conv-repo>/steps/<conv-id>/<NNN>/assistant.staging.json`,
   appending each content block as it completes; segment authority
   (§4.4) truncates it on an `Error` attempt and the settling `Finish`
   seals it. When the model call completes, the sealed file is renamed
   into the worktree as `messages/NNN-assistant.json` — a JSON array of
   canonical `Content` blocks — and committed. `NNN` is the branch's
   transcript counter, max-present-plus-one from the `messages/`
   listing, evaluated at commit time.
6. **Step loop (§2.5).** If the completion's terminal reason is
   `Finish{ToolUse}`, run every emitted `tool_use` block through the
   tool executor — the per-call records land under
   `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/` (out of
   every worktree, §3.3). As each tool resolves the transcript writer
   commits `messages/NNN-tool.json` (its canonical `tool_result` block,
   §3.3) — then assemble step `<NNN+1>` whose user message carries one
   `tool_result` block per emitted call. Step ≥2 has no *dispatch*
   commit, but each step's transcript entries (assistant output, tool
   results) do advance the branch tip, which is that step's read state
   (§2.10). Loop until the terminal reason is anything other than
   `ToolUse`. **The transcript runs alongside the shipped in-memory
   path:** both are written; the accumulator is still what assembles the
   next request. Re-pointing the assembler at the committed transcript
   (deleting the accumulator and the `output.json` read path) is a
   separate task.
7. Dispatch the terminal compactor (§2.7) off the conversation tip
   by re-entering the binary as `lernie dispatch compactor <repo>
   <conv-id>` (subprocess invocation per §3.4). The compactor spawns
   branch `<conv-id>-<cmp-id>` (hyphenated descent per §2.2) off the
   conversation tip, writes a placeholder `summary/001.md`, and
   `--no-ff` merges back into the conversation branch. The compactor
   is a stub — it does not call a model; the shape exists so v0.4+ can
   layer real semantics without moving call sites.
8. Rebase the conversation branch onto the current `main` tip and
   `--no-ff` merge it into `main` (§2.6), running the merge inside
   `<repo>/root/`. Remove the conversation worktree; the branch
   ref stays for the retention window (§2.3).
9. Print the conversation branch name on stdout.

After `lernie prompt` returns, inspect the merge from the primary
worktree (`root/` is where `main` is checked out):

```
cd /path/to/my-conversation/root
git log --oneline --decorate main -4
git show --stat main              # the --no-ff merge commit
cat summary/001.md                # terminal summary
```

The unmerged branch count health metric (ARCH §8) is read straight
from git refs — no sidecar file:

```
git -C /path/to/my-conversation/root \
    branch --list '*-*' --no-merged main | wc -l
```

A ballooning count indicates a silent failure in the merge pipeline.

## Stopping a conversation

```
lernie stop /path/to/my-conversation <conv-id>
```

Sends `SIGTERM` to the process group of the harness driving `<conv-id>`
(and any subagent harnesses on hyphenated descendants — `<conv-id>-…`),
with a 5-second flush deadline before `SIGKILL`. This is the same
cascade pattern adapter (§4.4) and tool (§3.3) cancellation use, applied
to the harness itself ([ARCH §2.9](docs/ARCHITECTURE.md#29-stopped-branches)).
The pid is discovered by scanning `/proc/<pid>/fd/*` for the writer
holding the latest step's `response.json` open — no sidecar pid file —
so the same `IN_CLOSE_WRITE` signal that drives the §3.5 in-flight UI
classification also targets the cascade. Linux only.

Behavior:

- **Idempotent.** A branch with no live writer (already stopped, or the
  harness exited cleanly) returns success without sending any signal.
- **Errors when** the branch doesn't exist, or is already merged into
  `main`. Both surface as a non-zero exit with a `lernie stop:` prefix
  on stderr.
- **No on-disk cancel marker.** The §2.9 signature of a stopped branch
  is the latest step's `response.json` closed without a terminal `end`
  event — produced for free by the kernel closing the harness's open fds
  when the process terminates (and `bz` dies with it, §4.4).

The frontend's stop button (per [ARCH §3.5](docs/ARCHITECTURE.md#35-ui-contract))
exec's this exact subcommand; there is no second control surface.

## Built-in tools (v0.3, +v0.4 Phase 2 dispatch)

The agent can call **built-in tools** that ship inside the `lernie`
binary as `lernie tool <name>` subcommands (ARCH §3.3 / §12). The tool
executor's resolution order — `<data-root>/tools/lernie-tool-<name>`
→ `PATH` → `lernie tool <name>` — falls through to this in-process
route for tools not externalized.

Each built-in is the triple §3.3 pins:

- **Binary** — the `lernie tool <name>` subcommand. Reads
  `tool_use.input` JSON from stdin, writes raw bytes to stdout, exits
  0 on success or non-zero on failure (stderr is concatenated after
  stdout into `tool_result.content` when `is_error` is set).
- **JSON schema** — at [`schemas/tools/<name>.json`](schemas/tools/),
  copied to `<data-root>/tools/<name>.json` by `make install`. Sent
  verbatim as the `input_schema` of the tool's entry in the model
  call's `tools: [...]` array.
- **Skill** — at [`skills/<name>/SKILL.md`](skills/), copied to
  `<data-root>/skills/<name>/`. The frontmatter `description` is
  the tool's description in `tools: [...]`; the body explains when to
  reach for it.

Built-ins:

- **`read_file`** — read the entire contents of a file at a given
  path. Rejects files larger than 1 MiB; v0.4+ adds the
  oversized-output auto-dispatch shim (ARCH §3.3 / §12). Try it
  directly: `echo '{"path":"README.md"}' | lernie tool read_file`.
- **`bash`** — runs a shell command via `sh -c` and returns its
  stdout. The shell runs in its own process group so a SIGTERM the
  harness sends is forwarded to the entire spawned tree (§2.9
  cascade). Try it directly: `echo '{"command":"ls"}' | lernie tool bash`.
- **`dispatch`** (v0.4 Phase 2) — spawns a subagent on a fresh
  branch with the supplied goal and returns
  `{"status":"in_progress","handle":"<sub-branch>"}` synchronously
  (ARCH §2.5). Input is `{role, goal}`;
  the role must resolve to `<conv-repo>/souls/<role>.md` and a
  `roles:` entry in `<conv-repo>/providers.yaml`. Reads the calling
  conversation's repo + branch from the harness-set
  `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` env vars (ARCH §3.3 env
  bullet); spawns through `lernie dispatch <role>` (§3.4). The handle
  it returns is the child's *address* — there is no polling tool to
  pair with it. The substrate redesign (ARCH §2.5 "Dispatch returns
  the child's address") dissolved the handle/`await` pair: the child's
  result comes back as a **deposit into the parent's inbox** carrying
  an epitaph (§2.6, §2.11), so `await`/`check` had nothing left to
  observe and are gone. That inbox return path is specced (ARCH §2.11)
  but not yet implemented; until it lands, `dispatch` spawns the child
  and returns its address, but the result does not yet flow back.

## Dispatching subagents directly

`lernie dispatch <role> <repo> <branch> [--goal <text>]` is the §3.4
re-entry point every subagent uses. The role name is positional, so
the surface generalizes across the v0.3 compactor, the v0.4 worker,
and future roles (verifier, critic, …) without a CLI shape change.

- `lernie dispatch compactor <repo> <conv-id>` runs the same
  terminal-compaction routine `lernie prompt` triggers internally.
  Useful for re-compacting a conversation branch that still exists as
  a ref, or for testing the compactor path independently. The
  compactor's goal is built-in boilerplate; passing `--goal` is
  rejected. The command only runs the compaction +
  compactor→conversation merge; merge into `main` is the caller's
  problem (§2.6).
- `lernie dispatch worker <repo> <parent-branch> --goal <text>`
  spawns a worker subagent off `<parent-branch>`'s tip. The new
  branch is `<parent-branch>-<sub-id>` (hyphenated descent, §2.2),
  with a sibling worktree at the same path; `goal.md` carries the
  supplied text and `soul.md` is loaded from
  `<repo>/souls/worker.md`, both committed as the dispatch commit
  (§2.3 step 2). v0.4 Phase 1 stops there — the worker's own step
  loop, the `dispatch` tool that puts a parent in front of it, and the
  inbox result-return path (§2.11) land in subsequent phases.

## Providers

Every model call goes through **brazen** — one small, stateless binary
(`bz`) that adapts every provider and wire protocol behind a single pipe
contract (see [ARCH §4.4](docs/ARCHITECTURE.md#44-the-provider-adapter-brazen)):

```
stdin (canonical request, JSON) → bz → stdout (v=1 event stream, NDJSON, one terminal `end`)
```

The harness execs `bz --json --provider <row>` once per attempt, pipes a
typed `brazen::CanonicalRequest` on stdin, and appends bz's stdout
verbatim to the step's `response.json`. lernie links the `brazen` crate
(`brazen = "=0.0.2"`) for the canonical *types* only — the data plane
always crosses the subprocess boundary (§3.4). Two facts follow:

- **Retry is the harness's.** brazen never retries — one `bz` process,
  one HTTP round-trip. On a retryable in-band `Error`
  (`CanonicalError::retryable()`, the linked crate's single home for the
  fact) the harness re-invokes `bz` up to the `workflow.yaml` attempt cap
  (§2.10). Each attempt appends one segment to `response.json`; the last
  is authoritative.
- **Auth and endpoints are brazen's.** Provider *rows* (endpoint,
  protocol, auth mode, model aliases) live in brazen's own config
  (`~/.config/brazen/config.toml`; `bz --dump-config`, `bz --login`).
  lernie references a row by name and never sees credential material
  (§4.1). A load-time guard (`bz --version` == the linked crate version)
  rejects a mismatched binary; `make install` installs the pin with
  `cargo install brazen --version =0.0.2`.

### Adding a provider

- **A new provider on a supported protocol** is a brazen config row — no
  code anywhere. Add the row (`bz` config), reference its name as a
  model's `provider:` in `<config-root>/models.yaml`, and point a role
  at that model in `<repo>/providers.yaml`.
- **A new wire protocol or auth mode** is a contribution to brazen.
- **An alternate adapter binary** that honors the same pipe contract
  slots in via the optional `adapter:` path in `models.yaml` (§4.2); the
  version guard is skipped for it and the in-band `MessageStart.v`
  handshake governs compatibility instead.

## UI (v0.5)

`lernie-ui-egui` is the desktop frontend: an egui/eframe window that renders
a conversation repo and issues user actions via `lernie <subcommand>`. Per
[ARCH §3.5](docs/ARCHITECTURE.md), it is stateless — every render is a pure
function of filesystem state — and one of potentially several frontends
(a future `lernie-ui-web` would share the pure-Rust view-model layer).

On startup, the binary loads the current git history of the target
conv-repo (ARCH §2.2 — the dir with `manifest.yaml`, `root/`, and any
sibling subagent worktrees) and renders a two-tier tree: `main`'s
first-parent trunk, with each `--no-ff` conversation merge showing its
step commits indented beneath; any unmerged conversation branches (root
in-flight or subagent) follow in their own section. Each merge node is
labeled with the conversation id and a truncated `messages[0].content`
preview pulled from `steps/<conv-id>/001/request.json`. If the tree
can't be read, a placeholder view is shown instead.

Four pure-Rust modules inside the crate (no egui dep on the view-model
side, reusable by a future `lernie-ui-web`) back the UI:

- `fs_watcher` — tracks the §3.5 repo paths via `notify` and coalesces
  change events. The watched set covers conv-repo root paths
  (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `version`,
  `souls/`, `steps/` — the last lives at the conv-repo root outside
  every worktree per ARCH §2.2 / §2.3), per-worktree contents under
  any subdir (`goal.md`, `soul.md`, `summary/`, `descriptions/`,
  `skills/`, `.gitattributes`), and the primary worktree's git refs
  (`root/.git/HEAD`, `root/.git/refs/`). Filesystem events drive the
  re-render tick; the renderer is a pure function of on-disk state at
  that tick (no in-memory accumulator), so a missed event at most
  delays a frame.
- `cli_outbound` — the frontend's sole command surface: `Cli::run(args)`
  spawns `lernie <subcommand>` with stream-chunked stdout/stderr and
  aggressive SIGTERM-then-SIGKILL cleanup on drop (ARCH §2.9). Override
  the binary via `LERNIE_BINARY`; default is `lernie` on `PATH`.
- `git_tree` — resolves the conv-repo path to its `root/` worktree
  (ARCH §2.2) and produces a view-model (`GitTree`) independent of
  egui: `main`'s first-parent trunk (including conversation merge nodes
  with their step commits) plus the set of unmerged conversation
  branches (`git for-each-ref --no-merged=main refs/heads/`, per
  PRINCIPLES.md single-source-of-truth). Conversation detection keys
  off the merged branch name in each `--no-ff` merge subject (step
  records are no longer in the merge tree per ARCH §2.3); the user-
  message preview is read from `<conv-repo>/steps/<conv-id>/001/
  request.json` on disk. Submodules layer the live indicators: streaming
  text folded from the latest step's `response.json` JSONL (`text_delta`
  events, ARCH §4.4); branch-state badges (in-flight / stopped / merged
  / conflicted) derived from refs + the §4.4 terminal event without
  sidecars (PRINCIPLES.md single-source-of-truth); tool-call pulses
  derived from `tools/<tool-id>/input.json` present without `output.json`
  (ARCH §3.3 in-progress-is-derived-state). A thin egui widget in the
  same module renders it.
- `actions` — the user-action surface: `ActionsState` holds the
  in-progress prompt input and selected branch (in-memory only per
  §3.5), and `dispatch_new_prompt` / `dispatch_stop` build the argv for
  `lernie prompt <repo> <message>` / `lernie stop <repo> <branch>` and
  invoke `cli_outbound`. Enable/disable derivation (`new_prompt_enabled`,
  `stop_enabled`) is a pure function of the input string and the
  unmerged-branch list. Per the §2.9 amendment landed in bl-abf3,
  there is no user-facing "resume" — continuing from a stopped branch
  is a fresh `lernie prompt` or fork-from-history.

```
lernie-ui-egui --repo /path/to/my-conversation
```

From a source checkout, `make ui REPO=/path/to/my-conversation` does the
same via `cargo run --bin lernie-ui-egui` and is what contributors
typically use during development.

egui is winit-based; its only runtime deps are the X11/Wayland libs already
present on any Linux desktop. No `apt install` step is required.

## Contributing

The instructions below are for contributors building lernie from source.
Users installing a release don't need any of this — `make install` from
**Quickstart** is the user-facing entry point.

### Contributor setup

```
make install-hooks
```

Sets `core.hooksPath` to `.githooks`. Required on every fresh clone — git
does not track `.git/config`, so the hook is not active until installed.

### Build targets

| Target                | What it does                                          |
|-----------------------|-------------------------------------------------------|
| `make build`          | `cargo build`                                         |
| `make release`        | `cargo build --release`                               |
| `make test`           | `cargo test`                                          |
| `make coverage`       | `cargo tarpaulin --fail-under 100` (llvm engine)      |
| `make lint`           | `cargo clippy --all-targets -- -D warnings`           |
| `make fmt`            | `cargo fmt`                                           |
| `make fmt-check`      | `cargo fmt --check`                                   |
| `make schemas`        | Regenerate `schemas/*.json` from the Rust types       |
| `make new-conversation DEST=<path>` | Scaffold a conversation repo from `template/` |
| `make ui REPO=<path>`  | Launch `lernie-ui-egui` against an existing conv-repo via `cargo run`        |
| `make check`          | `fmt-check` + `lint` + `coverage`                     |
| `make ci`             | Alias for `check`                                     |
| `make install-hooks`  | Point git at `.githooks/`                             |
| `make install` [`INSTALL_PREFIX=<p>` `LERNIE_HOME=<h>`] | Release-build; drop `lernie`/`lernie-ui-egui` into `$INSTALL_PREFIX/bin` (default: `~/.local/bin`); install the provider adapter `bz` via `cargo install brazen --version =0.0.2` (the ARCH §4.4 version pin); lay down the harness root — config root (default `~/.config/lernie`) with a default `models.yaml`, data root (default `~/.local/share/lernie`) with the `agents/default/` template; `LERNIE_HOME` collapses both |
| `make uninstall` [`INSTALL_PREFIX=<p>` `LERNIE_HOME=<h>`] | Remove the installed binaries; leaves the harness homes (config + data roots) in place |

### Workflow

All changes land on `main` via `bl` squash-merges. Direct commits to `main` are
rejected by the pre-commit hook.

```
bl prime --as <you>
bl claim <task-id>              # creates a worktree; cd into it
# ...edit, test, commit...
bl review <task-id> -m "..."    # squash-merges into main
bl close  <task-id> -m "..."    # from the repo root
```

See `bl skill` for the full guide.

### Pre-commit hook

`.githooks/pre-commit` enforces three rules on every commit:

1. **No direct commits to mainline.** `main` and `master` are rejected unless
   the commit is the tail of a merge (`MERGE_MSG`/`SQUASH_MSG` present), which
   is how `bl review` lands squash-merges.
2. **300-line cap on code files.** Docs (`*.md`, `*.txt`), config (`*.toml`,
   `*.yaml`, `*.yml`, `*.json`, `*.lock`), `Makefile`, `.gitignore`,
   `LICENSE`, and anything under `.githooks/` are exempt.
3. **100% line coverage.** `cargo tarpaulin --fail-under 100` runs on every
   commit that touches a Cargo project. The tarpaulin version is pinned (see
   `tarpaulin.toml` and `.github/workflows/ci.yml`) so the denominator means
   the same thing locally and on CI — newer releases have silently dropped
   inline `#[cfg(test)] mod tests;` files from the count, weakening the floor.
   `make coverage` aborts with an install hint if the local version drifts.

There is no `--no-verify` escape hatch in the workflow. If the hook rejects a
commit, fix the underlying issue rather than skipping.

## License

MIT. See [`LICENSE`](LICENSE).
