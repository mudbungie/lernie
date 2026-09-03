+++
title = "three window-level reads reach no gesture: the roster refreshes itself, and help and search have no home at all"
created = 1788397359
updated = 1788402386
claimant = "Gesture"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Three `control`-classed ops with no interactable here, and they are the ones scoped to the window rather than to a wall or a conversation: `workspaces`, `help`, `search`.

`workspaces` is the genuine catch the parity assertion turned up (yog docs/PARITY.md §2: the interactable a query owes a seat is the affordance that reaches the view it populates). The roster IS that view and it is populated by a standing off-frame read; no gesture reaches it, refreshes it, or fails visibly when the read does. Every other read this seat performs hangs off a control — aiming at a wall reaches `conversations`, selecting a conversation reaches `transcript` and `follow` — and this one hangs off nothing.

`help` and `search` have no pane at all: a verb's own page is readable only from the command line, and there is no way to find text across balls, workspaces, conversations and transcripts.

Exemption rows in `parity.toml` cite this ball until the surfaces land.