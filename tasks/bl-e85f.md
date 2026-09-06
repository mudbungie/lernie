+++
title = "a refused act clears the composer while the notice says nothing happened and it is safe to do it again"
created = 1788673692
updated = 1788675181
claimant = "Cantaloups-S3"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 daily lane. lernie 0.1.30, window under Xvfb.

The notice raised when an act cannot reach the engine is excellent, and then the seat throws away the thing the notice tells you to do again.

Reproduction: type into the composer, stop the engine, click `send`. The notice bar reads, verbatim:

    `message` was not sent: it never left this seat (connect 127.0.0.1:7741: Connection
    refused (os error 111)), so nothing happened — it is safe to do it again.

`rounds/1/shots/daily/15-notice.png`. The composer beneath it is empty: it shows its placeholder again, and the typed text is gone. To "do it again" the operator retypes it.

The sentence is right, its reasoning is right, and it is the best refusal this lane saw anywhere in the suite. It is also the one case where the composed text is provably still needed: the notice's own claim is that the act did not happen. A lost reply — the doubt case bl-3969 landed — is the opposite situation and may well want the clear; this one does not.

Expected: an act that provably never left the seat leaves the composed text where it was. A one-line goal is a small loss; the composer is where a person types a paragraph-long goal, and that is the case that hurts.

Severity p2 — small, cheap, and directly contradicts the sentence the seat itself prints.