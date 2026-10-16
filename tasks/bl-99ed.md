+++
title = "an open pane leads the Tab order, and a slider rings and shows its trough"
created = 1792122135
updated = 1792122569
claimant = "Surveying"
parent = "bl-5369"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Fix for bl-6a7b: the frame a pane opens, its first painted control takes the keyboard, so Tab walks the pane before the bar (the switcher excepted, whose keys are the walk); theme::paint::slider paints the brand ring while it holds the keyboard and a visible trough under its rail.