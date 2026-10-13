+++
title = "no wall aimed, no box: the window verbs (/help /doctor /attention /find /conversations) are unreachable by typing while the seat panel says /help for commands"
created = 1791867538
updated = 1791869487
claimant = "LaneVerbs"
parent = "bl-957a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: launch the seat with an engine reachable and nothing aimed (first launch, or a saved place with no wall), type / (or click the bottom line and type /).

Expected (DESIGN §4.42 items 4-5): the seat panel says '/help for commands'; the window's own verbs (refresh, attention, help, doctor, trail, board, prices, find, conversations) need no wall, so / opens the menu with those rows.

Observed: the composer paints the sentence 'pick a workspace to say anything or begin anything' (composer::NOWHERE) and no field at all. Nothing typed lands; / opens nothing. The only way in is Ctrl+X. The seat panel's own two hints ('/help for commands', 'begin one in the box below') point at a box that is not there.

Why it matters: it is the first frame every operator sees. The help the window advertises cannot be reached the way it says.