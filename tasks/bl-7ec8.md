+++
title = "/nudge fires silently: a successful act clears the notice, so the window says nothing where the CLI says 'nudged — the advance is running'"
created = 1791868404
updated = 1791868404
parent = "bl-957a"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: select a quiescent conversation, type /nudge, Enter. Expected: some receipt — the engine's own sentence in the state slot, as the CLI prints it. Observed: the box clears and nothing else on the glass changes (the engine did detach an advance, per the ops trail); a nudge with nothing to advance looks identical to a nudge that never left. Model::absorb maps a successful Outcome to no notice at all. Why it matters: the one verb whose whole point is 'did it move?' gives no answer.