# User stories: the promise suite for lernie 0.0.1

**Status:** living document. Deliverable of bl-0135.
**Sources:** [`README.md`](../README.md), [`docs/ARCHITECTURE.md`](ARCHITECTURE.md), [`docs/TAXONOMY.md`](TAXONOMY.md). This document **cites** them; it never restates them.

## 1. What this document is, and its vocabulary

A **user story** here is one promise lernie 0.0.1 makes to someone outside the codebase, written so that a machine can decide whether the shipped binary keeps it. Each story carries five parts, and the definitions below are this document's own — they are terms of art introduced here, not lernie architecture terms:

- **id** — `US-nn`, stable across edits. Renumbering is forbidden; a retired story is struck, not reused.
- **actor** — one of **end user** (drives the CLI by hand), **frontend author** (composes on the public surfaces per ARCH §3.5), **crate consumer** (takes the linked binding as a Cargo dependency), **operator** (installs, sweeps, archives).
- **scenario** — the situation, in one sentence.
- **commands** — the exact invocation under each binding that exists. ARCH §3.4: *"The **exec binding** is the `lernie` binary … The **linked binding** is a Cargo dependency … that constructs the same argument structs and invokes the same entry functions in-process."* Where both exist, both are given.
- **acceptance** — machine-observable state only: exit codes, refs, files on disk, stdout products. Never the agent's own claim. This is the settled §9.1 stance, quoted verbatim: *"a machine-checkable `check` (shell; exit 0 is the sole pass signal, so success is observable state, never the agent's own claim)"*.

