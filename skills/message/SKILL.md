---
name: message
description: Deposit a message into an existing agent's inbox — the user, your parent, a child you dispatched, a sibling, or yourself. The message is delivered at the recipient's next step boundary; if the recipient is quiescent, the deposit wakes it. Use to steer a running child, report upward to a parent, or leave yourself a note — content addressed to an agent that already exists, not a new dispatch.
---

# message

Deposits content into an existing agent's inbox (ARCH §2.11). Unlike
`dispatch`, it starts no new agent and creates no branch — the recipient
must already exist, addressed by its agent id. Delivery happens at the
recipient's next step boundary; a message to a quiescent agent revives
it. The deposit is synchronous and returns immediately.

## Input

```json
{ "agent": "<agent-id>", "content": "<message text>" }
```

- `agent` — the recipient's id (its full hyphenated descent, which is
  also its branch name and address). A child's id is the handle its
  dispatch returned; your parent's id is your own id minus its last
  segment; your own id is a legal recipient (a note to self).
- `content` — the message body, delivered verbatim as a user-role
  entry in the recipient's transcript.

## Output

A JSON object on `tool_result.content`:

```json
{ "status": "deposited" }
```

The deposit landed. There is nothing to poll and no address to capture
— the message either drains at the recipient's next boundary or wakes a
quiescent recipient (ARCH §2.11).

## When to use

- Steer a child you dispatched while it is still working — a course
  correction, a new constraint, a piece of context it now needs.
- Report upward to a parent as a long-lived child (a watchdog, a
  reminder, an adversarial critic running alongside).
- Leave yourself a note your own next step will see.

## When not to use

- Starting new work toward a goal — that is `dispatch`, which spawns a
  child agent and returns its address.
- Rewriting an agent's goal — a message adds context *beside* the
  pinned goal (ARCH §2.8); it does not replace it.

## Notes

- The sender recorded on the message is your own agent id, taken by the
  harness from `LERNIE_CONV_BRANCH` — you cannot forge it, and the
  recipient treats every sender uniformly (ARCH §2.11).
- Delivery is deferred to a step boundary, never mid-step: a message
  cannot interrupt in-flight work (stopping is a separate user action).
- The message is delivered once and becomes an ordinary transcript
  entry the recipient (or a compactor) may later curate away.
