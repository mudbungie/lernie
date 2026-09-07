+++
title = "the learning loop has no control on the glass: the config pane owes proposals a listing and its two verdicts"
created = 1788754622
updated = 1788754622
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r3"]
+++
The learning loop's operator half reached this seat at PROTOCOL 18 (bl-c515):
`reply/proposals` is decoded, `lernie proposals <ws> [<id>]` and
`lernie proposal <ws> <id> <accept|reject>` are composed, and the CLI paints
the listing and the proposal whole. **What has no control is the window**, so
`parity.toml` carries a line per op and this ball is what deletes them.

## What upstream says the pair is (yog REMOTE §9.22, yog bl-dd88)

A reviewer agent that learned something writes it as a real config patch and
parks it on `proposal/<reviewer-id>` — a branch no lineage points at until
somebody says so. The whole learning loop turns on a person reading that patch
and vetoing it, and until 18 nothing on this wire could: `lineages` enumerates
`config/*` only and `governing` answers the commit a conversation resolves,
never a candidate. So the veto lived at `litany proposal` on the engine's own
box, which on a server install is an `ssh` and a container exec — the thing the
§8.5 boundary exists to make unnecessary.

## Where it goes, and why that is not a new pane

A proposal is a **candidate config commit**, which is the same subject the
config pane already browses (DESIGN §4.30: the lineages a wall holds, and one
file's bytes). So the read hangs on that pane rather than beside it, one
standing read on the pane's own terms, and the two acts hang on the row:

- **the listing** — id, standing, the lineage it would move, the diffstat and
  the reviewer's subject. `fresh`/`stale` crosses as the engine's own reading
  and is never inferred here from an empty `lineages` (REMOTE §9.4).
- **the whole** — naming a row opens its message and diff, one op at two
  depths, and the diff is painted verbatim because a diff a seat reformatted is
  a diff nobody can apply.
- **accept and reject on the row.** Accept is destructive in the direction that
  matters — it fast-forwards a lineage every conversation on it then resolves
  at its next step — and reject throws a reviewer's work away. §4.20's arming
  is the rule for both; neither may be a bare button.

## Done when

`parity.toml`'s two lines are gone (deleting them is the gate re-reddening, the
severability test), the snapshot walk sees `act:proposals` and `act:proposal`,
and `src/ui/model/absorb.rs`'s dropped `Reply::Proposals` arm files the answer
instead of discarding it.