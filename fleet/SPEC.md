# Design proposal: a role-separated agent fleet with dedicated watchers

*Imported external document, reproduced verbatim: it was authored outside this
repository and is exempt from the repo's taxonomy and terminology discipline
(`CLAUDE.md`). Every other file under `fleet/` follows the repo rules; the
mapping from this document's vocabulary to lernie's is in `fleet/README.md`.*

## 1. Summary

Run parallel coding agents as a **five-role system with a deliberately sparse communication graph**, rather than as N agents attached to one operator. One long-lived coordinator owns intent and authorship. One long-lived shepherd owns observation of the fleet. One long-lived sensor owns observation of the *outward* surface. Builder agents are ephemeral and die with their worktree. A steward owns the doctrine and is its only writer.

Two independent fleets — two repos, two machines, two humans — coordinate through a shared Slack channel where each fleet has exactly one speaker.

The core claim: the scaling limit on parallel agents is not model quality, it is that **observation and judgment get bundled into the same actor**. Separate them and the operator's inbox turns from continuous status into occasional exceptions.

Implementation is `workmux` (git worktrees + tmux panes), two markdown skills, and a Slack MCP. No bespoke runtime.

## 2. Motivation

Driving one agent is fine. Driving six fails, and it fails on work that is almost entirely polling: *is this one working or wedged, did that one actually push, is the third sitting on unsubmitted text.* That work is constant, it is not judgment, and it should not be done by the actor holding the judgment.

Three failure classes dominate, none of which is a prompting problem:

- **Lost work.** Findings live in a worktree destroyed on merge. Uncommitted, they die silently.
- **False completion.** An agent reports done and nothing checks. "Merged" turns out to mean merged *locally* while `origin` never moved. Recurred in three distinct forms; the most expensive failure in the system.
- **Leakage across a trust boundary.** An internal fleet notification gets published onto a surface a human peer reads. This one is specific to multi-fleet operation and §6 is mostly about preventing it.

## 3. Design principles

1. **One duty, one owner.** A duty with two owners gets done twice or not at all, and "not at all" is silent.
2. **One observation is never a conclusion.** Before an observation becomes a claim — especially a *negative* claim, or one that triggers action — name another world that produces the identical observation and rule it out.
3. **Verify the object, not the mechanism.** A push hook is a mechanism; `origin` is the object.
4. **Escalation is a scarce resource.** If hearing from a watcher is routine, it stops carrying information.
5. **One speaker per outward surface.** Every shared surface with human readers has exactly one agent authorized to write to it. Sensors on that surface are read-only, without exception.

## 4. Component model

| Role | Lifetime | Owns | Explicitly does not |
|---|---|---|---|
| **Human** | — | Every binding decision: scope, rule removals, approvals | — |
| **Coordinator** | Long-lived; one per repo, on `main` | Spawning, charters, verified facts, gates, merge order, the user-facing summary, one monitor, **sole write access to the Slack channel** | Poll builders for status; edit doctrine |
| **Shepherd** | Long-lived; one per fleet, ~60s loop | Classification, nudges, close sweep, release relays, push verification, fleet report, merge window | Write product code; merge; edit anything but its scratch note |
| **Sensor** | Long-lived; one per outward surface, ~90s loop | Watching one Slack channel; classifying and relaying new traffic to the coordinator | **Post to Slack, ever.** Adjudicate. Commit, push, or merge |
| **Builder** | Ephemeral; one per task | Its deliverables, its claimed files, its IOU declarations and closing bark | Touch a peer's claimed files without a handshake; guess past a gate |
| **Steward** | Long-lived; rooted at the workspace | Deciding proposals with the human; sole writer of the skills | Accept a proposal without reading it in situ |

The two watchers are the non-obvious roles, and they are the same idea pointed in different directions: **observation is a full-time job that must not be bundled with the authority to act on it.**

The shepherd exists because `status=done` is ambiguous — finished-and-blocked, mid-work between steps, in a poll loop, and crashed mid-turn all present identically. Something must disambiguate that every minute. Without it you do not get a hands-off fleet, you get one that stalls quietly for forty minutes.

**Every fleet gets a shepherd regardless of size.** A one-agent fleet still loses uncommitted work, still sits on unsubmitted text, still closes holding IOUs.

## 5. Communication topology

