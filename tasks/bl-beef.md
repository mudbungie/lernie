+++
title = "the transcript paints a model's markdown raw: code fences, '##' headings and backticks are prose, a code block is not monospaced or set apart, and a long code line wraps as prose"
created = 1791868202
updated = 1791868206
claimant = "LaneDaily"
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: ask a live model for an explanation with a shell code block. Expected: a code block reads as code — monospace, set apart from the prose, fences not painted; the transcript is the pane the operator reads most. Observed: three backticks on their own line, the code in the body font, then three more; '## What I found' painted literally. theme::paint::prose lays one LayoutJob in the body font for everything said. Why it matters: models answer in markdown; code is the content an operator copies, and it is indistinguishable from the sentence around it.