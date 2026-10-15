+++
title = "an opened trail paints a tool result's trailing newline as a blank line at its foot"
created = 1792035467
updated = 1792035467
parent = "bl-4792"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: open a trail (click its head) whose tool result ends in a newline, which is what nearly every shell result does. Expected: the result's last line is the last thing in the bubble, with the bubble's usual padding under it. Observed (0.1.97, the busy fixture with a grafted Bash call whose result is two lines and a trailing newline): the bubble's foot stands about one line height further below the last word than a prose bubble's does, because the result's text reaches the painter with its trailing newline and the galley lays it as an empty line. Why it matters: every opened trail wears a blank line at its foot, which reads as a missing row. Fix is a trim of trailing newlines where the trail's result rows are projected (the projection decides what is said, never the painter), with a paint-probe test reading the bubble's height against a result with and without the newline.