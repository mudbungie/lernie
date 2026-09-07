+++
title = "follow calls a HELD conversation 'at rest (quiescent)' and names two remedies that are both wrong: the engine now says the park on the frame and the row, and the seat still reads only the state"
created = 1788746782
updated = 1788746782
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r2"]
+++
Filed by the yog Y10 lane beside yog bl-58bb, which fixed the engine half.

## What landed on the engine (yog bl-58bb, PROTOCOL 17)

A `reply/follow` tool-window entry now carries **`held`** — the capability
control's own reason — beside the `tool_use` id and the tool name:

    {"tool_use": "toolu_02", "tool": "box2_service_status",
     "held": "box2_service_status {} classified opaque (…)"}

and the follow lane no longer ends the stream on a park: a held step has not
committed, so `Frame::Over` would be a claim that it had. REMOTE §5.5 states
both halves.

## What is left, and it is the seat's

`seat/follow.rs` decides rest off the agent row's `state` alone:

    match row.state {
        AgentState::Live | AgentState::InFlight => Rest::Working,
        settled => Rest::At(settled.label()),
    }

so a held conversation is `Quiescent` and the watch prints

    <name> is at rest (quiescent) — nothing more will arrive until it is
    nudged or messaged

which is exactly wrong: something *will* arrive, as soon as the operator
reading that line answers the hold. The row already carries `held` (the same
three fields), so the fact is one field away.

Two asks, and they are one change:

- **A held conversation is a distinct rest kind, and the ending line names it**
  — the tool, and the control's reason — instead of "quiescent … nudged or
  messaged", which names two remedies that are both wrong for a park. The
  remedy is `/answer`.
- **The frames the lane already prints render `held`** the way they render a
  dispatch and an exit code, so an operator watching a foot lane sees the park
  arrive rather than only learning about it at the end. On a foot lane every
  call to a non-shell tool is held, so this is most of the conversation.

p2 for the same reason the engine ball was: the live view is the one an
operator has open at the moment they are the blocker.