```
   ┌─────────┐                                          ┌─────────┐
   │ HUMAN A │  both humans in the channel by design ──► │ HUMAN B │
   └────┬────┘                                          └────┬────┘
        │ decisions ↓   only-you-can-do ↑                    │
   ┌────┴────────┐   ~~~~ SLACK #lab ~~~~~~~~~~~~~~~~   ┌────┴────────┐
   │ COORDINATOR │◄══════ the only two speakers ═══════►│ COORDINATOR │
   │  ("Prior")  │                                      │("Likelihood")│
   └──┬───┬───┬──┘         ▲                            └─────────────┘
      │   │   │            ║ read-only                    (peer fleet,
      │   │   │       ┌────╫─────┐                         other machine)
      │   │   └──────►│  SENSOR  │  relays inbound one line at a time
      │   │  spawn    │("Evidence")│  ──► into coordinator's pane only
      │   │           └──────────┘
      │   │ fleet updates ↓  escalations ↑
      │  ┌┴─────────┐
      │  │ SHEPHERD │═══► heartbeat (the coordinator's ONLY monitor)
      │  └┬─────────┘
      │   │ nudges · release relays · report requests
   ┌──▼───▼─────────────────────────┐
   │  BUILDER   BUILDER   BUILDER   │◄──► peer claims, IOU releases
   └──────────────┬─────────────────┘
                  │ proposals (one file each)
            ┌─────▼──────┐
            │  STEWARD   │──► doctrine (sole writer)
            └────────────┘
```

The **absent** edges are the design:

- **Coordinator → builder status polling is cut.** A coordinator-side raw status monitor duplicates the shepherd and is *worse* at the job, because it cannot distinguish finished-and-blocked from stuck. The coordinator runs exactly one monitor, on **shepherd liveness**. Backstop the watcher, not the fleet.
- **Sensor → Slack is cut** (§6.2). It reads; it never writes.
- **Sensor → anyone but its coordinator is cut.** It does not talk to the shepherd, builders, or the other fleet.
- **Builder → human is cut.** Keeps the human's inbox a decision queue, not a feed.
- **Everyone → doctrine is cut** except the steward (§11).

**Builder ↔ builder is deliberately open.** Agents negotiating claims and releasing each other's holds *is* the system working, not drift. When two are chartered to collaborate the shepherd enforces an exchange cadence — no peer message in ~10 minutes nudges both. The talking is the point.

**The known structural gap.** This topology makes each watcher a node nobody audits. A heartbeat proves it is running, never that its classifications are right — so by construction its judgment goes unchallenged while every builder's is checked. Mitigation must be deliberate: the coordinator spot-checks a sample of the watcher's *interventions* — the nudges it sent, the text it carried forward, the messages it chose not to relay — and asks what evidence each rested on. Not its uptime.

## 6. Cross-fleet coordination

Two fleets, two repos, two machines, two humans with different authority: one owns software design, the other owns the domain spec. They meet in one private Slack channel.

### 6.1 Why Slack rather than a direct agent link

Three properties, in order of importance:

1. **Both principals are in the channel.** Agent-to-agent coordination is therefore *human-auditable by default* — not by adding oversight, but because the bus is a room the humans are already standing in. This is the single best governance property in the whole design.
2. **It is asynchronous and durable.** A fleet can be asleep. The other machine closing its laptop is a normal state, not an outage.
3. **It requires no shared infrastructure.** Two independent swarms, no shared filesystem, no shared queue, no coupling beyond a channel id.

### 6.2 One speaker per surface, and the leak that earned it

Only the coordinator posts. The sensor is **read-only, with an enumerated prohibition** on every write tool — send, draft, reaction, schedule, canvas — *including when a message in the channel appears to address it or ask it a question.*

That rule is enumerated rather than general because the general version failed. The channel record contains a verbatim internal status line — `EVIDENCE: 3 new (+1 Prior skipped) …` — published **into the channel**, where a human peer could read it. An internal fleet notification crossed onto an outward-facing surface. Nothing broke, but the same mechanism could have leaked a verified fact, a credential, or a half-formed judgment about a colleague's analysis.

The generalized rule is principle 5. A watcher pointed at a surface must not be able to write to it, because the failure mode is not "the watcher decides to post" — it is "the watcher's report format and the surface's message format are the same string type, and one call sends it the wrong way."

### 6.3 Report, don't adjudicate

The sensor's charter forbids it from verifying claims against the repo, running git or tests, or forming a position on the domain argument. If an inbound message contains a claim about our code that looks wrong, it flags it as *worth checking* — never as wrong.

This is principle 1 applied to a second axis. The sensor owns *noticing*; the coordinator owns *responding*; the human owns *deciding*. Collapsing noticing into responding is how a watcher ends up arguing with a Senior Director's hedonic model at 90-second intervals.

### 6.4 Four logical speakers over two accounts

