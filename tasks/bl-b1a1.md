+++
title = "the box never holds the caret on its own: not at launch, not after Ctrl+X picks a conversation, not after Esc clears the menu or closes an overlay — every keyboard path dead-ends until the operator clicks the box"
created = 1791867943
updated = 1791867943
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture (real window, XTEST keys only): launch with a wall aimed, or Ctrl+X → walk → Enter on a conversation, then type /help. Or: click the box, type /re, Esc, type /help.

Expected (DESIGN §4.42 keyboard, whole; the reach bound 'every pane is two gestures from the field: /, the word' — and back): the prompt line is where typing lands whenever nothing else takes text, as in the baseline.

Observed: nothing lands. The composer requests focus only when a row menu prefills it (composer.rs 'wanted'); egui's TextEdit surrenders focus on Esc, and an overlay or the switcher unpaints the box so egui drops its focus. After every one of those gestures the next keystrokes go nowhere, silently.

Why it matters: the shape is keyboard-first (Hermes baseline). As shipped, every pane is two gestures plus a mouse click away, and an operator who types after Ctrl+X loses what they typed.