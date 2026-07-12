+++
title = "Rename assistant origin token: model id authors the entry [substrate]"
created = 1783832724
updated = 1783832824
claimant = "Vireo"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["substrate"]
+++
Design decision (user-approved): 'assistant' as the transcript origin token is wire-vocabulary editorializing. The origin token of a step's model output entry becomes the MODEL ID that produced it, as it rode the canonical request — messages/NNN-<model-id>.json (e.g. messages/007-claude-fable-5.json). Rationale: the author is the fact with weight; with role-based model assignment (§4.3) the transcript becomes self-describing. Provider row deliberately excluded: deployment routing, its home is brazen config + diagnostic request.json, and free-form row names would force filename sanitization (a second spelling of one fact).

The general rule to state in §2.3: every transcript entry's origin token names its author — the sender for .md delivered messages, the model id for model output, 'tool' for tool results. Role framing stays path-derived: .md composes user-role; .json composes canonical blocks, tool_result vs model output distinguished by the reserved token. 'tool' is the ONE reserved .json token ('assistant' ceases to exist); 'user' stays reserved in .md sender space. A model id colliding with a reserved token is declined (decline illegal operations), not munged.

Scope: docs/ARCHITECTURE.md §2.3 (origins list, the transcript writer passage, reserved-token sentence, any examples), §3.5/§5 mentions if any; docs/TAXONOMY.md if it names the token; README; code: dispatch/assembler.rs origin parsing, dispatch/transcript.rs commit_assistant (thread the model id from the canonical request to commit time), staging filename (steps/<agent-id>/<NNN>/assistant.staging.json → staging.json — the 'assistant' there is the same editorializing), tests. grep -rn 'assistant' across src/ docs/ README to catch stragglers; wire-role 'assistant' inside canonical block JSON is brazen vocabulary and stays.