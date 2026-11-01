+++
title = "a chat row's state glyph and ink say nothing on hover: the row's help sentence names the state in words, and the failure clause where the row is red"
created = 1793512653
updated = 1793512666
claimant = "State-Words"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Operator 2026-10-31, on the left-hand chat list: 'There is color coding and an icon (checkbox, red square). It's not clear what it means. It should show on mouseover pause (like most interface elements).'

Today: a bar row (and a switcher row) leads with convs::mark — ✔ at rest, ▶ working, ■ stopped, ? asking, a trailing ? where the server could not observe — each in its state's ink (theme::state_ink / accent), plus a state rule at the left edge and a tone (Tone::Bad = the failure hue). The row's hover sentence is convs::SELECT_HELP, 'shows this conversation', shown after HELP_DELAY (0.5 s) through theme::paint::told. The glyph and the colour are legend-less: nothing on the glass or under the pointer says what ✔ or a red ■ means.

Fix: the row's help sentence is derived from the row (one function in convs.rs beside mark/rule/glyph, so the switcher and the bar say the same words): it leads with the state in words — 'asking, N waiting' / 'working' / 'at rest' / 'stopped' / the unknown word verbatim — with 'as far as anyone has looked' where uncertain, then the failure clause verbatim where ConvRow::failure is Some (the red row's words, which today only the switcher paints beneath), then what the click does. One sentence (DESIGN §4.46 item 2's rule): e.g. 'stopped · provider refused: <clause> — click shows it'. The trashcan's help is unchanged. Tests: one per state, the uncertain suffix, the failure clause. Amend DESIGN §4.44 item 1 / §4.46 item 2 where the row's help is named.