---
name: slack_post
description: Post one message to the shared outward Slack channel. Pass the message body as `text`; it returns `ok` plus the `ts` the message landed at. This is a coordinator-only surface — the one outward channel the fleet writes to, read by human peers as well as agents — so post only what a human peer should see, and always sign the text on its own final line with your persona ("— Persona").
---

# slack_post

Appends one message to the outward channel shared with the peer fleet and
both humans.

## Input

```json
{ "text": "v2 weights landed; pipeline confirmed on our side.\n— Prior" }
```

## Output

```json
{ "ok": true, "ts": "1785375162" }
```

## Coordinator-only

This is the fleet's single outward speaker surface. Exactly one role holds
this grant — the coordinator — and the agent that *watches* this channel
structurally cannot reach it. The rule exists because a watcher's internal
report and a channel message are the same string type: an internal fleet
status line was once published into the channel where a human peer read it,
and the same mechanism could have carried a credential or a half-formed
judgment about a colleague's work.

So before posting, check the text is something a human peer should see: no
internal fleet status, no unverified claim stated as fact, no credential.

## Always sign

End every post with your persona on its own final line — `— Persona` (the
coordinator of this fleet signs `— Prior`). The client appends the agent
marker `[Sent using <@Claude>]`; the signature is yours to write, and it is
what lets the peer fleet's sensor tell which of the two agents on this
account spoke.

## Your own posts come back

The sensor sees your post in its next read and **counts** it rather than
relaying it. That count is your confirmation that the message landed;
absence of the count is the signal to check.
