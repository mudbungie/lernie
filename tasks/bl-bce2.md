+++
title = "the capability boundary reaches no gesture: a held tool call cannot be answered, and auto-approval cannot be revoked or restored"
created = 1788581513
updated = 1788581513
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The capability boundary is where a conversation's tool call waits for the
operator (yog's `docs/REMOTE.md` §5, §9.11). Three of its ops are classed
`control` by the engine's own help table, so this seat owes each a control, and
`parity.toml` cites this ball for all three. bl-e53c landed the other half of
that ledger entry — `clients`, the machines pane — and left these, because they
are a different subject: the pane it built is about a MACHINE, and these three
act on a CONVERSATION.

## The three frames, exactly

    {"op": "answer",  "workspace": W, "agent": A, "verdict": "pass"|"hold"|"refuse"}
      -> {"ok": true, "kind": "answered", "tool": S, "tool_use": S,
          "verdict": S, "advanced": bool}
    {"op": "revoke",  "workspace": W, "agent": A}
      -> {"ok": true, "kind": "floored", "standing": bool}
    {"op": "restore", "workspace": W, "agent": A}
      -> {"ok": true, "kind": "floored", "standing": bool}

All three are rows of named strings, so they are `src/verbs/rows.rs`-shaped and
need no typed door. Their two reply kinds sit in `corpus/unreadable/` today
(`answered.json`, `floored.json`) and the commit that paints them is the commit
that moves them to `corpus/answers/` — the ledger diff DESIGN §4.9 describes.

`answer` is scoped to the exact call that is held, read from the conversation's
own hold mark: nothing is typed and no call can be answered by naming one. It
also DRIVES the conversation on where it passes or refuses, which is what
`advanced` reports and which makes it an act with a receipt worth painting.

## Where they go, and the one thing that is not answerable yet

The held call's identity is already on the glass: `reply/attention`'s row
carries `held: {tool, tool_use, reason}` and the decision queue paints it
(DESIGN §4.19). That is the natural seat for `answer` — the pane is *what is
waiting on you*, and a parked tool call is exactly that.

`revoke` and `restore` are the conversation's own acts and belong beside `stop`
and `retarget` on the composer's second row (DESIGN §4.11), which is where an
act on the selected conversation goes.

**The rank is the open question.** They are assertions and not a toggle — the
pin pair's rule (DESIGN §4.25): the control names the act it fires, and a row
is offered the one that is not already true of it. But whether a conversation is
floored right now is a fact this seat cannot read: `floored`'s `standing` is the
receipt of an act just performed, and the standing fact lives on `reply/agent`,
which is bl-3257's. So this ball either lands behind bl-3257 and reads the rank,
or offers both controls and lets the engine's refusal be the answer — which
would be the first control in this window that fires an act the far end may have
already decided against, and the pane doctrine is written against exactly that.
Decide it in the ball, and record the decision.

## What is not owed

The rest of §5 is a MACHINE's surface — `advertise`, `invocations`, `complete`,
`invoke`, `capture` — and the help table classes all five `machine`, so no seat
owes them a control. `src/verbs/clients.rs` states why `invocations` in
particular must never be asked from here: it drains the queue addressed to the
certificate that asked.