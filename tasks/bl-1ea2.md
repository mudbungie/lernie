+++
title = "the first Ctrl+X after a click on a switcher row opens nothing; a second press opens the switcher"
created = 1791872275
updated = 1791941071
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: Ctrl+X opens the switcher; click a workspace row (it aims and the switcher goes down); press Ctrl+X once. Expected: the switcher opens. Observed on a real window (Xvfb, xdotool XTEST, isolated fixture engine), every time: nothing opens; a second Ctrl+X opens it. Seen four times across two worlds. Likely the click leaves a control holding the keyboard and the chord's Event::Cut is spent elsewhere, or the switcher toggles twice; not diagnosed. Why it matters: the one door to a conversation silently ignores the first press right after the operator used it.