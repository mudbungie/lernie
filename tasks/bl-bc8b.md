+++
title = "the settings pane and the knob table: the wheel, the delay, the scale, the bar's four, bubbles, fold, confirm, dock; the place file carries them"
created = 1792120741
updated = 1792120741
parent = "bl-5369"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-2e2d"
on = "claim"
+++
DESIGN 4.46 items 1 and 4.

src/settings.rs: the knob table (key, label, help, kind with unit, floor, ceiling and step, default) and the typed Settings the readers read; from JSON and to JSON, writing only knobs off their default. The knobs: wheel 100, help on, help_delay 0.5, scale 1, linger 0.75, ease 0.12, bar_shut 80, bar_open a quarter of the desk, bubbles on, fold on, confirm on, dock off. place.json gains a settings object; Place::dragged and the bar_shut/bar_open keys go, a drag writes the two knobs. theme::install installs the wheel step (Options::line_scroll_speed), the tooltip delay (infinite while help is off) and the zoom factor from the settings each frame, and turns egui's own zoom_with_keyboard off. The bar reads linger, ease and the widths; the chat reads bubbles and fold; the delete reads confirm; the dock reads dock for its first frame.

src/ui/settings.rs: the pane, /settings on the menu and Ctrl+, in keys::handle (the chord moves into the bindings table when that lands): one row per knob, a slider for a number and a check for a toggle over the sentence in weak ink, the pin as a thirteenth row on the same fact as bar::PIN, and defaults at the foot. A change applies at the next frame; nothing here crosses the boundary.