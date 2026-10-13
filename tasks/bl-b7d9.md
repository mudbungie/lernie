+++
title = "/roles' model editor is a bare text box: it lists nothing the provider offers, warns of nothing, and a role left on a model id the provider does not offer is painted like any other — while 'lernie model' already fetches the list and warns"
created = 1791867822
updated = 1791867826
claimant = "LaneAsks"
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: set the worker role to a model id that does not exist (claude-session-direct, claude-nonexistent-9), send a message; the conversation stops with 'model: claude-nonexistent-9; ... is not in the model cache; run bz --list-models ...'. Open /roles, press 'model…' on worker.
Expected: the pane says which rows name a model their provider does not offer (in the error accent, on that row), and the editor offers the provider's own list to pick from — the read 'lernie model' already spends (seat/model.rs: "claude-session-direct" does not offer "claude-nonexistent-9" — ... It offers: claude-haiku-5-5, ...).
Observed: the row reads 'claude-session-direct  claude-nonexistent-9' in the same weak ink as every healthy row; 'model…' opens two plain text fields with set/cancel; nothing on the glass names a valid id. The operator has to leave the window for 'bz --list-models'. (Typing a valid id and 'set' does work.)
Also: the failed conversation's ask box offers only '1. seen' and its failure line is cut by the engine at 120 chars (yog bl-8550); nothing near it points at /roles or /nudge, the two acts that mend it.
Why it matters: a provider refusal is the commonest failure an operator meets, and the window can show it but not help mend it.