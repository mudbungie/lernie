+++
title = "the transcript's bubbles by role, and a trail that folds itself once its calls are done"
created = 1792034031
updated = 1792034031
parent = "bl-4792"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-b7cb"
on = "claim"
+++
DESIGN §4.45 items 5 and 7. Four bubbles under what is said, each a filled rounded block at RADIUS behind the row with the gutter glyph kept: the operator's turn on a brand tint, a trail on the ground inside a hairline, intermediate prose (prose a further trail follows within the same turn, and the live turn's prose while it streams) on SURFACE, the turn's final prose on RAISED — the elevation ramp rising toward what is read. The fills are named once in theme (theme::bubble) and derived from the palette, no new literal. The trail fold's default is open while any of its steps is live or its turn still runs, and shut once the turn settled; an explicit toggle is remembered as now. The trail's head and every other clickable seat in chat.rs wear the hover wash. chat.rs is at 252 lines: split the trail painter out (chat/trail.rs) before adding. Snapshot tests read the four fills off the probe.