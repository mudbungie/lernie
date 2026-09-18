+++
title = "the start mode is the same composer: the field at the same rows with start inside it, and the offers row carrying the one parameter a start has, the role (DESIGN §4.39)"
created = 1789705010
updated = 1789705267
claimant = "Cruises-C"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ux-overhaul"]

[[blockers]]
id = "bl-46e5"
on = "claim"
+++
Read lernie docs/DESIGN.md §4.39 first — the paragraph *One composer, in both of its modes* is the ruling for this ball, and §4.38's *The composer is the conversation pane's focal element* is the shape it must match.

Today `src/ui/composer/start.rs` paints a bare `theme::paint::field` and a `start` button, while the deposit (`src/ui/composer.rs`) paints `theme::paint::composer` — the field `COMPOSER_ROWS` tall with the send inside it — then the offers row and the `…` strip. The two are one control with two subjects (start.rs's own module doc) and do not look like one.

What to build:
- The start is laid through `theme::paint::composer` at the same rows, the act inside the field worded `START` where the deposit says `SEND`; Enter fires it as today; the `act:` tags (`PREPARE`, `PROMPT`) stay on that control (§4.16 — the parity ledger must not move).
- Under it, the same row shape (`composer/offers.rs`), carrying what applies to a start: the role the conversation is born on (`verbs::start::ROLE`, PROTOCOL 18 — read `src/verbs/start.rs`'s module doc on the role; today only plan mode sets it). `records…`, `interrupt`, `stop`, `nudge` and the `…` strip are acts on a conversation and are ABSENT in start mode, not greyed. Decide the role control's shape (the existing plan-mode control is the precedent — find it) and keep it a glyph and a word.
- The empty/held sentences (start in flight, receipt) keep their current behaviour.
- Tests: the start field's laid rect is the same height as the deposit's (off the glass, as §4.38's assertions are); the start control still carries both act tags; the offers row in start mode carries the role control and none of the four conversation acts.

This ball touches `src/ui/composer/**` and `src/ui/theme/paint/field.rs` only. Two other balls land in parallel on `src/ui/shell.rs`, `src/ui/roster*` and `src/place.rs`; do not touch those. Update DESIGN's module map rows for what you touch. 300-line cap, 100% coverage, `make check` green before `bl close`.