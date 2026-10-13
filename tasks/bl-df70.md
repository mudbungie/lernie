+++
title = "/delete fires on any word, an empty one included, and deletes the selected conversation; and /flag, /interrupt and /delete with their words missing fire nothing and say nothing about why"
created = 1791867688
updated = 1791867953
parent = "bl-957a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: select a leaf conversation (yog fixture busy, c-001), type '/delete wrongname' Enter.
Expected (DESIGN 4.42 item 5: '/delete <name> is the arming'; 4.20: the arming is an enablement, an unmaking states its subject before it offers the act): a word that is not the conversation's name arms nothing and fires nothing; a deletion that does fire says so and leaves the conversation.
Observed: the conversation is gone ('lernie delete-agent home c-001 x' now answers 'refused: unknown conversation'; it is missing from 'lernie conversations home'). The wire's grammar is that the typed name arms only the CHILDREN — a leaf is deleted by any word, empty included (yog boundary/dispatch/delete_exec.rs) — and the slash verb fires whatever follows it. No notice on the rule, the transcript and the conversation panel stay painted, and the ask box goes on asking for a deleted conversation.
Why it matters: the one destructive act a conversation offers fires on a mistyped confirmation and reports nothing; the operator believes the word protected them.

Same mechanism, found by LaneVerbs on the same fixture: '/delete' with nothing after it deleted c-006 outright (the trail reads 'litany delete … c-006'). '/flag' and '/interrupt' with no words fire nothing — correct — but the window says nothing, so the operator cannot tell a refused verb from a dead key (DESIGN §4.42 item 5: 'one with its parameter missing fires nothing, as the disabled control it replaces fired nothing'; this lane's brief: '…fires nothing and says why').