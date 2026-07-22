+++
title = "AdvanceOutcome::Terminal(Epitaph) and AdvanceHandoff::Done payloads are carried but read only by tests"
created = 1784525324
updated = 1784698050
claimant = "Prostheses-4f6d"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-9e2d's dead-code sweep; not config machinery, so split out.

Two carried-but-unread enum payloads held by `#[allow(dead_code)]`:
- src/prompt/dispatch/advance.rs — `AdvanceOutcome::Terminal(Epitaph)`. The epitaph is already written to disk by the §2.11 exit protocol (inbox::deposit), so the copy in the outcome is a second representation of a disk fact (PRINCIPLES: single source of truth) read only by tests.
- src/prompt/dispatch/advance/cli.rs — `AdvanceHandoff::Done(AdvanceOutcome)`. `cmd::advance::outcome_of` matches `Done(_)` and discards it; only tests read it.

Either surface them (a `lernie advance` that reports which terminal event ended the chain — an Outcome::Line rather than Quiet) or drop the payloads to `Terminal`/`Done` and have tests assert against the disk record instead. Decide which; do not leave the markers.