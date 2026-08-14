+++
title = "the mint corpus is CC BY 4.0 data in an MIT-only package and it mints hostile names: replace it with a clean-room neutral allowlist"
created = 1786677252
updated = 1786677252
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["publication"]
+++
Source: yog publication audit follow-up 2026-08-13 (item 3). Filed here
because lernie now holds the one mint authority: the same EFF-derived
7,395-word list exists in lernie 0.0.7, and yog `bl-cd38` is queued to consume
it. Fix the authority FIRST; yog then consumes it and deletes its duplicate.
Moving yog onto lernie's CURRENT mint moves neither the license nor the
reputation problem.

Two defects in one corpus.

**1. Mixed-license metadata.** yog's copy of the header (`src/names/words.txt`)
reads:

> SOURCE: EFF's Long Wordlist ... LICENSE: Creative Commons Attribution 4.0
> International ... This header is the attribution and rides into the binary
> via include_str!.

The header gives meaningful attribution and identifies modifications, so
redistribution is NOT forbidden — CC BY 4.0 permits sharing and adaptation and
requires credit, a license link, and indication of changes
(https://creativecommons.org/licenses/by/4.0/). The finding is notice debt:
`Cargo.toml` says `license = "MIT"` and `LICENSE` contains only MIT, so
ordinary package metadata tells users MIT-only while compiled-in data is
CC BY 4.0.

**2. The list emits hostile identities.** It contains `carnage`, `chokehold`,
`cruelty`, `deceit`, `depraved`, `despair`, `evil`, `hate`, `humiliate`,
`stench`, `threaten`, `traitor`, `trash`, `wrath`. `humiliate` and `wrath` were
both minted as real names in yog drive evidence. yog's tests assert character
shape, uniqueness, exclusion of `unknown`, and count — no semantic safety
property at all.

Required: a small, independently authored positive allowlist of neutral words,
with human review and recorded provenance. Do NOT derive it by removing words
from the EFF list — "EFF minus bad words" is still an adaptation and still
carries the CC BY notice obligation. Pin the approved set (or its digest) in a
test so every future addition is an explicit review event.

Permanence, for planning: a published crate version cannot be overwritten or
deleted, and yanking does not delete it
(https://doc.rust-lang.org/cargo/reference/publishing.html). Any already-
published lernie version containing the list cannot be scrubbed from
crates.io — future versions replace it, old versions can only be yanked.