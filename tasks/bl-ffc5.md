+++
title = "tool results omit the exit code and drop stderr on success — return both, always, like codex does"
created = 1785647733
updated = 1785647733
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Filed from the 2026-08-02 yog session (follow-up recorded in bl-298c's body). lernie's tool_result gives the model only is_error: the exit code is never surfaced, and stderr is dropped entirely when the command exits 0 — so a model can't distinguish exit 1 from exit 127, and warnings on successful runs are invisible. Codex always returns 'Exit code: N', wall time, and aggregated stdout+stderr; gpt-5.x models are tuned to read that. This is an ARCH §3.3 stdio-contract change affecting every tool, not just bash — design the result envelope once (e.g. exit code line + merged or labeled streams), update every tool's result path and the SKILL.md descriptions to match (they must promise what the wire delivers, cf. bl-298c's discipline), and pin the new shape in the toolspec tests. Operator approved filing 2026-08-02.