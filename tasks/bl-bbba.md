+++
title = "lernie advance on a nonexistent agent exits 0 silently AND mkdirs an orphan inbox — the id-existence guard is missing at exactly the verb README says has it"
created = 1785130169
updated = 1785130228
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Repro

```
$ lernie new /tmp/ws2
/tmp/ws2
$ lernie advance /tmp/ws2 ghost
$ echo $?
0
$ find /tmp/ws2/inbox
/tmp/ws2/inbox
/tmp/ws2/inbox/ghost
```

Exit 0, empty stdout, and a fresh `inbox/ghost/` directory for a name that has
no `agents/*` ref and never had one. Reproduced on both the crates.io 0.0.1
binary and a `make install` build of `main` (`6a4a47d`).

Every sibling verb declines the same id correctly:

```
$ lernie message /tmp/ws2 ghost hi
lernie message: no agent "ghost" in this workspace — a message is addressed to an existing agent (ARCH §2.11); check the id against the workspace's `agents/*` refs, or start an agent with `lernie prompt` / `lernie dispatch`   [exit 1, writes nothing]
$ lernie stop /tmp/ws2 ghost
lernie stop: branch "ghost" does not exist in repo                              [exit 1]
$ lernie bundle /tmp/ws2 ghost /tmp/out
lernie bundle: no branch matches agent id "ghost" in the workspace              [exit 1]
```

The *shape* guard is present at `advance` (`lernie advance /tmp/ws2 ../evil`
declines with the single-path-component message). Only the *existence* half
is missing.

## Expected, per the docs

README §"Messaging an existing agent directly":

> The id guard is the same rule at every verb taking an agent id from outside
> — `message`, `advance`, `stop`, `dispatch`, `bundle`.

`docs/USER_STORIES.md` US-13 records the same as the bl-bdc7 resolution:

> the id must be a single path component ... **and an `agents/<id>` ref must
> exist** (`workspace::agent_exists`) ... the same id guard was applied to
> `advance`, `stop`, `dispatch` and `bundle`

Observed: `advance` got the path-component half and not the ref half.

README §"The exit protocol and the operator scan" separately asserts the
opposite of the observed behaviour:

> a driver launched for a name with no branch dies on `invalid reference` on
> this pass and every pass after.

It does not die. It exits 0.

## Why the mkdir matters

The directory `advance` creates is precisely the pathological state the same
README section describes `scan` as *reporting*:

> an inbox directory with no matching ref is reported (`inboxes with no agent
> branch: N`) and left in place rather than driven

So an operator typo at `lernie advance` manufactures workspace litter, gets no
feedback that anything was wrong, and — because the counter only tallies
inboxes holding pending files — `lernie scan` reports `inboxes with no agent
branch: 0` and never mentions it either. (Verified: planting a file in the
orphan inbox does flip the counter to 1, so the counter itself is correct.)

## Suggested fix

Apply `workspace::agent_exists` at `advance` alongside the existing
`require_agent_id` shape check, *before* the inbox-directory lock path
creates the directory. Decline with the `message` verb's voice and exit 1.

Open question the fix should settle deliberately: `advance` is also the verb
every launch seam spawns, and US-14 promises *"Losing the lease is a clean
no-op: exit 0"* — so the decline must be a missing-ref decline specifically,
not folded into the lease no-op.

## Severity

Defect. Contradicts two literal README sentences and the recorded bl-bdc7
resolution, swallows an operator typo silently, and writes state into the
workspace on a path that should write nothing.

Filed by an outside evaluation pass (wharfinger) walking 0.0.1 from the public
docs only; not claimed, not fixed.