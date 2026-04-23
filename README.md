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
| `make install` [`INSTALL_PREFIX=<p>`] | Release-build, then copy `lernie`, `lernie-provider-anthropic`, and `lernie-ui-egui` into `$INSTALL_PREFIX/bin` (default: `~/.local/bin`) |
| `make uninstall` [`INSTALL_PREFIX=<p>`] | Remove the installed binaries from `$INSTALL_PREFIX/bin` |

### Install

```
make install                          # -> ~/.local/bin/{lernie, lernie-provider-anthropic, lernie-ui-egui}
make install INSTALL_PREFIX=/usr/local # -> /usr/local/bin/...
```

`make install` runs a release build and copies the binaries into
`$INSTALL_PREFIX/bin` with `install -m 0755` (atomic overwrite, no
symlinks). Make sure that directory is on your `PATH`. Re-run after
every rebuild to pick up changes. `make uninstall` removes them.

## Configuration schemas

JSON Schemas for the `.agent/*.yaml` config files (per
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) §2.2) are generated from the
Rust types under `src/config/`. `make schemas` writes them to `schemas/` for
editor integration and external validators.

## Conversation repos

Each conversation is a self-contained git repository copied from
[`template/`](template/) — the versioned skeleton described in ARCH §2.2.
Create one with the `lernie` binary:

```
lernie new /path/to/my-conversation
```

Or via the Makefile wrapper:

```
make new-conversation DEST=/path/to/my-conversation
```

The binary embeds `template/` at build time (via `include_dir`), copies
it to the destination, runs `git init -b main`, and lands a single
`init conversation repo` commit. The destination must either not exist
or be an empty directory. `.agent/goal.md` is intentionally not in the
template — it is written at dispatch time (ARCH §2.8).

## Sending a prompt (v0.2)

```
ANTHROPIC_API_KEY=... lernie prompt /path/to/my-conversation 'hello'
```

`lernie prompt` is the v0.2 exchange-branch path (ARCH §2.3, §2.8,
§2.10). Each invocation spawns its own branch off `main`, commits a
snapshot before the model call, lands the response as a follow-up
commit, and leaves the branch open. `main`'s HEAD is not touched —
merge-back (§2.6) is a separate step:

1. Load `<repo>/.agent/providers.yaml` and `<repo>/.agent/agents.yaml`;
   cross-validate `agents.worker.model` against the declared models.
2. Invoke `lernie-provider-<name> describe` (discovered on `PATH`) to
   read the adapter's `endpoint_env` (§4.4).
3. Spawn branch `ex/<ts>-<short-id>` off `main` and allocate a
   worktree at `<repo>/.lernie/worktrees/ex/<ts>-<short-id>/`.
4. Write the branch goal to `.agent/goal.md` (§2.8) and the model
   call's input to `exchanges/<ts>-<short-id>/steps/001/request.json`;
   commit both on the branch. This is §2.10's "commit before model
   call" — the tree the model reads is the tree of this snapshot
   commit.
5. Invoke `lernie-provider-<name> complete`, setting each env var
   named in `endpoint_env` to `providers.<name>.endpoint`; pipe the
   Messages-API-shaped request on stdin; read one JSON document back.
   Credential env vars like `ANTHROPIC_API_KEY` propagate by normal
   process inheritance.
6. Write the normalized response (assistant text, `model_id`,
   `provider`, `usage`, `stop_reason`, `started_at`, `ended_at`) to
   `exchanges/<ts>-<short-id>/steps/001/response.json` and land it as
   a follow-up commit on the same branch. The snapshot commit's tree
   stays intact so replay and retry (§2.10) see exactly what the model
   saw.
7. Print the branch name on stdout.

After the call, inspect the branch with standard git tooling:

```
cd /path/to/my-conversation/.lernie/worktrees/ex/<ts>-<id>
git log --oneline        # two commits ahead of main
cat .agent/goal.md       # the user message
cat exchanges/*/steps/001/response.json
```

The unmerged branch count health metric (ARCH §8) is read straight
from git:

```
git -C /path/to/my-conversation branch --list 'ex/*' | wc -l
```

A ballooning count indicates a silent failure in the merge pipeline.

Merge-back to `main` (§2.6) is tracked under a separate v0.2 task and
not part of this command today; open exchange branches therefore show
up in `git branch --list ex/*` until merge ships.

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
`complete` reads one Messages-API request on stdin and writes one JSON
object on stdout — either the upstream response or an in-band error
object (`{"type":"error", "kind":"retryable"|"fatal", ...}`). Exit code
`0` covers both; non-zero is reserved for adapter-side crashes. The
upstream URL is read from the env var named in `endpoint_env`
(`LERNIE_PROVIDER_ANTHROPIC_ENDPOINT` for the reference adapter), with
a built-in default if unset. The v0.1 adapter is non-streaming;
streaming support is tracked separately (bl-d15d).

Harness-side adapter discovery and invocation (§4.4 "Discovery" /
"Endpoint") is wired into the `lernie prompt` subcommand — see
**Sending a prompt (v0.1)** above.

### Dropping in a custom adapter

Adapters are discovered on `PATH` by name. To test a new one:

1. Copy the binary into any `PATH` directory (e.g. `~/.local/bin/` —
   the same location `make install` uses) with the name
   `lernie-provider-<name>`.
2. Add a provider entry in your conversation repo's
   `.agent/providers.yaml` keyed by `<name>`, with the `endpoint:` and
   `auth:` the adapter expects.
3. Declare at least one `models:` entry whose `provider:` points at
   that key, and point an agent role (`worker` for v0.1) at that model.
4. Run `lernie prompt <repo> '...'`.

## UI (v0.5, skeleton)

`lernie-ui-egui` is the desktop frontend: an egui/eframe window that renders
a conversation repo and issues user actions via `lernie <subcommand>`. Per
[ARCH §3.5](docs/ARCHITECTURE.md), it is stateless — every render is a pure
function of filesystem state — and one of potentially several frontends
(a future `lernie-ui-web` would share the pure-Rust view-model layer).

On startup, the binary loads the current git history of the target repo
and renders it as a linear commit list (v0.1-shape: one
`exchanges/<ts>-<id>.json` per commit, with a truncated user-message
preview). If the tree can't be read, a placeholder view is shown instead.
Three pure-Rust modules inside the crate (no egui dep on the view-model
side, reusable by a future `lernie-ui-web`) back the UI:

- `fs_watcher` — tracks the §3.5 repo paths via `notify` and coalesces
  change events. Live updates wire through here in a follow-up.
- `cli_outbound` — the frontend's sole command surface: `Cli::run(args)`
  spawns `lernie <subcommand>` with stream-chunked stdout/stderr and
  aggressive SIGTERM-then-SIGKILL cleanup on drop (ARCH §2.9). Override
  the binary via `LERNIE_BINARY`; default is `lernie` on `PATH`.
- `git_tree` — walks the repo's linear commit history and produces a
  view-model (`GitTree`) independent of egui. A thin egui widget in the
  same module renders it.

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
