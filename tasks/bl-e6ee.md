+++
title = "the seat speaks protocol 4 and the released engine speaks 5: reply/governing dropped branch for follows, and oid changed meaning under the same key"
created = 1788319846
updated = 1788319847
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
  lineage's head. A seat painting the old value as *what this conversation
  runs* would keep painting and keep being wrong.

The doctrine behind it inverted: fork-is-the-freeze became follow-the-tip.

## What this seat owes

`PROTOCOL` 4 -> 5, and the corpus re-vendored so the stamp check stops failing.

**Whether anything here paints it is the question to answer, not assume.**
`reply/governing` sits in `corpus/unreadable/` — the ledger — and this seat
carries nine reply kinds, none of them governing. Verify before writing any
paint: if some pane renders policy or config-lineage state under the old
`branch`/frozen vocabulary it must move to follows/held, and if none does then
the shape stays in the ledger and the correct amount of new UI is none (DESIGN
§4.9: a kind nothing renders is a kind nobody has to carry, and the ball that
lands a pane is the ball that adds its kind).