+++
title = "lernie model takes a model id the workspace's own 'lernie models' does not list, and the failure only shows up a turn later, truncated"
created = 1788673706
updated = 1788673706
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 daily lane. lernie 0.1.30 against yog 0.0.38.

`lernie model <ws> <role> <provider> <model>` accepts a model id the same seat can prove does not exist, answers ok, and the workspace is then broken until somebody reads a step record.

    $ lernie model proj worker claude-session-direct not-a-real-model-xyz
    {"exit":0,"kind":"outcome","ok":true,"stderr":"","stdout":""}

    $ lernie roles proj
    ... {"model":"not-a-real-model-xyz","provider":"claude-session-direct","role":"worker"} ...

The next deposit dies. The seat's own row then carries it, truncated (filed against yog separately):

    failure: "model: not-a-real-model-xyz; `not-a-real-model-xyz` is not in the model cache; run `bz --list-models` to refresh or enab"

The seat already holds the answer. `lernie models <workspace> <provider>` is a verb in the same table — it answers this row's eleven ids off the same channel, in one round trip, and `not-a-real-model-xyz` is not among them.

The documented rule is litany's — "a model id is validated by the wire at the first live model call" — and that is the right rule for the ENGINE, which must not become a second authority on a provider's catalogue. It is not an argument for the seat staying silent when it has just been given a word it can check against a list it already knows how to fetch.

Recovery, for the record, is clean and worth keeping: set the model back, `lernie nudge`, and the conversation resumes and finishes. That half works well.

Expected: `model` warns when the id is not in the row's offered set — a warning, not a refusal, since a fresh cache and partial matching are both real. Naming the nearest few would cost the same round trip.

Severity p3: recoverable, and it costs a whole turn plus a drill-in to diagnose.