+++
title = "an act's receipt outlives what it was about: 'seen X' stands in the state slot over the next conversation's state, and a dropup's model/effort/priority pick leaves a bare 'ok' over the state word until Esc"
created = 1791870719
updated = 1791870719
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: answer seen on one conversation, then + new and begin another (or pick a model on the rule's model dropup). Expected: the state slot says the selected conversation's state (live · 3s, quiescent · 2m); a receipt about another conversation is gone once the selection moves, and a selector pick's receipt is the segment itself repainting. Observed: 'seen <other> — off the queue' stood over the new conversation's whole turn, hiding live/quiescent; a model pick left 'ok' in the slot until Esc. Why: since bl-9760 every successful act posts Notice::Said until Esc; the deposit exemption (message, interrupt) was not extended to the role writes the selectors spend, and nothing retires a receipt when the selection moves. Round 1 had neither: role writes said nothing.