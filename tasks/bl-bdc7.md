+++
title = "Validate agent id at the inbox boundary; lernie message errors on a nonexistent agent"
created = 1784955529
updated = 1784959146
claimant = "Ivory"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Harrow audit findings 1+2 (same root cause, two fixes differing in kind). (a) src/prompt/inbox/mod.rs:52 inbox_dir joins a raw model-controlled agent_id into the workspace path: 'lernie message $W ../../victim/pwned hello' writes outside the workspace, exit 0; an absolute path REPLACES the base (Path::join semantics). Fix: validate at the boundary with the is_component idiom already written in src/prompt/tool/builtin/load_skill/mod.rs:147-154 ('the name is a fact, not a slot to munge') — lift it to a shared helper; or better an agent-id shape check (hyphenated descent per ARCH §2.3). (b) lernie message to a NONEXISTENT agent silently succeeds (exit 0, phantom inbox dir) — silent message loss; lernie bundle already resolves the id against agents/* refs and errors ('no branch matches agent id ...'); message must do the same. Sell as integrity, not threat model (the bash tool already grants the model execution).

## Decisions (Ivory, implementation)

**Shape check, not a shape *grammar*.** The predicate is "exactly one path component" (non-empty, not `.`/`..`, no `/`, `\`, NUL), lifted out of load_skill into a new `src/name.rs` (`is_component`, `require_agent_id`, `NotAnAgentId`) and used by both. A stricter §2.3 grammar (even token count of `<ts>-<short>` segments) was rejected: it is not what any consumer needs (every consumer needs "joinable and ref-nameable"), and it would outlaw the short ids fixtures and operators legitimately use.

**Guarded at the command surface, one call per verb.** `message`, `advance`, `stop`, `dispatch`, `bundle` each call `name::require_agent_id` before touching disk (`src/cmd/*.rs`), so the decline carries that verb's own `lernie <verb>: ...` prefix. §3.4 makes this total: both bindings enter through a verb, and the model's `message`/`dispatch` tools re-enter through the CLI. Verbs deriving ids from refs or the clock (`prompt`, `scan`, `replay`, `new`, `config`) take nothing from outside and are unguarded by design.

**Ref-existence: enforced.** ARCH §2.11 opens "A **message** is content addressed to an *existing* agent" — unambiguous, so `lernie message` now requires an `agents/<id>` ref (`workspace::agent_exists`, the one home of the question; `stop`'s BranchInspector now delegates to it) and fails `lernie message: no agent "<id>" in this workspace — a message is addressed to an existing agent (ARCH §2.11); ...`, exit 1, before any write. No legitimate deposit-before-branch case exists: `lernie dispatch` forks the child branch *before* depositing its dispatch message (§2.5, and `child_dispatch.rs` does exactly that), and the root's initial user message is deposited by its own executor after the fork.

**Addendum (phantom inbox poisons `lernie scan`): fixed, skip-and-report.** The scan's inbox flush now intersects the `inbox/*` listing with the `agents/*` refs — the §8 enumeration seam the sweep already uses, so this is the existing single source of truth applied to the other half of the pass, not a new mechanism. An inbox dir with no ref is never launched (a driver for it dies on `invalid reference` forever) and is reported as `ScanReport::inboxes_without_branch` / `inboxes with no agent branch: N`. The scanner deletes nothing — it moves nothing either; deleting debris stays an operator act. This heals workspaces that already carry a stray inbox.

**Not guarded (deliberate):** the deposit *sender*, which is harness-derived from `LERNIE_CONV_BRANCH` per §2.11 ("never model-supplied"), and `lernie advance` on a valid-shaped id with no branch, which §2.11 pin 1 defines as a silent no-op.

Docs updated: ARCH §2.11 *Deposit* (the enforced "existing agent" contract + the id guard) and the flush paragraph; README `lernie message` + `message` tool + `lernie scan` sections.