+++
title = "the leak self-test flagged nothing on one fixture line once, and passed on the next run"
created = 1788582351
updated = 1788582651
claimant = "Animations-R"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
`make check` failed one `leak-scan --self-test` run with:

    self-test: [quoted-dialogue] line 2 of scripts/leak-fixtures/quoted-dialogue.txt was NOT flagged

Two runs of `make leak-scan` immediately after, on the same tree and the same
shell, both passed. Nothing in that fixture or in `leak-rules.sh` had been
touched by the work in flight.

## Why it matters more than a re-run

The disclosure gate's own regression half is the thing that stops the scanner
rotting into a pattern that matches nothing. A self-test that can report a
false NEGATIVE-of-the-negative — a live rule reported dead — is one an agent
learns to re-run rather than read, which is exactly how a real dead rule gets
waved through. It also reddens a close at random, for both this repository and
yog, which share the script.

## Where to look

`scan_rule` greps with `-HIonE`. `-I` treats a file grep judges BINARY as
having no matches at all and says nothing about it, and the fixture carries
multi-byte UTF-8 (an em dash on line 3). A run under a locale where those bytes
are not decodable would answer no hits for the whole file — but only line 2 was
reported unflagged here, which does not fit that shape and is the fact to
explain before believing any theory.

Worth ruling out in order: whether `fixture_lines` can race the scratch tree
`leak-scan.sh` materializes with `git checkout-index`; whether a hit line was
produced but its `:N  [` anchor did not match; and whether the harness's
`grep -qE ":$ln  \["` can be defeated by a truncated finding on another line.

## What a fix owes

Whatever the cause, the self-test should fail LOUDLY on the ambiguous case
rather than reporting a rule dead: *this rule produced no hits for this file*
and *this file could not be read as text* are two different sentences, and only
the second is an infrastructure fault.

---

## Root cause: `pipefail` reads the status off a dead writer

Not the locale, not the scratch tree, not a truncated finding. `printf … |
grep -q PATTERN` is a race under `set -o pipefail`: `grep -q` exits the instant
it matches and closes the read end, the writer is killed by SIGPIPE part-way
through its own write, and `pipefail` then takes the pipeline's status from that
DEAD WRITER rather than from the reader that answered. **The pipeline reports
failure exactly when the pattern MATCHED.**

`PIPESTATUS` captured at four independent false answers, in an instrumented copy
of the harness: `141 0` — writer killed by SIGPIPE, `grep` exited 0 having
matched. Re-running `scan_rule` inside the same failed iteration reproduced the
finding the check had just called missing, byte for byte. That is also why only
ONE line is ever named: each of the eight lines is its own pipeline, and the
race is per-pipeline, not per-file. The em dash and `grep -I` were innocent.

## Measured

Reproduction, 12 workers x 100 whole self-test runs on a loaded box: 4 false
answers in 1,200 runs, on four different rules (`home-path`, `mac-address`,
`session-artifact`, `vendor-token`) and four different lines — the shape the ball
reported, at random.

Isolated, one blob of three finding-lines, matching the first, 4 concurrent
workers:

    printf '%s\n' "$blob" | grep -qE ':2  \['    36 false / 16,000
    grep -qE ':2  \[' <<<"$blob"                  0 false / 16,000

After the fix, 12 workers x 100 self-test runs: 0.

## Fixed

A `grep -q` reads its subject from a herestring, never from a pipe — there is no
second process to die, under either setting of `pipefail`, and the semantics are
byte-identical. Three sites in `leak-selftest.sh`, two in `leak-scan.sh`.

**One of them was never a flake.** `scan_paths` is `… | grep -qE
"$FORBIDDEN_PATH" && printf … [forbidden-path]`: a false 141 means the `&&`
never fires, so a real credential-shaped path was reported by NOBODY. The gate
missed findings in silence, at a rate that rose with load. That is the half of
this defect that was not a red close.

**The ban is on the shape, not on the option**, because a sourced file cannot
see whether its caller set `pipefail` — `leak-selftest.sh` does not set it and
inherits it from the scanner that sources it, which is how the defect reached
the one file whose whole job is to prove the gate is not lying. `self_test` now
refuses the shape in every tracked bash script under `scripts/`, both directions
proven (a planted violation fires, and a planted one in a POSIX `/bin/sh` script
does not — `deploy/` is `sh` on purpose and has neither the option that makes
the shape wrong nor the herestring that fixes it). The pattern is written `[|]`
so it cannot match its own text, the idiom `leak-rules.sh` already uses for
`Fil[e]`.

## What the ball asked for and got

`fixture_lines` now asks whether the fixture reads as text BEFORE judging the
rule, and answers in its own sentence naming `LC_ALL` and `LANG`. *This rule
matched nothing* and *this file could not be read as text here* are two
sentences, and only the second is an infrastructure fault.

## Upstream

Filed as yog bl-e33a: the same five sites, plus the piped `grep -q` beats in
`scripts/drive/`, where the natural home for the guard is `beat-audit.sh`'s
third shape. brazen carries the same port (its bl-39f1) and the same five sites.
