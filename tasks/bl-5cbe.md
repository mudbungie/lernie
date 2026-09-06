+++
title = "the unprovisioned-channel hint prints 'yog wire-certs WIRE_LEAF=<name>', which is not a command that works: WIRE_LEAF is an env var and must precede the verb"
created = 1788673830
updated = 1788673830
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 multitenant lane.

## The text

Printed by `lernie entries`, `lernie workspaces` and every verb that reaches an
unprovisioned channel:

    (this box's own engine)
        nothing provisioned at <data-root>/wire: the pair is minted on the host
        that issued it (`yog wire-certs WIRE_LEAF=<name>` there) and carried
        here by hand; the seat mints nothing

## Why it does not work

`WIRE_LEAF` is an environment variable read by `yog wire-certs`, not an
argument it parses. Typed as printed, `yog wire-certs` sees an unexpected
argument and does not issue a leaf. The working spelling is:

    WIRE_LEAF=<name> yog wire-certs

which is what `yog wire-certs --help` itself prints:

    `WIRE_LEAF=<common-name>` asks for the other act instead: issue ONE extra
    client leaf under that name, over the CA already here

and what `yog wire-certs`' own success message prints:

    issue another client with: WIRE_LEAF=<common-name> yog wire-certs, and a
    tool host's with WIRE_FOOT=1 beside that

## Expected

The hint spells the assignment before the verb, matching yog's own two
spellings of the same recipe.

## Severity

p3, and worth fixing because of where it lands: this string is the FIRST thing
a new seat prints, before anything works, to a person who by construction has
not provisioned a channel before. It is the one line they will copy.