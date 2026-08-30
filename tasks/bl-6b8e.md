+++
title = "the seat's decoder is judged by the wire conformance corpus, not by its own fixtures"
created = 1788068979
updated = 1788070327
claimant = "OrderProver"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
yog now generates a canonical fixture corpus from its own codec and commits it (yog bl-32cb; REMOTE §3, `corpus/` in the yog repository). It is one file per wire shape — `corpus/request/<op>.json` and `corpus/reply/<kind>.json` — each holding that shape's frames verbatim, stamped with the protocol version at which those fields last moved, plus `corpus/shapes.json` as the standing record.

The seat reimplements the reply vocabulary rather than linking a shared crate (the ruling on bl-4174, which is closed and landed), so the seat is exactly the consumer the corpus exists for: nothing but a fixture can catch a field the seat's decoder silently drops.

Scope: point the seat's decoder tests at the corpus instead of at hand-written fixtures.

- Decode every frame under `corpus/reply/` — the refusal envelope with no `kind` included — and every frame under `corpus/request/` that the seat emits.
- Round-trip what the seat encodes: decode then re-encode must return the frame exactly.
- A shape the seat does not paint is still one it must not misread. The seat decodes only what it renders (bl-4174's rule), so a skipped fixture is a decision recorded in the test — a named skip list — never a silent pass. That list is then the honest answer to \"how much of the vocabulary does the window carry\".

How the corpus arrives is the open question and belongs to this ball: vendor the directory, or read it from a yog checkout at test time. There is no published artifact and no endpoint. Vendoring pins a snapshot that can go stale against yog's PROTOCOL; reading a checkout needs a path nobody has to configure. Pick one and say why.

Also worth stating in the seat's own docs: a fixture that fails is a seat defect until proven otherwise. The corpus is generated from the codec that is the protocol authority, so a disagreement is the seat's, not the corpus's — unless PROTOCOL itself moved, which the fixture's own stamp says.