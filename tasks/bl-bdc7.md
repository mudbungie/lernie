+++
title = "Validate agent id at the inbox boundary; lernie message errors on a nonexistent agent"
created = 1784955529
updated = 1784959172
claimant = "Ivory"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Harrow audit findings 1+2, one root cause: an unvalidated model/CLI-supplied agent id reaching the filesystem. (a) `inbox_dir` joined a raw agent_id into the workspace path — `lernie message <ws> ../../victim/pwned hi` wrote OUTSIDE the workspace with exit 0, and an absolute id REPLACED the base (Path::join). (b) `lernie message` to a nonexistent agent silently succeeded (exit 0, inbox dir nothing would ever read) — silent message loss.

## STATE (Ivory, 2026-07-25): implemented, verified, committed on work/bl-bdc7; delivery blocked on a close/main race

Branch `work/bl-bdc7` = 7a12462 (Merge main 803aa00 into work/bl-bdc7) on top of 1e987b0 "validate the agent id at every verb; message requires an existing recipient [bl-bdc7]". Worktree CLEAN (0 modified). Nothing left to author.

Verified on this exact tree: `make check` GREEN — fmt-check, clippy -D warnings, tarpaulin **100.00%, 4435/4435 lines, zero uncovered**, lib suite 853 passed / 0 failed. (Logs: /tmp/claude-1000/-home-mark-dev-lernie/fe8c0ced-96b4-45ac-8547-30d9edb8e1e3/scratchpad/{check4,c10,close}.log.)

## What is implemented

- **src/name.rs** (new, crate-private `mod name;` in lib.rs): `is_component` — non-empty, not `.`/`..`, no `/`, `\`, NUL — lifted verbatim out of `load_skill` (which now calls it), plus `require_agent_id` / `NotAnAgentId` whose Display is `agent id "X" is not a single path component — an agent id is its branch name, the hyphenated descent \`<a>-<b>-…\` (ARCH §2.3); pass the id exactly as \`lernie prompt\` / \`lernie dispatch\` printed it`.
- **Per-verb guards** in src/cmd/{message,advance,stop,dispatch,bundle}.rs: one `crate::name::require_agent_id(...)` call before any disk touch, each mapping to its own `lernie <verb>: …` prefix (`dispatch <role>` for dispatch). Rule documented in the src/cmd/mod.rs module header.
- **Ref-existence**: `workspace::agent_exists(ws, id, git)` — `git rev-parse --verify --quiet refs/heads/agents/<id>` — is the ONE home of the question; `prompt::stop::inspector::GitInspector` now delegates to it. `inbox::cli_run` calls it after `workspace::require` and before the deposit, failing `MessageError::UnknownAgent` → `lernie message: no agent "X" in this workspace — a message is addressed to an existing agent (ARCH §2.11); check the id against the workspace's agents/* refs, or start an agent with lernie prompt / lernie dispatch`, exit 1.
- **scan flush skip-and-report** (the coordinator's addendum): `scan()` derives the `agents/*` candidate set once and passes it to both halves; `flush` intersects the `inbox/*` listing with it, so an inbox dir with no ref is never launched (a driver for it dies on `invalid reference` on every pass forever) and is counted in `ScanReport::inboxes_without_branch` / `inboxes with no agent branch: N`. The scanner deletes nothing.
- **Tests**: src/cmd/tests/agent_id.rs (traversal + absolute declined for message with proof nothing lands outside the ws; unknown-agent decline; one decline per verb), src/e2e/message_cli.rs (real-binary repro: escaping id exits non-zero and writes nothing outside; unknown recipient errors; valid recipient still deposits), probe.rs cli_run decline, scan flush no-branch test, name.rs unit tests.
- **Docs**: ARCH §2.11 *Deposit* (the enforced existing-agent contract + the id guard) and the flush paragraph; README `lernie message`, `message` tool, and `lernie scan` sections.

## Design decisions

- **Component check, not an id grammar.** The predicate is "exactly one path component", not a §2.3 `<ts>-<short>` token grammar: every consumer needs "joinable and ref-nameable", and a grammar would outlaw the short ids fixtures and operators legitimately use.
- **Guarded at the command surface, one call per verb.** §3.4 makes it total — both bindings enter through a verb and the model's message/dispatch tools re-enter through the CLI — and the per-verb call keeps each decline under that verb's own error prefix.
- **Ref-existence enforced** (ARCH §2.11 opens: "A message is content addressed to an *existing* agent"). No legitimate deposit-before-branch case: `lernie dispatch` forks the child branch before depositing (child_dispatch.rs), and a root's first user message is deposited by its own executor after the fork.
- **Unguarded by design**: the deposit *sender* (harness-derived from LERNIE_CONV_BRANCH, §2.11 "never model-supplied"); `lernie advance` on a well-shaped id with no branch (§2.11 pin 1 = silent no-op); verbs whose ids come from refs or the clock (prompt, scan, replay, new, config).
- No new terms of art were coined (deliberately avoided naming the no-branch inbox anything).

## Remaining steps to close (a cold session can do exactly this)

1. `cd /home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/lernie/bl-bdc7 && git merge main --no-edit` (resolve if main moved; bl-4a6c/bl-2503/bl-ae66 already reconciled — all clean so far).
2. From /home/u/dev/lernie: `PATH=<bz-0.0.3-bin>:$PATH bl close bl-bdc7 --as Ivory -m "..."`. **The ambient `bz` is 0.0.4 while this branch pins brazen =0.0.3 (bl-8c92 in flight): without a 0.0.3 `bz` first on PATH, five real-bz e2e tests fail.** A pinned copy is at /tmp/claude-1000/-home-mark-dev-lernie/fe8c0ced-96b4-45ac-8547-30d9edb8e1e3/scratchpad/bz003/bin (rebuild with `cargo install brazen --version =0.0.3 --root <dir>`). Run it detached (`setsid nohup … &`) — a gate pass currently takes 4-8 h.
3. Sweep the gate children after close: bl-6d74 (tests), bl-7cd3 (docs), bl-18bc (alignment).

## Blocker to be aware of

`bl close` folds main at the START, then runs the ~4-8 h gate (12-16 concurrent sibling tarpaulin runs; the box swaps ~17 GB). By the time it finishes main has moved, and bl-delivery aborts on its no-resurrection invariant ("the squash carries path(s) never authored on it since its fork"). Seen against bl-2503, then main moved again to bl-ae66 mid-run. Nothing is lost when it aborts — the branch stays claimed and clean — but the close cannot converge while gate duration exceeds the interval between main landings. Needs a quiet window or serialized deliveries.