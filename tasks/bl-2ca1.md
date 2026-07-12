+++
title = "Delete await_tool: await/check have no referent once step 5 is total [substrate]"
created = 1783829720
updated = 1783830754
claimant = "Wren"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["code"]

[[blockers]]
id = "bl-65d8"
on = "claim"
+++
Blocked on bl-65d8 (the doc must land first).

Delete `src/prompt/tool/builtin/await_tool/` entirely: `poll_until_terminal`, the 500ms `POLL_INTERVAL` sleep-poll, `validate_descent` / `Error::NotADescendant`, the four merge-era statuses (`merged` / `stopped` / `conflicted` / `budget_exhausted`), and the `/proc/<pid>/fd/*` scan **as used by await** (the scan survives for `lernie stop` per §2.9 and for §3.5 `in_flight` classification — do not delete `PgidFinder`/`ProcFsFinder` themselves).

Per bl-65d8: once every terminal event deposits a result message, the inbox is the only return path and there is nothing left for `await`/`check` to observe. `check` was never built (`skills/await/SKILL.md`: *"A non-blocking `check(handle)` is filed for v0.5+ and not part of v0.4."*) — it must NOT be built. Also delete `skills/await/`.