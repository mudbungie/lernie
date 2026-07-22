# lernie

[![CI](https://github.com/mudbungie/lernie/actions/workflows/ci.yml/badge.svg)](https://github.com/mudbungie/lernie/actions/workflows/ci.yml)

A git-backed agent harness. Design spec: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
Principles catalog: [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md).
Vocabulary reference: [`docs/TAXONOMY.md`](docs/TAXONOMY.md).

CI runs `make ci` (`fmt-check` + `lint` + `coverage` with the 100% gate) on every push and pull request to `main`. The Rust toolchain is pinned in `rust-toolchain.toml` — CI, the pre-commit gate, and every contributor build under the same `rustc`/`rustfmt`/`clippy`.

## One command surface, two bindings

lernie is defined once as a **command surface** — the set of verbs, their arguments, and their products (ARCH §3.4). It is consumable two ways, and both are the *same* control plane:

- **Exec binding** — run the `lernie` binary: `exec("lernie", args)` with env-var auth. This is what the CLI and every frontend use.
- **Linked binding** — depend on the `lernie` crate and drive the same verb entries in-process. The crate's entire public API is `lernie::cmd` (the `Cli`/`Command` clap surface, one `run` entry per verb, the `Fx`/`Outcome`/`Error` binding seam, and the `prelude` binding preludes). The linked binding is **pin-exact 0.x only** — no semver stability, the posture brazen takes toward lernie.

Parity between the two is enforced mechanically, not by convention: `tests/command_surface_parity.rs` walks the crate's actual public surface (via `syn`) and asserts a bijection with the CLI's introspected verb set (via clap) — the crate exposes nothing public that is not a verb's entry, its arguments, its products, or the binding preludes, and no verb lacks its entry. It rides `make check` (hence the pre-commit hook and GitHub Actions), so a divergence between the linked surface and the CLI fails the build.

## Quickstart

```
make install              # lay down the XDG harness homes + binaries on PATH
lernie new ~/work/chat    # create a workspace (bare repo.git + config/default)
ANTHROPIC_API_KEY=... lernie prompt ~/work/chat 'hello'
```

## Install

```
make install                                  # default: ~/.local/bin, XDG homes
make install INSTALL_PREFIX=/usr/local        # binaries -> /usr/local/bin/
make install LERNIE_HOME=/opt/lernie          # collapse both homes -> /opt/lernie/
```

`make install` runs a release build and then:

1. Installs `lernie` and `agent-eval` into `$INSTALL_PREFIX/bin`
   with `install -m 0755` (atomic overwrite, no symlinks). Make sure
   that directory is on your `PATH`.
2. Installs the provider adapter — brazen's `bz` — with
   `cargo install brazen --version =<pin> --locked`, where the pin is
   the `brazen = "=<pin>"` dependency in `Cargo.toml` — its one home;
   the Makefile and the load-time guard both derive from that line.
   One binary serves every provider
   (ARCH §4.4); the harness resolves `bz` on `PATH`, and a load-time
   guard rejects any `bz` whose version differs from the pin.
3. **Founds the harness root by invoking `lernie prime`** — the single
   verb that seeds the installation substrate (ARCH §2.2), so the
   Makefile no longer duplicates the seeding. `prime` resolves the roots
   (XDG split, collapsed by `LERNIE_HOME`) and lays down the default
   `models.yaml` under the **config root** (model capabilities + context
   windows — no endpoints or auth, which are brazen's), the `tools/` and
   `skills/` pools and the `workspaces/` tree under the **data root**,
   and the empty `workflows/` templates dir. It is **seed-if-absent
   throughout**: a second run changes nothing, and a hand-edited
   `models.yaml` (or any operator-added pool entry) survives a re-install.
   The shipped assets are embedded in the binary, so `prime` needs no
   source tree — `LERNIE_HOME=<dir> lernie prime` seeds any fresh home.
   There is no frozen profile pool: the config a workspace runs under is
   its own `config/default` commit, authored from
   [`template/`](template/) at `lernie new` (fork is the freeze, ARCH §2.2).
4. Smoke-tests the freshly installed binaries with `lernie --version`
   and a throwaway `lernie new`. Failure aborts the install with a
   non-zero exit.

Provider endpoints, auth, and wire dialects live entirely in brazen's
own config (`~/.config/brazen/config.toml`; inspect with
`bz --dump-config`, authenticate with `bz --login --provider <id>`).
lernie references a provider *row* by name and never sees credential
material (ARCH §4.1).

`make uninstall` removes the `lernie`/`agent-eval` binaries; `bz`
(installed via cargo) is removed with `cargo uninstall brazen`. The
harness homes (the config and data roots, holding config and
workspaces) stay put — clean them up manually if you want a true
uninstall.

## Configuration schemas

JSON Schemas for the harness-root and config-commit control files (per
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) §2.2, §4.1) are generated
from the Rust types under `src/config/`. `make schemas` writes them to
`schemas/` for editor integration and external validators. Generation is a
golden test (`config::schemas::write_to` vs the checked-in `schemas/`):
`make schemas` runs it with `UPDATE_SCHEMAS=1` to rewrite the directory,
and the same test under `make check` fails if `schemas/` ever drifts from
the source types — so the tree is always current, with no separate binary
to run.

| File                          | Backed by Rust type                        | Config-commit / on-disk file          |
|-------------------------------|--------------------------------------------|---------------------------------------|
| `schemas/version.json`        | `config::version::Version`                 | `version` (config commit)             |
| `schemas/manifest.json`       | `config::manifest::Manifest`               | `manifest.yaml` (config commit)       |
| `schemas/workflow.json`       | `config::workflow::Workflow`               | `workflow.yaml` (config commit)       |
| `schemas/providers.json`      | `config::per_repo_providers::PerRepoProviders` | `providers.yaml` (config commit, `roles:`) |
| `schemas/models.json`         | `config::models::Models`                   | `<config-root>/models.yaml`           |

## Layout: harness root and workspaces

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
  (default `~/.local/share/lernie`). Holds the `tools/` and `skills/`
  pools plus the `workspaces/` tree. Shared across every workspace.
- **Workspace** — one git repository per workspace, at
  `<data-root>/workspaces/<workspace>/` (ARCH §2.2): a bare `repo.git`
  holding config branches (`config/<name>`) and agent refs
  (`agents/<agent-id>`) — **no `main`**. The control files
  (`providers.yaml` `roles:` only — §4.3, `manifest.yaml`,
  `workflow.yaml`, `version`, `souls/`) live in the **config commit**,
  read from each agent's governing config commit (`git merge-base`
  against the `config/*` heads — derived from ancestry, never stored).
  Agent worktrees are siblings under `agents/<agent-id>/`; `steps/` and
  `inbox/` sit at the workspace root, outside every worktree.
  Workspace repositories are never pushed to a remote.

`lernie new` creates a workspace and authors its **first config commit**
— an orphan root on `config/default` — from [`template/`](template/),
the versioned skeleton embedded into the `lernie` binary at build time:

```
lernie new                     # auto-id under <data-root>/workspaces/
lernie new /path/to/my-workspace
```

Or via the Makefile wrapper:

```
make new-workspace DEST=/path/to/my-workspace
```

The binary runs `git init --bare -b config/default <dest>/repo.git`,
materializes a transient authoring checkout, extracts the template's
control files into it, snapshots the data-root pools into
`descriptions/{tools,skills}/` (ARCH §3.3 descriptions-always), commits
(`config: init [config/default]`), and tears the checkout down. The
workspace is left with exactly one ref — the config commit every fresh
root agent forks off (fork is the freeze, §2.2). The destination must
either not exist or be an empty directory. With no path argument, the
destination is `<data-root>/workspaces/<auto-id>/`; the created path is
printed on stdout. `goal.md` and `soul.md` are intentionally not in the
template — they are written per-branch at dispatch time (ARCH §2.3,
§2.8), which also **removes the control files from the agent's tree**
(§2.2: control is read from the config commit; worktrees hold only
context).

**Pre-v1 clean break (ARCH §10):** the retired per-conversation layout
(a `root/` worktree with loose control files) is refused with an
actionable error, not migrated — create a fresh workspace with
`lernie new`.

**First-run smoke test (required).** `lernie new` authors the default
`providers.yaml` with a concrete model id, but validates it against
nothing — id validity is brazen's fact, and lernie runs no model-list
reconciliation (ARCH §4.2, the settled stance). A wrong id surfaces only
at the first live model call. The required next step after creating a
workspace is therefore a live `lernie prompt` (see the quick start
above): it is the cheapest — and, by that stance, only — check that the
authored id actually resolves on the wire.

## Authoring config commits

`lernie new` authors a workspace's *first* config commit. Every later
one — the general harness-assisted user act of ARCH §2.2 — is
`lernie config`:

```
lernie config <workspace>                       # advance config/default
lernie config <workspace> <name>                # advance config/<name>
lernie config <workspace> <name> --from <src>   # fork config/<name> off config/<src>
lernie config <workspace> <name> --orphan       # fresh orphan lineage
```

The verb materializes a transient checkout of the target config lineage,
refreshes the `descriptions/**` snapshot from the data-root pools (ARCH
§3.3), opens the checkout in `$EDITOR` (falling back to `vi`) so you edit
the control files (`workflow.yaml`, `providers.yaml`, `manifest.yaml`,
`souls/`, `version`), commits, and tears the checkout down. `<name>`
defaults to `default`. `--from` and `--orphan` are mutually exclusive and
only apply when creating a new branch. An authoring pass that changes
nothing is declined (git's empty-commit refusal) — the branch does not
move. This is the **only** act that advances a config branch (ARCH §2.3);
agents forked before it keep their governing config, and agents forked
after it govern under the new head (fork is the freeze, §2.2).

## Sending a prompt

```
lernie prompt /path/to/my-conversation 'hello'
```

`lernie prompt` is the root-conversation path (ARCH §2.3, §2.6, §2.7,
§2.8, §2.10). Each invocation spawns its own `agents/<conv-id>` branch
off the default config branch's head (§2.2–§2.3 — there is no `main`),
drives the model call through brazen's `bz` (§4.4), and runs the
terminal compactor off the tip — the compaction merge lands into the
conversation's own branch. Merge-back is gone (§2.6): the root branch
persists on its own ref (§2.4), and a child returns by depositing a
result message into its parent's inbox (§2.6):

1. Resolve the harness root (`LERNIE_HOME`, else XDG homes, ARCH
   §2.2) and guard the workspace layout (a non-workspace, or the
   retired per-conversation layout, is refused — §2.2, §10). Load
   `<config-root>/models.yaml` (capabilities + context windows +
   optional `adapter:` override — §4.2) and, from the config commit's
   tree (`git show <config-commit>:providers.yaml`, §2.2),
   `providers.yaml` (`roles:` block — §4.3); cross-validate
   `roles.worker.{provider,model}` against `models.yaml`. For a fresh
   root the config commit is `config/default`'s head — the very commit
   the new agent forks off; `lernie advance` derives an existing
   agent's **governing config commit** from ancestry instead
   (`git merge-base` against the `config/*` heads — never stored).
2. Run the load-time version guard: `bz --version` must equal the
   linked brazen crate version (§4.4). Under an `adapter:` override the
   guard is skipped and the in-band `MessageStart.v` handshake governs.
   Read the worker soul from the config commit's `souls/worker.md`
   (§2.2, §4.3).
3. Spawn branch `agents/<conv-id>` (§2.3 — the id is the bare
   hyphenated descent; the `agents/` prefix is the ref namespace) off
   `config/default` and allocate a worktree at
   `<workspace>/agents/<conv-id>/` (§2.2). Write the branch goal to
   `goal.md` and the role soul to `soul.md`, remove the config commit's
   control files from the tree (§2.2 — the worktree holds only
   context), and commit — that commit's tree is step 1's read state
   (§2.10).
4. Build a typed `brazen::CanonicalRequest` (linked crate — the
   fail-open `extra` map stays unreachable), mirror it to
   `<workspace>/steps/<conv-id>/001/request.json` (a diagnostic
   artifact, outside every worktree, never read at runtime, §2.3).
5. **Model call, harness-owned retry loop (§2.10, §4.4).** Exec
   `bz --json --provider <row>` once per *attempt*, canonical request
   on stdin, appending each attempt's stdout verbatim to
   `<workspace>/steps/<conv-id>/<NNN>/response.json` as brazen `v=1`
   NDJSON — one self-delimiting segment per attempt, each ending in a
   terminal `end`. On a retryable in-band `Error`
   (`CanonicalError::retryable()`, never re-derived) the harness
   re-invokes `bz` with the identical request, up to the `workflow.yaml`
   attempt cap with exponential backoff. brazen never retries; auth and
   endpoints are entirely its own. The `response.json` fd is held open
   across every attempt and backoff sleep — its close is the §3.5
   IN_CLOSE_WRITE completion signal. As the events stream, the harness
   tracks only their *framing* — the terminal `end`, an in-band `Error`,
   the handshake `v` — for retry/classification; `meta.json` carries
   `{commit, started_at, ended_at}`. The events' *content* streams into
   the **transcript writer**'s (§2.3) staging file
   `<workspace>/steps/<conv-id>/<NNN>/staging.json`,
   appending each content block as it completes; segment authority
   (§4.4) truncates it on an `Error` attempt and the settling `Finish`
   seals it — one stream, two sinks (diagnostic `response.json` +
   transcript), never read back. When the model call completes, the
   sealed file is renamed into the worktree as
   `messages/NNN-<model-id>.json` — its origin token is the model that
   authored it (§2.3), the body a JSON array of canonical `Content`
   blocks — and committed. `NNN` is the branch's transcript counter,
   max-present-plus-one from the `messages/` listing, evaluated at
   commit time. The initial user message now enters through the front
   door like any other (§2.11): the executor deposits it into the agent's
   own inbox, and the step-boundary drain delivers it as the first
   transcript entry `messages/NNN-user.md` (bl-1129) — no bespoke
   initial-message path beside the drain.
6. **Step loop (§2.5).** At each step boundary the executor first
   **drains the inbox** (bl-1129, §2.11): after committing any
   renamed-but-uncommitted stray a prior death left in `messages/`, it
   moves each pending `inbox/<agent-id>/<sender>-<NNN>.md` into the
   worktree as `messages/<counterNNN>-<sender>.md` (a literal `rename(2)`
   — one home at every instant) and commits the move, in a deterministic
   `(mtime, filename)` order, ahead of the read-state capture so a
   delivered message is part of the commit the model call assembles from.
   Each step then re-assembles its model-facing history
   from the read-state commit's tree — `readdir` of `messages/`, sorted
   by the filename's `NNN` prefix, each entry composed by its origin
   token (`NNN-<sender>.md` → user text, `NNN-<model-id>.json` → the
   assistant message — any `.json` token but the reserved `tool`,
   `NNN-tool.json` → `tool_result` in the following
   user message), with consecutive same-side entries grouped into one
   alternating wire message. There is no in-memory history and no
   git-log walk; running, retry, and replay are one code path against
   one input, the commit's tree (§2.3, §5). If the settled model-output
   entry carries any `tool_use` block, run every one through the tool
   executor — the per-call records land under
   `<workspace>/steps/<conv-id>/<NNN>/tools/<tool-id>/` (out of every
   worktree, §3.3; written but never read at runtime), and as each tool
   resolves the transcript writer commits `messages/NNN-tool.json` (its
   canonical `tool_result` block) — then loop into step `<NNN+1>`. A
   step with no `tool_use` block is terminal. Step ≥2 has no *dispatch*
   commit, but each step's transcript entries (assistant output, tool
   results) do advance the branch tip, which is that step's read state
   (§2.10). `tool_use`/`tool_result` pairing holds by construction: a
   tool result commits immediately after its emitting step's model-output
   entry, so it always lands in the immediately following user message.
7. **Terminal return (§2.6, §2.3 step 5).** Every terminal event —
   normal completion (`final-response`), budget exhaustion
   (`budget-exhausted`, §6), and stop (`stopped`, §2.9 — the executor's
   SIGTERM handler deposits on its way out) — deposits a **result
   message** into the parent's inbox: an ordinary deposit whose
   frontmatter adds `epitaph:`
   and `terminal_ref:` (the branch tip) and whose body is the terminal
   response iff the agent spoke. For a root this is a structural no-op —
   a root has no parent inbox; its response answers the user (§2.4). The
   deposit is executor-side, never a model tool call ("Return is not a
   verb"). At delivery, a message carrying `terminal_ref:` applies the
   fork-point→terminal **work-product transfer** as one commit before its
   delivery commit, filtered to work products; a diff that fails to apply
   is declined at `refs/lernie/conflicted/<agent-id>` (§2.6).
8. On a normal completion, dispatch the terminal compactor (§2.7) off the
   conversation tip by re-entering the binary as `lernie dispatch
   compactor <repo> <conv-id>` (§3.4). The compactor spawns branch
   `<conv-id>-<cmp-id>` (hyphenated descent per §2.2) off the tip, writes
   a placeholder `summary/001.md`, and lands the **compaction merge** —
   a plain `--no-ff` merge into the conversation branch, the one merge
   left in the system (§2.6), conflict-free with no rebase or alignment.
   The compactor is a stub — it does not call a model; the shape exists so
   v0.4+ can layer real semantics without moving call sites. **Merge-back
   is gone (§2.6):** the root branch persists on its own ref (§2.4);
   nothing merges anywhere, and the conversation worktree is not torn
   down (quiescence, not teardown, §2.3 step 6).
9. Print the agent id (the bare conv-id) on stdout.

After `lernie prompt` returns, inspect the conversation against the
bare workspace repository:

```
cd /path/to/my-workspace
git -C repo.git log --oneline --decorate agents/<conv-id> -4
git -C repo.git show --stat agents/<conv-id>  # the compaction merge commit
git -C repo.git show agents/<conv-id>:summary/001.md # terminal summary
```

The root branch persists unmerged by design (§2.4), so the health metric
is no longer branch count but silent deaths and undelivered returns
(ARCH §8) — read straight from git refs, the executor lock, and inbox
listings, with no sidecar file.

## Stopping a conversation

```
lernie stop /path/to/my-conversation <conv-id> [--stop-children]
```

Sends `SIGTERM` to the process group of the **one executor** driving
`<conv-id>`, with a 5-second flush deadline before `SIGKILL`. This is the
same cascade pattern adapter (§4.4) and tool (§3.3) cancellation use,
applied to the harness itself
([ARCH §2.9](docs/ARCHITECTURE.md#29-stopped-branches)). The group signal
reaches that executor's own `bz` and tool subprocesses — its limbs — and
**stops at the agent boundary**: a dispatched child harness has taken its
own process group, so a bare stop does not fell it. A running child
outlives the stopped parent and revives it later by depositing its result
(§2.11) — stopping a parent strands nothing.

`--stop-children` opts into the agent→agent cascade: it walks the id
namespace — the descendants of `<conv-id>` are exactly the inbox
directories prefixed `<conv-id>-` (§2.3), one prefix scan reaching every
depth — and folds each descendant executor's group into the same sweep.
The pid is discovered by scanning `/proc/<pid>/fd/*` for the process
holding the agent's inbox-directory lock fd open — the executor lock
(§2.11), held for the whole step loop, so a stop lands even during tool
execution when no `response.json` is open — no sidecar pid file. Linux
only.

The group signal reaches every member independently: `bz` installs no
handler and dies at once (leaving the missing-`end` signature, §4.4),
while the **executor catches its own copy** — SIGTERM is catchable — and,
instead of dying on the spot, deposits its branch's `stopped` result on
its way out (§2.9 step 3, executor-side, "Return is not a verb") and then
exits cleanly. Catching shields nobody: the kernel already delivered to
`bz` and the tools. For a root the deposit is a no-op (no parent inbox);
the observable is the clean exit.

Behavior:

- **Idempotent.** A branch with no live writer (already stopped, or the
  harness exited cleanly) returns success without sending any signal.
- **Errors when** the agent branch (`agents/<conv-id>`) doesn't exist.
  Surfaces as a non-zero exit with a `lernie stop:` prefix on stderr.
  (The old "already merged" refusal died with `main`: nothing merges,
  so there is no merged state to refuse — an already-terminal branch is
  simply the idempotent no-holder case above.)
- **No on-disk cancel marker.** The §2.9 signature of a stopped branch
  is the latest step's `response.json` closed without a terminal `end`
  event — produced by `bz` dying mid-stream on its own SIGTERM (§4.4);
  the executor's `stopped` deposit is an independent write to the inbox
  tree and never touches that signature.

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
  seeded to `<data-root>/tools/<name>.json` by `lernie prime` (which
  `make install` invokes, ARCH §2.2). Sent verbatim as the
  `input_schema` of the tool's entry in the model call's `tools: [...]`
  array.
- **Skill** — at [`skills/<name>/SKILL.md`](skills/), seeded to
  `<data-root>/skills/<name>/` by `lernie prime`. The frontmatter `description` is
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
  the role must resolve to `souls/<role>.md` and a `roles:` entry in
  `providers.yaml` — both read from the calling branch's governing
  config commit (§2.2). Reads the calling
  conversation's repo + branch from the harness-set
  `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` env vars (ARCH §3.3 env
  bullet); spawns through `lernie dispatch <role>` (§3.4). The handle
  it returns is the child's *address* — there is no polling tool to
  pair with it. The substrate redesign (ARCH §2.5 "Dispatch returns
  the child's address") dissolved the handle/`await` pair: the child's
  result comes back as a **deposit into the parent's inbox** carrying
  an epitaph (§2.6, §2.11), so `await`/`check` had nothing left to
  observe and are gone. The return path — the result-message deposit
  and the delivery-time work-product transfer — is now built (bl-4ce8,
  bl-9f53, §2.6); the epitaph deposits (final-response, budget-exhausted,
  and stop — the executor-side SIGTERM handler, §2.9) are wired at the
  root's terminal events but fire for a child only once children run a
  step loop (`worker.rs` still stops at the dispatch commit), so today
  `dispatch` spawns the child and returns its address while the child does
  not yet reach a terminal event.
- **`message`** — deposits content into an *existing* agent's inbox
  (ARCH §2.11). Input is `{agent, content}`; the recipient is addressed
  by its agent id (its branch name / hyphenated descent). Unlike
  `dispatch` it starts no branch and returns no address — it deposits
  synchronously and returns `{"status":"deposited"}`. The sender is the
  calling agent's id, taken from the harness-set `LERNIE_CONV_BRANCH`
  (never model-supplied), so provenance cannot be forged. It goes
  through the front door — `lernie message` (below) — like `dispatch`
  goes through `lernie dispatch`. **Shipped state:** the deposit lands
  and the step-boundary drain delivers it (bl-1129) — the next driver to
  step the branch moves the inbox file into `messages/` as a transcript
  entry at its next boundary. A deposit into a *quiescent* agent is
  self-delivering: the free-lease probe detach-spawns `lernie advance`
  (§6, below), which acquires the lease, delivers the deposit, and steps
  the branch.
- **`load_skill`** — copies a pooled skill's body into the calling
  agent's worktree at `skills/<name>/`, where the next context assembly
  composes it (ARCH §3.3 *Body-on-demand*, §5.2). Input is `{name}`; the
  data-root pool + target worktree come from `LERNIE_HOME`/XDG and
  `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH`. Returns
  `{"status":"loaded","path":"skills/<name>"}` on a fresh copy or
  `already_loaded` when the worktree already holds it (the loaded copy is
  the snapshot the branch is pinned to; `rm` and reload to refresh). An
  unknown or non-single-component name is declined (`is_error`, naming
  the available pool). **Shipped state:** the copy commits with the tool
  result — a tool commit now stages the whole worktree (`git add -A`,
  `commit_tool`), landing any tool's worktree side effects with its
  result entry (ARCH §2.3).

## Messaging an existing agent directly

`lernie message <workspace> <agent> <content>` deposits a message into
`<agent>`'s inbox and, finding the recipient quiescent, launches a
driver to deliver it (ARCH §2.11, §3.4). The sender is read from
`LERNIE_CONV_BRANCH` — the calling agent's id when the `message` tool
re-enters the verb, else `user` for a bare invocation.

- The deposit is a create-only file at `<workspace>/inbox/<agent>/
  <sender>-<NNN>.md` (temp-path + atomic rename), with `from:` /
  `deposited_at:` frontmatter and the content as its body. `<NNN>` is
  the sender's own sequence, derived as max-present-plus-one over its
  existing files in that inbox.
- After depositing, the verb probes the **executor lock** (`flock` on
  the inbox directory): the same lease the shipped `lernie prompt` step
  loop holds for its whole run, releasing it on exit. A held lease means
  a driver is already stepping the branch (it will deliver at its next
  boundary); a free lease means the branch is quiescent.
- On a free lease the verb launches a driver — `lernie advance
  <workspace> <agent>` (ARCH §6) — as a **detached spawn** (§2.11):
  `setsid` (its own session and process group), stdio to null,
  fire-and-forget. The driver outlives the `lernie message` process, so
  messaging is scriptable: the verb returns as soon as the deposit and
  spawn land, and delivery + stepping continue in the driver.

## Driving a branch: `lernie advance`

`lernie advance <workspace> <agent>` is the §6 driver verb — the
process every launch seam spawns, and the same verb an operator runs by
hand. One invocation is one **hop**: take the lease (adopt the
`LERNIE_LOCK_FD` fd published by a predecessor hop, else try-acquire
the executor lock — losing it is a clean no-op), deliver pending inbox
messages through the real drain (rematerializing a torn-down worktree
first), derive warrant from the transcript tail (ends user-side → a
model call is due; ends assistant-side without `tool_use`, or empty →
exit silently; assistant `tool_use` with uncommitted results → decline
loudly, the one non-replayable state), run one step, and hand off: a
step that emitted `tool_use` runs its tools and **exec's the successor
`lernie advance`** with the lock fd deliberately inherited (close-on-
exec cleared just before exec; the successor fstat-validates the fd
against the inbox directory and restores close-on-exec), while a
terminal event ends the chain through the §2.11 exit protocol. Because
the successor is `exec`'d in the same process, the pid, process group,
and flock lease all survive the hop — `lernie stop` lands on whichever
hop is current, and no rival driver can wedge between hops.

## The exit protocol and the operator scan

Normal operation needs zero scanning (ARCH §2.11): `lernie message`
deposits, probes the executor lock, and launches a driver if the agent
is quiescent; the executor drains its inbox at every step boundary. The
graceful-exit crack — a deposit landing after an executor's final drain
but before its lock release — is closed by the **exit protocol**
(§2.11, bl-5846): one terminal sequence, no agent kinds — deposit the
result message (a structural no-op for a parentless agent) → release
own lock → spawn a driver at own agent, fire-and-forget → exit. Two
pins terminate the recursion: a driver that acquires and finds nothing
to deliver exits silently (no step, no epitaph, no further launch —
`dispatch::driver::drive` is that entry), and the launch is decided by
epitaph value — a final response launches; `stopped` and
`budget-exhausted` never do. The exit launch rides the same launcher
seam as the writer probe, so it is the same detached `lernie advance`
spawn (§6); the decision logic, ordering, driver entry, and the spawn
itself are live and tested.

Crashes are accepted as a failure class (§2.11): everything is on disk,
so a hard death strands results and messages *late*, never lost, and
the next touch heals. That touch is a user reprompt — or the operator
verb **`lernie scan <workspace>`** (§2.11, §8, bl-d148 + bl-5846): one
workspace-wide pass, run by hand or by cron if you want a heartbeat,
never wired into any driver hot path or default schedule (the events it
compensates for happen at crash rate, not step rate). Two derived
actions, no watcher (an idle workspace stays unswept until the next
touch, by design):

- **Silent-death sweep.** Every agent branch with no live executor (the
  §2.11 executor-lock probe) that either died mid-work (its latest step's
  `response.json` closed without a terminal `end`) or — for a child —
  never deposited a result message is a *silent death* (the §8 health
  count). For each hard-crashed **child** in that set, the sweep deposits
  a `died`-epitaph result message *on the child's behalf* (sender = the
  child — the sweep is the scribe, not the author), so the parent is
  revived rather than stalled. The "never deposited" test reads both the
  parent's inbox (undelivered) and its transcript (delivered), so a prior
  sweep's own deposit is seen on re-scan and never re-deposited —
  idempotent by construction.
- **Inbox flush.** Every agent with pending inbox files and a free lock
  gets a driver **launched** — never drained: the scanner moves no files
  and commits nothing; only an agent's own lock-holding executor
  delivers. An agent whose lock is held is left alone. The sweep's own
  deposits are picked up by the flush that follows in the same pass.

**Shipped state.** The scan (silent-death sweep + inbox flush) ships
behind `lernie scan` and *only* there — driver startup (`lernie prompt`,
`lernie dispatch`, `lernie advance`) runs no workspace scan. The flush
and the exit launch reuse the same driver-launch seam as `lernie
message`, and the spawn is real: each seam decides *when* a driver is
needed and detach-spawns `lernie advance` (§6) for it. Because a child
does not yet
run a step loop (the worker path stops at the dispatch commit), a real
"died child" cannot arise from a run today — the derivation is exercised
against constructed on-disk states.

**Namespace note.** The candidate enumeration is the `agents/*` ref
namespace, exactly as ARCH §8 writes it (a root is `agents/<conv-id>`,
a child `agents/<parent>-<sub-id>`); config branches are excluded
structurally by the prefix — there is no `main` (§2.2).

## Dispatching subagents directly

`lernie dispatch <role> <repo> <branch> [--goal <text>]` is the §3.4
re-entry point every subagent uses. The role name is positional, so
the surface generalizes across the v0.3 compactor, the v0.4 worker,
and future roles (verifier, critic, …) without a CLI shape change.

- `lernie dispatch compactor <workspace> <conv-id>` runs the same
  terminal-compaction routine `lernie prompt` triggers internally.
  Useful for re-compacting a conversation branch that still exists as
  a ref, or for testing the compactor path independently. The
  compactor's goal is built-in boilerplate; passing `--goal` is
  rejected. The command runs the compaction + the compaction merge
  into the conversation branch — the one merge in the system (§2.6).
- `lernie dispatch worker <workspace> <parent-id> --goal <text>`
  spawns a worker subagent off the parent's tip. The new id is
  `<parent>-<sub-id>` (hyphenated descent, §2.2), its ref
  `agents/<parent>-<sub-id>` (§2.3), its worktree
  `agents/<parent>-<sub-id>/`; `goal.md` carries the supplied text and
  `soul.md` is read from the parent's governing config commit
  (`souls/worker.md`, §2.2), both committed as the dispatch commit
  (§2.3 step 2). v0.4 Phase 1 stops there — the worker's own step loop
  is a later milestone. The inbox result-return path it will use is
  already built (result-message deposit + work-product transfer, bl-4ce8,
  §2.6); it fires once the child runs a step loop and reaches a terminal
  event.

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

The desktop frontend lives in its own repository, `yog`: an
egui/eframe window that renders a workspace and issues user actions via
`lernie <subcommand>`. It composes on lernie's public surfaces only —
the CLI and the on-disk workspace layout (ARCH §3.5, §7.1) — and takes
no Cargo dependency on this crate, so it builds, versions, and installs
independently (`make install` there drops `yog` next to
`lernie`). Keeping frontends out of this workspace is deliberate:
lernie ships as a composable component, and anything that composes it
(a GUI, a web view) lives outside it and meets it at those surfaces.

## Evaluation: archival and the task suite (§9)

**Archive a run.** A "run" is an agent subtree, not a whole workspace (§9.2).
`lernie bundle <workspace> <agent> <out-dir>` writes the subtree — the
`agents/<agent>` branch and its `agents/<agent>-*` hyphen-descendants (§2.3),
with all the ancestry those refs reach, the governing config commit included
(§2.2) — as one `git bundle`, and copies the matching `steps/<id>*` and
`inbox/<id>*` diagnostic slices beside it. One bundle plus two slices is the
whole run.

```
lernie bundle /path/to/workspace <agent-id> /path/to/archive
```

**Replay a run.** `lernie replay <archive>` reconstructs a scratch workspace
under `LERNIE_HOME`'s data root at `replays/<primary-id>/` (the primary id is
the subtree's root agent), fetches every branch out of the bundle into a
fresh bare `repo.git`, materializes the primary's worktree under `agents/`,
restores the slices, and prints the scratch path. Point the ordinary frontend
at it — replay is not a mode (§2.3). Set `LERNIE_HOME` to an isolated
directory to keep the replay sandboxed. The governing config rides the
bundle's ancestry (§2.2): the config commit is an ancestor of every agent
branch, so no config sidecar exists.

```
LERNIE_HOME=/tmp/replay lernie replay /path/to/archive
```

**Task suite.** The evaluation suite lives as data under `tests/suite/` — 50
tasks with machine-checkable `check` scripts, tagged by the seven §9.1 failure
categories (≥10 per category), format in `tests/suite/README.md`,
well-formedness enforced by `tests/suite.rs`.

**Run the suite.** The `agent-eval` runner (a separate crate, `crates/agent-eval`,
ARCH §9.3) executes an experiment against the suite N times per task and reports
pass@1 (with 95% Wilson intervals) and pass@5, overall and per category:

```
agent-eval --config baseline --suite tests/suite --runs 5
```

`--config <name>` names an experiment — a `workflow.yaml` variant under
`experiments/<name>/` (a config diff, no code changes; see `experiments/README.md`).
Per run the runner seeds a fresh isolated `LERNIE_HOME` and working directory,
runs the task `setup`, invokes the agent, then runs the task `check` — **exit 0
is the sole pass signal** (§9.1), so success is observable state, never the
agent's own claim. The agent invocation is `--agent <cmd>` (an external
harness-driver receiving the prompt, with `LERNIE_HOME`/`LERNIE_EXPERIMENT` in
the env); `--bundle-dir <dir>` archives failing runs for triage via `lernie
bundle` (§9.2). The runner is fully tested against a faked agent, so it needs no
live model to validate — a live-model run of the full suite is a separate,
deliberately manual step.

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

The Rust toolchain is pinned in `rust-toolchain.toml` (channel `1.95.0`, with
`rustfmt`, `clippy`, and `llvm-tools-preview`). rustup reads it automatically
for every `cargo` command in the tree and installs the pinned toolchain on
first use — no manual `rustup` step. This is what keeps `fmt-check` and
`lint` from drifting between your machine, another agent's, and CI.

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
| `make new-workspace DEST=<path>` | Create a workspace (bare repo.git + first config commit from `template/`) |
| `make check`          | `fmt-check` + `lint` + `coverage`                     |
| `make ci`             | Alias for `check`                                     |
| `make install-hooks`  | Point git at `.githooks/`                             |
| `make install` [`INSTALL_PREFIX=<p>` `LERNIE_HOME=<h>`] | Release-build; drop `lernie`/`agent-eval` into `$INSTALL_PREFIX/bin` (default: `~/.local/bin`); install the provider adapter `bz` via `cargo install brazen --version =0.0.2` (the ARCH §4.4 version pin); then invoke `lernie prime` to found the harness root — config root (default `~/.config/lernie`) with a default `models.yaml`, data root (default `~/.local/share/lernie`) with the `tools/`/`skills/` pools and the `workspaces/` tree — seed-if-absent (ARCH §2.2); `LERNIE_HOME` collapses both |
| `make uninstall` [`INSTALL_PREFIX=<p>` `LERNIE_HOME=<h>`] | Remove the installed binaries; leaves the harness homes (config + data roots) in place |

### Workflow

All changes land on `main` via `bl` squash-merges. Direct commits to `main` are
rejected by the pre-commit hook.

```
bl prime --as <you>
bl claim <task-id>              # creates a worktree; cd into it
# ...edit, test, commit...
bl close  <task-id> -m "..."    # squash-merges into main; run from the repo root
```

See `bl skill` for the full guide.

### Pre-commit hook

`.githooks/pre-commit` enforces three rules on every commit:

1. **No direct commits to mainline.** `main` and `master` are rejected unless
   the commit is the tail of a merge (`MERGE_MSG`/`SQUASH_MSG` present), which
   is how `bl close` lands squash-merges.
2. **300-line cap on code files.** The cap is a repo *invariant*, not a
   per-commit property, so the hook sweeps **every tracked code file in the
   tree** (`git ls-files`), not just the staged set — a file that crosses the
   cap in one commit and is untouched afterward is still caught. Docs (`*.md`,
   `*.txt`), config (`*.toml`, `*.yaml`, `*.yml`, `*.json`, `*.lock`),
   `Makefile`, `.gitignore`, `LICENSE`, and anything under `.githooks/` are
   exempt.
3. **`make check`** on every commit that touches a Cargo project: `fmt-check`
   (formatting), `lint` (`clippy -D warnings`), and `coverage` (`cargo
   tarpaulin --fail-under 100`). The hook invokes `make check` rather than
   re-listing the commands, so the close gate is always exactly what `make
   check` is — the Makefile is the single source. Formatting and lint drift
   therefore cannot land invisibly. The toolchain is pinned in
   `rust-toolchain.toml` and the tarpaulin version in `tarpaulin.toml` (also
   `.github/workflows/ci.yml`) so `fmt-check`, `lint`, and the coverage
   denominator mean the same thing locally and on CI — newer tarpaulin
   releases have silently dropped inline `#[cfg(test)] mod tests;` files from
   the count, weakening the floor. `make coverage` aborts with an install
   hint if the local tarpaulin version drifts.

There is no `--no-verify` escape hatch in the workflow. If the hook rejects a
commit, fix the underlying issue rather than skipping.

## License

MIT. See [`LICENSE`](LICENSE).
