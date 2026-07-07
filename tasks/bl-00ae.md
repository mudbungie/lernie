+++
title = "v0.6 — brazen provider layer (fold ~/dev/harness into lernie)"
created = 1783464230
updated = 1783464230
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-f739"
on = "claim"

[[blockers]]
id = "bl-507a"
on = "claim"

[[blockers]]
id = "bl-56ee"
on = "claim"

[[blockers]]
id = "bl-d653"
on = "claim"

[[blockers]]
id = "bl-660b"
on = "claim"

[[blockers]]
id = "bl-a46f"
on = "claim"
+++
The design exploration at ~/dev/harness (specs/architecture.md, 2026-07-03) folds into lernie. Its keeper is the adapter boundary: brazen (~/dev/brazen, github.com/mudbungie/brazen — our own project, pre-1.0, changeable to fit) replaces the bespoke per-provider adapter contract wholesale. One stateless binary bz adapts every provider/protocol behind a pipe contract: canonical request JSON on stdin -> canonical v=1 event NDJSON on stdout, one process per attempt, exactly one HTTP round-trip, sysexits exit codes, errors in-band, no retry (caller's job — brazen non-goal §2). What lernie gains: multi-provider (anthropic/openai-chat/openai-responses/google/ollama) via config only, automatic prompt caching, server-tool passthrough, reasoning-effort knob, OAuth/credstore auth lernie never touches. What moves into lernie: transient-error retry (attempt loop). What dies: crates/lernie-provider-anthropic, describe/complete subcommand contract, auth_env/endpoint_env forwarding, the bespoke event vocabulary (message_stop et al). Spec of record once the design child lands: docs/ARCHITECTURE.md §4.4 (Draft v0.4). The other ~/dev/harness keepers (budgets, WASM sandbox) are filed as standalone balls, not children of this epic.