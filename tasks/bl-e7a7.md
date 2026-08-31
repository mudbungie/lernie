+++
title = "the installed desktop entry names a bare Exec, so a shell whose PATH lacks the cargo bin dir launches nothing"
created = 1788147567
updated = 1788147635
claimant = "OrderScribe2"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Landed. `make icon-seats` installs `assets/lernie.desktop`, and that file carries `Exec=lernie`.

A desktop entry's `Exec` is resolved by the desktop environment, which starts from the session's own environment rather than a login shell's. On a box where the binary lives in a user-local bin directory that only an interactive shell profile adds to `PATH`, the entry resolves nothing and the launcher fails silently — no window, no message, nothing in a log.

## What landed

The substitution happens at seat time, into the INSTALLED copy only, recomputed on every run rather than patched — so a box already fixed by hand converges on the same answer instead of being edited twice, and re-running is idempotent. rename(2) atomicity through a temp name in the same directory, for the reason the installed binary already has it.

The ladder is two rungs and a refusal:

1. `$(INSTALL_BIN)/lernie`, this Makefile's own installation. It goes FIRST precisely because the defect is that directory not being on a `PATH` — including the `PATH` `make` itself was handed, which is why a `command -v` first rung would still miss on the very box that has the bug.
2. `command -v lernie`, for a binary some other hand installed.
3. Neither, or a resolution that is not absolute: refuse, naming what was looked for. An entry whose `Exec` resolves nowhere is the whole defect, so writing one is not a fallback — it is the bug with a success message.

The tracked asset stays generic: it is one repository's source for every box, `mark::tests` reads it for the three-way spelling agreement, and a real absolute path in it is a disclosure as well as a lie everywhere else.

## The ordering trap this found

`icon-seats` was a PREREQUISITE of `install`, so it ran before the binary was laid down. With a refusal in it that breaks the fresh install — the one case that has to work — because `release` builds into the target directory and puts nothing on any `PATH`. It is now the last recipe STEP of `install` instead, and the Makefile says why the order is load-bearing.

## What is machine-checked and what is not

The tracked asset's side is: `mark::tests` now pins that `Exec` is the bare name and that there is exactly ONE `^Exec=` line — the count being what the substitution rides on, since a second one would be rewritten to the same path and a duplicate key in one group is a malformed entry, which launchers answer by ignoring the file rather than by complaining.

The target's own behaviour is not, and deliberately: driving it from the suite would need a child process, and the confinement rules put every fork in one file that exists for the certificate mint. It was exercised by hand over a throwaway prefix — refusal with no binary, each rung, an idempotent re-run byte-identical, mode 0644, no temp file left behind, and convergence when rung 1 appears under a box already seated off rung 2.