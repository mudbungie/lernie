+++
title = "an empty world answers rows:[] and stops: nothing tells a first-time user that starting a conversation is the next act"
created = 1788673659
updated = 1788673659
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 install lane of the suite usability campaign (yog docs/STORIES.md S0/S9).

## Scenario step

The seat is provisioned and dialling. The user asks the first question a
provisioned seat can answer.

## Gesture and reply

    $ lernie workspaces
    (this box's own engine)
        {"kind":"workspaces","ok":true,"rows":[]}

    $ lernie attention
    (this box's own engine)
        {"kind":"attention","ok":true,"rows":[]}

Both correct, both terminal. A fresh world genuinely holds nothing, and the
protocol's answer for "nothing" is an empty array — that is right and should not
change.

## What is missing

The one fact a new user needs at this exact moment is that a world bootstraps
its first workspace when a conversation is started in it (yog DESIGN §3.1: the
fixed name `home`, used without asking), so the next act is

    lernie start home "<goal>"

Nothing says it. `lernie help` lists forty-odd verbs alphabetically-ish with no
ordering by when you need them, and `start` is documented in the trailing group
of "four words this binary answers itself" — the last place a reader looks.

## Expected

An empty roster is a knowable state with exactly one sensible next act. Say it,
once, under the empty result. This is the cheapest possible fix and it is the
difference between a seat that has just connected and a seat that appears
broken.

## Severity

p3. Nothing is wrong; the product is merely silent at the one moment a person
is most likely to conclude it does not work. Filed as a rough edge rather than
a defect.

## Comparator note

`hermes doctor` (hermes-agent 0.19.0) ends every run with a numbered
"Found N issue(s) to address" list, each naming the command that fixes it. The
suite has no equivalent verb on any of its four components.