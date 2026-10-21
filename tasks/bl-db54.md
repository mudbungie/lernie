+++
title = "the settled working's line says what it hides by kind — N tool calls · M thoughts · K intermediate responses — on a full-width lit door; '+ worked · N calls · M notes' was a weak-ink run nobody reads as a control"
created = 1792561594
updated = 1792561594
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
## The defect, as reported and as reproduced

Operator report 2026-10-20: the tool collapse goes too far and there is no way to reopen it; everything between the operator's prompt and the model's final response should fold into a status line that says what is hidden, in the shape *16 tool calls, 13 thoughts, 4 intermediate responses*.

Reproduced on the shipped 0.1.106 against a real transcript on an isolated Xvfb seat: a settled turn folds to `+ worked · 8 calls · 0 notes` in `INK_WEAK` at body size, whose click seat is the text run alone inside a full-width trail bubble. The click DOES open it (and a compact step DOES open whole) — the door works; it is not legible as one. So the defect is the line: it lumps thoughts and intermediate prose as "notes", it is a label-shaped run, and nothing on it says it is the one way back in.

## The ruling, applied (amends DESIGN §4.45 item 5's one-line head)

1. **The line counts by kind, zeros left out, in this order**: `tool calls` (Kind::Call | Answered | Result), `thoughts` (Kind::Thinking), `intermediate responses` (Kind::Said in the model's weight), `notes` (any other Said inside the working — a peer's, a compaction). Through `render::counted` for the singular. `worked` goes: the counts are the sentence. Example at rest: `+ 16 tool calls · 13 thoughts · 4 intermediate responses`; open: the same behind `−`.
2. **The line is the row idiom** (`theme::paint::Row`, as the compact step already is): the full width of the bubble is the seat, it lights under the pointer across its width, it holds the keyboard, and its help sentence says what the click does and that this is the one way to see the hidden steps.
3. Nothing else about the three stages changes: in flight whole, finished compact, settled one line; the memory stays per conversation and per row (bl-b83d); the `fold` knob still turns the self-folding off.
4. The phone follows in yog-android (its ball is filed beside this one): same words, same categories, so the two seats read alike.

## Where

`src/ui/chat/working.rs` (`counted`, `working`), `src/ui/chat/tests/stages.rs` and `idiom.rs` (`head`), DESIGN §4.45 item 5 (amend in place, cite this ball), STYLE §5 if it names the old words.