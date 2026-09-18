+++
title = "the middle column is gone: a wall's conversations stand beneath it in the engines accordion, a + on the engine row begins a conversation, the keyboard walks one list, and the width policy has two columns (DESIGN §4.39)"
created = 1789705011
updated = 1789705882
claimant = "Cruises-F"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ux-overhaul"]

[[blockers]]
id = "bl-46e5"
on = "claim"

[[blockers]]
id = "bl-8da3"
on = "claim"

[[blockers]]
id = "bl-cff1"
on = "claim"
+++
Read lernie docs/DESIGN.md §4.39 in full first; this is the last of its four balls and the one that changes the window's shape. It is gated on the drag ball (bl-8da3) and the accordion ball (bl-cff1), which must be on main before you start — read what they landed (`git log`, `bl show`) rather than the design's prediction of it.

What to build:
- Under the open engine, each wall row is followed by that wall's conversations: the rows `src/ui/convs.rs` paints today (`convs::row::conversation` with rails, elbows, the §4.23 menu) called from the roster, for the AIMED wall using `Model::rows` (a started conversation's claim row included). For a wall that is not aimed, decide and state whether its conversations paint (the design says *under each wall row the conversations that wall holds*; the standing read set today asks only about the aimed wall — §4.12 — so a non-aimed wall's rows may be unknown; paint what the model holds and say in the doc what a non-aimed wall shows). Clicking a conversation selects it and aims its wall in one act.
- The `+` at the right edge of every engine row: clears the selection, keeps/sets the aim to the engine's aimed wall, opens the engine if closed (a start is a use — update the last-opened instant), and focuses the composer's box (`keys::BOX_ID`). It crosses no wire; no `act:` tag.
- `shell::policy`: two columns. `Column` has two variants (Engines, Conversation); `widths` yields ONE list pane (worth `CONVS`) to `CHAT_FLOOR`; the narrow floor is where that can no longer hold; the drag from bl-8da3 clamps to the new range. `SIDE_FLOOR` unchanged. The bar names two.
- `keys`: one cursor track in paint order (engine rows, the open engine's walls, their conversations); `Pane::Conversations` goes; landing on an engine stands, Enter/Space opens; wall aims; conversation selects. Every pointer act reachable by key (QUALITY F1).
- Delete `convs.rs`'s pane-level `render`, its `HEADING` and its pane emptinesses; keep `headline`, `age`, `continues`, `row.rs`, `menu.rs` (move them under `roster/` if that reads better; update the module map either way).
- `snapshot::worlds`, `snapshot::reach`, the kittest matrix, `parity`: re-read against two columns. The parity ledger must stay empty of anything this fold removed — every control that fired an op still fires it.
- Empty sentences: every emptiness names the next act (§4.38); the old *pick a workspace under channels* wording dies with the word.

Amend DESIGN §4.11's opening (already annotated) and its narrow-shape paragraphs to say two columns, and the module map. 300-line cap, 100% coverage, `make check` green before `bl close`.