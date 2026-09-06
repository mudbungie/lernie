+++
title = "the two left columns are fixed width at every window size, their rows wrap after three words, the preview is cut without an ellipsis, and a 160px band beside them is always empty"
created = 1788673700
updated = 1788674445
claimant = "Cantaloups-S3"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 daily lane. lernie 0.1.30, window under Xvfb, captured at 800x600 and 1440x900.

The two left columns are fixed width and the extra 640 pixels all go to the pane that needs them least. `rounds/1/shots/daily/00-default-800x600.png` and `01-roster.png` are the same layout at the two sizes: the channels column is about 175 px in both, the conversations column about 320 px in both, and every pixel of the resize lands in the conversation pane.

At both sizes:

- Roster rows wrap after two or three words. `tasks (named) 7 conversations 7 waiting` renders as three lines; `what these engines answer…` wraps inside its own button.
- The conversation-list preview is CUT by the panel edge rather than elided: "What changed in the last 20 commits of this repository" ends flush against the next panel with no `…`, so a reader cannot tell the sentence continues. The same seat elides correctly elsewhere — the wire's own `preview` field arrives ending in `…` — so this is the pane overrunning its own clip rect.
- A vertical band roughly 160 px wide sits empty between the conversation list and the conversation pane, at every width. `03-proj-selected.png` onward; it is blank in all of them.

Net at 1440x900: about 500 px of usable width for the two navigation columns and the dead band, against 780 for a pane whose content is already too wide to need it.

Expected: the two list columns grow with the window (or are draggable), the dead band is either explained or reclaimed, and the list preview elides at its own edge.

Severity p3. It is what makes the window feel cramped on a large display, which is the display a daily seat runs on.