+++
title = "an engine that goes down is invisible on the glass: the rule keeps '1 engine' and the stale green 'N waiting', and the seat panel has no dial-state word at all — only a typed act's 'not sent' notice says anything"
created = 1791867708
updated = 1791868396
claimant = "LaneAsks"
parent = "bl-957a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: a seat on one engine (yog fixture busy), a conversation selected; kill the engine; wait 30 s; then aim nothing and read the seat panel.
Expected (DESIGN 4.42 item 2: the rule counts 'how many engines this seat reaches'; item 4: the seat panel paints 'the engines this seat reaches, one row each with its dial state, walls and counts'; 4.19/bl-e620: an unreachable channel is that channel's state, said on its own row): the rule's engine segment says one is unreachable, in the error accent; the seat panel's row for it carries the wire's own sentence (connection refused at the address); the waiting count does not keep counting an engine nobody can reach as if it were current.
Observed: status::facts counts model.roster.len() — the channels HELD, not reached — so it reads '1 engine' with the engine dead; the waiting count stays green at its last value; chat/panel::seat paints walls and counts from the last answer and never reads Chunk::held, so Held::Unheld(why) (where unanswered::unreachable files the sentence) reaches no glass but the switcher's, which Ctrl+X cannot open on a real window (bl-8c49). The only signal is a typed act failing with "'message' was not sent". Recovery when the engine returns works without relaunch.
Why it matters: an operator reads a resting window as healthy while every read is failing.

---

Also on a seat holding three entries — its own engine up, one at a refused address, one whose ca.pem is another engine's CA: the panel paints both broken ones as '— 0 walls, 0 conversations' and the rule says '3 engines'. 'lernie workspaces' on the same roots prints the dial sentence for each (connection refused; the handshake did not verify (BadSignature) … must hold the anchors of THAT engine's CA). Same cause, same fix. And with NO material at all (first run) the panel says '0 walls, 0 conversations' and the rule '1 engine' where 'lernie entries' says 'nothing provisioned at … the act is a COPY …'.

---

Protocol skew (a scratch seat built with PROTOCOL 18 against engines speaking 19): the CLI says 'wire protocol mismatch: this seat speaks version 18, the engine speaks 19 ... upgrade the older component'; the window says 'ready · 2 engines', each engine '— 0 walls, 0 conversations', and offers the start hint on an aimed wall it cannot reach. Same cause: Held::Unheld is painted nowhere but the switcher, and the switcher shows the engine rows with no sentence either.

---

Acceptance case: first run with an empty data root (no wire material at all). The seat's own chunk is Held::Unheld carrying the 'nothing provisioned at ~/… the act is a COPY' sentence that 'lernie entries' prints; the seat panel must paint that sentence whole and wrapped instead of '0 walls, 0 conversations', and the rule must not claim it reaches an engine. The snapshot world 'unprovisioned' is that state.
