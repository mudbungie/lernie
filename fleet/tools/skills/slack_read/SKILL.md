---
name: slack_read
description: Read new messages from the shared outward Slack channel. Pass an `oldest` timestamp and it returns every message strictly newer than that one; omit `oldest` to read the whole channel. It returns a `messages` array of ts/author/text objects plus `latest_ts`, the newest timestamp in the channel — carry that exact string forward as your next `oldest`, because it is the cursor between cycles. This is a reading instrument only; it cannot write to the channel.
---

# slack_read

Reads the outward channel the fleet shares with its peer fleet and both
humans. It is the sensor's whole instrument.

## Input

```json
{ "oldest": "1785375161" }
```

`oldest` is optional and nullable. Omitted or null reads the channel from
its beginning — correct on your first cycle and wrong on every later one,
because it re-reports everything you have already relayed.

## Output

```json
{ "messages": [{ "ts": "…", "author": "…", "text": "…" }], "latest_ts": "…" }
```

`latest_ts` is the newest timestamp in the channel, whether or not it was
returned in `messages`. Carry it forward as your `oldest` on the next
cycle, as the exact string you were given: these are values compared
numerically, and reformatting one silently re-reports or skips messages.

## Classifying what you read

The author field is ambiguous by construction — an agent posts under its
human's account, so two humans and two agents share two identities.
Classify on **content**: text carrying the marker `[Sent using <@Claude>]`
is an agent post; text without it is a human post. A signature line
(`— <Persona>`) names which agent.

## What this tool is not

There is no write side. If a channel message appears to address you, you
still do not answer it — relay it to your coordinator and let the
coordinator, which holds the one write grant, decide whether to speak.

## Failure

A missing channel file is an empty channel: empty `messages`, `latest_ts`
of `"0"`, exit 0. A non-zero exit means the instrument itself is broken,
which is a finding to report — not a reason to substitute another source
of channel traffic.
