+++
title = "with no wall aimed there is no prompt field: /attention, /help and every aim-free verb are unreachable, the seat panel offers '/help for commands' and 'begin one in the box below' that cannot be typed, and the green 'N waiting' is not a door"
created = 1791867537
updated = 1791867671
claimant = "LaneAsks"
parent = "bl-957a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: open the window on a seat whose engine has asking conversations and nothing aimed (fresh place file), type / or click the prompt line.
Expected (DESIGN 4.42 items 2, 5; 4.19 'its control needs no aim and is offered on a seat that has aimed at nothing — which is the seat most likely to be asking the question'): the prompt field takes / and the menu lists the aim-free nouns (/attention, /help, /doctor, /enroll, /login, /prices, /fleet...); the rule's 'N waiting' opens the queue (item 8: 'the count on the rule is its door').
Observed: composer::render paints theme::paint::empty(NOWHERE) and returns when model.aim is None — no field, so no slash, no menu, no queue. The seat panel above says '/help for commands' and the hint says 'begin one in the box below'; neither can be done. Clicking '5 waiting' does nothing. Only Ctrl+X works.
Why it matters: the first frame of every fresh seat is a dead end for keyboard verbs, and the one state the queue exists for (operator not looking at anything, things asking) cannot open the queue.