**Shipped status vocabulary** (this document's own, again):

| Status | Means |
|---|---|
| **fulfilled** | Verified. The acceptance criteria were observed to hold, either hands-on against a built binary or by a test that exercises the real verb. |
| **partial** | Verified, and part of the acceptance holds while a named part does not. The shortfall is named inline. |
| **unfulfilled** | Verified to not hold. |
| **unverified** | Not checked by this pass. A separate evaluation pass fills these in. Never a guess dressed as a verdict. |

**Verification environment for this pass.** `lernie 0.0.1`, worktree of `main` at `8d8638c`, `cargo build` (debug). Hands-on observations were made at `e3123ba` and re-checked against `8d8638c` after merging; where a later commit changed the answer, the story says so. Hands-on runs used a scratch harness root under `/tmp` (`LERNIE_HOME=<scratch>`) and a live wire through `bz` against the `local` (ollama) provider row, reached by the §4.2 `adapter:` override — see US-25 for why the ordinary path could not be used on this machine. Statuses attributed to tests name the test.

**Second pass: the 0.0.1 outside-evaluation walk.** A later pass walked the *published* 0.0.1 from the public docs only — the crates.io crate, the `v0.0.1` release tarball, and a `make install INSTALL_PREFIX=<p>` from a fresh clone of `main` (`6a4a47d`) — each into an isolated prefix with isolated XDG homes. Statuses it changed say so inline and are marked **(0.0.1 walk)**. It filed six balls: bl-bbba (`advance` accepted a nonexistent agent), bl-63c1 (a missing `bz` named neither `bz` nor the fix), bl-33ef (the two binary distribution routes were undocumented), bl-8efa (`lernie new` at an existing *file*), bl-55e0 (`lernie config --from <nonexistent>` leaks git argv), bl-4bd1 (the built-in tool set is undiscoverable from the CLI).

**Terminology.** All lernie terms below are used in the senses ARCH §2.1 and `docs/TAXONOMY.md` pin. The banned bare forms ("call", "turn", "session", "compression") do not appear.

---

## 2. Install and substrate

### US-01 — the operator installs lernie and gets a ready harness root

- **actor** operator
- **scenario** A fresh machine with no lernie state. One command must produce runnable binaries and a founded harness root.
- **commands** (three routes; only the third is a Makefile act)
  ```
  cargo install lernie --locked           # (a) crates.io
  <the v* release tarball>                # (b) GitHub release asset
  make install                            # (c) ~/.local/bin, XDG homes
  make install INSTALL_PREFIX=/usr/local
  make install LERNIE_HOME=/opt/lernie
  ```
- **acceptance**
  - exit 0.
  - `$INSTALL_PREFIX/bin/lernie`, `$INSTALL_PREFIX/bin/agent-eval`, and `$INSTALL_PREFIX/bin/lernie-eval-agent` exist, mode `0755` — the three `PATH_BINARIES` the Makefile installs. *(The third was missing from this clause until the 0.0.1 walk counted them on disk.)*
  - `bz` resolves on `PATH` at the version `Cargo.toml`'s `brazen = "=<pin>"` names.
  - Routes (a) and (b) lay down `lernie` alone and run nothing after it — no `bz`, no `prime`. That is acceptable *only if documented*, since both are publicly reachable artifacts: README §Install must state, per route, what is laid down, that `bz` at the pin is required for any prompting, the literal `cargo install brazen --version =<pin> --locked`, and where the harness root goes.
  - The harness root carries what US-02 asserts, because install founds it *through the verb* — README: *"**Founds the harness root by invoking `lernie prime`** — the single verb that seeds the installation substrate (ARCH §2.2), so the Makefile no longer duplicates the seeding."*
  - The install's own smoke step passes: `lernie --version` exits 0 and a throwaway `lernie new` exits 0.
- **status** **fulfilled (0.0.1 walk).** Route (c) was walked hands-on in the `INSTALL_PREFIX` form and was flawless end to end: exit 0, three binaries at `0755`, `bz` recognised as already-at-pin by `install-bz`, `prime` run through the verb, `install-verify` passing, and a closing banner naming the PATH, both roots, and the `bz --dump-config` / `bz --login` follow-ups. `lernie --version` printed `lernie 0.0.1 (brazen 0.0.4)`. Routes (a) and (b) were walked into isolated prefixes too and both **worked** — `lernie --version`, `lernie new`, and the harness root founding all held — but each laid down `lernie` alone with **no documentation anywhere for the route**, so a first-time user hit `lernie prompt` and stopped dead at a missing `bz`. That was the gap, and it was a docs gap, not a mechanism gap: **RESOLVED (bl-33ef)** — README §Install now covers all three routes with what each lays down, the required `bz` at the pin with the literal install command, and where the harness root goes; the release tarball now carries `README.md` and `LICENSE` beside the binary; and `src/prompt/tests/pin.rs::every_brazen_version_the_readme_spells_is_the_pin` holds every pin the README spells equal to `brazen_pin()`, so the prose cannot drift (the third recurrence of that class — bl-143e, bl-8c92). The runtime half of the same stranding is bl-63c1 (US-24). `tests/install.rs::make_install_lays_down_skeleton_idempotently` is now under the gate too, via the uninstrumented `test-install` step in `make check` (bl-f01f landed).

### US-02 — `lernie prime` founds a harness root and never clobbers

- **actor** operator; also any nested world that owns a `LERNIE_HOME`
- **scenario** Seed a fresh home; re-seed a live one without losing hand edits.
- **commands**
  - exec: `LERNIE_HOME=<dir> lernie prime`
  - linked: `lernie::cmd::prime::run(prime::Args {}, &mut fx)`
- **acceptance**
  - exit 0, **and stdout is empty** — ARCH §2.2: *"`prime` is product-less: it prints nothing on success (the CLI's one-product convention, §3.4), reporting only failures."*
  - Under the config root: `models.yaml`, `workflows/`.
  - Under the data root: `tools/<name>.json` and `skills/<name>/SKILL.md` for each of `bash`, `dispatch`, `load_skill`, `message`, `read_file`; plus `workspaces/`.
  - A second run exits 0 and changes **no** byte of a hand-edited `models.yaml` or any operator-added pool entry — ARCH §2.2: *"It is **seed-if-absent throughout — it creates what is absent and never clobbers what exists**"*.
  - No source tree is read: the assets are embedded (`include_dir`), so seeding works from an installed binary alone.
- **status** **fulfilled.** Observed hands-on: prime on an empty `LERNIE_HOME` produced exactly the tree above with empty stdout and exit 0; the second run left `models.yaml`'s md5 unchanged. `tests/prime_cli.rs::prime_founds_a_fresh_nested_home_idempotently` asserts the same through the real binary.

---

## 3. Workspaces and configs

### US-03 — the end user creates a workspace with its first config commit

- **actor** end user
- **scenario** Start a place for agents to live.
- **commands**
  - exec: `lernie new` (auto-id) or `lernie new /path/to/ws`
  - linked: `lernie::cmd::new::run(new::Args { path: Some(dest) }, &mut fx)` → `Outcome::Line(<dest>)`
- **acceptance**
  - exit 0; **stdout is the created absolute path and nothing else** (the verb's one product).
  - `<dest>/repo.git` is a bare repository whose **only** ref is `config/default` — README: *"The workspace is left with exactly one ref — the config commit every fresh root agent forks off (fork is the freeze, §2.2)."* No `main`.
  - `config/default` is an **orphan root**: exactly one commit, subject `config: init [config/default]`.
  - Its tree holds `version`, `manifest.yaml`, `providers.yaml`, `workflow.yaml`, `souls/worker.md`, `souls/compactor.md`, and a `descriptions/{tools,skills}/` snapshot of the data-root pools (ARCH §3.3 descriptions-always).
  - `goal.md` and `soul.md` are **absent** — they are written per-branch at dispatch (ARCH §2.3 step 2).
  - No transient authoring checkout survives.
- **status** **fulfilled.** Observed hands-on: `lernie new` printed the path, `git branch -a` showed only `config/default`, `git log --oneline config/default` showed the single `config: init` commit, and `ls-tree -r` matched the list above exactly. `src/template/tests_scaffold.rs` asserts each clause against the real binary (`workspace_has_exactly_one_ref_and_no_main`, `config_commit_is_an_orphan_root_with_one_commit`, …).

### US-04 — `lernie new` declines rather than damages

- **actor** end user
- **scenario** Point `new` at an occupied directory, or at the retired pre-substrate layout.
- **commands** exec: `lernie new <occupied-path>`
- **acceptance**
  - Non-empty destination → exit 1, stderr `lernie new: destination <path> already exists and is not empty`, and **nothing is written**.
  - Destination exists and is not a directory (e.g. a plain file) → exit 1, stderr `lernie new: destination <path> already exists and is not a directory`, and **nothing is written**.
  - An existing *empty* directory is accepted.
  - The retired per-conversation layout is **refused with an actionable error, never migrated** — ARCH §10: *"The retired pre-substrate layout … is **refused, not migrated**: every verb's layout guard declines it with an actionable error naming what was found and what the current layout is."*
- **status** **fulfilled (0.0.1 walk).** The non-empty refusal and the empty-directory acceptance were observed hands-on (exit 1 with that literal stderr; exit 0 respectively) and are covered by `src/template/tests_scaffold.rs::binary_refuses_non_empty_destination`. The **retired-layout refusal was then observed hands-on** by the 0.0.1 walk against a constructed retired-layout directory: the guard declined with the actionable error naming what was found and what the current layout is, exit 1, nothing written. The promise therefore holds in full; what remains is an *evidence* gap, not a shortfall — the automated proof for `new` specifically is still unit-only (`src/workspace/tests.rs::require_refuses_the_retired_layout_with_an_actionable_error`), with integration coverage only at `stop` and `bundle`.

  The non-directory case (bl-8efa: an existing plain file said the bare `I/O error: Not a directory (os error 20)` instead of naming the path and the rule) is now the same guard's third arm, same voice, exit 1, nothing written — `src/template/tests_dest.rs::check_dest_rejects_a_plain_file`, `src/template/tests.rs::scaffold_refuses_a_plain_file_dest`, `src/cmd/tests/verbs.rs::new_at_an_existing_plain_file_names_the_rule_not_the_errno`, and observed hands-on.

### US-05 — the end user authors a later config commit

- **actor** end user
- **scenario** Change souls, roles, workflow, or manifest for agents forked from now on.
- **commands**
  - exec: `lernie config <ws>` / `lernie config <ws> <name>` / `... --from <src>` / `... --orphan`
  - linked: `lernie::cmd::config::run(config::Args { workspace, name, from, orphan }, &mut fx)`; the `$EDITOR` spawn is `Fx::editor`, supplied by the binding (ARCH §3.4 *"Process effects stay at the binding"*).
- **acceptance**
  - Advance: `config/<name>` gains exactly one commit; `descriptions/**` is refreshed from the data-root pools in the same pass.
  - `--from <src>`: a new `config/<name>` exists and `git merge-base config/<name> config/<src>` **succeeds**.
  - `--orphan`: a new `config/<name>` exists and `git merge-base` against any other config branch **fails**.
  - `--from` with `--orphan` → exit 1.
  - An authoring pass that changes nothing is **declined** and the branch does not move — README: *"An authoring pass that changes nothing is declined (git's empty-commit refusal) — the branch does not move."*
  - No agent branch moves; this is the only act that advances a config branch (ARCH §2.3 branch advancement).
- **status** **fulfilled (0.0.1 walk).** Advance, `--from`, and `--orphan` are proven through the real binary with a scripted `$EDITOR` by `tests/config_cli.rs::config_verb_advances_forks_and_orphans_via_editor`, including the `merge-base` succeeds/fails pair. The two clauses that were unit-only — the `--from`/`--orphan` exclusion and the empty-commit decline — were **both observed hands-on through the binary** by the 0.0.1 walk: the flag pair exits 1, and an authoring pass that changes nothing is declined with the branch not moving. As with US-04, what remains is an evidence gap rather than a shortfall: the automated proofs are still the unit tests (`src/template/authoring/tests.rs::a_no_op_edit_is_declined_as_an_empty_commit`, `::from_cli_declines_from_and_orphan_together`).

  **Separately found by the same walk and still open:** `lernie config --from <nonexistent>` dumps the raw git argv and the internal `.config-author` path instead of naming the missing lineage. Filed as bl-55e0.

---

## 4. Prompting: the root path over a live wire

### US-06 — the end user sends a prompt and gets an agent on a ref

- **actor** end user
- **scenario** The first real model call in a fresh workspace.
- **commands**
  - exec: `lernie prompt /path/to/ws 'hello'`
  - linked: `lernie::cmd::prompt::run(prompt::Args { repo, message }, &mut fx)` → `Outcome::Line(<agent-id>)`. The binding must first perform `Command::Prompt.preludes()` — `become_pgid_leader` and `install_stop_handler` — or US-17 cannot land on it.
- **acceptance**
  - exit 0; **stdout is the bare agent id**, containing no `/` (the `agents/` prefix is the ref namespace, ARCH §2.3).
  - `refs/heads/agents/<id>` exists in `<ws>/repo.git`.
  - Its history is, oldest first: `config: init [config/default]` → `step 001: dispatch [<id>]` → `transcript 001: user [<id>]` → `transcript 002: <model-id> [<id>]`. The initial user message rides the **inbox front door**, not a bespoke path — ARCH §2.11: *"The initial user message now rides this same path: the root executor deposits it into the agent's own inbox and the step-1 drain delivers it."*
  - The worktree at `<ws>/agents/<id>/` holds `goal.md`, `soul.md`, `descriptions/`, `messages/` — and **no** `providers.yaml`/`workflow.yaml`/`manifest.yaml`/`version` (ARCH §2.2: control is removed from the tree at the dispatch commit).
  - `messages/NNN-<model-id>.json` is a JSON array of canonical `Content` blocks; `messages/NNN-user.md` is the delivered user message. The origin token is the model id, not a role.
  - `config/default` does **not** advance.
  - The wire actually answered: no `{"type":"error"}` line in any `steps/<id>/*/response.json`, and at least one `{"type":"content_delta"}`.
- **status** **fulfilled.** Observed hands-on over a live wire (local ollama via the §4.2 `adapter:` override): exit 0, id `20260725T001733Z-0bc74bb7` with no slash, the exact four-commit history above, `messages/002-qwen3.5:9b.json` = `[{"type":"text","text":"pong"}]`, control files absent from the worktree, `response.json` terminating `usage` → `finish` → `end` with no `error`. `src/e2e/prompt_end_to_end.rs::prompt_subcommand_persists_conversation_without_terminal_compaction` asserts the same over real `bz` against a mocked endpoint.

### US-07 — the frontend author reads step diagnostics without reading context

- **actor** frontend author
- **scenario** Render a live model call and classify its outcome from disk alone.
- **commands** none — this is the read half of the ARCH §3.5 contract.
- **acceptance**
  - `<ws>/steps/<id>/<NNN>/` holds `meta.json`, `request.json`, `response.json`, and `tools/<tool-id>/` where tool calls ran. The tree sits **outside every worktree** and is untracked by git.
  - `response.json` is NDJSON of canonical events, one **attempt segment** per adapter invocation, each terminated by `{"type":"end"}`. A retryable in-band `Error` produces a further segment; the last is authoritative.
  - Liveness classification is derived, never a sidecar: `live` = the executor holds the inbox-directory lock; `in_flight` = the latest `response.json` fd is still open; `quiescent` = closed and ending in `end`; `stopped` = closed and **not** ending in `end`.
  - Nothing under `steps/` is read for *content* at runtime — ARCH §2.3: *"**response.json content: never read, by anyone, for anything.**"*
- **status** **partial.** The step-record layout, the terminal `end`, and the multi-segment retry shape were observed (hands-on for the single-segment case; `src/e2e/prompt_retry.rs::retryable_529_then_clean_writes_two_segments_and_completes` drives a raw TCP server through 529-then-clean and asserts exactly two segments). The `stopped` signature is proven by `src/e2e/stop_cli.rs::stop_cascades_sigterm_and_leaves_response_without_terminal_end`. **`in_flight` (the fd-open sub-state) is not asserted by any test as a frontend-observable fact** — no test opens the inotify/`IN_CLOSE_WRITE` seam README and ARCH §3.5 promise a frontend.

---

## 5. Built-in tools

Each built-in is the ARCH §3.3 triple — binary (`lernie tool <name>`), JSON schema, `SKILL.md`. Common acceptance for all of US-08…US-12: the invocation reads `tool_use.input` JSON on stdin, writes raw bytes on stdout, exits 0 on success and non-zero on failure; a per-call diagnostic record lands at `steps/<id>/<NNN>/tools/<tool-id>/{input.json,output.json}`; and the model-facing result is the committed transcript entry `messages/NNN-tool.json`, never the diagnostic record.

Linked binding for every one of them: `lernie::cmd::tool::run(tool::Args { name }, &mut fx)` → `Outcome::Code(n)`, with stdin/stdout/stderr supplied as `Fx::tool_stdin` / `tool_stdout` / `tool_stderr`.

### US-08 — `read_file`

- **actor** end user (directly) or an agent
- **commands** `echo '{"path":"README.md"}' | lernie tool read_file`
- **acceptance** exit 0 and the file's bytes on stdout; a missing path exits 1 with `lernie tool read_file: open <path>: …` on stderr; a file over 1 MiB is **rejected, not truncated**.
- **status** **fulfilled** for the read and the miss (observed hands-on: exit 0 with README bytes; exit 1 with that literal stderr). The oversize rejection is unit-covered (`src/prompt/tool/builtin/read_file/tests.rs`). Note the deferred piece is honestly deferred, not silently missing: ARCH §3.3 — *"Oversized-output auto-dispatch … is not in v0.3."*

### US-09 — `bash`

- **actor** end user (directly) or an agent
- **commands** `echo '{"command":"ls"}' | lernie tool bash`
- **acceptance** exit 0 with the command's stdout; the shell's own non-zero code propagates verbatim as lernie's exit code; the shell runs in its own process group so a harness SIGTERM reaches the whole spawned tree (ARCH §2.9 cascade).
- **status** **fulfilled.** Observed hands-on: `{"command":"echo hi"}` → `hi`, exit 0; `{"command":"exit 3"}` → **exit 3**, the code passed straight through. The signal cascade is covered by real SIGTERM/SIGKILL tests in `src/prompt/tool/builtin/bash/tests.rs` and end-to-end by `src/e2e/stop_cli.rs::stop_lands_during_tool_execution_via_inbox_lock_fd`.

### US-10 — `dispatch`

- **actor** an agent
- **commands** `echo '{"role":"worker","goal":"…"}' | lernie tool dispatch`, with `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` set by the harness
- **acceptance** exit 0 and stdout `{"status":"in_progress","handle":"<parent>-<sub-id>"}`; the branch `agents/<parent>-<sub-id>` exists; the role resolved against **both** `souls/<role>.md` and a `roles:` entry in the governing config commit's `providers.yaml`, and an unresolvable role exits non-zero. The handle is the child's **address**, not a poll token — ARCH §2.5: *"A dispatch's `tool_result` is the **child's address** (its agent id, §2.11), never its result and never a handle to poll: there is nothing to await."*
- **status** **fulfilled.** `tests/dispatch_tool_e2e.rs::dispatch_tool_returns_handle_and_spawns_worker_branch` drives the real `lernie tool dispatch` and asserts the JSON product plus the spawned branch; `::dispatch_tool_surfaces_unknown_role_as_nonzero_with_stderr_message` covers the decline. Note `status` is a hardcoded constant (`STATUS_IN_PROGRESS`), not a state read — correct given there is nothing to poll, but it is not an observation.

### US-11 — `message`

- **actor** an agent
- **commands** `echo '{"agent":"<id>","content":"…"}' | lernie tool message`
- **acceptance** exit 0 and stdout `{"status":"deposited"}`; a create-only file appears at `<ws>/inbox/<agent>/<sender>-<NNN>.md`; the sender is `LERNIE_CONV_BRANCH`, **never** the model's input — ARCH §2.11: *"The sender identity is taken from `LERNIE_CONV_BRANCH` (§3.3) — harness-derived, never model-supplied, so an agent cannot forge provenance."* No branch is created and no address is returned.
- **status** **fulfilled (0.0.1 walk).** The deposit semantics, the unforgeable sender, and the JSON product are covered at unit level (`src/prompt/tool/builtin/message/tests.rs`, 13 tests, with the re-entry sender injected), and the 0.0.1 walk drove `lernie tool message` **as a real subprocess** against a live workspace: exit 0, the `{"status":"deposited"}` product on stdout, and the create-only inbox file on disk with the harness-derived sender. The shim is therefore observed, not merely inferred; the standing evidence gap is that no *test* drives it as a subprocess.

### US-12 — `load_skill`

- **actor** an agent
- **commands** `echo '{"name":"bash"}' | lernie tool load_skill`
- **acceptance** exit 0 and stdout `{"status":"loaded","path":"skills/<name>"}` on a fresh copy, `already_loaded` when the worktree already holds it; the pooled body is **copied** (not symlinked) into `<ws>/agents/<id>/skills/<name>/`; the copy is committed with the tool result because a tool commit stages the whole worktree (`git add -A`); an unknown name or a non-single-component name is **declined** with `is_error`, naming the available pool — never fuzzy-matched, never sanitized (ARCH §3.3).
- **status** **fulfilled (0.0.1 walk).** All of the above is unit-covered (`src/prompt/tool/builtin/load_skill/tests.rs`, 13 tests) and the pool seeding is integration-proven (`tests/prime_cli.rs`). The 0.0.1 walk then drove `lernie tool load_skill` **as a real subprocess**: the `loaded` product with the `skills/<name>` path on a fresh copy, `already_loaded` on the repeat, and the copied body present in the agent's worktree. The standing evidence gap is that no *test* drives it against the real binary.

  **Separately found by the same walk and since fixed (bl-4bd1):** the built-in tool set was undiscoverable from the CLI — `lernie tool --help` showed a bare `<NAME>` and the unknown-tool decline listed nothing, while `load_skill`'s sibling decline named its whole pool. Both surfaces now render the pool from one list (`prompt::tool::builtin::NAMES`): `lernie tool --help` documents `<NAME>` as `Built-in tool to run; one of: bash, dispatch, load_skill, message, read_file`, and an unknown name declines non-zero with `unknown built-in tool: "<name>"; available: bash, dispatch, load_skill, message, read_file`. Every verb's positionals gained help text in the same pass.

---

## 6. Messaging, driving, sweeping, stopping

### US-13 — the end user messages an agent and it wakes up

- **actor** end user (or an agent, via US-11)
- **scenario** Reprompt a quiescent root, or steer a running one. ARCH §2.4: *"**Reprompt is a message.**"* There is no separate resume verb.
- **commands**
  - exec: `lernie message <ws> <agent> 'content'`
  - linked: `lernie::cmd::message::run(message::Args { workspace, agent, content }, &mut fx)` → `Outcome::Quiet`. `Fx::driver_target` is the executable the launch seam spawns — ARCH §2.11: *"**The driver target is injected at the binding, not resolved by name**"*.
- **acceptance**
  - exit 0, **stdout empty**, and the verb **returns immediately** — it launches a driver, it does not become one (ARCH §2.11 Writer/driver totality).
  - A create-only `<ws>/inbox/<agent>/<sender>-<NNN>.md` appears with `from:` and `deposited_at:` frontmatter and the content as body; `<sender>` is `user` for a bare invocation.
  - If the agent was quiescent, a detached `lernie advance` runs to completion without the caller waiting: the inbox file disappears, a `transcript NNN: <sender>` delivery commit lands, and a further model-output commit follows.
- **status** **fulfilled.** Observed hands-on end-to-end: `lernie message` on a quiescent root returned at once with exit 0 and empty stdout; 45 s later the branch had gained `transcript 003: user` and `transcript 004: <model-id>`, the inbox was empty, and `steps/<id>/002/` existed. `src/e2e/advance_cli.rs::message_launches_a_detached_advance_chain_that_batons_through_tools` proves the same chain with a tool call in the middle.

  **Shortfall found during this pass, since RESOLVED at `8e79e76` (bl-bdc7)** — kept as the record of the find (G5 below). At the time of writing the verb did **not** validate that the recipient exists: `lernie message <ws> ghost 'hi'` exited 0 and created `inbox/ghost/user-001.md` for a branch that was never dispatched, against ARCH §2.11's *"content addressed to an **existing** agent"*. It now declines twice over: the id must be a single path component (`src/name.rs::require_agent_id`, so a `..`/absolute id is refused rather than sanitized) and an `agents/<id>` ref must exist (`workspace::agent_exists`), giving `lernie message: no agent "X" in this workspace …` and exit 1. Covered by `src/cmd/tests/agent_id.rs` (7 tests) and `src/e2e/message_cli.rs` (3 tests through the real binary).

### US-14 — the operator drives a branch by hand with `lernie advance`

- **actor** operator; also every launch seam
- **scenario** One hop of the workflow chain — the same verb whether a human or a launcher runs it.
- **commands**
  - exec: `lernie advance <ws> <agent>`
  - linked: `lernie::cmd::advance::run(advance::Args { workspace, agent }, &mut fx)` → `Outcome::Exec(cmd)` or `Outcome::Quiet`. The `execve` is the **binding's** act, not the library's (ARCH §3.4).
- **acceptance**
  - The agent id is guarded before anything else: a single path component (ARCH §2.3) **and** an `agents/<id>` ref must exist — a name that is no agent is refused with `lernie advance: no agent "<id>" …` and exit 1, ahead of the lease, so no `inbox/<id>/` is created. This is deliberately *not* the lease no-op below: that one is a live agent already driven, this one is no agent at all.
  - Losing the lease is a **clean no-op**: exit 0, empty stdout, nothing written.
  - An empty inbox on a terminal branch exits **silently** — the recursion terminator of ARCH §2.11 pin 1.
  - Warrant is derived from the transcript tail: user-side tail → a model call is due; assistant-side without `tool_use`, or empty → silent exit; assistant `tool_use` with uncommitted results → **declined loudly** (the one non-replayable state).
  - A step that emitted `tool_use` runs its tools and then `execve`s the successor **in the same process**: pid, process group, and `flock` lease all survive the hop, and `LERNIE_LOCK_FD` is adopted and fstat-validated by the successor.
- **status** **fulfilled.** `src/e2e/advance_cli.rs::message_launches_a_detached_advance_chain_that_batons_through_tools` drives real subprocesses and real `bz` through three model calls and a real `bash` tool call, proving the exec baton and the lease inheritance; `::advance_on_a_name_with_no_agent_ref_refuses_and_mints_no_inbox` and `::advance_verb_surfaces_an_unusable_workspace_loudly` cover the refusals. Warrant derivation is covered by `src/prompt/dispatch/advance/tests.rs`.

  **Shortfall found by the 0.0.1 outside-evaluation pass, since RESOLVED (bl-bbba).** The existence half of the id guard was never actually applied at `advance`, only the path-component half — so `lernie advance <ws> ghost` exited 0 in silence *and* minted `inbox/ghost/`, manufacturing exactly the orphan directory `lernie scan` reports as debris (G5 below recorded the guard as applied at all five verbs; for `advance` that was true of the shape check only). The guard now lives in `src/prompt/dispatch/advance/cli.rs::cli_run_with`, ahead of `take_lease`, in the `lernie message` voice. Covered by `src/cmd/tests/agent_id.rs::advance_declines_an_agent_that_does_not_exist` (exact line + no directory), the e2e above, and the unit refusal in `src/prompt/dispatch/advance/cli.rs`.

### US-15 — the operator sweeps a workspace after a crash

- **actor** operator
- **scenario** A hard death (SIGKILL, OOM, panic) stranded a result or left mail undelivered. ARCH §2.11: *"Crashes are a failure class, not a scan trigger."*
- **commands**
  - exec: `lernie scan <ws>`
  - linked: `lernie::cmd::scan::run(scan::Args { workspace }, &mut fx)` → `Outcome::Line(<report>)`
- **acceptance**
  - exit 0; stdout is exactly `silent deaths: <n>; died deposits swept: <n>; drivers launched: <n>`.
  - Silent-death sweep: for each hard-crashed **child** with no live executor that never deposited, a `died`-epitaph result message is deposited **as the child** (sender = the child), and a re-scan does not re-deposit it.
  - Inbox flush: every agent with pending files and a free lease gets a driver **launched, never drained** — the scanner moves no files and commits nothing.
  - It runs **only** here: no driver startup performs a workspace sweep.
- **status** **fulfilled.** Observed hands-on: `lernie scan` on a workspace holding one stranded child result (stranded by the pre-`cbfaaef` revival gap of US-17, which supplied a convenient real specimen — not a state normal operation now produces) printed `silent deaths: 0; died deposits swept: 0; drivers launched: 1`, exit 0; the launched driver then delivered the message (inbox emptied, `transcript 005: <child-id>` landed on the parent) — proving the flush's launch is real, not a report. `src/e2e/scan_cli.rs::scan_verb_heals_a_crash_stranded_child` asserts the `died` deposit and the `silent deaths: 1` line; `::prompt_hot_path_runs_no_workspace_scan` and `::dispatch_hot_path_runs_no_workspace_scan` assert the no-hot-path rule. The `died` derivation itself is exercised against constructed on-disk states, which ARCH §2.11 states plainly and is its honest unit — reproducing a SIGKILL mid-run deterministically is impractical.

### US-16 — the end user stops one agent, and only that agent

- **actor** end user; the frontend's stop button execs this exact verb
- **scenario** Kill a runaway agent without felling its children, or with them.
- **commands**
  - exec: `lernie stop <ws> <agent>` / `lernie stop <ws> <agent> --stop-children`
  - linked: `lernie::cmd::stop::run(stop::Args { repo, branch, stop_children }, &mut fx)` → `Outcome::Quiet`
- **acceptance**
  - The **executor exits 0**, not by dying: SIGTERM is catchable, the handler sets a flag, and the loop deposits its `stopped` epitaph on the way out. For a root the deposit is a structural no-op.
  - The stopped step's `response.json` closes **without** a terminal `end` — that absence is the on-disk signature; no cancel marker is written.
  - Idempotent: a branch with no live writer returns success with no signal sent.
  - A missing agent branch → exit 1, stderr prefixed `lernie stop:`.
  - The pid is found by scanning `/proc/<pid>/fd/*` for the holder of the agent's **inbox-directory lock fd**, so a stop lands even mid-tool-call. No sidecar pid file. Linux only.
  - Bare stop **stops at the agent boundary**; `--stop-children` folds every `<agent>-` prefixed descendant into the one sweep.
- **status** **fulfilled.** The single-agent stop is fully proven with real signals: `src/e2e/stop_cli.rs::stop_cascades_sigterm_and_leaves_response_without_terminal_end` (executor exits 0, no terminal `end`, ref persists) and `::stop_lands_during_tool_execution_via_inbox_lock_fd` (lock-fd discovery, not response-fd). Idempotence and the missing-branch error are covered by `src/e2e/stop_idempotence.rs`; the missing-branch exit 1 with `lernie stop: branch "nosuch" does not exist in repo` was also observed hands-on. The cascade and the boundary are proven against two **live** executors in `src/e2e/stop_children.rs` (bl-3761): a root mid-model-call plus a dispatched child mid-model-call, in distinct process groups. `::stop_children_fells_the_live_child_executor` asserts the child's executor is gone, its `response.json` closed without a terminal `end`, and its `epitaph: stopped` result deposited in the parent's inbox (the child case of the §2.9 step-3 exit, where the deposit is not the root's structural no-op); `::bare_stop_leaves_the_live_child_running` asserts the negative — after a bare stop reaps the parent, the child still holds its inbox lock under its original pgid. The walk's enumeration remains unit-covered too (`src/prompt/stop/tests/orchestration.rs`, `src/prompt/stop/cascade.rs`).

---

## 7. Dispatch and compaction

### US-17 — a dispatched child runs its own step loop and returns

- **actor** an agent (via US-10) or an operator by hand
- **scenario** Fan work out to a child and get its result back.
- **commands**
  - exec: `lernie dispatch worker <ws> <parent-id> --goal '<text>'`
  - linked: `lernie::cmd::dispatch::run(dispatch::Args { role, repo, branch, goal }, &mut fx)`. Prelude: `become_pgid_leader`.
- **acceptance**
  - exit 0; **stdout is the child's agent id** `<parent>-<sub-id>`.
  - `agents/<parent>-<sub-id>` exists, forked off the parent's tip, with a `dispatch: <role>` commit pinning `goal.md` and `soul.md` (the role's soul read from the governing config commit) and the control files removed.
  - `--goal` is **required** for `worker` and **rejected** for `compactor`; a role absent from the config's `roles:` is refused.
  - The child then **runs**: its branch gains a delivery commit for the dispatch message and at least one model-output transcript entry, and `steps/<child-id>/` appears.
  - At its terminal event the child deposits a result message into its parent's inbox carrying `epitaph:` and `terminal_ref:` frontmatter, with the terminal response as body **iff the agent spoke**.
  - At delivery, a message carrying `terminal_ref:` applies the fork-point→terminal **work-product transfer** as one commit before its delivery commit, filtered to work products; a diff that fails to apply is declined at `refs/lernie/conflicted/<agent-id>`.
- **status** **partial — and better than the README says.** Observed hands-on over a live wire: `lernie dispatch worker` printed the child id, exit 0; the child branch gained `dispatch: worker` → `transcript 005: <parent-id>` (the delivered dispatch message) → `transcript 006: <model-id>`; `steps/<child-id>/` appeared; and a result message landed at `inbox/<parent>/<child>-001.md` with `epitaph: final-response` and `terminal_ref: e327bb3…`. **This falsifies README's claim that a child does not reach a terminal event** (G3 below). The body was empty because that child's terminal entry held only a `thinking` block — correct per ARCH §2.6 (*"terminal response — present iff the agent spoke"*), since thinking is not speech.

  **Parent revival: was broken at `e3123ba`, fixed at `cbfaaef` (bl-4a6c).** Observed hands-on at the earlier commit: the child's result sat in the parent's inbox for 60 s with the parent unadvanced, and only an explicit `lernie scan` delivered it — `inbox::deposit_child_result` deposited and returned without launching. ARCH §2.5 and §2.11 promise the opposite (*"with no step in flight, the child's deposit **revives** it — the deposit starts a driver"*). bl-4a6c closed it by putting the wake in the terminal path (`dispatch::terminal`'s `revive_parent`, deliberately beside `exit_launch` so the launch decision has one home) rather than in the deposit, which stays a pure return. Now covered by `src/prompt/tests/parent_revival.rs::a_child_final_response_revives_the_parent_which_delivers_and_steps` and `::a_parent_with_a_held_lease_gets_no_second_driver`. **Not re-verified hands-on** at the merged commit — the status above rests on those tests.

  The **work-product transfer** is unverified by this pass: the child produced no work product, so no transfer commit was expected or seen.

### US-18 — compaction runs at a checkpoint and lands the one merge

- **actor** an agent (workflow-elected) or an operator
- **scenario** A branch's context is rebuilt at a configured checkpoint.
- **commands**
  - exec: `lernie dispatch compactor <ws> <agent-id>` (no `--goal`)
  - linked: as US-17 with `role: "compactor"`
- **acceptance**
  - exit 0; stdout is the compactor child's id.
  - The compactor is an **ordinary child** with a real model call — not a stub. Its injected toolset is exactly `write_summary` and `mark_for_deletion`, available to that role alone and never declarable in a `providers.yaml` `tools:` list. Its request additionally *declares* the tools its inherited transcript names, so the wire's referential integrity holds (ARCH §3.3 closure); it still cannot **call** them — a compactor reaching for an inherited tool is declined in-band and nothing executes.
  - `mark_for_deletion` can remove and never write content (a `git rm`) — deletion-only structurally, so the worst case is lost compaction, never corrupted work.
  - The **compaction merge** is a plain `--no-ff` merge into the dispatching branch — the one merge in the system. A work product the live branch rewrote since the checkpoint resolves **live-branch-wins** (the compactor's deletion is dropped).
  - Checkpoint triggers (`every_n_commits`, `every_t_seconds`, `on_flush`) are read from the config's `workflow.yaml`; a branch with no configured trigger never compacts; a malformed threshold fails closed.
  - **There is no terminal compaction stage** — ARCH §2.7: *"There is no *terminal* compaction stage anymore."*
- **status** **partial.** The absence of terminal compaction is proven positively-by-negation through the real binary: `src/e2e/prompt_end_to_end.rs::prompt_subcommand_persists_conversation_without_terminal_compaction` asserts the tip has exactly one parent and that `git show <branch>:summary/001.md` **fails** — and hands-on, the live `lernie prompt` run's branch ended at `transcript 002` with no merge commit and no `summary/`. The merge mechanics, live-branch-wins overlap, checkpoint predicate, and the two tools are unit-proven against real git (`src/prompt/compactor/**`, 23 tests, incl. `merge/tests.rs::overlap_drops_work_product_deletion_live_branch_wins`). **No test lands a compaction merge end-to-end through the verb**, and the verb itself does not merge (G4 below) — the merge fires when the *parent* interprets the returning compactor's result message.

---

## 8. Archival, replay, evaluation

### US-19 — the operator archives a run

- **actor** operator
- **scenario** A "run" is an agent subtree, not a workspace (ARCH §9.2).
- **commands**
  - exec: `lernie bundle <ws> <agent> <out-dir>`
  - linked: `lernie::cmd::bundle::run(bundle::Args { workspace, agent, out_dir }, &mut fx)` → `Outcome::Quiet`
- **acceptance** exit 0, empty stdout; `<out-dir>/agents.bundle` is one `git bundle` of `agents/<agent>` and its `agents/<agent>-*` hyphen-descendants, carrying all ancestry those refs reach — **the governing config commit included, with no config sidecar**; `<out-dir>/steps/<id>*` and `<out-dir>/inbox/<id>*` are the matching slices. An unknown agent exits 1.
- **status** **fulfilled.** Observed hands-on: `lernie bundle` exited 0 with empty stdout and produced exactly `agents.bundle` plus `steps/<id>/` and `inbox/<id>/`. `src/e2e/bundle_replay_cli.rs` drives the real `git` transport for the round trip and the unknown-agent decline.

### US-20 — the operator replays a run in a sandbox

- **actor** operator; then any frontend
- **commands**
  - exec: `LERNIE_HOME=/tmp/replay lernie replay /path/to/archive`
  - linked: `lernie::cmd::replay::run(replay::Args { archive }, &mut fx)` → `Outcome::Line(<scratch-path>)`
- **acceptance** exit 0; **stdout is the scratch workspace path** at `<data-root>/replays/<primary-id>/`; that path holds a fresh bare `repo.git` with every branch fetched from the bundle, the primary's worktree materialized under `agents/`, and the `steps/`/`inbox/` slices restored. The ordinary frontend then inspects it — ARCH §9.2: *"replay is not a mode (§2.3)."*
- **status** **fulfilled.** Observed hands-on into an isolated `LERNIE_HOME`: exit 0, the path printed, `repo.git` carrying `agents/<id>`, the worktree materialized, and both slices restored. `src/e2e/bundle_replay_cli.rs::bundle_then_replay_round_trips_the_subtree` and `::replay_cli_lands_under_lernie_home` assert the same.

### US-21 — the operator measures an experiment against the task suite

- **actor** operator
- **scenario** Evaluate harness *configuration*, holding tasks and models fixed (ARCH §9.3).
- **commands** exec only (a separate binary crate, deliberately outside the harness):
  ```
  agent-eval --config baseline --suite tests/suite --runs 5 \
             [--experiments-dir <dir>] [--agent <cmd>] [--bundle-dir <dir>] [--lernie <bin>]
  ```
- **acceptance**
  - `tests/suite/` holds **50 tasks across the 7 §9.1 failure categories**, each with an `id`, a `prompt`, an optional `setup`, and a machine-checkable `check` where **exit 0 is the sole pass signal**; ≥10 tasks per category via secondary tags. Well-formedness is a gate, not documentation.
  - Per run the runner seeds a fresh isolated `LERNIE_HOME` and working directory, runs `setup`, invokes the agent, runs `check`.
  - It reports pass@1 with 95% Wilson intervals and pass@5, overall and per category.
  - `--bundle-dir` archives failing runs through `lernie bundle`.
- **status** **partial.** The suite is verified well-formed by `tests/suite.rs::suite_is_well_formed` — 50 tasks, 7 categories, unique ids, closed tag set, file-stem-is-primary-tag — and no suite task is ever executed by `cargo test`, which is correct. The runner is fully covered as a library (22 tests, all in `crates/agent-eval/tests/`: `runner.rs` drives `runner::evaluate` with a `FakeAgent` over real shell `setup`/`check`, `stats.rs` proves the Wilson intervals). **The production wiring is unfulfilled** (G7 below): the default `--agent lernie-eval-agent` names a binary that exists nowhere in the repo, and `LERNIE_EXPERIMENT` — set by the runner as the channel that selects the experiment — is read by **nothing** under `src/`, so an experiment's `workflow.yaml` does not reach the harness. The ~40% baseline pass@1 is a property of running the suite against a live model and is asserted by no crate.

---

## 9. The two bindings

### US-22 — the crate consumer drives lernie in-process

- **actor** crate consumer (the motivating case is a linked frontend, ARCH §3.5)
- **scenario** Embed lernie without exec'ing a binary.
- **commands**
  ```toml
  # Cargo.toml — pin-exact, 0.x only
  lernie = "=0.0.1"
  ```
  ```rust
  use lernie::cmd::{Command, Fx, Outcome, new, prompt};

  for prelude in Command::Prompt(/* … */).preludes() { prelude(); }
  let mut fx = Fx { driver_target, adapter_target, editor, tool_stdin,
                    tool_stdout, tool_stderr, stop };
  let outcome = prompt::run(prompt::Args { repo, message }, &mut fx)?;
  ```
- **acceptance**
  - `lernie::cmd` is the crate's **entire** public API — `src/lib.rs` exposes exactly `pub mod cmd;`.
  - Every verb module exposes exactly `Args` and `run(args: Args, fx: &mut Fx) -> Result<Outcome, Error>`. One shape, twelve verbs.
  - `cmd`'s own public surface is exactly `{Cli, Command, Outcome, Fx, Error, <verb modules>, prelude}`; `prelude` exposes exactly `become_pgid_leader`, `install_stop_handler`, `stop_flag`.
  - The library performs **no** process-global effect: `Outcome::Exec` returns the successor for the binding to `execve`, `Outcome::Line` the line for it to print, `Fx::editor` the `$EDITOR` spawn.
  - The library resolves **no** `current_exe`: every re-entry seam reads `Fx::driver_target`, so a linked host is never assumed to answer to `lernie tool …`.
  - The promise is **pin-exact 0.x only** — ARCH §3.4: *"The linked binding promises **pin-exact consumption only** — 0.x, no semver stability, the same posture brazen takes toward lernie (§4.4)."*
- **status** **fulfilled.** Enforced structurally by `tests/command_surface_parity/` (11 tests over `syn`-parsed source), which links only `lernie::cmd`: `items::the_public_surface_is_exactly_the_command_surface` (every externally reachable declaration is a verb's entry, its arguments, its products or a binding prelude), `entries::the_prelude_re_exports_are_mechanisms_and_no_types`, and `graph::every_source_file_is_reachable` / `::nothing_below_the_surface_reaches_into_it`, which prove the walk opened every `src/**/*.rs` and that nothing below the surface names it. The `driver_target`-not-`current_exe` rule is stated and realized per ARCH §3.4's bl-eb98 note.

  One wrinkle worth an eval pass's attention: `lernie dispatch`'s product is `println!`'d **inside the library** rather than returned as `Outcome::Line` — the one verb whose product bypasses the `Outcome` seam. Parity is unaffected (the surface shape is identical), but a linked host cannot capture that product without capturing stdout.

### US-23 — CLI and crate cannot drift

- **actor** frontend author, crate consumer
- **scenario** The guarantee that makes US-22 worth relying on.
- **commands** `make check` (and therefore the pre-commit hook and GitHub Actions, which both run it)
- **acceptance** A bijection is asserted between the crate's **actual** public surface (walked with `syn`) and the CLI's **introspected** verb set (walked with clap): the crate exposes nothing public that is not a verb's entry, its arguments, its products, or the binding preludes, and no verb lacks its entry. A divergence **fails the build**. The prelude-per-verb map is a query on the surface (`Command::preludes()`), not a match duplicated in each binding.
- **status** **fulfilled, and stronger since `df4e443` (bl-4762).** `tests/command_surface_parity/` is the checker and rides `make check`. It now asserts the bijection at three depths: `entries::every_verb_entry_is_its_variants_payload` pairs each `Command` variant with its module's entry **as function values**, so the compiler — not an assertion — proves the two share one argument type and one product type; `entries::the_verb_table_is_exhaustive` closes the verb set; and `arguments::every_verb_argument_is_a_field_of_its_entrys_args` asserts, per verb, that the clap-introspected argument set is exactly that verb's public `Args` fields — same names, same arity, same named-vs-positional form. Verified by inspection of the CLI (12 verbs) against the 12 verb modules.

---

## 10. The provider seam

### US-24 — every model call crosses one pipe, and a mismatched adapter is refused

- **actor** end user, operator
- **scenario** lernie never sees a credential; the adapter is a pinned, replaceable binary.
- **commands**
  - `bz --dump-config`, `bz --login --provider <id>` (brazen's own surface, not lernie's)
  - `make smoke` / `make smoke SMOKE_PROVIDER=<row> SMOKE_MODEL=<id>`
- **acceptance**
  - The harness execs `bz --json --provider <row>` once per **attempt**, canonical request on stdin, and appends bz's stdout verbatim to the step's `response.json`. Retry is the harness's; brazen never retries.
  - Provider endpoints, auth, and wire dialects live entirely in brazen's config; lernie references a row **by name** and never sees credential material.
  - A **load-time version guard** rejects any `bz` whose `--version` differs from the linked brazen crate version, with an actionable message.
  - The same voice covers the guard failing one step earlier — **no `bz` at all**: `lernie prompt: provider adapter "bz" not found (§4.4 — the default adapter is `bz` on PATH; install the pinned binary: cargo install brazen --version =<pin> --locked, or name an adapter you have with `adapter:` in the harness root's models.yaml): No such file or directory (os error 2)`. The errno is trailing detail, never the headline, and the pin is the linked one.
  - Under an `adapter:` override in `<config-root>/models.yaml` the guard is **skipped** and the in-band `MessageStart.v` handshake governs.
  - Adding a provider on a supported protocol is a brazen config row plus a `models.yaml` entry plus a `providers.yaml` pointer — **no code anywhere**.
- **status** **partial.** The guard, the override, and the row-by-name indirection are all verified hands-on. The guard fired exactly as promised: `make smoke SMOKE_PROVIDER=local SMOKE_MODEL=qwen3.5:9b` failed with `lernie prompt: bz version "0.0.4" does not match the linked brazen crate "0.0.3" (§4.4 — install the pinned binary: cargo install brazen --version =0.0.3)`, exit 1. Re-running the same target with `adapter: /home/mark/.cargo/bin/bz` in `models.yaml` skipped the guard and completed a real model call end-to-end (US-06) — so the override is real and the handshake tolerated the newer adapter.

  **The shipped default is still unvalidated (G10 below):** every live check in this pass ran against the `local` row. Provider `anthropic` with model `claude-sonnet-5` — the `make smoke` default — has never been exercised on the wire; `make check` mocks it and structurally cannot catch a bad id (this is exactly how `claude-sonnet-4-7` once shipped).

  **The pin is stale again (G6 below):** `Cargo.toml` pins `brazen = "=0.0.3"` while brazen 0.0.4 is published and installed on this machine. Any user whose `bz` is current gets the refusal above on every verb that drives a model call.

  **Shortfall found by the 0.0.1 outside-evaluation pass, since RESOLVED (bl-63c1).** The "actionable message" clause held only for a `bz` at the *wrong* version. With no `bz` at all the same guard failed one step earlier and rendered a bare `lernie prompt: adapter subprocess: No such file or directory (os error 2)` — naming neither the adapter, nor the pin, nor the fix, on the one command every binary-install user reaches first. Both spawn seams (the `--version` probe and every model call) now classify the launch failure through `prompt::adapter::spawn_error`, so `ErrorKind::NotFound` becomes `Error::AdapterMissing` in the guard's voice and everything else stays `AdapterSpawn`. Covered by `src/e2e/prompt_adapter_failure.rs::a_missing_bz_names_the_adapter_the_pin_and_the_install_command` (real binary, a `PATH` carrying `git` and no `bz`), `src/prompt/tests/errors.rs::run_names_a_missing_adapter_and_the_command_that_installs_it`, and `src/prompt/dispatch/model_call/tests/cases.rs::a_missing_adapter_surfaces_as_adapter_missing_at_the_model_call_too`.

---

## 11. Promises the code does not keep

Each entry is a gap this pass **observed**, not inferred. They are backlog candidates.

| # | Gap | Evidence |
|---|---|---|
| ~~**G1**~~ | ~~**A child's result deposit does not revive a quiescent parent.**~~ **RESOLVED at `cbfaaef` (bl-4a6c)** while this document was being written. Found hands-on at `e3123ba`: the child's `epitaph: final-response` message sat in the parent's inbox for 60 s unadvanced, and only an explicit `lernie scan` (`drivers launched: 1`) delivered it. The wake now lives in `dispatch::terminal::revive_parent`, tested by `src/prompt/tests/parent_revival.rs`. Kept here as the record of the find, not as open work. | See US-17. |
| **G2** | **README promises a terminal compactor that is deleted.** README's `lernie prompt` step 8 describes dispatching a compactor and landing a compaction merge, and its inspection block tells the user to run `git show agents/<id>:summary/001.md`. ARCH §2.7 deleted the stage. | Hands-on: the live `lernie prompt` branch ended at `transcript 002` — one parent, no merge, no `summary/`. The README's two inspection commands fail for any user who follows them. `src/e2e/prompt_end_to_end.rs` asserts this negative deliberately. |
| **G3** | **README (and two live doc comments) claim children do not run.** README says `worker.rs` "still stops at the dispatch commit" and that a child "does not yet reach a terminal event"; the same claim is in `src/prompt/inbox/scan/mod.rs` and `src/prompt/dispatch/result_deposit.rs`. `worker.rs` does not exist and ARCH §2.5 says it is deleted. | Hands-on: `lernie dispatch worker` produced a child that ran a full step loop and deposited `epitaph: final-response` into its parent's inbox. The doc understates the code; the gap is in the evidence, not the behavior. (Since bl-3761 a child *is* exercised at integration level — `src/e2e/stop_children.rs` dispatches one that reaches its own model call and deposits a result — though it is stopped rather than run to a final response.) |
| **G4** | **`lernie dispatch compactor` does not run the compaction merge** the README attributes to it. The verb forks, deposits, and launches; the merge fires only when a parent interprets the returning compactor's result message. | `src/prompt/dispatch_cli.rs` performs role validation → `child_dispatch::run` → `println!`. No merge call. No test asserts the verb-level claim. |
| ~~**G5**~~ | ~~**`lernie message` accepts a nonexistent recipient.**~~ **RESOLVED at `8e79e76` (bl-bdc7)** while this document was being written. Found hands-on: `lernie message <ws> ghost 'hello'` → exit 0, `inbox/ghost/user-001.md` created for a branch that never existed, against ARCH §2.11's *existing* agent. The verb now requires the id to be a single path component and requires the `agents/<id>` ref to exist; the same id guard was applied to `advance`, `stop`, `dispatch` and `bundle`, and `scan`'s flush skips-and-reports an inbox with no agent branch. Kept here as the record of the find, not as open work. **Correction (bl-bbba):** at `advance` only the path-component half actually landed; the existence half arrived with bl-bbba — see US-14. | See US-13. |
| **G6** | **The brazen pin is stale.** `brazen = "=0.0.3"` while 0.0.4 is published. Every model-call verb refuses on a machine with a current `bz`. This is the second recurrence (bl-143e bumped 0.0.2→0.0.3). | Hands-on: `make smoke` failed the load-time guard with the literal version-mismatch message. |
| **G7** | **`agent-eval`'s production wiring is a dangling seam.** The default `--agent lernie-eval-agent` names a binary that exists nowhere in the repo (one grep hit — the default-value literal). `LERNIE_EXPERIMENT`, the env the runner sets to select the experiment, is read by **nothing** under `src/` — so an experiment's `workflow.yaml` never reaches the harness. | Repo-wide grep. `make eval` without an explicit `AGENT=` cannot spawn. |
| ~~**G8**~~ | ~~**The shipped config template declares an action the interpreter does not implement.**~~ **RESOLVED (bl-0e79).** `template/workflow.yaml` bound `user_message: [spawn_root_agent]`, and `spawn_root_agent`/`spawn_exchange` parsed as `config::action::Action` while `src/prompt/workflow_actions.rs` had an arm for neither. ARCH decided it: §2.4 leaves no circumstance a hop could fire them from — "Reprompt is a message" (a user message resumes the agent's own branch), a new root agent is forked explicitly, and an exchange "owns no branch, no merge, no lifecycle". Both were **subtracted** from the closed action set (ARCH §6) and from every shipped config; naming one now fails at load with the reason, the idiom bl-a1a1 used for the retired `overflow: summarize`. `workflow_actions::execute` lost its catch-all — the match is exhaustive over the closed set, so a new action cannot be added without deciding on the spot whether it executes or is a tracked deferral. | Kept as the record of the find. The guard is `src/prompt/tests/workflow_vocabulary.rs`: it sweeps every `workflow.yaml` this repo ships (the embedded template plus `experiments/*/`) through `Workflow::parse` and the interpreter, and pins the deferred set exactly — so a verb the interpreter cannot run cannot enter a shipped config unnoticed. |
| ~~**G9**~~ | ~~**`--stop-children` has no integration proof.**~~ **RESOLVED (bl-3761).** The shipped walk held as specified — no behavior change was needed, only the missing evidence. `src/e2e/stop_children.rs` builds a live parent/child pair (both mid-model-call, distinct process groups) and proves both directions: the flag fells the child, a bare stop does not. Kept here as the record of the find. | See US-16. |
| **G10** | **The shipped default is unvalidated on the wire.** Provider `anthropic` / model `claude-sonnet-5` has never completed a live model call. Needs a user credential. | Every live run in this pass used the `local` row. `make check` mocks the wire by construction. |

## 12. What this pass did not check

Named so they are not mistaken for passes: US-01 (`make install` end-to-end), the `in_flight` fd-close frontend signal (US-07), the work-product transfer with an actual work product (US-17), the compaction merge through a real checkpoint (US-18), `lernie tool message` and `lernie tool load_skill` as subprocesses (US-11, US-12), and any run of a `tests/suite/` task against a live model (US-21).
