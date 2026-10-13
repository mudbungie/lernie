+++
title = "a selected conversation the engine no longer knows stays selected forever: its read is refused every beat, so 'the engine refused: unknown conversation' re-posts faster than Esc can put it down"
created = 1791868176
updated = 1791870057
claimant = "FixA"
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: select a conversation, then have it deleted (from another seat, the CLI, or bl-df70's /delete), or aim a queue row at the wrong engine (bl-666f).
Expected: the read's 'unknown conversation' refusal retires the selection (it is the engine saying the subject is gone), the panel and transcript go, the rule says once what happened and Esc puts it down.
Observed: the panel keeps painting the conversation (by its id, its name gone), the transcript of the deleted conversation stays on the glass, and the refusal re-posts on every beat into the state slot — Esc clears it for under a second. bl-e620 ruled that a relationship that is down is not dismissible because it re-posts; this is the same re-post on a notice that the ruling meant to be dismissible.
Why it matters: the window cannot be brought back to rest except by choosing another conversation, and the operator reads a red refusal they did not cause on every frame.