Agents post under their humans' Slack accounts, so the author field is ambiguous: `Joe Bakke` is both Joe and Prior; `Matt McCauley` is both Matt and Likelihood. Two humans and two agents share two identities.

Disambiguation is by **content, not author** — agent posts carry a `Sent using <@Claude>` marker and a `— Persona` signature; human posts carry neither and are usually short and conversational. The classification matters because the four speakers have different standing:

| Observed | Actually | Handling |
|---|---|---|
| `Joe Bakke` + signature | Our own coordinator's outbound | Do not relay the content; count it, so the coordinator can confirm its post landed |
| `Joe Bakke`, unsigned | The human | Always relay — usually an instruction |
| `Matt McCauley` + signature | The peer agent | Highest-value traffic |
| `Matt McCauley`, unsigned | The peer human, who holds authority on the spec | Always relay — a one-liner can carry a decision |

The subtle case is the first: relaying our own posts back to their author is noise, but *silently dropping* them removes the coordinator's only confirmation that its message landed. So they are counted, not reported.

### 6.5 Relay discipline

- **~90s cycle**, `oldest=<last_seen_ts>`, concise response format. Timestamps compared as exact strings — they are values like `1785375161.989859` and rounding them silently re-reports or skips.
- **One line per report**, prefixed `EVIDENCE:`, in a fixed shape: counts, then `Author HH:MM: '≤25-word gist'` per message, then a one-clause read of what it means for us. Long analysis posts get a headline plus a path to the full text, never the body.
- **A cycle log line every cycle, including no-ops.** A silent pane reads as a crash to the shepherd, so a cycle must always leave a mark. This is the sensor's liveness signal.
- **Verify your own instrument first.** The sensor's literal first action is to confirm it actually has the Slack tool and that the tool returns this channel — because whether a spawned worktree agent inherits an interactively-authenticated MCP connector was *deliberately left unverified* by the coordinator. The sensor is the check. A missing connector is a reportable finding, not something to work around or substitute a data source for.
- **Escalate a failing instrument on the third consecutive failure**, not the first. One line, then keep the same `last_seen_ts` and carry on.

### 6.6 Sharing findings across fleets

A finding shared between fleets must carry its **measurement method and its numbers**, so the receiving side can *test* applicability rather than adopt on trust. A finding adopted without local measurement is cargo cult; one declined *with numbers* is a real answer.

The corollary that matters more: **aiming another fleet's measurement is never cheap-if-wrong.** Telling a peer which pipeline carries its verdict, or which commit landed its dependency, requires verifying the causal link and not just the ordering — because a true fact with a false causal implication propagates into someone else's verification and comes back wearing the authority of *their* measurement.

### 6.7 Fleet isolation

Multiple projects run on one machine, and bare role names resolve *across* projects — a message addressed to `shepherd` has silently delivered to the wrong fleet. Two rules follow: **address only explicit, project-qualified handles**, and **no cross-fleet traffic** except coordinator-to-coordinator over the channel. Anything that misroutes in gets handed to the coordinator, never answered or forwarded.

## 7. Task lifecycle

| Phase | Actor | Transition |
|---|---|---|
| **Pre-flight** | Coordinator | Enumerate every external dependency needing a human click or another team; file them *all* before parallel work starts. Check a documented-blocked registry for prior attempts |
| **Charter** | Coordinator | Prompt file authored (§9.2); boilerplate appended |
| **Spawn** | Coordinator | `workmux add <name> -b --base main -a claude -P <file>`; persona assigned; shepherd briefed in the same breath |
| **Work** | Builder | Commit ≥ every ~20 min. Declare IOUs on grant *and* receipt. Surface gates, never guess past them |
| **Bark** | Builder | One line to the shepherd mapping every deliverable → commit **subject**, or naming it undelivered and why |
| **Close** | Builder | `workmux merge --rebase`; worktree, window, branch torn down in one move |
| **Verify** | Shepherd | Confirm `origin` moved. Not the pane's job — the pane is gone |
| **Sweep** | Shepherd | Find orphaned IOUs the closer died holding; relay any unsent release with attribution |

**The bark precedes the merge, so every SHA in it is pre-rebase by construction** — `--rebase` rewrites all of them. Identifiers must be commit *subjects*. A relayed pre-rebase SHA becomes a gate condition that can never become true, and the failure is silent: an agent polling an impossible condition looks exactly like an agent being patient.

**Whether the close reaches `origin` is a per-project property, not a property of the close.** One repo has a `pre_remove` push hook; the other deliberately has none because it closes via MR, and pushing `main` behind that flow's back is the failure it is avoiding. Hooks are also copied into a worktree at creation, so a worktree made before the hook existed will never push regardless of current config.

