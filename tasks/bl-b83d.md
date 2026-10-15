+++
title = "a step or working toggle in one conversation toggles the same row in every conversation: the fold's memory is keyed by row index alone"
created = 1792037250
updated = 1792037250
parent = "bl-4446"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: in a settled conversation, click the worked head and then its first call so the call stands whole; switch to a second conversation on the same wall; click that conversation's first finished call to fold it; switch back.

Expected (§4.45 item 5): an explicit toggle is the toolkit's memory of ONE step's view state — the step the operator clicked, in the conversation it was clicked in. The second conversation's finished call should have been compact on arrival (nobody toggled it), and folding it should leave the first conversation's call as the operator left it.

Observed on 0.1.102: the second conversation's first finished call arrived open, and folding it folded the first conversation's call too. The cause is in src/ui/chat/working.rs: the head's id is make_persistent_id(("working", at)) and the step's is make_persistent_id(("step", at)), where at is the row's index in the transcript, and the panel's id path is the same whichever conversation is painted — so index 3 of every conversation is one id. src/ui/chat/trail.rs's folded() has the same shape (make_persistent_id(at)).

Why it matters: a running turn's finished steps appear open or shut by what the operator did somewhere else, which contradicts the stage rule (the stages decide where nobody toggled) and the principle that the glass shows this conversation's state. Fix shape: salt the id with the conversation's identity (the key the tab or the place file already names), so the memory is per conversation and per row; a test that paints two conversations through one panel and toggles one must find the other unchanged.