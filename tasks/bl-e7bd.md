+++
title = "everything the window does is reachable from the keyboard"
created = 1788069437
updated = 1788069437
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The window (bl-428f) sends on Enter and does everything else with a pointer: aiming a wall, selecting a conversation, dismissing a notice. A face an operator has to leave the keyboard for, once per selection, is a face they use through the command line instead.

Upstream settled this as a standing rule rather than a feature — everything keyboard-operable — and the seat inherits the obligation without inheriting the implementation, because the window here is four panes rather than yog's whole surface and the shape that fits four panes is not the shape that fits thirty.

What it wants:

- A focus that moves between the panes and within a list, and paints where it is. A focus that cannot be seen is a focus nobody can use.
- The two selections and the dismiss, bound.
- The acceptance discipline the window already has: a keyboard beat drives real key events into a persistent context and reads back the glyphs on the glass, exactly as the pointer beats do. `crate::paint_probe::frame` already holds the driver and the key-press helper.

What it must not do is grow a second table of what a control means. A binding names a control that already exists; a binding that could fire something a click cannot is a second surface.