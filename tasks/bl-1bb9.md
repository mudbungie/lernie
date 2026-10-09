+++
title = "lernie price and lernie ceiling fan a WRITE to every engine this box holds: the command-line forms must name the engine they act on"
created = 1791514217
updated = 1791514223
claimant = "Mystical-1bb9"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Seen while landing bl-9111. `src/cli/prices.rs` says of the two money acts: *"Both name no workspace, so both fan — every channel this box holds is the subject, exactly as `lernie ack` and `lernie clear-trail` are."*

The precedent is wrong for these two. `ack` and `clear-trail` are idempotent housekeeping an operator means everywhere. A price row and a spend ceiling are per-engine facts (yog DESIGN §4.1: world facts, one world per engine), and an operator typing one number means one engine. A seat holding two engines today writes the same ceiling into both, and the engine-side `released` count comes back twice for two different worlds. This is the shape bl-4855 refused for a config write naming no workspace, and the parity ledger's own comment records why: *a write naming no workspace would have been FANNED to every channel this box holds.*

The window already has it right: the prices pane paints one section per engine and each `price`/`ceiling` control acts on the engine whose section it sits in (DESIGN §4.41).

## The work

- `lernie price` and `lernie ceiling` take a target: the entry name (`<leaf>`, DESIGN §4.6) or a workspace name that resolves over the entries to one channel (§4.7 `route` is the one place that mapping is spent — use it, do not add a second resolver). With exactly one channel on the box the target may be omitted and the one channel is the subject; with more than one it is refused, naming the entries, the way a gesture naming no reader refuses in §4.7.
- `lernie prices` (the read) may keep fanning: a listing of every engine's table is what the read is for, and its sections are already labelled by channel.
- Fix the header in `src/cli/prices.rs`, the verb/door text (`crate::verbs::doors`), README's verb list, and DESIGN §4.41's command-line paragraph.
- Tests: one channel without a target → that channel; two channels without a target → refused naming both; a target that is an entry; a target that is a workspace held on an entry; the read still fans.