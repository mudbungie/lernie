+++
title = "the decision queue has no pane, so a flag raised on a conversation is readable nowhere from this seat"
created = 1788319410
updated = 1788319410
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
PROTOCOL 4 landed in bl-d774 and the seat consumed one of the four fields the
two bumps carried. This is the rest of them, and they are one surface rather
than three fields.

REMOTE §6 is the engine's *"what needs you"* — a flattened roster across every
enumerated workspace, filtered to the rows that are asking. The wire answers it
as `reply/attention`, and this seat files that shape under `corpus/unreadable/`
because no pane here reads it. Three facts ride that row and are readable
nowhere from this seat:

- **`flag`** (REMOTE §9.11, PROTOCOL 4) — when somebody raised a flag on the
  conversation and why, in the raiser's own words, `null` when nobody did. The
  point of a flag is that a second party asks the operator to look; a seat that
  cannot show it is the one place the ask goes to die.
- **`flagged`** — the signal token that fires beside it, in the same array as
  `held` and `mail`. A new token in an existing vocabulary is not a shape
  change and rung 3 already carries an unknown word verbatim, so the cost here
  is a badge and not a decode.
- **`failure`** on the queue row (REMOTE §9.10, PROTOCOL 3) — the same clause
  bl-d774 put on the conversation row, spelled `null` rather than absent
  because that row's encoder already spells `held` that way.

Also on the row and equally unread: `held` — the invocation parked at the
conversation's capability boundary, which is what makes a queue row answerable
(`/answer pass`) rather than merely readable.

## What this is not

It is not a decode ball. `src/reply/` gains a kind the day a pane paints it
(DESIGN §4.9), and the ledger under `corpus/unreadable/` is the standing record
that this one is not painted yet. So the deliverable is a **pane**, and the
kind arrives with it.

## What it is next to

The `agent` answer gained the same `failure` clause at PROTOCOL 3 and is also
unread here. That half belongs with the provider-rung work (bl-b180, bl-e3c5)
rather than with this: those two are about a wall that cannot reach a model
before a goal is spent, and the clause is the same fact after it has been.