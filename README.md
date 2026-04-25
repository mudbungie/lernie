# lernie

A git-backed agent harness. Design spec: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
Principles catalog: [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md).
Vocabulary reference: [`docs/TAXONOMY.md`](docs/TAXONOMY.md).

## Setup

```
make install-hooks
```

Sets `core.hooksPath` to `.githooks`. Required on every fresh clone — git
does not track `.git/config`, so the hook is not active until installed.

## Build

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
| `make check`          | `fmt-check` + `lint` + `coverage`                     |
| `make ci`             | Alias for `check`                                     |
| `make install-hooks`  | Point git at `.githooks/`                             |
| `make install` [`INSTALL_PREFIX=<p>` `LERNIE_HOME=<h>`] | Release-build, lay down the harness root (default: `~/.lernie/`), drop `lernie`/`lernie-ui-egui` into `$INSTALL_PREFIX/bin` (default: `~/.local/bin`) and `lernie-provider-anthropic` into `$LERNIE_HOME/adapters/` |
| `make uninstall` [`INSTALL_PREFIX=<p>` `LERNIE_HOME=<h>`] | Remove the installed binaries; leaves the harness root config in place |

### Quickstart

```
make install              # lay down ~/.lernie/ + binaries on PATH
lernie new ~/work/chat    # scaffold a conversation repo
ANTHROPIC_API_KEY=... lernie prompt ~/work/chat 'hello'
```

### Install

```
make install                                  # default: ~/.local/bin + ~/.lernie/
make install INSTALL_PREFIX=/usr/local        # binaries -> /usr/local/bin/
make install LERNIE_HOME=/opt/lernie          # harness root -> /opt/lernie/
```

`make install` runs a release build and then:

1. Lays down the harness root skeleton at `$LERNIE_HOME` (default
   `~/.lernie/`) — `adapters/`, `workflows/`, `tools/`, `skills/`,
   `agents/`, and `conversations/` (ARCH §2.2). Re-runs are idempotent:
   directory creation uses `mkdir -p`; existing config files are kept.
2. Installs `lernie` and `lernie-ui-egui` into `$INSTALL_PREFIX/bin`
   with `install -m 0755` (atomic overwrite, no symlinks). Make sure
   that directory is on your `PATH`.
3. Installs `lernie-provider-anthropic` into `$LERNIE_HOME/adapters/`
   — the harness resolves `lernie-provider-<name>` there before
   falling back to `PATH` (ARCH §4.4), so per-harness-root credential
   isolation is preserved.
4. Drops a default `$LERNIE_HOME/providers.yaml` (anthropic endpoint
   + a couple of model rows) and a `$LERNIE_HOME/agents/default/`
   skeleton copied from [`template/`](template/) — only when those
   paths don't already exist, so rotated credentials and hand-edited
   profiles survive a re-install.
5. Smoke-tests the freshly installed binaries with `lernie --version`
   and a throwaway `lernie new`. Failure aborts the install with a
   non-zero exit.

`make uninstall` removes the installed binaries; the harness root
(config, conversations, custom adapters) stays put — clean it up
manually if you want a true uninstall.

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
| `schemas/global-providers.json` | `config::providers::Providers`           | `<harness-root>/providers.yaml`       |

## Layout: harness root and conversation repos

There are two distinct on-disk locations (ARCH §2.2):

