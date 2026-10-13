+++
title = "Ctrl+O over an open overlay does nothing: the model dropup opens under the pane and the covered check closes it the same frame, where Ctrl+X puts the pane down and opens"
created = 1791870720
updated = 1791870720
parent = "bl-957a"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: /help (or any overlay), then Ctrl+O. Expected: the overlay goes down and the model dropup opens over the rule, as Ctrl+X replaces an overlay with the switcher (bl-339d). Observed: nothing; after Esc closes the overlay no dropup stands. Why: status::dropup::render closes any dropup while model.covered().