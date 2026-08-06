+++
title = "the dispatch and workflow prompt surfaces misstate the parent/child relationship: a compaction stage that was deleted, budgets advertised per-agent that are whole-tree, and no hint that a live child is addressable"
created = 1785906111
updated = 1785995940
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
**Operator ruling (2026-08-05): the word `subagent` stands.** An agent is an
agent. `subagent` is fine wherever it names a *relationship* — an agent created
by another agent, whose lifecycle is expected to be circumscribed by its
creator. What is not fine is a string implying a child holds fewer powers than
its parent: that it cannot be addressed, cannot dispatch children of its own,
or answers to different controls. Renaming `subagent` → `child agent` is a
lexical treadmill and is **out of scope**, as is `conversation` → `agent` /
`workspace` wherever the word is merely retired and the sentence is true.

This supersedes the ball's original framing (revoice every retired term). The
mechanism claims are the deliverable.

**The implementation already satisfies the ruling** — verified 2026-08-05, no
code change needed:

- `schemas/tools/message.json` — the recipient is "either an agent id or a
  unique agent display name"; parent, child, and sibling are all addressable
  with no asymmetry. Sibling delivery is pinned by the ARCH §2.6 shipped-state
  note (bl-a96a): "a sibling's reply included — delivers as an ordinary
  message."
- `template/souls/worker.md` grants `dispatch` to every worker. The only
  limiter on a child dispatching its own child is `max_depth`, a budget axis
  (`template/workflow.yaml` default 4) — an explicit prohibition, not a
  structural block.
- `src/prompt/dispatch_cli/mod.rs:36` — "**The role set is open (§4.3).** This
  CLI enumerates no role names"; roles "differ only in the pinned soul
  (`souls/<role>.md`) and in where the goal comes from".

Only the shipped strings are wrong.

**Defect 1 — `schemas/tools/dispatch.json:13` describes a deleted mechanism.**
The shipped string promises "only the terminal **compacted** result". Terminal
compaction is gone: `docs/ARCHITECTURE.md` §2.7 shipped-state note (bl-9dbd)
reads "**No terminal compaction.** The stage the spec deleted is gone from
code: `terminal::finish` no longer dispatches a compactor at a final
response." What the parent actually receives (ARCH §2.6, §2.11) is the child's
own terminal response verbatim — iff it spoke — carried with `epitaph:` and
`terminal_ref:` frontmatter. Fix: drop "compacted"; name the terminal response.

**Defect 2 — `schemas/tools/dispatch.json:13` is silent on messaging a live
child.** "the dispatching agent does not see the subagent's intermediate work —
only the terminal … result" is true about *visibility*, but reads as one-shot:
nothing tells the model it may `message` a running child. That is the precise
misreading the ruling guards against. Fix: one clause pointing at `message`
(ARCH §2.11).

**Defect 3 — budgets are advertised per-agent and are actually whole-tree.**
Three sites say "**Per-conversation** spend limits": `schemas/workflow.json:11`,
`schemas/workflow.json:82`, and the `budgets:` block comment in
`template/workflow.yaml`. Line 82's own next clause contradicts it: "a
**whole-tree ceiling**: any driver (root or subagent) checks the tree's total
spend against the single frozen limit, with **no per-dispatch inheritance**." A
model reading "per-conversation" believes each child carries its own budget; it
does not. This is a claim about which controls a child answers to, so it is in
scope under the ruling. Fix: "Whole-tree spend limits". **Keep** "root or
subagent" on line 82 — it affirms sameness and is exactly the sanctioned use.

**Riding along — "Per-call goal text" (`schemas/tools/dispatch.json:13`).**
Independent of this ruling: `docs/ARCHITECTURE.md` §2.1 bans "**call**" without
a qualifier "*as a term for an interaction with a model, tool, or API*", and the
bl-1966 carve-out sanctions the programming sense alone — "*Call site,
callback, callee, function call, system call … stay as they are*". "Per-call
goal text" is the banned sense. → "Per-dispatch goal text". The same violation
sits in two internal doc comments, `src/prompt/dispatch_cli/mod.rs:113` and
`src/prompt/subagent/mod.rs:67` ("per-call soul"); fix them while the file is
open.

**Deliverable 4 — carry the ruling into `docs/TAXONOMY.md`.** The taxonomy
today (L290) reads: lernie "**deletes 'subagent' as a category** — a child
agent is just an agent". That sentence is what made this ball look like a
rename. Amend it to state the ruling: the *category* is deleted — no separate
powers, controls, or addressability, and every agent may be messaged and may
dispatch — while the *relational word* is kept for "an agent created by another
agent, whose lifecycle its creator circumscribes". Without this amendment the
next alignment pass re-raises the same rename.

**Tests:** `src/install/tests/toolspec.rs` asserts the dispatch schema's
descriptions and moves with the strings.

**Out of scope:** the `src/prompt/subagent/` module rename (~20 refs) — dead
under the ruling, no follow-on ball. The unspecified `max_depth` boundary is
filed separately: `src/prompt/budget/mod.rs:31` flags that "§6 does not spell
out the depth boundary" and the module picked `depth(branch) > max_depth`
itself, which is precisely how far a child's own children may go.

Discovered by the bl-404d rescue, which flagged rather than acted because that
ball's body had specified the `subagent` wording. Under this ruling that wording
was never the defect.