+++
title = "a committed model entry with an empty thinking block paints a blank row, which the live path's own rule forbids"
created = 1788329660
updated = 1788581010
claimant = "Animations-J"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Found while diagnosing bl-6952, and reproduced against the live engine rather
than against a fixture.

## The defect

`src/ui/chat.rs` states the rule at `live_rows`, in its own words:

> The two halves of a turn in flight, each a row of its own and each omitted
> when it is empty: a model that has only thought so far, or one that answered
> without reasoning. **An empty half is simply no row — never a blank one,
> which would claim something was said.**

The live path obeys it. **The committed path does not.** `block()` turns every
`Block::Thinking` and `Block::Text` into a row unconditionally, so a committed
model entry carrying an empty thinking block paints a `<model> (thinking)`
header with a separator above it and nothing under it — which is precisely the
blank row the rule forbids, and it reads as *the model thought something and
this seat lost it*.

## The evidence

A real conversation on this box, read over the wire, decoded by the seat's own
reader and projected through `chat::rows`:

    who="user"                       said_len=189
    who="gpt-5.6-terra (thinking)"   said_len=0      <- the blank row
    who="gpt-5.6-terra"              said_len=7993
    who="user"                       said_len=97
    who="gpt-5.6-terra"              said_len=418

The engine emits `{"kind":"thinking","text":""}` as an ordinary block, so this
is not a malformed answer and not something to refuse — it is a half of a turn
with nothing in it, which the live path already knows how to say nothing about.

## The shape of the fix, and the trap in it

One rule with one home, obeyed by both paths — not a second copy of the filter.

**`Block::Unknown` must keep its blank row.** It is deliberately `said:
String::new()`, and the module's own header says *"Nothing is dropped,
including what nothing could parse … a transcript that quietly skipped an entry
would be a conversation the operator reads as shorter than it was."* So a
blanket *drop every row with an empty `said`* would delete the rung-3
surfacing, which is the opposite defect. The rule is about the two halves of a
turn, and it has to stay about them.