+++
title = "/plan on is not shown on the rule: only the box's hint changes, so the mode is invisible once a goal is typed over the hint"
created = 1791868202
updated = 1791868207
claimant = "LaneDaily"
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: Ctrl+X, '+', type /plan, Enter. Expected (§4.42 item 3): '/plan becomes a toggle shown on the rule while it is on'. Observed: the rule reads 'ready │ no model │ 5 waiting │ 1 engine ─ home' with or without plan; only the empty box's hint says 'begun as a plan', and the hint vanishes at the first typed character. Why it matters: a start fired in the wrong mode is a conversation begun as the wrong kind of thing, and nothing on the glass says which kind is armed while the goal is being typed.