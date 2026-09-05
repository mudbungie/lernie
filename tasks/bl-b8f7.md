+++
title = "the trail pane reads but cannot act: an alarm cannot be acknowledged and nothing truncates the trail"
created = 1788581504
updated = 1788582907
claimant = "Animations-O"
parent = "bl-4c48"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
bl-4c48 landed the trail's READ half — `crate::ui::trail` paints `reply/ops`
rows, standing while the pane is open, fanned across every channel. Its two
acts were left, and this is them.

## The two frames, exact

Both name no workspace, so both take the poster's fanned path
(`crate::offframe::poster`: *"a gesture naming no workspace is FANNED, not
routed"*) — the pane is the union across channels and so are its acts.

    {"op": "ack"}            ->  {"ok": true, "kind": "acked"}
    {"op": "clear-trail"}    ->  {"ok": true, "kind": "trail-cleared"}

Both fixtures are already vendored, in `corpus/unreadable/` — `acked.json` and
`trail-cleared.json`. Landing the acts moves both to `corpus/answers/`, and
that diff is the record of what this ball added.

## What each one means

`ack` appends the acknowledgement watermark every failure-derived alarm reads
past: it is the second of the two ways an alarm comes down, the first being
retirement by a newer clean run of the same verb (REMOTE §9.17). The rows
already on the glass say which ending each alarm had, so the pane can state
what the control did rather than predicting it — the next standing read
answers `acked` on the rows that were `live`.

`clear-trail` truncates the trail and logs the clear as the new trail's first
row. It is destructive in the sense the unmaking is: what is gone is gone, and
the row that replaces it is not the rows it replaced.

## The two open questions, and neither is a detail

**Both acts fan, and `clear-trail` fanning means one control truncates every
engine's trail.** That is what the wire's own shape gives — there is no
workspace field to address one channel with, and no seam in `Posted` for a
channel either. Either the fan is the right reading and the control says so in
words, or `Posted` grows a channel and the pane offers the act per section.
Decide it before writing the control, not after.

**`clear-trail` needs the unmaking's arming or an argument that it does not**
(DESIGN §4.20). `delete-workspace` is armed by typing the name back; a truncation
that a single click spends, across every engine, is the case that rule was
written for.

## Where it goes

`src/verbs/trail.rs` (the family's home, holding the `ops` door already),
`src/ui/trail.rs` for the controls, `src/ui/model/trail.rs` for the doors,
`src/reply/read.rs` for the two receipt kinds. Delete the `ack` and
`clear-trail` rows from `parity.toml` when each control lands.