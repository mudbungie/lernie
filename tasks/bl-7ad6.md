+++
title = "the leak self-test flagged nothing on one fixture line once, and passed on the next run"
created = 1788582351
updated = 1788582391
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