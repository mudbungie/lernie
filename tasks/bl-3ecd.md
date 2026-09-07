+++
title = "lernie follow right after lernie message answers 'is at rest' in 0s: the obvious pair — say something, then watch it — never watches anything"
created = 1788745985
updated = 1788752231
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r2"]
+++
Round 2, lane swdev. Seat from lernie main 1763302 (constant raised, bl-183b), engine yog main 5cd95986.

## The gesture

    lernie --json message <ws> <agent> "Now add a README section … Take your time."
    {"exit":0,"kind":"outcome","ok":true,"stderr":"","stdout":""}
    lernie follow <ws> <agent>

answers in 0 seconds, exit 0:

    <agent> is at rest (quiescent) — nothing more will arrive until it is nudged or messaged

The deposit landed and the driver had not yet taken the lease, so `follow`
read the standing as of that instant and correctly said "at rest" — about a
conversation that started a step twelve seconds later and then ran for 78.

The same `follow`, run once the driver is up, is excellent: it held for 78
seconds across four steps and streamed the models prose at write cadence, 69
lines. That is bl-f076 fixed and bl-3dca fixed (a quiescent conversation now
answers in 0s with that sentence instead of blocking 31 seconds and exiting 1).
The residual is only the race.

## Why it matters

`message` then `follow` is the pair every operator will type — it is the
whole interactive loop. Getting "nothing more will arrive" as the answer to
"I just sent something" reads as a refusal, and the recovery (wait, then ask
again) is not written anywhere.

## Expected

`follow` on a conversation with undelivered mail waits for the driver rather
than answering at rest — mail queued is not rest. Failing that, the sentence
distinguishes them: "at rest, with 1 deposit not yet taken".

## Severity

p3. The information is right, the moment is wrong, and the workaround is a
sleep.

---

Fixed by bl-87ab's item 2, which is the same defect reached through `start` instead of `message`. The mechanism is the one this ball's Expected asked for: `follow` asks `inbox` when the state read says rest, and a rest whose inbox still holds mail is not an ending — the watch holds through it exactly as it holds on a live conversation, and says once (with the gutter, naming the state the engine gave and the count waiting) that it is waiting, so the hold never reads as a hang. Ctrl-C is the way out it always was.

Three properties, all asserted in src/seat/follow/tests/waiting.rs: it costs one extra read only on the path that was wrong (a genuinely at-rest conversation has an empty inbox and answers as fast as it did; a working one is never asked); an inbox this seat could not READ is treated as no mail, on every way the probe can fail, so an unanswered question can never hold a connection open forever; and --json says nothing while it waits, because in the machine form this seat narrates nothing at all.

Verify against a live engine and close if it holds — I could not drive one from this lane (the seat spoke PROTOCOL 13 against a yog on 16 when I started).
