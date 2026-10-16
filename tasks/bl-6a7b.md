+++
title = "Tab under an open settings pane walks the bar's pin, + and every row before it reaches the pane's own controls, and a slider that holds the keyboard shows no ring"
created = 1792122022
updated = 1792122022
parent = "bl-5369"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Seen on the verification lane for bl-5369 (DESIGN 4.46 item 3, every control is a Tab stop): with /settings open, the first Tab lands on the bar pin, the next on + new and the rows, and only the eighth or so on the pane first check box; the sliders after it take the arrows but paint no brand ring while focused, so a keyboard operator cannot see which knob the arrows will move. Two candidates: an open overlay takes the Tab order first (its controls lead while it stands), and theme::paint::slider paints the brand ring on its handle while it holds the keyboard, as a door does (bl-8e3e).