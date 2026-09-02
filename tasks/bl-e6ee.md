+++
title = "the seat speaks protocol 4 and the released engine speaks 5: reply/governing dropped branch for follows, and oid changed meaning under the same key"
created = 1788319846
updated = 1788319887
claimant = "Vellum"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
0.1.2 shipped speaking 4 (bl-d774) and the wire moved again the same day. The
released engine's handshake refuses a seat that does not speak 5, and there is
no negotiation and no compat shim — that is REMOTE §3 by design, so a seat one
integer behind is a seat that cannot dial.

## What moved (REMOTE §9.12, upstream bl-e654)

`reply/governing` is the whole of it, and it is the *second* clause of §3's
bump rule rather than the first — what a spelling already in use is taken to
say:

- **`branch` is gone; `follows` and `diverged_lineages` replace it.** `follows`
  is the config lineage's name with the count at 0, or `null` with the count
  saying how many distinct lineage tips reached the conversation and therefore
  held it on its fork commit. The pair cannot decode to a state the encoder
  could not have written.
- **`oid` changed meaning without changing key.** It named the `config/*`
  ancestor an agent's branch forked off — a commit that never moved. It now
  names the commit control actually reads at each step boundary: the followed
  lineage's head.

The doctrine behind it inverted: fork-is-the-freeze became follow-the-tip.

## What landed

- `corpus/` re-vendored through `scripts/refresh-corpus.sh`. **Nothing added,
  nothing retired**; three files moved and none of them is in a class this
  build paints: `shapes.json` (protocol 4 -> 5, `reply/governing`'s signature
  and `since`), `unreadable/governing.json`, and `unreadable/help.json` — the
  help table's `/retarget` wording, which followed the same ruling upstream.
- `PROTOCOL` 4 -> 5.

## The paint question, answered rather than assumed

**No pane here paints governing or config-lineage state, so the correct amount
of new UI was none.** Checked three ways before writing anything: `governing`
is not among the five reply kinds this seat decodes, so it falls to
`reply::read`'s unknown-kind arm; its fixture sits in `corpus/unreadable/` and
the replay asserts exactly that; and the only config-adjacent field the seat
already receives — the roster row's config-lineage tip — is documented as
carried-and-unpainted because there is no model picker here. `WsRow::pinned` is
the workspace pin list and is unrelated. So the §9.4 rendering precedent
("policy follows config/<name>, now at <oid>" / "policy held at <oid> — N
diverged config lineages") has no site to be applied at, and applying it would
have meant inventing a pane — which DESIGN §4.9 forbids: a kind nothing renders
is a kind nobody has to carry, and the ball that lands a pane is the ball that
adds its kind.

## What was written instead of paint

The trap, at `PROTOCOL` itself and echoed in DESIGN §4.9. **A shape whose
meaning moves under an unchanged spelling is the one drift a corpus replay
cannot catch** — the bytes are well-formed and the class is right — so a future
reader that took `oid` for the fork commit would paint a plausible number that
has been wrong since this bump. The ledger catches a shape that changed; only
prose catches a shape that changed its mind, and the prose belongs where
whoever lands the pane will be reading.

## Release ritual

**There is no changelog promotion step in this repository and that is a
decision, not a gap.** `release-plz.toml` sets `changelog_update = false` and
records the evidence: commit subjects here are a short imperative plus a
`[bl-xxxx]` trailer rather than Conventional Commits, and the sibling
repository measured git-cliff stripping a leading `word:` token from any
subject that happens to parse as a conventional type, doubled because
release-plz runs each commit through the processing twice. There is no
`CHANGELOG.md` to promote. The stated consequence is that the GitHub Release
body is empty and the tagged tree is the record.

Gate: `make check` green — fmt-check, line-cap 160 files, leak-scan clean
(332 tracked files), clippy -D warnings, 11 rules audited, cargo-deny ok,
100.00% coverage (2152/2152), 404 tests.