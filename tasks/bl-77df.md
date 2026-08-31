+++
title = "the window paints an entry that points at the box's own engine as a second server: every workspace appears twice, with no address on either group to say why"
created = 1788150505
updated = 1788151597
claimant = "OrderJoiner2"
priority = 4
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ux"]
+++
The window groups its channels list by entry, and an entry's group header is the
entry's directory name. On a box provisioned with an entry whose `address` file
holds the same address the box's own engine listens on, every workspace of that
engine is painted twice — once under `(this box's own engine)` and once under
the entry — with nothing on either group saying they are the same server.

Observed rendering, one engine, three workspaces, plus one genuine remote:

    (this box's own engine)
      dev  (named)  N conversations  N waiting
      lab  (named)  0 conversations
      ops  (named)  N conversations  N waiting
    <remote entry>
      <remote ws>  (named)  1 conversations  1 waiting
    lab
      dev  (named)  N conversations  N waiting  — this seat holds no name for it
      lab  (named)  0 conversations
      ops  (named)  N conversations  N waiting  — this seat holds no name for it

`lernie entries` already knows: it prints the address under each entry, and two
of the three rows there carry the identical `127.0.0.1:7737`. The window drops
the one fact that would explain the duplication.

## The suffix makes it worse, not better

`— this seat holds no name for it` is attached to exactly the rows that are
duplicates, so it reads like a diagnosis of the duplication and is not one. Its
actual meaning is a different fact: the entry's directory names one workspace,
the channel enumerates every workspace that client is registered in, and the
extras have no entry directory of their own. A reader has no way to get from the
sentence to that. It reads as an error about the row above it.

## What this is and is not

The provisioning is arguably the operator's mistake — an entry pointing at the
box's own engine duplicates a channel the seat already has. But the seat is the
only thing that can see it, it can see it cheaply, and today it renders the
mistake as though it were a second server. Two things would close it, either
alone helping:

- Put the address on the group header, the way `entries` does. Then a duplicate
  is self-evident and so is a mis-provisioned one.
- Say something when two entries resolve to one address, rather than painting
  both silently.

The suffix wants rewording regardless, since it fires on a normal, correct
configuration.