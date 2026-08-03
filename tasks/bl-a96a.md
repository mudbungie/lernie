+++
title = "a child's terminal response deposits to its dispatcher even when the turn was prompted by someone else"
created = 1785733795
updated = 1785734812
claimant = "Routing"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-eb84"
on = "close"

[[blockers]]
id = "bl-1d7f"
on = "close"

[[blockers]]
id = "bl-78d9"
on = "close"
+++
Diagnosed from a live yog session, 2026-08-02 (yog dev workspace,
`~/.local/share/yog/workspaces/dev`). **Shipped 2026-08-03.**

## Symptom the operator hit

The operator typed a question into a **child** agent's own conversation. The
child answered. The answer was deposited into the **parent's** inbox, which
revived the parent, which spent a step reporting the child's unrelated answer to
the operator. The operator's next message:

> are you responding to me with info intended for a subagent?

## The bytes

Child `20260802T223426Z-1277061f-20260802T223711Z-2962f775`:

- `messages/027-user.md` — `from: user`, `deposited_at: 2026-08-03T00:35:55Z`,
  body `does shift enter not sent`
- `messages/029-…json` — its terminal answer about Shift+Enter

Parent `20260802T223426Z-1277061f` (energize):

- `messages/042-<child-id>.md` — `epitaph: final-response`,
  `deposited_at: 2026-08-03T00:35:59Z`, carrying that same Shift+Enter answer
- `messages/043-…json` — the step the parent burned on it

## The ruling

**A reply answers the last prompter; an obituary reports to the dispatcher.**
The recipient is decided by the epitaph's *value*, like everything else about a
result message — two speech acts, not two message kinds:

- **Reply** (`final-response`) → the **last prompter**: the sender of the newest
  delivered `messages/NNN-<sender>.md` entry, skipping two non-prompts — a
  returning child's result message (a return, not a question; without the skip
  every parent would address its own answer to the last child that returned) and
  the agent's own note to itself (a self-reply never terminates). Derived, never
  stored. `from: user` addresses no inbox at all — the answer is read in the
  agent's own conversation, the same structural no-op a root's deposit is. No
  surviving prompt (compaction squashed it) falls back to the id.
- **Obituary** (`stopped` / `budget-exhausted` / `died`) → the **dispatcher**
  (`parent_of`). It is not an answer but a structural fact about the tree, and
  the dispatcher is the party with a standing interest in it; descent fixes that
  address, so no conversation can move it.

For the dispatch step the two coincide (the goal arrives as the dispatcher's
message), so the old parent-addressed rule is the reply rule's **first case** —
one rule replaces two.

§2.11's `revive_recipient` (renamed from `revive_parent`) revives the address
the deposit derived, computed once under the still-held lease so a rival
executor cannot move the tail between deposit and wake-up.

## Question 1 — the §2.6 work-product transfer at a non-parent

**Ruled: it does not apply, and neither do the §6 delivered-child-result
bindings.** §2.6's return is the *dispatcher's* business — the transfer diffs
the child's fork point, a commit on the dispatcher's own branch, and §2.5's
disjoint-write-path guarantee (what makes the apply conflict-free by
construction) is a statement about a dispatcher and its own children. Diffing
across two lineages would drag the dispatcher's intervening commits into a tree
that never forked from them. So a reply delivered anywhere else is an **ordinary
message**: it lands in the transcript with its `epitaph:`/`terminal_ref:`
frontmatter model-visible, nothing is applied to the tree, no §6 binding fires.
Nothing is lost — the terminal ref names every byte of the work. One predicate,
`child_result::own_result_ref` (`parent_of(sender) == recipient` + carries a
`terminal_ref:`), read at the drain and at the interpreter alike. No scope
growth: this also stops a foreign compactor's return from rebasing the wrong
branch.

## Attacked and accepted

- **A hijacked child's dispatcher parks.** The ordinary parked state; its
  recovery is the ordinary primitive — it messages the child, and the child's
  next reply answers it. Procedure children are out of this by an existing rule:
  ARCH §2.3 already says *a procedure child is not an agent an operator speaks
  to* (they are dispatched unnamed).
- **Agents can now hold a conversation** — the point of the reframe. An
  unbounded exchange is bounded by the §6 `budgets:` ceiling derived over the
  tree, not by a new governor.
- **A root whose last prompter is an agent replies to that agent.** Falls out of
  the one rule and is correct: a child that asked its parent a question gets the
  answer. The user still reads the response in the root's own transcript.

## Shipped

- `src/prompt/dispatch/result_deposit.rs` — `recipient()` (the rule's one home)
  + `last_prompter()`; `+ result_deposit/tests.rs`.
- `src/prompt/dispatch/terminal.rs` — `conclude` derives the recipient
  pre-release; `revive_parent` → `revive_recipient`.
- `src/prompt/dispatch/child_result.rs` — `own_result_ref`; `load_results` /
  `has_pending_result` scoped by it.
- `src/prompt/dispatch/drain.rs` — leaves only own-child results for the §6
  interpreter.
- `src/prompt/inbox/deposit.rs` — `deposit_child_result` **deleted**;
  `deposit_result` takes the recipient it is handed.
- Docs: ARCH §2.3 step 5, §2.5 (workflow appendix), §2.6 (the ruling + a
  shipped-state note), §2.9, §2.11 (exit protocol, pin 2, shipped note);
  TAXONOMY (reply / obituary / last prompter); PRINCIPLES ("Return is not a
  verb"); README; USER_STORIES; CHANGELOG.
- Tests: `prompt/tests/reply_address.rs` (end-to-end on the real child path),
  plus drain / child_result / result_deposit units. `make check` green, 100%
  coverage.

Related, same conversation, filed separately: the dispatch skill teaches neither
the wait nor the do-it-yourself default.