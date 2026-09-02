+++
title = "a region of the window paints nothing and the desktop flickers through it"
created = 1788329772
updated = 1788329772
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Continues bl-6952, whose diagnosis eliminated the paint-regression and
changed-shape classes and could not name the pane. Two operator facts arrived
after it closed and together they name the class:

- **The `enroll a box` pane is open and sits to the RIGHT of the black box.**
  The enrollment is the central panel by construction, so the black region is
  NOT the enrollment and NOT the chat pane. It is the region the conversation
  list occupies — the middle of the three.
- **The region FLICKERS TRANSPARENTLY**: on some frames the desktop shows
  through it, and the operator says it is otherwise unclear what the region
  even is.

The second fact is the one that decides the class. **Paint logic that chose the
wrong ink could never show the desktop.** A region that is sometimes the
compositor behind the window is a region *nothing painted on that frame* — the
surface's own clear showing through, opaque black where the alpha path is
opaque and the desktop where it is not. So this is a coverage defect, not a
colour defect, and the question is which pane owns that rect and why it paints
nothing over it on some frames.

The QR symbol is eliminated: it is in the pane to the right, and it is 50% ink
on a white ground that no frame would render as a hole.