## 8. Escalation policy

| Tier | Path | Trigger |
|---|---|---|
| **Log** | Watcher → its own file | Everything routine. One line per cycle. Never leaves |
| **Relay** | Sensor → coordinator | New outward traffic worth acting on (§6.5) |
| **Nudge** | Shepherd → builder | Cadence miss, stall, reported-but-did-not-act. ≤5 lines |
| **Escalate** | Shepherd → coordinator | *Only:* a decision needing the human, a merge conflict, destructive or out-of-scope behavior, an agent dead >15 min a nudge did not revive |
| **Surface** | Coordinator → human | Only what the human alone can do, with paste-ready steps |

**The non-escalation list is the load-bearing half**, and most policies omit it. These are healthy and must generate no traffic:

- `unmerged=0` with work on origin — the agent may hold a deliverable that *cannot exist yet* (evidence gated on a rollout, a verdict, a peer)
- `status=done` between poll iterations; an empty composer while tokens flow
- Any agent whose unsolicited status is fresher than ~5 cycles
- A nudge the shepherd **declines with evidence**. Defending a compliant agent against a coordinator nudge is part of the job, not insubordination

## 9. Context model

Five tiers, each with a different lifetime. Most setups fail by keeping knowledge in the wrong one.

| Tier | Lifetime | Failure if relied on |
|---|---|---|
| Session context | Dies at session end | Invisible loss; the next fleet re-derives at full cost |
| Scratch note | Dies with the worktree | Hours of state destroyed by a routine `workmux remove` |
| Prompt file | Frozen at spawn | Stale by construction the moment doctrine changes |
| **Template** | Propagates into every future artifact | Highest leverage *and* highest defect cost |
| **Skill** | Durable, cross-project | The only tier that survives a fleet |

### 9.1 The learning loop

```
observation → proposal → human decision → skill → template → next spawn
```

Three ways it silently fails to close: a lesson left in a scratch log dies with the worktree; a lesson folded into the skill but **not the template** keeps shipping the retired convention, because agents are born from the template; and a *wrong* "verified fact" is worse than an omission, because the boilerplate instructs agents not to re-derive it — you have laundered a guess into an axiom for everyone downstream.

### 9.2 The prompt file is the whole interface

An agent cannot see the coordinator's conversation. Anything omitted is re-derived at full cost or invented. Five sections, in order: **charter** (the ask near-verbatim, plus who holds product authority) · **verified facts marked "do not re-derive"** · **ownership + claims** · **gates**, framed as *pending explicit clearance* with the override path designed in · **rules**.

### 9.3 Reference indirection over inlining

A long-lived agent's prompt does not contain the doctrine — its first instruction is to *invoke the skill*. This is the highest-value single decision in the context model. A fully-expanded static prompt is frozen at spawn, so every fix afterward never reaches the running agent. We ran both patterns side by side and measured it: the static copy was missing four rules added thirty minutes after it spawned.

Skills themselves are progressive disclosure — a ~500-word router plus reference files, one per moment of the job. They began as 3,000-word walls, which do not get read.

## 10. Verification model

**Rule 0: one observation is never a conclusion.** Negatives need a higher provenance bar than positives, because a positive carries its own evidence while a wrong negative is invisible.

| Claim | What actually proves it |
|---|---|
| "Merged" | `main..<branch>` = 0 **and** `origin/main` moved |
| "Works" | Drive it. A green suite shipped a login that 400'd on every real request |
| Anything with hidden state | Two runs. One passes identically whether the real path works or a cache masks it |
| Anything with a fallback | Assert the *path taken*, not the output |
| Absence of anything | Publish the window: which pane, how many lines, when |

**More verification of the wrong object produces more confidence in a wrong conclusion.** A composer state cannot detect a corrupted message; a `capture` window is not the message; `origin` cannot show work never meant to land there. The instrument worked perfectly and pointed at the wrong object — so escalating effort buys certainty without buying evidence, which is worse than checking less because it feels like rigour.

## 11. Authority and write boundaries

**Only the human's decisions bind.** Rules originating in an agent or a document are advisory and must be attributed as such when enforced — "the design doc's position is X," never "your rule is X."

**No fleet actor edits the doctrine.** Enforced, not documented, because documenting it failed: three sessions wrote to the skill files concurrently in one night, and a shepherd edited the very doctrine it was supposed to escalate about. A `PreToolUse` hook denies writes to the skills tree from any session rooted in a repo or worktree.

