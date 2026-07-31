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
| One speaker per outward surface | Per-role `tools:` grants, and therefore **structural at both declaration and execution**: `slack_post` is granted to `worker` alone, the fork prunes every descriptor a role does not grant so the sensor's model call has no way to name the tool (ARCH §3.3), and even a tool the sensor's inherited transcript names but its grant omits is declined in-band rather than run (ARCH §3.3 *Declaring is not permitting*, bl-5a1f). |
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
coordinator's grant survives their returns (bl-475a: a parent's
`descriptions/**` survives child returns); B seeds three channel lines and
asserts the sensor relays one `EVIDENCE:` line while its request carries
`slack_read` and cannot name `slack_post`; C charters a builder over the
in-model `dispatch` tool and asserts its first model call is accepted
(bl-4231: agent-initiated dispatch produces wire-valid children), then
measures the work-product transfer on the CLI dispatch path; D runs a
shepherd cycle; E asserts a new signed post reached the channel.

All assertions are expected to pass on this harness build.

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

## Harness defects this demo surfaced (fixed)

Four facts about lernie that a heterogeneous fleet hit immediately and a
single-role conversation never had — reported, not worked around in harness
code, and since fixed on main. The demo's configuration and assertions now
reflect the fixed contracts, not the defects.

1. **Fixed (bl-a900).** A child's descriptors now derive from the governing
   config commit filtered to the child's own grant, never the dispatcher's
   tree (ARCH §3.3 *The fork derives the branch's descriptors from the config
   commit, filtered to the role's grant*). Previously the fork pruned the
   *parent's tree*, so a chain of dispatches intersected grant after grant and
   a role's `tools:` could only narrow down the chain — a `sensor` granted
   `[slack_read, message]` off a coordinator that did not itself grant
   `slack_read` composed a request with no `slack_read` and no diagnostic.
   That forced the coordinator's `tools:` in `providers.yaml` to be a union of
   every descendant's grant. With the fix, grants no longer compose that way:
   the coordinator holds exactly `slack_post` plus its own job's tools, and
   `slack_read` is the sensor's alone.

2. **Fixed (bl-475a).** The work-product transfer's `CONTEXT_EXCLUDES` now
   also excludes `descriptions/**` from the fork-point→tip diff it applies,
   alongside the branch-scoped paths (`goal.md`, `soul.md`, `messages/**`,
   `summary/**`, `skills/**`). Previously a child's dispatch-commit prune of
   `descriptions/**` to its own narrower grant rode the diff back into the
   parent on return, deleting descriptors the parent itself was still
   granted — a coordinator's declared toolset shrank every time a child
   returned, with a second symptom on the *next* return (the already-applied
   deletions no longer applying, declined at `refs/lernie/conflicted/<id>`).
   `fleet/test.sh` scenario A asserts the coordinator's declared toolset is
   identical before dispatching its watchers and after both have returned.

3. **Fixed (bl-4231).** A child of the in-model `dispatch` tool no longer
   forks between the parent's `tool_use` commit and its `tool_result` commit,
   so it no longer inherits a dangling `tool_use` and its first model call is
   no longer refused (`"message":"No tool output found for function call
   call_…"`). `fleet/test.sh` scenario C dispatches a builder through the
   in-model tool and asserts its first model call is accepted, then separately
   measures the work-product transfer via the CLI dispatch path.

4. **Fixed (bl-5a1f).** *Declaring is not permitting* now holds for every
   role, not the compactor alone: a `tool_use` naming a tool outside the
   calling role's grant is declined in-band, whatever the role's inherited
   transcript names. Previously only the compactor had this execution-time
   check; every other role could run a tool its grant omitted but its
   inherited history declared — a sensor dispatched after its coordinator had
   used `slack_post` would carry `slack_post` in its request array with
   nothing stopping the call, so the one-speaker property held only until the
   coordinator had used the guarded tool once. The one-speaker property is
   now structural at execution as well as declaration (see the mapping table
   above).
