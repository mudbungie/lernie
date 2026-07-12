+++
title = "Re-point context assembler at the transcript; delete accumulator and output.json read path [substrate]"
created = 1783831462
updated = 1783831462
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["substrate"]

[[blockers]]
id = "bl-4798"
on = "claim"
+++
ARCHITECTURE §2.3: assembly becomes readdir+sort of messages/ in the read-state commit's tree — no git-log walk, no index; consecutive same-side entries group into alternating wire messages; tool_use/tool_result pairing holds by construction. Delete the in-memory assistant-event accumulator and the tools/<tool-id>/output.json tool_result assembly path (diagnostic-only contract: zero runtime content reads under steps/). Running, retry, and replay assembly are one code path against one input: a commit sha. Needs the transcript writer ball (entries must exist before the assembler can read them).