+++
title = "the Login pane: provider rows over the channel that hosts the wall, and the sign-in posted as a boundary act"
created = 1788138703
updated = 1788138703
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Re-homed from the server repo's board after the severance; original id bl-1ddb.
The original had two halves; only the pane half is here.

**The act is the server's and stays there.** `Action::Login` — spawning `bz`
inside the named workspace's wall on the ENGINE, its one-run-per-pair holder,
its hour sweep, and the follow-class lane that streams the run's lines — is the
server repo's bl-c285, filed on that board and not re-homed. This crate never
spawns a sign-in and never holds a run; it asks, it posts, and it paints what
comes back. That separation is the whole point of the surface-by-surface
migration REMOTE §9.5 describes, and it is what makes the pane portable at all.

## What this lands (REMOTE §8.3; needs the server's `Action::Login` and its lane)

The seat has no Login surface at all today, and nothing under `src/ui/` reads or
writes a credential. So this is new construction rather than a migration. What
it gains:

- **Rows.** `Query::Providers { workspace }` over the channel that hosts the
  aimed workspace: this box's own engine for a flat-root wall, the entry's
  channel on the entry's own material for one held elsewhere. The routing is
  free — `seat::route` already spends the leaf-to-host mapping at exactly one
  place — so the only new work is the reply frame and its strict decode
  (`src/reply/`, rung 1 field readers, and a corpus fixture per the vendored
  conformance corpus rather than a fixture of this crate's own).
- **Sign-in.** The control posts `Action::Login` through the outbox like every
  other composed gesture (DESIGN §4.11: the frame never dials), and the painted
  stream is the follow-class lane. Closing the pane terminates nothing — the run
  is engine-owned and bounded there, which is a property this side gets for free
  and must not re-implement.
- **Which sphere the sign-in is for, said in the surface.** The wall's name plus,
  for a wall held elsewhere, that it is held elsewhere — the roster's own
  client-side channel stamp (`ui::Channel`), never a host address in the
  sentence.
- **Run-by-hand fallback, per channel.** The original spelled a local wall's
  fallback as an `exec` on this box's own engine; that verb is not this crate's
  and there is no exec here at all. Both cases are therefore the same case now —
  an act the operator runs on the box that HOLDS the wall — so the fallback is
  one sentence naming the host, not two.
- **The loopback remedy sentence** (REMOTE §8.3). When the aimed wall is held
  elsewhere AND the row is browser-only, say beside the run that the authorize
  URL redirects to the ENGINE's loopback: complete it in a browser on that box,
  or forward the port by hand. A stated operator remedy, never a channel
  feature.

## Dropped from the original, with the reason

- **"`LoginHolder`'s in-process reads and its `bz`/`wall`/`creds_dir` fields
  go."** There is no holder here to strip; the pane is new construction rather
  than a migration, which is the one way this ball got cheaper.
- **"The auth-failed banner's inline seat migrates with the pane; both seats
  keep working."** That banner does not exist here and neither does the machinery
  under it. If the seat should carry a second inline seat for the same act, that
  is a decision to make when there is a first one.

## Discipline

Paint assertions through `crate::paint_probe` only
(`rules/no-hand-rolled-paint-walk`) — a galley reports the string that went in.
The pane must not become a fifth panel by accident: DESIGN §4.11 fixes the face
at four panes and a notice bar, so where a Login surface sits is this ball's one
layout decision and it should be made explicitly. Test material is minted by the
suite (`test_support::mint`), never a committed certificate. Run
`make line-cap LINE_CAP=199` before extending any module this touches.