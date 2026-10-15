+++
title = "the ask's reason line names the tool twice: the seat's prefix and the engine's sentence both begin with it"
created = 1792037251
updated = 1792037251
parent = "bl-4446"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: a parked Bash call stands in the ask.

Observed: the ask's second line reads 'waiting for your approval: Bash — Bash {"command":…} classified process (…)'. The seat writes 'waiting for your approval: <tool> — ' and then the engine's says sentence, which itself begins with the request name (yog control::reason::reason formats '{name} {input} classified {class} ({why})'). The tool is read twice in one line.

Expected: one naming. The seat should not restate what the sentence opens with — either drop its own tool prefix when a says sentence is present (bl-5507's rule: the engine's own sentence carries the row), or keep the prefix only for the raw-signal case where no sentence came. Rough edge, not a wrong answer.