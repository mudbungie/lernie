+++
title = "the rule's new segment controls take a Tab and show no ring: the keyboard lands on them invisibly"
created = 1791868221
updated = 1791868221
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: with the caret in the prompt line (seated window), press Tab.
Expected (bl-2d6b, keys::tests::focus: the focused thing is visibly the focused thing — whatever holds the keyboard wears the brand ring): the control Tab lands on is ringed.
Observed: the focus moves to a small click-sensing control on the status rule (the model segment bl-4e55 made a control; rect ~44x13 on the rule's line) and nothing on the glass is ringed. Measured in the headless window: focused id is that control, brand-ringed rects = 0. The old ring test only ever tabbed from nothing to the first control, so it could not see this.
Why it matters: a keyboard operator who tabs off the box loses the cursor with no mark of where it went.