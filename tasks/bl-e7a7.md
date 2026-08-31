+++
title = "the installed desktop entry names a bare Exec, so a shell whose PATH lacks the cargo bin dir launches nothing"
created = 1788147567
updated = 1788147593
claimant = "OrderScribe2"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
`make icon-seats` installs `assets/lernie.desktop` verbatim, and that file carries `Exec=lernie`.

A desktop entry's `Exec` is resolved by the desktop environment, which starts from the session's own environment rather than a login shell's. On a box where the binary lives in a user-local bin directory that only an interactive shell profile adds to `PATH`, the entry resolves nothing and the launcher fails silently — no window, no message.

## What to build

`make icon-seats` resolves the binary AT SEAT TIME (`command -v lernie`) and writes the absolute path into the INSTALLED copy only. The tracked asset stays generic: it is the source of the entry, it is read by `mark::tests` for the three-way spelling agreement, and an absolute path in it would be both a disclosure and a lie on every other box.

Absent a resolvable binary the target REFUSES with a sentence saying what it looked for and what to do — not a silent install of a broken entry. `install` has `icon-seats` as a prerequisite and orders `release` first, so the ordinary path always has a binary to find.

Re-running the target must be idempotent and must preserve correctness on a box already hand-fixed: the rewrite is of the installed copy, computed fresh each run, never a patch of what is already there.

Record the reasoning at the target, in the Makefile's own voice — the Makefile is the build authority and this is a deviation between a tracked asset and its installed form, which is exactly the kind of thing that gets "cleaned up" by whoever reads the target next.