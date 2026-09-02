+++
title = "the window has no narrow layout: at a phone-shaped viewport the panes collapse to unreadable columns"
created = 1788330085
updated = 1788330085
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Found by the snapshot harness bl-dc07 built, and its evidence is a picture: run `make snapshots` and open `target/snapshots/enrolling--phone.png`.

At a 400x800 viewport the shell lays out all four surfaces side by side, as it does at every other width. The two list panes go to their floor (`ui::shell::SIDE_FLOOR`, 140 points each), which spends 280 of the 400, and every remaining surface divides what is left:

- the enrollment pane becomes a column roughly 45 points wide, so its text wraps to **one letter per line** — the heading reads as a vertical stack of single characters, and the form grows past 800 points tall;
- the control that CLOSES that pane, and so the control that forgets the private key it is holding, is laid out below the bottom of the window;
- a band of the window is left unpainted entirely.

`ui::shell::widths` already states, in its own words, what the layout does as the window narrows: the side panes yield in proportion until they hit their floor, and past that nothing yields and the conversation goes under `CHAT_FLOOR`. So this is not a surprise to the code — it is the documented end of the policy, and what is missing is an answer for what happens after it.

The shape of an answer is probably a mode rather than more yielding: below the width where the conversation still gets its floor, show ONE pane at a time with a way between them, the way a hand-held client does. That is a design decision and this ball is where it gets made, not assumed.

What the harness does about it meanwhile: every size is rendered and photographed, and the two GEOMETRY assertions are judged only at widths `ui::shell::widths` still promises a shape for (`snapshot::promised`). The reachability assertion is not gated by that and passes at 400 — the enrollment control is reachable there, it is the pane behind it that is unusable. When this ball lands, the phone row starts being judged with no change to the harness.