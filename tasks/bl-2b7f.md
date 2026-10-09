+++
title = "verify the canonical scene end to end: an engine on one box, this window on another, over a stated address"
created = 1788138699
updated = 1791513616
claimant = "Mystical-2b7f"
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

---

Drive result (seat 0.1.67, engine yog 0.0.75, the engine box reached over its stated tailnet address from the seat box). ACCEPTANCE MET on all four counts.

1. Construction, from the CLI: `lernie entries` lists the flat root, the per-relationship entry for the engine box, and lab, each with its own address. `lernie ask workspaces` down that entry answered 13 conversations, 3 waiting, in about 0.16s. The hello did not refuse, so the protocol numbers agree.
2. Roster: the window paints one section per channel. Clicking the remote channel's header aimed it, moved its section to the top and painted its wall as "3 waiting 13 conversations (named)", with five top-level conversations and their show-N children (2+5+1+2+3 = 13). This matches `lernie conversations`.
3. Deposit: a NEW conversation was started from the composer with a one-word prompt. The pane said "starting in <channel>: <goal>". About 3s later the roster showed it [in-flight] and the wall's counts went to 14 conversations and "running". The CLI read afterwards gives 14 conversations, 4 waiting, matching the glass.
4. Reply: within about 12s of the click the pane showed the deposit, the model entry "pong" (3632 tokens), the header "quiescent", and the composer "it is waiting on you". `lernie transcript` agrees: two entries, 3627 tokens in and 5 out. The reply was one word, so the drive could not tell a token-by-token stream apart from a single frame.

Findings:
- DEFECT, filed as bl-f6b5: after the turn commits, the pane paints the answer twice. The committed "gpt-5.6-sol pong" is followed by a "«live» pong" row that stays after rest. The CLI transcript has no streaming entry, so the duplicate comes from the seat: Model::live is never retired when the turn commits or the lane ends.
- Minor, not filed: during the in-flight seconds the pane header said "waiting to hear what this conversation is" with the transcript empty. The operator's own deposit did not show until the first transcript read landed.
- Driving hazard (environment, not this crate): on a Wayland session, DISPLAY=:99 alone does not confine the window. winit prefers WAYLAND_DISPLAY, so the first launch opened on the operator's real display and was killed at once, with no input sent. Safe launch: `env -u WAYLAND_DISPLAY DISPLAY=:99 lernie`. Also, `xdotool --window` delivers synthetic events that the window ignores; plain XTEST input on :99 (windowfocus, then type/click) works.

The conversation to remove on the engine box: BrackenBramble in workspace NoodleZoo.
