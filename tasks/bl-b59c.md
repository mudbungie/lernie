+++
title = "the mint corpus is CC BY 4.0 data in an MIT-only package and it mints hostile names: replace it with a clean-room neutral allowlist"
created = 1786677252
updated = 1786677413
claimant = "Pesto"
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

---

Audit body verified against the lernie tree before any edit. Every claim holds; three corrections/additions.

**Verified.** `src/workspace/agent_name/mint/words.txt` carried 7,395 non-comment words (matching the pinned `assert_eq!(words.len(), 7395)`) under the header "SOURCE: EFF's Long Wordlist ... LICENSE: Creative Commons Attribution 4.0 International (CC BY 4.0) ... This header is the attribution and rides into the binary via include_str!." All fourteen named words were present, one occurrence each: carnage, chokehold, cruelty, deceit, depraved, despair, evil, hate, humiliate, stench, threaten, traitor, trash, wrath. `unknown` was absent. `Cargo.toml` said `license = "MIT"`; `LICENSE` held MIT alone. No other CC-BY / Apache / GPL notice existed anywhere in the tree.

**Correction 1 — the test finding is lernie's, not only yog's.** The body attributes the four thin assertions ("character shape, uniqueness, exclusion of `unknown`, and count") to yog's tests. lernie's own `src/workspace/agent_name/mint/tests.rs::embedded_wordlist_holds_its_invariants` asserted exactly the same four and nothing about meaning. The authority had the same gap, so fixing the authority fixes both.

**Correction 2 — the header carried a stale premise.** Its curation note said the four hyphenated entries were dropped because "they would yield a three-segment name and break the two-word split." lernie's mint draws ONE word (`mint_from` takes a single wordlist entry; `mint_over_the_embedded_list...` asserts `!name.contains('-')`). The two-word split is yog-era and was already dead. Gone with the header.

**Correction 3 — the CC BY notice was not the only third-party debt in that header.** The curation also filtered against the first-names + surnames corpora at github.com/dominictarr/random-name. The header argued it was "used only as a filter; not redistributed, so its terms do not ride into this artifact" — that argument is exactly the "EFF minus bad words" reasoning the ball rejects. A clean-room rewrite retires it too; nothing here is derived from anything.

**Delivered.** 541 words, authored from scratch, replacing 7,395.

**Why 541.** The pool sizes the occupied-set problem, and the occupied set is one workspace's *living* agents (ARCH §2.3) — tens, and recycled the instant retention (§9.2) deletes a ref. A collision is not a failure, only one step of the wraparound scan; exhaustion needs all 541 worn simultaneously and is a loud `MintError::Exhausted`, never a loop. With 50 living agents the expected scan is 1.1 steps; with 100 it is 1.23. Entropy is therefore not the binding constraint and never was — the mint is a collision-avoidance device, not a security one (`SplitMix64::from_entropy` doc). The binding constraint is the one this ball creates: the list is now *approved* data, and approval means a human read it end to end. 7,395 words cannot be reviewed by anyone; 541 takes about ten minutes. A smaller pool also buys names an operator can say aloud and recall. The cost — names recur across workspaces and over time — is not a cost: names are per-workspace unique and lawfully recyclable by design.

**Clean-room method.** The new list was written out by category from ordinary English vocabulary (weather, landscape, trees, plants, food, materials, colours, tools, instruments, buildings, animals, textures, crafts) and never compared, diffed, filtered, sampled or intersected against the EFF file or any other corpus. The old file was overwritten, not edited. Screening was by hand plus mechanical rules applied to the new list alone.
