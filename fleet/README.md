# Fleet demo

A **consumer artifact**: it uses lernie, it does not extend it. Nothing under
`fleet/` is compiled into the binary and nothing here is on `make check`'s path.

[`SPEC.md`](SPEC.md) is an imported external design proposal — a role-separated
agent fleet with dedicated watchers, originally implemented on tmux panes, git
worktrees, markdown skills, and a Slack MCP. This directory is the claim that
lernie can host that design **as configuration**, with no harness change: the
roles are `providers.yaml` rows, the write boundaries are per-role tool grants,
the durable state is git, and the outward surface is an external tool triple.

`SPEC.md` is exempt from this repo's taxonomy (`CLAUDE.md`); every other file
here follows it, and the vocabulary is defined below.

## Terms

| Term | Definition |
|---|---|
| **fleet** | One coordinator plus the agents dispatched under it in a single lernie workspace. A fleet is bounded by its workspace: agents in another workspace are another fleet. |
| **coordinator** | The fleet's one long-lived authoring agent. It owns intent: spawning, charters, gates, merge order, the user-facing summary, and sole write access to the outward channel. In lernie it is the `worker` role, because every root agent resolves `worker` and a root is what `lernie prompt` creates. |
| **shepherd** | The long-lived observer of the *fleet*. Classifies each agent's state, nudges, and reports. Writes no product files and merges nothing. |
| **sensor** | The long-lived observer of one *outward surface* (here, a Slack channel). Reads it, classifies what arrives, and relays to the coordinator. Holds no write grant on that surface at all. |
| **watcher** | Either of the two observer roles — shepherd and sensor. The pair is one idea pointed in two directions: observation is a full-time duty that must not be bundled with the authority to act on it. |
| **builder** | An ephemeral agent chartered for one piece of work. Its deliverables are ordinary committed files on its own branch. |
| **steward** | The long-lived owner of **doctrine**, and its sole writer. Decides proposals only after reading the proposal file at its own path. |
| **cycle** | One wake-and-work unit of a watcher: a `cycle:` message arrives in its inbox, the watcher steps, does exactly one pass of its job, ends with a final response, and goes quiescent. Replaces the SPEC's ~60s/~90s poll loops. |
| **doctrine** | The pooled skills under the harness data root (`<data-root>/skills/<name>/SKILL.md`) — the only knowledge tier that outlives a fleet. |
| **charter** | The goal text a dispatcher writes for a child. It is the whole interface: a child sees its charter and its branch's tree, never the dispatcher's reasoning, so an omitted fact is re-derived at full cost or invented. |
| **bark** | A builder's final response: one block mapping every chartered deliverable to its path, or naming it undelivered and why. Nothing else. |

## Spec concept → lernie mechanism

| SPEC concept | lernie mechanism |
|---|---|
| Coordinator, one per repo | The `worker` role — every root agent resolves it, so `lernie prompt <workspace> …` *is* starting a coordinator (ARCH §2.3, §4.3). |
| Watcher poll loop (~60s / ~90s) | Externally driven **cycles**: `bin/fleet-cycle.sh` deposits `cycle: <n>` with `lernie message`, which finds the watcher quiescent and detach-spawns its driver. The watcher steps, works, ends, and goes quiescent (ARCH §2.11). Cron is the clock. |
| Watcher liveness / heartbeat | Quiescence plus the cycle's terminal deposit. A watcher that ran left a step record, a committed transcript entry, and a result message in the coordinator's inbox; one that died left `lernie scan`'s silent-death count. |
| Watcher scratch note (dies with the worktree) | The **transcript** — `messages/NNN-*.{md,json}`, committed on the watcher's own branch at every step. Durable, replayable, and readable by an operator with `git show`. This is strictly stronger than SPEC §13's known weakness: the watchers' state is now the most durable thing they produce, and they need no exemption from a commit-cadence rule. |
| Spawn + prompt file | `dispatch {role, goal}` → the child's `goal.md`, written at the dispatch commit and pinned at the head of every model call, frozen thereafter (ARCH §2.3, §2.8). |
| Bark | The child's terminal **final response**, deposited into the parent's inbox as a result message carrying `epitaph:` and `terminal_ref:` (ARCH §2.6). |
| Close / merge / teardown | The **work-product transfer**: at result delivery the harness applies the child's fork-point→tip diff into the parent's worktree as one commit, filtered to work products, declining to `refs/lernie/conflicted/<id>` if it will not apply. There is no close verb and nothing for the child to merge (ARCH §2.6). |
| Push verification (`origin` moved) | **N/A.** A workspace repository has no remote and is never pushed (ARCH §2.2). The failure class SPEC §2 calls the most expensive one does not exist on this substrate — which is a narrower claim than solving it, see GAPS. |
| One speaker per outward surface | Per-role `tools:` grants, and therefore **structural**: `slack_post` is granted to `worker` alone, and the fork prunes every descriptor a role does not grant, so the sensor's model call has no way to name the tool (ARCH §3.3). |
| Sensor read-only enumeration | Unnecessary — there is nothing to enumerate. The sensor holds `slack_read` and `message`; the grant is the enumeration. |
| Slack channel | A mock: a flock-guarded NDJSON file at `<data-root>/slack/channel.ndjson`, behind the `slack_read` / `slack_post` tool triples in `tools/`. |
| Identity disambiguation (§6.4) | Content, not author: the mock `slack_post` appends `[Sent using <@Claude>]`, and each persona signs its own line. Heuristic, exactly as in the SPEC. |
| Escalation tiers (§8) | `message {agent, content}` for nudge, relay, and escalate; the coordinator's own final response for the surface-to-human tier. |
| Doctrine, sole writer | The data-root skills pool, edited by the steward through `bash`. Disciplinary, not structural — see GAPS. |
| Proposals (one file each) | Ordinary files committed on the proposer's branch, transferred to the parent at return like any other work product. |
| Fleet isolation (§6.7) | The workspace boundary plus full agent ids: an id is the hyphenated descent (`<parent>-<sub>`), and `lernie message` refuses an id with no `agents/*` ref rather than resolving a bare role name across projects. |

