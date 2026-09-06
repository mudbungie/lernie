+++
title = "lernie enroll prints only a QR, and nothing in the suite prints the yog-enroll envelope the phone app asks you to paste"
created = 1788673822
updated = 1788674420
claimant = "Cantaloups-S2"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 multitenant lane. yog REMOTE §8.4 is the payload authority.

## What the two ends say

`lernie enroll` renders a QR symbol and states outright that the text does not
exist anywhere:

    $ lernie enroll alpha phone-emu operator
    phone-emu — operator at <address>
    [QR symbol]
    not written down anywhere — scan it now, or enroll again

The Android seat's enrollment screen, verbatim, offers exactly one typed
intake beside the scanner:

    or paste the envelope a seat minted
    one line of JSON beginning {"yog-enroll": 1
    [paste here]  [enroll] [scan QR]

So the app asks for a line of text that the seat deliberately never produces.

## The nearest thing, and why it is not it

`lernie ask` reaches the op and prints the reply:

    $ lernie ask '{"op":"enroll","workspace":"alpha","name":"phone-seat","grade":"operator"}'
    {"address":"…","ca":"…","cert":"…","grade":"operator","key":"…",
     "kind":"enrolled","name":"phone-seat","ok":true}

That is the wire ANSWER, not the envelope. REMOTE §8.4 states the difference:

    `ok` and `kind` do not travel: they say what a *wire answer* is, and a
    photograph is not one. `yog-enroll` is the marker a scanner recognizes it
    by and the version it will be told about if the fields ever move.

So the paste text has to be reassembled by hand — drop two fields, add the
marker, re-emit compact — by someone who has read §8.4. The app's own
validator refuses anything else, correctly and by name:

    the envelope's cert will not read: base64 decode error: InvalidTrailingPadding

## Expected

`lernie enroll` prints the envelope line as text beside the symbol, or takes a
word that asks for it. The custody argument for not writing it down is about
files and caches, and it is a good argument — but the text is on the screen
either way, drawn as a picture of itself; the QR is not more private than the
line it encodes.

## Severity

p2. The phone is one of the four components and this is the only supported way
to seat one that does not require shell access to the device. Today the
advertised path is closed unless the operator can decode their own QR.

---

Same change as bl-1554, landed there: the seat now says the REMOTE 8.4 envelope as a line of text beside the symbol, unconditionally. One artifact (Enrolled::envelope), three renderings — symbol, line, and --into <dir> which lays it down as the four entry files. The line is exactly what the android paste box takes: compact JSON, the six fields under the marker, no ok/kind, on one line. One note for the phone lane: the envelope's key order is serde_json's own, which is sorted, so the marker lands LAST — the paste screen's hint text says 'one line of JSON beginning {"yog-enroll": 1'. Order is not semantic (a scanner parses the object, and 8.4 fixes none) but the hint is wrong about what the line begins with; either the hint changes or the encoder puts the marker first at both ends.
