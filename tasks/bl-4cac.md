+++
title = "an engine that goes down is invisible on the glass: the rule keeps '1 engine' and the stale green 'N waiting', and the seat panel has no dial-state word at all — only a typed act's 'not sent' notice says anything"
created = 1791867708
updated = 1791867708
parent = "bl-957a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: a seat on one engine (yog fixture busy), a conversation selected; kill the engine; wait 30 s; then aim nothing and read the seat panel.
Expected (DESIGN 4.42 item 2: the rule counts 'how many engines this seat reaches'; item 4: the seat panel paints 'the engines this seat reaches, one row each with its dial state, walls and counts'; 4.19/bl-e620: an unreachable channel is that channel's state, said on its own row): the rule's engine segment says one is unreachable, in the error accent; the seat panel's row for it carries the wire's own sentence (connection refused at the address); the waiting count does not keep counting an engine nobody can reach as if it were current.
Observed: status::facts counts model.roster.len() — the channels HELD, not reached — so it reads '1 engine' with the engine dead; the waiting count stays green at its last value; chat/panel::seat paints walls and counts from the last answer and never reads Chunk::held, so Held::Unheld(why) (where unanswered::unreachable files the sentence) reaches no glass but the switcher's, which Ctrl+X cannot open on a real window (bl-8c49). The only signal is a typed act failing with "'message' was not sent". Recovery when the engine returns works without relaunch.
Why it matters: an operator reads a resting window as healthy while every read is failing.