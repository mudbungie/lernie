+++
title = "A non-retryable in-band model-call error silently strands a root agent's branch forever — no epitaph, undetected by lernie scan, exit 0 on the follow-up"
created = 1785124361
updated = 1785129438
claimant = "Marlin"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["user-story"]
+++
## Repro (live wire, local/ollama provider — the documented tool-free
recipe from README's "What SMOKE_PROVIDER=local does and does not
prove")

1. `lernie new <ws>`, `lernie config <ws>` to point `providers.yaml`
   worker at `local`/a pulled ollama model (e.g. `qwen3.5:9b`, per
   README's documented override recipe).
2. `AID=$(lernie prompt <ws> 'What is 2+2? Use whatever means you like.')`
   — the model calls `bash`, gets a tool_result, and (as README already
   documents, brazen bl-fba7) the *next* model call errors:
   `{"type":"error","kind":"parse_input","message":"user accepts only
   text content"}` then `{"type":"end"}`. Expected/known per README; not
   filing that part.
3. `lernie message <ws> "$AID" 'are you there?'` — exit 0, empty stdout,
   as documented. Wait: the delivery commit (`transcript NNN: user`)
   lands, but the follow-up model call hits the identical class of error
   again and the branch advances no further.
4. `lernie scan <ws>` — reports `silent deaths: 0; died deposits swept:
   0; drivers launched: 0`. The branch is invisible to the operator's one
   sweep tool.

## Root cause / why scan misses it

ARCH §2.3 "Crash and recovery" / "Silent death is derived": the
silent-death test is "its latest step's `response.json` closed without a
terminal `end`". A non-retryable in-band `Error` segment still ends with
`{"type":"end"}` (that's how the segment terminator works, §4.4) — so a
branch stuck on a clean-but-fatal API error is on-disk indistinguishable,
to the sweep, from a normal branch that's merely idle between messages.
Nothing about "no live executor + last response ended in an Error, not a
Finish" is checked.

Compounding this: ARCH §2.3 "The transcript writer" says plainly "A
model call that never settles complete — retries exhausted, a
non-retryable error, a stop — commits nothing: the transcript gains no
entry, the branch tip does not move." So by design this is NOT a
terminal event (final-response/stopped/budget-exhausted/died) — no
epitaph is ever deposited, so for a ROOT agent (which has no parent to
notify anyway) there is now no signal anywhere on disk or on any CLI
exit code that this conversation is permanently wedged. A user re-running
`lernie message` gets exit 0 forever, and `lernie scan` reports a clean
bill of health forever.

(This is a distinct gap from the known ollama_chat tool_result rejection
itself — that part is documented and tracked in brazen as bl-fba7. This
finding is that *any* non-retryable in-band adapter error, from any
provider, produces a permanently-wedged, invisible branch on the root
path — a structural gap in the silent-death derivation, not a provider
quirk.)

## Expected

Either: (a) the silent-death sweep's "died mid-work" test should also
catch "latest attempt segment terminated in a non-retryable Error" (not
just "no terminal end"), so `lernie scan` reports it and an operator can
act; or (b) a root branch whose model call exhausts retries / hits a
non-retryable error should get *some* on-disk, user-visible signal (the
docs don't promise a specific one, but "the user has to manually read
steps/<id>/NNN/response.json to learn their conversation died" is not a
promise README or ARCH make anywhere either).

## Resolution (Bosun, 2026-07-26): reframe chosen and shipped

Delivered together with bl-1c94 on work/bl-1c94 (commit "Epitaph-gate
compaction_merge; classify failed model calls as silent deaths
[bl-1c94][bl-ee80]"); this ball closes empty against that landing.

**Reframe: the record was already truthful — the derivation read half
of it.** brazen's segment contract records the failure exactly
(`Error` event, then the segment's clean terminal `end`), and
`provider::segment::classify` already distinguishes `Failed` from
`Complete` and `NoTerminal`. The executor fabricates nothing (option
(a) was rejected: suppressing the segment terminator would falsify the
§4.4 record to satisfy a consumer — the classic two-representations
drift). The bug was that the silent-death derivation tested only
"closed without a terminal `end`", i.e. read the framing tail for one
of its two death shapes. The one invariant that dissolves the special
case, root and child alike: **a branch with no live executor whose
latest step never settled complete (§2.3) is dead** — `NoTerminal`
(killed/stopped mid-stream, §2.9) and `Failed` (retries exhausted or
non-retryable, §2.10) are both "never settled complete". No root-only
mechanism, no second detection path, no new on-disk state.

Shipped:
- `step::latest_step_outcome` — the single framing-only read (latest
  step dir + `classify`), shared by scan and the message verb.
- `scan::derive::died_mid_work` classifies `NoTerminal | Failed`.
- `ScanReport.silent_deaths` is now the candidate **ids** (count
  derives as len, SSOT): a dead root gets no `died` deposit (no parent
  inbox), so its NAME in the `lernie scan` summary
  (`silent deaths: 1 (<id>)`) is its whole surfacing, pointing the
  operator at `steps/<id>/`.
- `lernie message` on a dead root: **queues-with-warning**, never
  errors. Docs imply queueing (ARCH §2.9: a message into a
  stopped/dead branch is THE resume path — refusing would block
  retry-after-fix; §2.11 declines a deposit only for a nonexistent
  agent). The verb prints a stderr advisory when the recipient is
  quiescent and its latest call `Failed` (exit code and stdout
  untouched). ARCH §2.10 now records the derived-flag semantics and
  the advisory; README's messaging + scan sections updated.
- Tests: scan sweep unit (`Failed` root counted and named; `Failed` is
  died-mid-work), e2e `scan_names_a_root_whose_model_call_failed`
  (Wharf's exact live-wire shape), cmd advisory test.

Distinct from brazen bl-fba7 (the ollama tool_result rejection): this
fix is provider-agnostic surfacing; the provider gap remains tracked
there.

## Landing (Oakum, 2026-07-26)

Landed on `main` inside commit **6d4412b** (titled for bl-1c94; it
carries both balls' work). CI run 30220130316 is **green**.

Verified present in that commit: `step::latest_step_outcome` (the one
framing-only read); `scan::derive::died_mid_work` matching
`NoTerminal | Failed`; `ScanReport.silent_deaths` as a `Vec<String>` of
ids rendered as `silent deaths: N (<id>, ...)`; the `lernie message`
stderr advisory (`branch_failed` / `failed_branch_note`, deposit and
exit code untouched); tests
`a_root_with_a_failed_model_call_is_a_named_silent_death`,
`a_failed_latest_response_is_a_death`, e2e
`scan_names_a_root_whose_model_call_failed`,
`message_advises_on_a_quiescent_branch_whose_latest_call_failed`; README
messaging/scan sections and ARCH §2.10 amended.

The code landed but the store-side close never completed (Marlin's
session ended first). This close is bookkeeping only — an empty
delivery over content already on `main`.
