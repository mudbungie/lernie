+++
title = "the unprovisioned refusal teaches the visiting-box recipe to a same-box user and names none of the four files it wants"
created = 1788673655
updated = 1788673655
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 install lane of the suite usability campaign (yog docs/STORIES.md S0/S9).

## Scenario step

A stranger has installed all five crates on ONE box, booted the engine, and
runs the seat for the first time.

## Gesture and reply

    $ lernie workspaces
    (this box's own engine)
        no wire provisioned at <lernie-data>/wire: the pair is minted on the host
        that issued it (`yog wire-certs WIRE_LEAF=<name>` there) and carried here
        by hand; the seat mints nothing

## Three problems, in order of cost

**1. It teaches the wrong recipe.** `WIRE_LEAF` mints an EXTRA client leaf for a
visiting box. On a single-box install the leaf already exists — the engine's own
boot minted `client.pem`/`client.key` beside `ca.pem` — and the act is a copy,
not a mint. Worse, a `WIRE_LEAF` leaf is registered in no workspace, which is the
dead end filed on yog's board as bl-6b14 and bl-bd48. So the sentence sends a
same-box user down a path that is both unnecessary and known-broken.

**2. It names no files.** "carried here by hand" — carried WHAT? The four
filenames (`ca.pem`, `client.pem`, `client.key`, `address`) appear in this
repository's README and in no message the binary ever prints. A user who did not
read the README cannot act on this sentence at all. yog's `wire-certs` output is
the model to copy: it lists each path and says what to rename it to.

**3. The recipe it should teach also fails.** Copying the four files from the
engine's wire directory gets the next refusal:

    <lernie-data>/wire/address names 127.0.0.1:0 — a kernel-chosen port only
    that engine's own window is told; a seat wants a stated address

which names no remedy either. That half is filed on yog's board (the mint and
the address file are yog's); this ball is the seat's sentence.

## Expected

The refusal distinguishes the two cases it is standing in front of:

- **this box's own engine** — copy `ca.pem`, `client.pem`, `client.key` and
  `address` out of the engine's wire directory, naming them; and say that
  `address` must name a stated port.
- **another box's engine** — the existing `WIRE_LEAF` sentence, which is
  correct for that case.

It already knows which it is: the heading it printed says
`(this box's own engine)`.

## Severity

p2. It is the first thing the seat ever says to a new user, and it is advice
that does not work.