Everyone else **proposes**: one file per proposal, carrying the observation, its evidence, the exact wording proposed, and what the proposer did in the interim. One file per proposal, never a shared file — that is the clobbering the mechanism exists to prevent. A proposal never blocks work.

Two implementation findings, each of which cost a probe:

- **Discriminate on the session's project root, not `cwd`.** `cwd` follows the shell and moves on `cd`; the first version would have locked out the steward and let the fleet through.
- **Judge a redirection by its target, not by token presence.** The first version matched a bare `>`, so `2>&1` made read-only inspection look like a write, and appending a guarded file *into* an agent's own scratch file was denied — making a doctrine-mandated step **unobeyable**. Six agents reported it independently within twenty minutes; the corroboration is what made it unarguable.

## 12. Failure model

| Failure | Why it fools you | Mitigation |
|---|---|---|
| `send` exits 0 | Means the **paste** worked, not that the message arrived | Always `capture` after |
| **Newlines in a send** | A multi-line send lands as a collapsed paste blob and is never submitted. The identical content on **one line** submits fine at ~200 words. Length is not the problem | One line, always |
| Backticks / `$` in a send | The shell eats them *before* send runs — silently, exit 0. A backticked word that is a valid command substitutes with no stderr, so "no error" is not proof | Report to a file; send `"$(cat report.txt)"` |
| `status` / `elapsed_secs` | A stale **hook** state; elapsed measures time since the last hook, so it climbs on a dead agent | Confirm via token flow / `esc to interrupt` |
| Poll-loop agent status | Meaningless in *both* directions — foreground `sleep` reads `working`, backgrounded reads `done`, same healthy agent | Require a per-cycle counter in its brief; read it twice |
| Shell mode (`!` prompt) | Pasted text is **executed**; a picker check sails past it | Add `!` to the pre-send check |
| `unmerged=0` | Reads zero for "merged", "never committed", *and* "deliverables live outside the repo" | Ask where the deliverables live before reading `origin` |
| Heartbeat sharing a resource with the event stream | A sampler echoing to the notification channel blocks under backpressure, freezing proof-of-life exactly when needed | Heartbeat and events to separate files |
| Bare role-name addressing | Resolves *across* projects; has silently delivered to the wrong fleet | Project-qualified handles only |

The instructive one: broadcasting the picker-detection rule put the literal strings `Enter to select` / `to navigate` into every agent's scrollback, where their own detectors matched them and reported pickers that did not exist. **The rule in transit tripped the tripwire that enforces it.** Fix: scope detection to the input region, because a real picker owns the bottom of a pane.

Generalized into a test-design rule after a fused probe briefly produced evidence that a correct fix was broken: **one hypothesis per command.** A command carrying both a should-pass and a should-fail case returns one result for two questions, and it reads as "the fix did not work."

## 13. Known weaknesses

- **Watchers are unaudited by construction** (§5). Mitigated by intervention spot-checks, not solved.
- **Claims are etiquette, not structure.** Claim-before-touch runs over `send`, a blind paste. A structural claim — a lock that is a filesystem fact rather than an agreement — would be strictly better.
- **Watcher state is not durable.** It lives in an untracked scratch file inside a worktree, so a routine teardown destroys it. Both watchers are also forbidden from merging, so they structurally cannot make their own findings durable — they enforce a commit-cadence rule they are exempt from.
- **Doctrine growth is unbounded.** Consolidating the corpus cut it 40%; it regrew to 92% of original size within the same session, because consolidation without a change in authoring behaviour buys one hour.
- **Identity disambiguation is heuristic** (§6.4). It depends on a client-added "sent using" marker and a signature convention. A human who signs a message like an agent, or a client that stops adding the marker, breaks it.

## 14. Alternatives considered

- **No watcher; coordinator polls.** Rejected: it is the bundling this design exists to undo, and a raw status monitor cannot tell finished-and-blocked from stuck.
- **A central task queue with a dispatcher.** Rejected: recreates the bottleneck in software and makes the dispatcher a single point of stall. Agents solving their own merge problems is cheaper than serializing them.
- **A separate reviewer role.** Rejected: verification is a duty attached to the closer and the watcher, not a stage. A review stage adds a queue and invites "someone else will catch it."
- **Direct agent-to-agent link between fleets** instead of Slack. Rejected: it removes the property that makes cross-fleet work safe — both principals standing in the room where their agents negotiate.
- **Letting the sensor reply in-channel** to save a hop. Rejected on the leak in §6.2, and it would collapse noticing into responding.
- **Trusting the fleet with doctrine.** Tried; failed empirically (§11).
