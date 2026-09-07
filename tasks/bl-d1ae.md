+++
title = "restyle every pane onto docs/STYLE.md: rows with a state rule, threaded connectors, ruled transcript blocks, folded tool results, a composer that glows when it is your turn, records, notices and enrolment on the same anatomy"
created = 1788754421
updated = 1788754473
claimant = "Cantaloups-D4"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r3"]

[[blockers]]
id = "bl-73d2"
on = "claim"
+++
Follows bl-73d2, which landed the tokens (src/ui/theme.rs), the egui adapter and docs/STYLE.md. This ball delivers STYLE.md section 5: the row helper (theme::paint) with a left accent rule per state, the roster's rows and its window-level band with the queue entry in the attention accent while anything waits, the conversation list's threaded rows with L-shaped connectors in faint ink, the chat pane's aligned ruled blocks with the speaker's weight and folded tool results in weak ink, the composer field tinted with the attention accent while the selected conversation is asking, the records pane and every covering pane on the same anatomy (heading, subject one step weaker, close at the line's end, hairline sections), and the notice bar in the state's ink with no box. Every assertion reads the glyphs that reached the glass, ink included. Before/after screenshots under Xvfb in the usability notes repository, round 3, design-desktop.