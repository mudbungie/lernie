+++
title = "lernie --json enroll prints an ANSI QR block instead of frames, and enrolment is one-shot: a scripted enroll destroys the material it was supposed to hand over"
created = 1788745981
updated = 1788746171
claimant = "Cantaloups-S7"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r2"]
+++
Round 2, lane swdev. Seat built from lernie main 1763302 (wire constant raised to the engines PROTOCOL by hand — see bl-183b), engine yog main 5cd95986.

## The gesture

    lernie --json enroll <ws> <name> foot | head -3 > material.json

`--json` is documented as the raw frame stream: "`--json` prints the frames
exactly as they crossed instead, one envelope per line, which is what a script
wants." What it actually writes, `cat -v`:

    <name> M-bM-^@M-^T foot at 127.0.0.1:7772
    ^[[30;47m   ▄▄▄▄ ▀█▄ ███▀▄ █▀  ▀ ▀▄▀ ██▀▄ ▀ ▄█▀  ▄▄▄▀▄█▀▀█ …  ^[[0m
    ^[[30;47m   … ^[[0m

An ANSI-coloured QR symbol under a rendered header. The `{"yog-enroll":1,…}`
envelope IS emitted, but ~25 lines down, after the symbol — so any
line-oriented capture (`head`, `read`, a pipe into `jq`) takes the picture and
not the bytes.

## Why it is worse than an ordinary rendering bug

Enrolment is one-shot and unrepeatable. The second attempt on the same name
answers, verbatim:

    <name> was enrolled already: its key left this box with that enrollment,
    so there is no material to hand over a second time and re-issuing would
    put two live certificates under one identity.

So a script that runs `--json enroll` and captures the first lines has
consumed the identity and kept a picture of it. The lane burned two names this
way before reaching `--into`, which is the flag that works.

## Expected

`--json` prints the envelope and nothing else — no header sentence, no symbol,
no colour. The QR is the human rendering, which is what the bare form is for
(ruling 4, bl-6ae7: "every lernie verb prints a human rendering by default;
`--json` is the raw frame stream").

Worth considering beside it: `--into` is the only form that leaves anything
recoverable, and the help says so in its last sentence. A `--json` that
silently keeps nothing is the trap; `--into` is the cure and the two should
not be this easy to confuse.

## Severity

p2 — the loss is irreversible per name, and enrolment is the first thing a new
box does.