+++
title = "the off-frame threads: the asker, the poster, the follow lane, and the union roster"
created = 1788068281
updated = 1788069441
claimant = "OrderGlazier"
priority = 4
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-428f"
on = "claim"
+++
A frame that never blocks means no read and no act happens on it. The window holds one standing-question set per channel and reads back what landed; the acts go out on their own thread and their receipts land later; a follow-class read holds one connection open on the focused conversation so the serial pass is never stalled by a read that deliberately never finishes.

The transport half of this already exists here: `Channel::follow` hands each frame over as it arrives, and `ask` is written in terms of it. What is missing is everything above the socket — the threads, the link that holds what is outstanding, and the roster composed as the union across channels with every row carrying the channel it came from (a client-side stamp: no origin crosses the wire).

This is also the ball that gives `src/state.rs` its first tenant. The lock confinement rule names that file already and it does not exist yet, which is the rule working: the first lock does not get to choose where locks live.