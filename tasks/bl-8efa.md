+++
title = "lernie new pointed at an existing FILE says 'I/O error: Not a directory (os error 20)' — the destination guard covers non-empty dirs but not non-dirs"
created = 1785130176
updated = 1785133161
claimant = "flop-8efa"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Repro

```
$ touch /tmp/afile
$ lernie new /tmp/afile
lernie new: I/O error: Not a directory (os error 20)
$ echo $?
1
```

Reproduced on the crates.io 0.0.1 binary and on a `make install` build of
`main` (`6a4a47d`). Nothing is written, so the refusal is correct — only the
message is.

## Expected, per the docs

The adjacent case in the same guard is exemplary and is a promise:

```
$ lernie new /tmp/ws1        # existing, non-empty
lernie new: destination /tmp/ws1 already exists and is not empty
```

`docs/USER_STORIES.md` US-04 pins that literal string as acceptance:
*"Non-empty destination → exit 1, stderr `lernie new: destination <path>
already exists and is not empty`, and **nothing is written**."* README
§"Layout" states the rule the guard implements: *"The destination must either
not exist or be an empty directory."* A plain file satisfies neither, so it is
the same refusal — but it falls through to a bare errno that names neither the
path nor the rule.

## One-line fix

Extend the existing destination guard: if the path exists and is not a
directory, refuse in the same voice, e.g.

```
lernie new: destination /tmp/afile already exists and is not a directory
```

## Also walked clean in the same pass (for the record)

`lernie new` with no argument (auto-id under the data root), at an existing
*empty* directory (accepted, exit 0), and at a path whose parents do not yet
exist (created, exit 0) — all correct. The retired-layout refusal was also
exercised hands-on for the first time against a fabricated `root/` +
`providers.yaml` layout: `lernie scan` on it gives the full actionable
refusal naming what was found, the current layout, and `lernie new`. US-04
records that refusal as *"unit-only ... integration-proven only for `stop` and
`bundle`"*.

## Severity

Papercut. Right decision, wrong voice, on the very first verb a new user runs.

Filed by an outside evaluation pass (wharfinger) walking 0.0.1 from the public
docs only; not claimed, not fixed.