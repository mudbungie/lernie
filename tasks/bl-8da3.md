+++
title = "every visible pane edge drags: the list's edge against the conversation and the composer's top, the dragged width and height are the seat's own in the place file, and the policy keeps only the default and the floors (DESIGN §4.39)"
created = 1789705009
updated = 1789705009
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ux-overhaul"]

[[blockers]]
id = "bl-46e5"
on = "claim"
+++
Read lernie docs/DESIGN.md §4.39 first — the paragraph *Every edge an operator can see is one they can drag* is the whole ruling, and it reverses bl-fef8's sentence in §4.11 (already annotated there).

What to build:
- The side panel in `src/ui/shell.rs` (`lists`) becomes `resizable(true)` with a `width_range` of the policy's floors: `SIDE_FLOOR ..= window − CHAT_FLOOR`. The pane's body begins with `ui.set_min_width(ui.available_width())` — that is the one-line reason the drag ever snapped back (egui 0.30 `containers/panel.rs`: SidePanel floors content at `width_range.min` and stores the content rect; TopBottomPanel fills its width itself). Verify that claim against the pinned egui source before relying on it.
- The shown width each frame is the seat's dragged width where one is held, else `policy::widths`'s answer, clamped to that range. A window narrower than the stored width CLAMPS it, never overwrites it. The narrow shape consults no drag.
- The composer's bottom panel gets the same treatment on its top edge: the dragged height sets the field's row count, `theme::COMPOSER_ROWS` becoming the default; keep §4.38's bound (the panel is handed a height, it never reads one back off its content — that was the grew-a-row-a-frame defect).
- Both values are held on the `Model` and written to `src/place.rs` beside `aim` (keys of your naming, e.g. `list_width`, `composer_rows`), once, after the event loop returns, exactly as the aim is (§4.13). Unknown keys ignored, missing keys absence, no way to fail to start.
- Tests: the policy's clamp as a pure function; a kittest frame that drags the edge and reads the width back the next frame (the snap-back regression); place round-trip with the new keys and with a file lacking them.

Landing in parallel: the accordion ball adds its own keys to `src/place.rs` — keep your keys additive and your `read`/`write` edits minimal so the merge is trivial. This ball does not touch `src/ui/roster.rs` or `src/ui/convs.rs`; the two-column fold is a later ball gated on this one, so the three-column policy stays as it is here (still two side panels; both drag).

Update DESIGN §4.11's width paragraphs and the module map rows for the files you touch. Add rows for any file you split. 300-line cap, 100% coverage, `make check` green before `bl close`.