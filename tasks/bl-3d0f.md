+++
title = "the notice bar does not wrap, so every refusal is cut at the window frame and the remedy is the half that is lost"
created = 1788150533
updated = 1788150533
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
`crate::ui::shell::notice` paints the notice as a `ui.horizontal` holding the
dismiss button and one `colored_label`. A horizontal layout does not wrap, so
the line runs to whatever width it wants and the panel cuts it at the window's
right edge. Nothing elides it, so there is no `…` to say that it was cut.

**The half that is cut is always the half that says what to do.** Every refusal
the channel layer composes puts the fact first and the remedy last. The
provisioning refusal is:

    no wire provisioned at <the directory>: the pair is minted on the host
    that issued it (`yog wire-certs WIRE_LEAF=<name>` there) and carried here
    by hand; the seat mints nothing

Driven headless with a data root holding no material, at the window's own
default size, the bar painted:

    ×  this seat could not reach it: no wire provisioned at <a long path, cut
       off against the window frame>

— the directory itself unfinished, and every word of the remedy off the glass.
That is the FIRST RUN of the seat on a new box: the operator has no channel,
the notice is the only thing on the window with an instruction in it, and the
instruction is the part that is not painted. A shorter path only moves where the
cut falls; it does not change which end is lost.

The same clipping hits every refusal long enough to reach the frame, and the
channel layer's refusals are deliberately long because they name an act on
another machine.

Two candidate fixes and the first looks right: wrap the notice's label, so a
long sentence costs a second line in a bar that is already sized to its
content. Failing that, the bar must at least elide, so that something on the
glass says the sentence continues — but a notice the operator cannot finish
reading is barely better than one they cannot see.

Related but separate: whose channel a notice is about is bl-e620.

Found by driving the window headless (X server, screenshots read back).