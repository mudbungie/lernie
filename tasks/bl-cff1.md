+++
title = "the roster's heading is engines and the pane is an accordion of them: one open at most, painted first, the rest ordered by when each was last opened on this seat, the open state and the order in the place file (DESIGN §4.39)"
created = 1789705010
updated = 1789705010
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ux-overhaul"]

[[blockers]]
id = "bl-46e5"
on = "claim"
+++
Read lernie docs/DESIGN.md §4.39 first — the paragraphs *The word on the glass is engines*, *The roster is an accordion of engines* and *What opens and in what order is the seat's own* are the ruling for this ball.

What to build:
- `roster::HEADING` becomes `engines`. The crate's word `channel` is NOT renamed anywhere in code; only the glass word changes (and the narrow bar follows for free — §4.11, one home).
- Each channel's section becomes an engine ROW in the paint anatomy (`theme::paint::row`, §4.38): `roster::header`'s words, the unheld/stale notes as today beneath it. At most one engine is open; its walls (as today, `ordered`) paint under it; a closed engine paints only its row. The open engine is painted first; the rest in order of last-opened instant, most recent first, name breaking ties. Clicking a row opens it and closes the other. Opening aims the wall last aimed under that engine, else its first by `ordered`.
- `src/place.rs` gains keys beside `aim`: the open engine's name, a map of engine name → last-opened instant, and a map of engine name → last-aimed wall. First run with no keys: the aim's channel is open, else the first by name. *Last used* = last OPENED by a click here (later the +), never a beat down the channel.
- The keyboard: `keys::walk` over `Pane::Roster` walks engine rows and the open engine's walls in paint order; landing on an engine row stands there, Enter/Space opens it; landing on a wall aims it as today. Do not open engines the walk merely passes through.
- `snapshot::worlds` gets a world with one engine open and another closed so the parity walk and reach walk see both states; `snapshot::reach`'s counts are re-read.
- Tests: the order as a pure function over (open, instants, names); the open/close as a model act; the place round-trip; the paint (which rows reached the glass, one open engine's walls and not the other's).

Landing in parallel: the drag ball adds its own keys to `src/place.rs` — keep yours additive and the `read`/`write` edits minimal. This ball leaves the middle conversations column in place; the fold that moves conversations under the wall and adds the + is a later ball gated on this one. Do not delete `src/ui/convs.rs`.

Update DESIGN's module map rows for the files you touch; §4.39 already states the design. 300-line cap (roster.rs is ~183 — plan a split, e.g. `roster/engine.rs` for the row and the order), 100% coverage, `make check` green before `bl close`.