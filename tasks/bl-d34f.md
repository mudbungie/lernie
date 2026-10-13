+++
title = "a provider refusal paints '«wound» (wounded, which this seat cannot read)' in the transcript, and its remedy line says 'log in to carry it on' without naming /login or the provider"
created = 1791867842
updated = 1791867842
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: set a role to a provider with no credential on the engine ('lernie model home worker anthropic claude-haiku-5-5' on a box with no anthropic key), send a message in the window.
Expected: the transcript says what failed and how to mend it in the seat's words — the provider that refused and the verb that mends it (/login, which lists 'anthropic — auth api_key · no credential stored'; or /roles to move the role elsewhere).
Observed: the transcript's last entries are '«wound» (wounded, which this seat cannot read)' — an unknown-entry fallback leaking the seat's own internals onto the glass — and '⚠ the last step failed on credentials — log in to carry it on'. The ask box carries the engine's 'no credential for provider anthropic: set BRAZEN_API_KEY ... or run bz…' cut at 120 chars (yog bl-8550). Nothing names /login or /roles.
Why it matters: the commonest first-day failure reads as a seat bug ('which this seat cannot read'), and the remedy is a phrase rather than a verb.