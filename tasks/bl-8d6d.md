+++
title = "the ask box stays up saying 'Enter sends' while that conversation's engine is down; and the model list cuts long model ids with … even at desk width"
created = 1791871671
updated = 1791871671
parent = "bl-957a"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Two round-2 leftovers (bl-0639). (1) With the aimed engine down the ask box keeps standing with its choices and the hint 'it is waiting on you — Enter sends', though nothing can be answered: while the aimed engine is down the box's choices go faint and the hint is the engine's own sentence (bl-4cac's state-slot reading). (2) The model dropup elides long model ids with … at 1440 wide where there is room; the list's width is its longest row's, capped at the pane, not the selector's share.