+++
title = "the seat composes no stop-with-children, and on a conversation with children it is the only stop that works — the records pane prints the offer beside no control"
created = 1788673676
updated = 1788674766
claimant = "Cantaloups-S1"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 daily lane. lernie 0.1.30 against yog 0.0.38, litany 0.0.10.

`lernie help stop` states the gap outright:

    It kills the driver and leaves everything else standing ... This is the BARE
    form. The wire also carries a `children` flag that takes the whole subtree
    down, and this seat composes no gesture that raises it — a cascade is a
    second control with a second confirmation, and it belongs beside the
    conversation records that would say what is under there.

Measured this lane: on a conversation with children, the bare form does not stop it. `lernie stop` answered `{"exit":0,"kind":"outcome","ok":true}`, the conversation read `stopped` for one poll and was `in-flight` again five seconds later, spending another 796,522 tokens over the next forty seconds. `lernie ask '{"op":"stop",...,"children":true}'` stopped it and it stayed stopped. Reproduced on two conversations in two workspaces. (Filed against the engine as its own defect; this ball is the seat's half.)

The consequence for the seat is that the only stop that works is reachable only by hand-writing the envelope — and the window is worse than the CLI, because the window has no `ask`.

The records pane already prints the engine's own offer, verbatim, in the conversation block:

    the engine offers nudge, stop with its children

so the seat renders the sentence that names the control and then does not draw it. The reasoning quoted from the help — "it belongs beside the conversation records that would say what is under there" — has since come true: the records pane exists, it lists the members, and the window's conversation list already shows `4 members`.

Expected: a cascade control, in the records pane if that is where it belongs, armed the way `delete-agent`'s typed name arms it. And a CLI form — `lernie stop <ws> <agent> children`, matching `/stop [children]`.

Severity p1: the one control an operator reaches for when something is wrong is the one that does nothing, and the seat knows the working spelling and will not say it.