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

## Providers

Model calls are intended to go through **provider adapters** — separate binaries the harness invokes over stdio, one per named provider (see [ARCH §4.4](docs/ARCHITECTURE.md#44-provider-adapters)). Auth, HTTP, and transient retry live inside the adapter; the harness only forwards the env vars the adapter declares. A new provider is a standalone executable, not a harness patch.

The v0.1 Anthropic path is still in-process (see `src/provider/`); extracting it to the adapter contract is planned follow-up work.

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
