+++
title = "neither list pane scrolls: what does not fit the roster or the conversation list is cut off mid-glyph, and the keyboard walks onto rows the glass never painted"
created = 1788150516
updated = 1788150516
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Two list panes hold a list, and neither has any answer for a list longer or
wider than its box. The overflow is not scrolled, not elided and not counted —
it is cut off at the panel edge, mid-glyph, and there is nothing on the glass
that says anything was cut.

**Vertically.** Driven headless at a 900x260 window against an engine holding
three walls and five conversations: the roster's last painted row is sliced
horizontally through the middle of its glyphs at the panel's bottom edge, with
one further channel section below it painted nowhere at all. No scrollbar
appears in either pane at any size. The conversation list does the same to its
last preview line.

A pointer cannot reach what is off the bottom. The KEYBOARD can — `Tab` walks
into a row past the fold and paints its focus ring on the sliced half-row, and
the arrow walk (`crate::ui::keys::walk`) moves the selection onto rows that are
not painted at all — so the two surfaces disagree about which rows exist, which
is exactly the property `crate::ui::roster::aimable` is documented to
guarantee: *"a key cannot walk onto a row a click cannot reach, and cannot walk
in an order the glass does not show."* It holds against the `NO_NAME_HERE`
rows and fails against the fold.

**Horizontally, the same absent policy with the opposite victim.** The two
`SidePanel::left`s keep their widths as the window narrows and the
`CentralPanel` absorbs the whole loss. At 900 points wide the conversation pane
is a ~140-point strip: its heading is legible, the composer's text box under it
paints as `what this conversati` with the rest cut, and `start` sits against
the frame. The pane the window exists for is the one that disappears, and the
roster — the pane whose whole content is a handful of short words — is the one
that keeps 280 points of it.

**What it is not.** This is not the elision the paint layer already does well:
a conversation row's preview elides honestly with `…` inside its own width. The
defect is the absence of a rule one level up — what the pane does when its
CONTENT does not fit, in either axis.

The two faces are one missing decision and probably one fix at the shell: the
list panes scroll, and the central panel has a floor the side panels yield to.

Found by driving the window headless (X server, screenshots read back) against
a local engine; the paint evidence above is off those captures.