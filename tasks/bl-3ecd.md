+++
title = "lernie follow right after lernie message answers 'is at rest' in 0s: the obvious pair — say something, then watch it — never watches anything"
created = 1788745985
updated = 1788745985
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