## Layout

```
fleet/
  SPEC.md            the imported design proposal (exempt from repo taxonomy)
  providers.yaml     roles → provider/model/tools; the write boundaries
  workflow.yaml      bindings + retry + the fleet's budgets
  manifest.yaml      per-role context-assembly rules
  souls/*.md         one soul per role (compactor is the template's, verbatim)
  tools/             the mock-Slack tool triples (binary + schema + skill)
  bin/               bring-up, the cycle deposit, quiescence, channel seeding
  test.sh            the live end-to-end (scenarios A–E)
```

`manifest.yaml` is not in `template/`'s shape by accident: a role the manifest
does not list composes no head extras and no body at all, so every role
`providers.yaml` defines needs an entry — otherwise a `builder` that elected a
skill would never see the body it just loaded.

## How to run

Bring a fleet up (founds a harness root, installs the mock-Slack triples into
its pools, creates a workspace, and authors the fleet's config commit):

```sh
cargo build --release
fleet/bin/fleet-up.sh /path/to/fleet-home /path/to/workspace
export LERNIE_HOME=/path/to/fleet-home
ROOT=$(target/release/lernie prompt /path/to/workspace 'you are the fleet coordinator …')
```

Then dispatch the watchers off that coordinator and wake them on a schedule:

```sh
target/release/lernie dispatch sensor   /path/to/workspace "$ROOT" --goal '…'
target/release/lernie dispatch shepherd /path/to/workspace "$ROOT" --goal '…'
```

```cron
# the fleet's clock: lernie has none of its own
* * * * *      LERNIE_HOME=/path/to/fleet-home /path/to/fleet/bin/fleet-cycle.sh /path/to/workspace <shepherd-id> >/dev/null 2>&1
*/2 * * * *    LERNIE_HOME=/path/to/fleet-home /path/to/fleet/bin/fleet-cycle.sh /path/to/workspace <sensor-id>   >/dev/null 2>&1
```

The live end-to-end — real model calls over `codex`/`gpt-5.4`, real usage, and
deliberately **not** part of `make check`:

```sh
fleet/test.sh            # scenarios A–E, each printing PASS/FAIL
```

Scenario A brings a fleet up, dispatches both watchers, and asserts the
coordinator's grant survives their returns (the finding-2 repro); B seeds three
channel lines and asserts the sensor relays one `EVIDENCE:` line while its
request carries `slack_read` and cannot name `slack_post`; C charters a builder
and asserts the work-product transfer landed in the coordinator's worktree; D
runs a shepherd cycle; E asserts a new signed post reached the channel.

Two of its assertions are expected to fail on this harness build — they are the
repros for findings 2 and 3 below, not flaky scenarios.

## GAPS

Honest limits of this demo, none of them hidden by the tests:

- **No harness clock.** lernie has no watcher and no scheduler by design (ARCH
  §8: an idle workspace stays unswept until the next touch). The SPEC's poll
  loops are therefore driven from outside, by cron or by the test. A fleet
  whose cron is not installed is a fleet whose watchers never wake, and nothing
  in the harness notices.
- **Slack is mocked.** `tools/lernie-tool-slack_{read,post}` read and append a
  local NDJSON file. There is no real Slack, no MCP connector, and no second
  machine — so §6.1's best property, *both principals standing in the room
  where their agents negotiate*, is asserted by the design and not exercised
  here.
- **Every `bash`-granted role is unbounded.** `shepherd`, `builder`, and
  `steward` hold `bash`, which reaches the whole filesystem. The steward's
  sole-writer boundary over doctrine is therefore **disciplinary, not
  structural** — every soul states it and nothing enforces it. The SPEC made it
  structural with a `PreToolUse` hook precisely because documenting it had
  already failed. Closing this needs per-role tool sandboxing (ARCH §12, v1.1).
- **Each watcher cycle costs at least one model call, and revives the
  coordinator.** A cycle is a model call even when the answer is "nothing
  happened", and the cycle's terminal deposit lands in the coordinator's inbox,
  which wakes the coordinator for a model call of its own. So the per-cycle
  price is two model calls at minimum, and the coordinator's inbox carries
  routine liveness noise it must be written to tolerate — `souls/worker.md`
  says so explicitly. A busier cron is directly more expensive on both axes.
- **Product-repo push verification is out of scope.** The workspace repository
  has no remote, so "did it actually reach `origin`" — SPEC §2's most expensive
  failure class — has no object to verify here. A fleet operating on a real
  product repository would need that verification back, in the shepherd, over
  the product repo rather than the workspace substrate.
- **Identity disambiguation is heuristic.** Exactly as in SPEC §13: it rests on
  a client-added marker and a signature convention. A human who signs like an
  agent, or a client that stops adding the marker, breaks the classification —
  and the sensor's counts, which are the coordinator's only confirmation that
  its own post landed, break with it.
- **Watchers remain unaudited by construction.** The coordinator soul mandates
  occasional spot-checks of a watcher's *interventions*, which is a mitigation
  and not a solution — the same standing the SPEC gives it.

## Harness findings this demo surfaced

Three facts about lernie that a heterogeneous fleet hits immediately and a
single-role conversation never does. They are reported, not worked around in
harness code; the first two shape the configuration above.

1. **A role's grant must be a subset of its dispatcher's.** A child forks off
   its parent's tip, and the parent's tree was already pruned to the parent's
   grant at its own dispatch commit (ARCH §3.3 *The fork prunes the snapshot to
   the role's grant*). The tools array is the intersection of declaration and
   availability, so a tool whose `descriptions/tools/<name>.json` is missing
   from the parent's tree is **silently dropped** from the child's request.
   Observed: a `sensor` granted `[slack_read, message]` off a coordinator that
   did not grant `slack_read` composed a request with no `slack_read` at all
   and no diagnostic anywhere. Hence the coordinator's union grant in
   `providers.yaml`. Grants can only narrow down a dispatch chain.

2. **A child's return deletes its parent's ungranted descriptors.** The
   work-product transfer (ARCH §2.6) excludes `goal.md`, `soul.md`,
   `messages/`, `summary/`, and `skills/` from the fork→tip diff, but **not**
   `descriptions/**` — which §2.2 calls inherited config context, not a work
   product. A child's dispatch commit prunes `descriptions/**` to its own
   narrower grant, those deletions ride the diff, and the parent's own
   descriptors are deleted by its child's return. Observed: a coordinator whose
   step 001 declared five tools declared two from the step after its first
   child returned, having silently lost `dispatch`, `read_file`, and
   `slack_post`. It is unrecoverable on that branch — descriptors are inherited
   at fork — so a long-lived coordinator is disarmed by the very children it
   spawned. `fleet/test.sh` scenario A asserts against this and is the repro;
   scenarios C and E each start a fresh coordinator so they measure their own
   claim rather than this one.

   The same defect produces a second symptom on the *next* return: the
   already-applied deletions no longer apply, `git apply` fails, and the
   transfer is declined at `refs/lernie/conflicted/<child-id>` — so a watcher
   woken twice leaves a conflicted ref behind on its second cycle.

3. **A child of the in-model `dispatch` tool is born with an unanswerable
   history.** The child branch forks off the parent's tip at the moment the
   tool runs — which is *after* the parent committed the assistant message
   carrying the `dispatch` `tool_use`, and *before* it commits the matching
   `tool_result`. The child therefore inherits a dangling `tool_use`, and its
   first model call is refused by the provider:
   `{"type":"error","kind":{"provider":{"status":400}},"message":"No tool
   output found for function call call_…"}`. The child dies having done
   nothing, and its parent is told only that a child returned. The `lernie
   dispatch` **CLI** path is unaffected — it forks after the parent's step has
   settled — which is why `fleet/bin/fleet-up.sh` and `fleet/test.sh` dispatch
   the watchers through the CLI, and why scenario C measures the work-product
   transfer on that path while asserting separately against this one.

4. **Declaration closure is not gated at execution for ordinary roles.** A
   request declares every tool its *inherited* history names, so the wire holds
   (`src/prompt/dispatch/tools.rs`) — and a child inherits its parent's
   transcript. For the compactor, calling such a tool is refused in-band
   (`compactor::refusal`); for every other role there is no such check, so a
   declared-by-closure tool would actually run. A sensor dispatched after its
   coordinator had used `slack_post` would therefore have `slack_post` in its
   array and nothing stopping the call. The structural one-speaker property
   holds only while the coordinator has not yet used the tool in the history
   the child inherits — which is a weaker guarantee than the grant implies.
