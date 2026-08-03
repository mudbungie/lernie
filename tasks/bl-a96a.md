+++
title = "a child's terminal response deposits to its dispatcher even when the turn was prompted by someone else"
created = 1785733795
updated = 1785733795
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Diagnosed from a live yog session, 2026-08-02 (yog dev workspace,
`~/.local/share/yog/workspaces/dev`). Evidence is on-disk bytes, cited below.

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
- `messages/043-…json` — the step the parent burned on it: "A delayed subagent
  response arrived, but it addressed an unrelated keybinding question rather
  than the requested XDG analysis."

The parent asked nothing. The user asked, in the child's own conversation, and
the reply fanned out to an agent that was not the sender.

## Where it lives

`src/prompt/dispatch/terminal.rs` — every terminal deposits to the parent and
`revive_parent`s it on a `final-response` epitaph, keyed on the epitaph value
alone (§2.11 pin 2). Nothing consults **who prompted the turn**. ARCH §2.6's "a
child's result message carries its own terminal response" is written as if the
dispatch were the only thing that ever prompts a child, but §2.11 makes any
agent — and the user — a legal sender to any existing agent, and §2.9 makes
messaging the resume path.

## The reframe to attack first (design ball — ARCH ruling before code)

A child's terminal response goes to **whoever last spoke to it**, not to its
dispatcher. For the dispatch turn the two are the same agent, so the current
behaviour is the new rule's first case rather than a rule of its own — one rule
replacing two, and the fan-out to an uninvolved parent disappears with it.

Check it against: a user-prompted turn (deposit to no agent — the operator reads
the child's own conversation); a sibling-prompted turn (deposit to the sibling);
a long-lived child reporting upward unprompted (that is `message`, not a
terminal, and is unaffected); a `stopped`/`died` terminal with no prompter at
all (the scan/stop paths, which must still tell the parent its child is gone —
likely the one case that stays parent-addressed, and if so say why in ARCH).

Related, same conversation, filed separately: the dispatch skill teaches neither
the wait nor the do-it-yourself default.