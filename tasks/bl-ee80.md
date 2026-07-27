+++
title = "A non-retryable in-band model-call error silently strands a root agent's branch forever — no epitaph, undetected by lernie scan, exit 0 on the follow-up"
created = 1785124361
updated = 1785125326
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