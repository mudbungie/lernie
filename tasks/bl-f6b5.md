+++
title = "the chat pane paints a committed answer twice: the live fold outlives the turn it was folding"
created = 1791513611
updated = 1791513611
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Found by the canonical-scene drive (bl-2b7f): an engine on one box, this window on another, over a stated address.

## What the glass shows

A new conversation started from the composer, asked for a one-word reply. After the turn committed and the conversation came to rest (header `quiescent`, roster `1 waiting`, composer "it is waiting on you"), the chat pane paints the answer TWICE and keeps doing so (observed for over thirty seconds after rest):

    user          Reply with exactly one word: pong. Use no tools.
    gpt-5.6-sol   pong
    «live»        pong

The CLI's `lernie transcript` of the same conversation shows exactly two entries (the deposit and the model entry) and no streaming entry, so the engine is not serving a stale tail in the committed read; the duplicate is the seat's own.

## Why, read from the tree

- `src/ui/model/absorb.rs`: `Reply::Transcript(t) => self.transcript = t` and `Reply::Follow(s) => self.live = Some(s)`. Nothing clears `Model::live` when the turn commits or when the follow lane ends; only `select` and an aim change do (`src/ui/model/acts.rs`).
- `src/ui/chat/rows.rs` `rows()`: the live fold replaces only a committed `Streaming` entry. Once the turn lands as a `model` entry there is no streaming entry to replace, so the live rows are appended after the committed answer. The function's own doc says "Appending would paint the answer twice" — which is what happens.
- `src/offframe/follow.rs` `tick`: when the engine ends the stream (REMOTE §5.5, a quiescent conversation with no mark ends the stream), the last fold stays in the model.

## Acceptance

After a turn commits, the pane shows the answer once. A test drives the model through a follow fold, then a transcript carrying the committed entry (and/or the lane's end), and asserts no `«live»` row remains beside the committed one. Decide in the ball which signal retires the fold (lane end vs. the committed read advancing) and say it in DESIGN if it is a rule.