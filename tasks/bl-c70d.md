+++
title = "a receipt is stamped with the aimed channel, not the one it was routed down"
created = 1788400516
updated = 1788400516
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
`src/offframe/poster.rs` stamps every receipt it files with the channel
`Standing::aimed()` answers, on the stated ground that an aim is "where a
composed gesture came from". It is not where the gesture GOES. A gesture is
routed by the address it carries (`seat::route`, DESIGN §4.7), and an operator
may compose one aimed at a wall on channel A while a control fires at a row on
channel B — the receipt is then filed under A.

## Why it is cheap today, and why it stopped being free

Only two filings on the model are keyed by channel. The roster's is, and no
control composes a `workspaces` gesture, so no receipt has ever reached it. The
decision queue's sections are the second (DESIGN §4.19), and `seen` DOES return
one — its reply is `reply/attention`, the queue that remains, not a receipt.

That ball took the hazard out rather than leaving it: a queue row's ADDRESS is
resolved against the roster (`Model::wall`), never against the section's stamp,
so a wrong stamp cannot aim a gesture at a different wall. What is left is a
section painted under the wrong header for one beat, until the standing read
replaces it. Cosmetic, bounded, and the next channel-keyed pane may not have a
roster to fall back on.

## The fix is where the fact already is

`seat::route` finds the entry by leaf and therefore already knows the seat-side
name of the channel it opened — `OWN` for the flat root, `holds::label` for an
entry — and discards it. Answering it beside the channel lets the poster stamp
what it actually dialled, and deletes the `unwrap_or_default()` empty channel a
gesture composed with no aim currently earns.

The cost is a signature change on a `pub fn` with three call sites (`seat::sent`,
`offframe::asker::aimed`, `offframe::poster::tick`). Keep the §8.2 mapping spent
at the one place it is spent now: the name must come OUT of `route`, never be
re-derived by the poster.