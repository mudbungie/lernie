+++
title = "the ball pane reads but cannot act: no ball can be filed, claimed, released, amended or closed from this seat"
created = 1788581980
updated = 1788583007
claimant = "Animations-T"
parent = "bl-d2af"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The act half of bl-d2af, which landed the read: `balls`, `board`,
`workspace-balls` and `marks` are painted (`crate::ui::board`, DESIGN §4.28)
and their four rows are gone from `parity.toml`. These five are what is left,
and they are the ops that CHANGE the store.

## The frames, exact — every fixture already vendored under `corpus/request/`

    {"op": "create",  "project": P, "name": N, "title": T}
    {"op": "create",  "project": P, "name": N, "title": T, "body": B}
    {"op": "create",  "project": P, "name": N, "title": T, "fields": [...]}
    {"op": "assign",  "project": P, "name": N, "id": I}
    {"op": "release", "project": P, "name": N, "id": I}
    {"op": "close",   "project": P, "name": N, "id": I}
    {"op": "update",  "project": P, "name": N, "id": I}
    {"op": "update",  "project": P, "name": N, "id": I, "title": T, "body": B, "note": M}
    {"op": "update",  "project": P, "name": N, "id": I, "fields": [...]}

`fields` is an array of objects — `{"field": "priority", "value": -2}`,
`{"field": "tag", "on": true, "value": "boundary"}`, `parent`, `needs` — so a
gesture carrying one is not rows of named strings and cannot be a row in the
verb table (§4.10). The bare forms are; `assign`, `release` and `close` are
three named strings apiece and nothing else.

All five answer with a **captured run** (`kind: "outcome"`), which this seat
already paints — so nothing new is decoded and the whole of this ball is
controls, their arming, and where each one hangs.

## The question to settle BEFORE writing a control, not after

**Not one of the five names a workspace.** They carry `project` and the `--as`
`name`, so `crate::verbs::Verb::addresses_a_workspace` is false for all five
and `crate::offframe::poster` FANS them: one click would file the same ball on
every channel this box holds, close it on every one, release it on every one.
That is the wire's own shape and it is not obviously wrong — a project is a
project — but it is not obviously right either, and it is the same question
bl-b8f7 asks about `clear-trail`. Either the fan is the right reading and the
control says so in words, or `Posted` grows a channel and each control is
offered per section. Decide it, in DESIGN, before the first control.

## What each act needs beyond the frame

- **A name to act AS.** All five carry `name`, and this seat has no identity of
  its own anywhere. It is not the wall's name and not the channel's; it is who
  the operator is. Where it comes from is this ball's second question — a box
  the pane holds, a seat-side setting, or the aimed wall's own `owner` off
  `workspace-balls`, which is the one answer already on the glass.
- **A project.** Every board row and every binding row carries one, so a
  control hanging off a ROW has its project; a bare `create` does not, and
  needs a box or a row to hang off.
- **`close` is the unmaking's case** (DESIGN §4.20): it delivers work and is
  the one of the five that cannot be undone by doing the other thing. Arm it,
  or state the argument that a close is recoverable.

## Where it goes

`src/verbs/balls.rs` (the family's home, holding the four reads), the controls
on `src/ui/board.rs` and `src/ui/board/wall.rs`, the doors on
`src/ui/model/board.rs`, and `parity.toml`'s five rows deleted as each lands.
The `update` and `create` field-list forms are recorded in
`src/verbs/tests/corpus/emits.rs`'s `UNEMITTED` ledger by count and reason
until a control composes them.