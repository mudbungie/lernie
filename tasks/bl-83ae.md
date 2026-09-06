+++
title = "the transcript anchors to the first message: selecting a conversation shows message 001, and a live one never follows its tail"
created = 1788673677
updated = 1788673677
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 daily lane. lernie 0.1.30, window under Xvfb at 1440x900.

Selecting a conversation lands the transcript on its FIRST message and leaves it there. A conversation that is streaming does not follow its own tail either — over thirty seconds of screenshots of a live conversation running its ninth, tenth and eleventh steps, the pane showed step 001 and 002, unmoved, scrollbar pinned at the top.

Evidence: `rounds/1/shots/daily/06-streaming.png` and `07-stream-4.png` — same pixels in the pane, four `lernie steps` rows apart. `10-stopped-conv.png` is a 29-step conversation freshly selected, showing message 001.

The pane does scroll by wheel (`08-scrolled.png`), so this is anchoring, not the closed bl-e5d2.

What it costs: a conversation this lane ran had 61 transcript entries and about 900 rendered lines. Reading what the agent last said means selecting it and then scrolling roughly thirty screens, every time, and there is no End key or "jump to latest" control. Watching one work is impossible.

The README's claim for the follow lane is "the follow lane holding one connection open on the focused conversation", which is presumably working — the bytes arrive, and the roster's state and member count update live. They arrive below the fold.

Expected: selecting a conversation shows its tail. A live conversation follows the tail until the reader scrolls away, and returns to following when the reader scrolls back to the bottom — the ordinary chat-pane contract.

Severity p2, and p1-shaped for daily use: it makes the conversation pane, which is the whole middle of the window, useless for the two things a person opens it for.