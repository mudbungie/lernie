+++
title = "stop mid-tool-window wedges the branch: the stopped exit leaves an unpaired tool_use tail, so the next advance declines with UnpairedToolUse and no deposit can ever revive the agent"
created = 1785824337
updated = 1785905559
claimant = "Tatter-b98d"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
A `lernie stop` that lands inside a **tool window** leaves the branch in the one
state `lernie advance` refuses, so the agent cannot be restarted by a deposit —
it is wedged until a human forks from history. Verified against the published
0.0.6 source, not inferred.

## What happens today

`run_exchange` (`src/prompt/dispatch/mod.rs`) commits the assistant entry —
`tool_use` blocks and all — at line 215 (`transcript::commit_assistant`), and
only then enters the tool window at line 249 (`run_tool_calls`). The §2.9 group
SIGTERM fells the in-flight tool subprocess; `tool_step` reads
`KilledBySignal` + the stop flag and returns `ToolWindow::Stopped`
(`src/prompt/dispatch/tool_step.rs:222-225`, `tool_step/seam.rs:63`); the loop
breaks and `terminal::conclude` deposits the `stopped` epitaph.

Nothing settles the window. The branch tip is now an assistant entry whose
`tool_use` has no committed `tool_result`. That is exactly `Warrant::Unpaired`:

    src/prompt/dispatch/advance.rs:192
    Warrant::Unpaired => Err(Error::UnpairedToolUse { branch: ... })

    src/prompt/error.rs:159-165
    "branch {branch} tip is an assistant entry with tool_use unmatched by
     committed tool results — a mid-step crash after the assistant entry
     landed; tool side effects are not replayable, so this is declined (§6).
     Recover by fork-from-history (§2.3)."

So the next deposit into that agent starts a driver (§2.11 release rule, as
designed) and that driver **errors out**. The deposit is stranded; every
subsequent one is too. The stop did not pause the agent, it retired it.

A stop landing in the **model-call window is already clean** — the loop breaks
at `mod.rs:195` *before* `commit_assistant`, so no assistant entry lands, the
tail stays user-side, and a later deposit resumes at `Warrant::ModelCallDue`.
The defect is confined to the tool window.

## The ask

**The stopped exit should settle its own tool window**, the same way the tool
seam already settles a decline: commit one in-band `is_error` `tool_result` per
unanswered `tool_use` id (`refusal` in `tool_step.rs` is the existing shape),
saying the invocation was interrupted. Then the tail is settled, the warrant is
`ModelCallDue`, a deposit revives the agent normally, and the model reads
"you were interrupted" in-band — which is the truthful record and the useful
one.

Deleting the tail (as `step_commit/unsettled.rs` does at a *fork*) is the wrong
repair here: on the agent's own branch it would discard the assistant's own
reasoning, and the model would never learn it was cut off.

**No new verb is wanted.** `lernie stop` already means "end the current
attempt"; it just does not currently leave the branch in a state anything can
resume. If the honest settlement is considered a behaviour change rather than a
bug fix, the alternative is a finer `interrupt` that ends only the current
attempt — but the settlement is needed either way, because a plain `stop`
mid-tool wedges the agent for every consumer, not just yog.

## Who is waiting

yog `bl-a33d` — composer send-and-interrupt (Ctrl+Enter): interrupt the
in-flight work, deposit the message, and let the deposit's driver-start be the
trigger. That composition is `stop` then `message`, and it is unbuildable while
`stop` lands agents in `UnpairedToolUse`: the gesture would brick any agent that
happened to be inside a tool call. Held pending this.