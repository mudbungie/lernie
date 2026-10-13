+++
title = "/nudge fires silently: a successful act clears the notice, so the window says nothing where the CLI says 'nudged — the advance is running'"
created = 1791868404
updated = 1791869886
claimant = "FixB"
parent = "bl-957a"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: select a quiescent conversation, type /nudge, Enter. Expected: some receipt — the engine's own sentence in the state slot, as the CLI prints it. Observed: the box clears and nothing else on the glass changes (the engine did detach an advance, per the ops trail); a nudge with nothing to advance looks identical to a nudge that never left. Model::absorb maps a successful Outcome to no notice at all. Why it matters: the one verb whose whole point is 'did it move?' gives no answer.

---

Not fixed in the lane: Model::absorb silences the four receipts on purpose (the comment over the Nudged|Flagged|Acked|TrailCleared arm: what each changed arrives on the transcript or the next queue, and a receipt that restated it would predict a listing). That holds when the nudge moves something; it fails when there is nothing to advance and nothing arrives. Operator's call: paint crate::render::answer's own receipt sentence as Notice::Said (one source with the CLI), or keep the ruling.

---

Fixed by bl-9760 (one fix): a successful act's receipt now stands in the state slot as Notice::Said in the Annotation ink, carrying crate::render::receipt — the CLI's own sentence ('nudged — the advance is running'). A deposit (message, interrupt) keeps the old silence: its receipt is the transcript.
