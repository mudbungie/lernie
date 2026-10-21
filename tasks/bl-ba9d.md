+++
title = "a place file written before the own channel's rename aims at '(this box's own engine)': no channel matches, and the bar says 'waiting to hear about this workspace' until the workspace is re-picked"
created = 1792561594
updated = 1792561619
claimant = "Unstop-place"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
## The shape

bl-dab9 swept engine → server, and the own channel's name — `seat::holds::OWN`, now `(this box's own server)` — is also the KEY the place file stores the aim and the open set under (`aim.channel`, `aimed`, `open`, `opened`). A seat that ran before the sweep holds `(this box's own engine)` there; after it, no channel on the roster matches, the aim resolves to nothing, the asker never asks for the workspace's conversations, and the bar reads `waiting to hear about this workspace` indefinitely. Re-picking the workspace from the foot's dropup cures it for the run. Reproduced 2026-10-20 on 0.1.106 with a copy of a real place file on an isolated seat; the user's own seat has the same file.

## The fix, by the single-source rule

A display name is not a key. Either the own channel is stored under a name that is not its glass words (one constant that never followed a wording sweep), or the read maps the one retired spelling onto the current one once. Prefer the first — a stable key — with the read accepting the retired spelling for exactly one release so an existing file is not thrown away. A test that loads a place file carrying the old spelling and finds the aim resolved.

## Where

`src/place.rs` (+ tests), `src/seat/holds.rs::OWN`, DESIGN §4.39 (the place file) — amend, cite this ball.