- **Harness root** — installation-global, defaults to `~/.lernie/`,
  overridable via `LERNIE_HOME`. Holds the global
  [`providers.yaml`](docs/ARCHITECTURE.md#41-provider-abstraction)
  (endpoints, auth, models — §4.1), `adapters/`, `workflows/`, `tools/`,
  `skills/`, and `agents/<profile>/` per-profile skeletons. Shared across
  every conversation; rotates with key rollover and infrastructure
  changes.
- **Conversation repo** — one self-contained git repository per root
  conversation, at `<harness-root>/conversations/<root-id>/`. Carries the
  per-repo `providers.yaml` (`roles:` only — §4.3), `manifest.yaml`,
  `workflow.yaml`, `souls/`, and a `root/` worktree where `main` is
  checked out. Subagent conversations are sibling worktrees of `root/`,
  one per `<a>-<b>-…/` directory. Conversation repos are never pushed to
  a remote.

Each conversation repo is scaffolded from [`template/`](template/) — the
versioned skeleton embedded into the `lernie` binary at build time, the
v0.3-shape default profile in the absence of a custom
`<harness-root>/agents/<profile>/`. Create one:

```
lernie new                     # auto-id under <harness-root>/conversations/
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
the destination is `<LERNIE_HOME or ~/.lernie>/conversations/<auto-id>/`;
the scaffolded path is printed on stdout. `goal.md` and `soul.md` inside
`root/` are intentionally not in the template — they are written at
dispatch time (ARCH §2.3, §2.8).

## Sending a prompt (v0.3)

```
ANTHROPIC_API_KEY=... lernie prompt /path/to/my-conversation 'hello'
```

`lernie prompt` is the v0.3 root-conversation path (ARCH §2.3, §2.6,
§2.7, §2.8, §2.10). Each invocation spawns its own branch off `main`,
commits a snapshot before the model call, lands the response as a
follow-up commit, runs the terminal compactor off the tip, and
`--no-ff` merges the compacted branch back into `main`:

1. Resolve the harness root (`LERNIE_HOME` or `~/.lernie/`, ARCH
   §2.2). Load `<harness-root>/providers.yaml` (endpoints, auth, model
   capabilities — §4.1) and `<repo>/providers.yaml` (`roles:` block —
   §4.3); cross-validate `roles.worker.{provider,model}` against the
   global file.
2. Read the worker soul from `<repo>/souls/worker.md` (§4.3 — by
   convention, no per-role path override).
3. Invoke `lernie-provider-<name> describe` (resolved at
   `<harness-root>/adapters/`, then `PATH`, per §4.4) to read the
   adapter's `endpoint_env`.
4. Spawn branch `<conv-id>` (the bare hyphenated id — no `ex/`
   prefix per §2.3) off `main` and allocate a sibling worktree at
   `<repo>/<conv-id>/` (§2.2). The `git worktree add` runs inside
   `<repo>/root/`, where the `.git` directory lives.
5. Write the branch goal to `goal.md`, the role soul to `soul.md`,
   and the model call's input to `steps/<conv-id>/001/request.json`
   in the new worktree; commit all three on the branch as the
   dispatch snapshot. This is §2.10's "commit before model call" —
   the tree the model reads is the tree of this snapshot commit.
6. Invoke `lernie-provider-<name> complete`, setting each env var
   named in `endpoint_env` to `providers.<name>.endpoint`; pipe the
   Messages-API-shaped request on stdin; read one JSON document back.
   Credential env vars like `ANTHROPIC_API_KEY` propagate by normal
   process inheritance.
7. Write the normalized response (the assistant's structured
   `content` blocks — text + `tool_use` per §3.3 — plus `model_id`,
   `provider`, `usage`, `stop_reason`, `started_at`, `ended_at`) to
   `steps/<conv-id>/<NNN>/response.json` and land it as a follow-up
   commit on the same branch. The snapshot commit's tree stays
   intact so replay and retry (§2.10) see exactly what the model saw.
8. **Step loop (§2.5).** If the response's `stop_reason` is
   `tool_use`, run every emitted `tool_use` block through the
   tool executor — ball #4 ships the real subprocess-driving impl
   that lands per-call records under
   `steps/<conv-id>/<NNN>/tools/<tool-id>/` per §3.3 — then assemble
   step `<NNN+1>` whose user message carries one `tool_result` block
   per emitted call. Steps 2+ commit only `request.json`
   (goal/soul are step 1's job). Loop until `stop_reason` is
   anything other than `tool_use`.
9. Dispatch the terminal compactor (§2.7) off the conversation tip
   by re-entering the binary as `lernie dispatch compactor <repo>
   <conv-id>` (subprocess invocation per §3.4 — the harness never
   shortcuts past the CLI for procedure-to-procedure calls). The
   compactor spawns branch `<conv-id>-<cmp-id>` (hyphenated descent
   per §2.2) off the conversation tip in a sibling worktree at
   `<repo>/<conv-id>-<cmp-id>/`, writes `goal.md` with the
   boilerplate compactor goal and lands it as a dispatch commit
   (§2.8, §2.10), then writes `summary/001.md` with the
   terminal-response summary and lands that as a follow-up commit,
   and `--no-ff` merges the compactor branch back into the
   conversation branch. The v0.3 compactor is a stub — it does not
   call a model, and `mark_for_deletion` is a no-op; the shape
   exists so v0.4+ can layer real semantics without moving call
   sites.
10. Rebase the conversation branch onto the current `main` tip and
    `--no-ff` merge it into `main` (§2.6), running the merge inside
    `<repo>/root/`. Remove the conversation worktree; the branch
    ref stays for the retention window (§2.3).
11. Print the conversation branch name on stdout.

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

## Built-in tools (v0.3)

The agent can call **built-in tools** that ship inside the `lernie`
binary as `lernie tool <name>` subcommands (ARCH §3.3 / §12). The tool
executor's resolution order — `<harness-root>/tools/lernie-tool-<name>`
→ `PATH` → `lernie tool <name>` — falls through to this in-process
route for tools not externalized.

Each built-in is the triple §3.3 pins:

- **Binary** — the `lernie tool <name>` subcommand. Reads
  `tool_use.input` JSON from stdin, writes raw bytes to stdout, exits
  0 on success or non-zero on failure (stderr is concatenated after
  stdout into `tool_result.content` when `is_error` is set).
- **JSON schema** — at [`schemas/tools/<name>.json`](schemas/tools/),
  copied to `<harness-root>/tools/<name>.json` by `make install`. Sent
  verbatim as the `input_schema` of the tool's entry in the model
  call's `tools: [...]` array.
- **Skill** — at [`skills/<name>/SKILL.md`](skills/), copied to
  `<harness-root>/skills/<name>/`. The frontmatter `description` is
  the tool's description in `tools: [...]`; the body explains when to
  reach for it.

v0.3 ships two built-ins:

- **`read_file`** — read the entire contents of a file at a given
  path. Rejects files larger than 1 MiB; v0.4+ adds the
  oversized-output auto-dispatch shim (ARCH §3.3 / §12). Try it
  directly: `echo '{"path":"README.md"}' | lernie tool read_file`.
- **`bash`** — runs a shell command via `sh -c` and returns its
  stdout. The shell runs in its own process group so a SIGTERM the
  harness sends is forwarded to the entire spawned tree (§2.9
  cascade). Try it directly: `echo '{"command":"ls"}' | lernie tool bash`.

## Dispatching the compactor directly

`lernie dispatch compactor <repo> <conv-id>` runs the same
terminal-compaction routine that `lernie prompt` triggers internally.
Useful for re-compacting a conversation branch that still exists as a
ref, or for testing the compactor path independently. The command
only runs the compaction + compactor→conversation merge; merge into
`main` is the caller's problem (§2.6).

## Providers

Model calls go through **provider adapters** — separate binaries the harness
invokes over stdio, one per named provider (see
[ARCH §4.4](docs/ARCHITECTURE.md#44-provider-adapters)). Auth, HTTP, and
transient retry live inside the adapter; the harness only forwards the env
vars the adapter declares. A new provider is a standalone executable, not a
harness patch.

The reference adapter is `lernie-provider-anthropic`, built alongside
`lernie` by `cargo build`:

```
lernie-provider-anthropic describe
ANTHROPIC_API_KEY=... lernie-provider-anthropic complete < request.json
```

`describe` prints the adapter's self-description JSON (name,
`schema_version`, capabilities, models, `auth_env`, `endpoint_env`).
`complete` reads one Messages-API request on stdin and writes either a
single JSON response object or — when the request carries `stream:
true` — a JSON Lines event stream of normalized §4.4 events
(`message_start`, `content_block_start`, `text_delta`,
`tool_use_delta`, `content_block_stop`, terminal `message_stop` with
`usage` + `api_calls`). Errors land in-band as `{"type":"error",
"kind":"retryable"|"fatal", ...}` in either mode; exit code `0` covers
both, with non-zero reserved for adapter-side crashes. The upstream
URL is read from the env var named in `endpoint_env`
(`LERNIE_PROVIDER_ANTHROPIC_ENDPOINT` for the reference adapter), with
a built-in default if unset. SIGTERM cancels in-flight work and emits
a terminal `error` event before exit (§4.4 "Cancellation"). The
v0.2 adapter advertises `streaming` in `describe.capabilities`; the
harness `lernie prompt` path stays non-streaming today (UI consumption
of the stream is the v0.5 ball).

Harness-side adapter discovery and invocation (§4.4 "Discovery" /
"Endpoint") is wired into the `lernie prompt` subcommand — see
**Sending a prompt (v0.3)** above.

### Dropping in a custom adapter

The harness resolves `lernie-provider-<name>` at
`<harness-root>/adapters/` first, then falls back to `PATH` (ARCH
§4.4). To test a new one:

1. Copy the binary into `$LERNIE_HOME/adapters/` (default
   `~/.lernie/adapters/` — the same location `make install` uses for
   the bundled anthropic adapter) with the name
   `lernie-provider-<name>`. A `PATH` install also works for
   experimentation but skips the per-harness-root isolation.
2. Add a provider entry in `<harness-root>/providers.yaml` (ARCH
   §4.1) keyed by `<name>`, with the `endpoint:` and `auth:` the
   adapter expects, and a `models:` entry whose `provider:` points at
   that key.
3. In the conversation repo's `<repo>/providers.yaml` (`roles:`
   block, ARCH §4.3), point the `worker` role at the new
   provider/model pair.
4. Run `lernie prompt <repo> '...'`.

## UI (v0.5, skeleton)

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

Three pure-Rust modules inside the crate (no egui dep on the view-model
side, reusable by a future `lernie-ui-web`) back the UI:

- `fs_watcher` — tracks the §3.5 repo paths via `notify` and coalesces
  change events. The watched set covers conv-repo control files
  (`manifest.yaml`, `workflow.yaml`, `providers.yaml`, `version`,
  `souls/`), per-worktree contents under any subdir (`goal.md`,
  `soul.md`, `summary/`, `steps/`, `descriptions/`, `skills/`,
  `.gitattributes`), and the primary worktree's git refs
  (`root/.git/HEAD`, `root/.git/refs/`). Live updates wire through here
  in a follow-up.
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
  off `steps/<conv-id>/<NNN>/...` paths introduced by each merge. A
  thin egui widget in the same module renders it.

```
lernie-ui-egui --repo /path/to/my-conversation
```

egui is winit-based; its only runtime deps are the X11/Wayland libs already
present on any Linux desktop session. No `apt install` step is required.

## Workflow

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

## Pre-commit hook

`.githooks/pre-commit` enforces three rules on every commit:

1. **No direct commits to mainline.** `main` and `master` are rejected unless
   the commit is the tail of a merge (`MERGE_MSG`/`SQUASH_MSG` present), which
   is how `bl review` lands squash-merges.
2. **300-line cap on code files.** Docs (`*.md`, `*.txt`), config (`*.toml`,
   `*.yaml`, `*.yml`, `*.json`, `*.lock`), `Makefile`, `.gitignore`,
   `LICENSE`, and anything under `.githooks/` are exempt.
3. **100% line coverage.** `cargo tarpaulin --fail-under 100` runs on every
   commit that touches a Cargo project.

There is no `--no-verify` escape hatch in the workflow. If the hook rejects a
commit, fix the underlying issue rather than skipping.

## License

MIT. See [`LICENSE`](LICENSE).
