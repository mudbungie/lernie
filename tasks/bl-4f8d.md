+++
title = "/find with a long needle widens the overlay past the window: the 'looked for' line does not wrap and the box loses its right edge"
created = 1791867945
updated = 1791867945
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: type /find followed by ~500 characters with no spaces (or any needle longer than the window), Enter.

Expected (§4.42 item 6, §4.32): the pane's content wraps or is cut inside the box; nothing is laid past the window.

Observed: the section's 'looked for "…"' line is laid on one run; the box grows with it, its right border leaves the glass, and the result line under it runs off the window. The needle field itself stays one line at the head.