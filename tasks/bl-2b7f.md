+++
title = "verify the canonical scene end to end: an engine on one box, this window on another, over a stated address"
created = 1788138699
updated = 1788138699
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Re-homed from the server repo's board after the severance; original id bl-320b.

## Both halves the original named are answered by construction

The original was about the server's in-process window, which built its client
material by forcing loopback over whatever address it held — so a box running
the engine and a box running the window were necessarily the same box, and
REMOTE §1's canonical scene ("a home server runs the engine and keeps every log;
a phone seat talks to a conversation") had no graphical half at all.

Neither half survives into this crate. Both were checked against this tree
rather than assumed:

- **A stated address is the only address there is.** `channel::material` reads
  `address` — one `host:port` per relationship, no flag, no second spelling —
  and nothing here rewrites it. There is no `loopback()` and no in-process
  engine that could force one. Port zero is refused with its own sentence rather
  than dialled (`seat::tests::routing`, *"a self-provisioned loopback root says a
  seat wants a stated address"*), which is the same fact said from the other
  side.
- **Material is per relationship, not per box.** The original's second half —
  *"a client reads its certificate material from one place, so a box can hold
  exactly one CA, one client leaf and one address; a machine that already runs
  its own engine cannot also be a client of somebody else's without its own
  window breaking"* — is dissolved by `channel::entries`. The flat root is this
  box's own engine, every other participation is its own directory with its own
  anchors, leaf, key and address, and they share nothing (DESIGN §4.6:
  *"Separation is the absence of a mechanism"*). A laptop that is a seat of a
  primary server and also runs an engine holds both at once.

## What is left, and it is the only thing left

The construction is verified; the SCENE is not. Nobody has driven the canonical
arrangement end to end — an engine on one box, this window on another, over a
stated address, with the leaf and anchors carried by hand.

Acceptance: from a second box, the roster paints the server's walls under that
channel's section, a conversation opens, a deposit lands, and its reply streams
back. Whatever that drive finds is this ball's real content; finding nothing is
also an answer and closes it.

Not a code ball unless the drive makes it one. Sequenced behind whichever
install ball actually stands two boxes up.