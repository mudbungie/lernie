+++
title = "the trail paints a call and its result as two steps under a tofu arrow: '→' has no glyph in the body font, tool-use ids lead both rows, the result is laid open six lines deep, and 'tool calls (N)' counts results as calls"
created = 1791867737
updated = 1791867737
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: a live conversation whose model calls bash once (claude-session-direct, haiku). Expected (DESIGN §4.42 item 4): one row per call under the fold, '├─ ● bash(<args head>) ✓', the result folded to its head and unfolded in place; the fold counts calls. Observed: the call row reads 'claude-haiku-5-5 ☐ bash toolu_01…' (the ☐ is the '→' in rows.rs, which the body font lacks), its JSON input on a second line, then a separate '└ ✓ toolu_01… returned' step with 'show all 13 lines' and six result lines open beneath; the head says 'tool calls (2)' for one call. Why it matters: the trail is the part of every working turn the operator skims; ids and a doubled count are noise, and the tofu box is the exact failure theme::tests exists to refuse.