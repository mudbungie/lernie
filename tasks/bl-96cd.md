+++
title = "bl-00f5's fold landed in the window only: the typed lernie conversations still lists compactors as peers, and it is the surface a headless operator has"
created = 1788746052
updated = 1788746052
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r2"]
+++
Round 2, lane swdev. Seat built from lernie main 63d2535 (lernie 0.1.35) with the wire constant raised to the engines 16 (bl-183b); engine yog main 5cd95986.

bl-00f5 is closed. Its fix (79d4faf) folds a conversations subtree under its
root — the right remedy, and the DESIGN paragraph it added argues it well. It
is applied to `Model::rows`, which is the WINDOWs one list. The typed verb has
no fold, and after one lane of ordinary work it answers:

    lernie conversations swdev
      Working directory: ~/w/int/rsroman  quiescent  9m   4 members  1 waiting
      TrellisCopper                       quiescent  13m  1 member   1 deep
        "You are the compactor for branch `20260906T103908Z-11537221`."
      ShoalTeacup                         quiescent  9m   1 member   1 deep
        "You are the compactor for branch `20260906T103908Z-11537221`."
      FrostyHillside                      quiescent  9m   1 member   1 deep
        "You are the compactor for branch `20260906T103908Z-11537221`."
      Working directory: ~/w/ballfix      quiescent  12m  1 member   1 waiting
      Working directory: ~/w/yog/tsreport quiescent  25m  3 members  1 waiting
      BriocheGorge                        quiescent  25m  1 member   1 deep
      …

Twelve rows, seven of them compactors — worse than the five bl-00f5 was filed
for, on the same amount of work. Three of the seven hang under one conversation.

The `1 deep` annotation IS new and is the fix showing through: the descent is
on the row, so the verb has everything it needs. What it does not do is use it.

This is not a re-open of bl-00f5 — the argument and the mechanism are right.
It is that the remedy stopped at the surface with a pointer, and the other
surface is the one used by anybody on a server, in a script, over ssh, or in
this campaigns own test lanes.

## Expected

`lernie conversations` prints roots, each with the count of what hangs under it
(the row already carries `members`), and the subtree on request — the same
shape the window now has, reached by a word instead of a click. `--json` keeps
answering every row, as it must.

## Severity

p3, the same as bl-00f5, and for the same reason it gave: an index whose
majority is machinery is an index people learn to stop reading.