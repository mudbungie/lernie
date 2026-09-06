+++
title = "follow on a quiescent conversation blocks 31 seconds, prints one blank byte and exits 1 — the same shape as a failed dial"
created = 1788673650
updated = 1788673650
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 install lane of the suite usability campaign (yog docs/STORIES.md S0/S9).

## Scenario step

First conversation reached its first reply. The user then follows it, which is
what the README teaches as the way to watch work happen.

## Gesture and result

    $ time lernie follow home PelicanQuiet
    (nothing)
    EXIT=1 after 31s
    bytes written to stdout: 1   (a lone newline)

The conversation is alive and healthy — `lernie conversations home` shows it
`"state":"quiescent"` and `lernie transcript home PelicanQuiet` shows the
model's reply committed.

**Following a LIVE conversation works correctly.** With a message in flight the
same command streams properly and immediately:

    $ lernie message home PelicanQuiet "What is 17 times 23? Answer with only the number."
    $ lernie follow home PelicanQuiet
    {"kind":"follow","ok":true,"stream":{"delta":"text","text":"391"}}

So the defect is only the quiescent case, which is the common one: a user
follows *after* looking at something, and by then the conversation has usually
settled.

## Expected

Either an immediate reply saying the conversation is quiescent and there is no
tail to hold (exit 0 — nothing is wrong), or a held line that streams when work
resumes. What it must not do is spend half a minute producing a blank line and
a failure code: exit 1 with no output is indistinguishable from a dial that
failed, a certificate that was refused, or an engine that died — every one of
which a first-time user is much likelier to suspect than "it worked, there was
just nothing to say".

## Severity

p2. It is not data loss, but `follow` is one of the five verbs the README puts
in front of a new user, and its failure mode is silence — the one output a
person cannot debug.