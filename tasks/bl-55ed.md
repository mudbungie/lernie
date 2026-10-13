+++
title = "/help lists the engine's op table spelled as slash words the box does not take (/scan, /workflow, /close, /assign…), and never lists the window's own words or keys"
created = 1791867944
updated = 1791867944
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: type /help, Enter.

Expected: the operator who asks the window for help learns what the window takes: its slash words (the completion menu's rows, from the same verb table lernie help reads) and its keys (Enter, Shift+Enter, /, Tab, Ctrl+X, Ctrl+T, Esc, digits on the ask). The seat panel advertises '/help for commands'.

Observed: the overlay is 'what these engines answer' — every engine op with a '/word [control]' headline and the engine's own detail page. Most of those words (/scan, /workflow <lineage>, /clear-workflow, /close [id], /assign, /release…) are not rows of the menu and typing them answers 'nothing here answers to that'. Some engine text is stale for this window ('Ctrl+Enter in the composer is this gesture'). The window's own words and keys are written nowhere on the glass but the menu.

Compared with lernie help (scenario 7): different subjects by design (commands.rs 'And it is not lernie help'), so same-words-same-order is not the bar; the defect is that the window's own help is missing and the engine table is dressed as the window's commands.