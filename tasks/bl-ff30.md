+++
title = "gate: docs"
created = 1785124418
updated = 1785124441
claimant = "Ratchet"
parent = "bl-9300"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-9300"
on = "claim"
+++
PASS — docs are already current; no edit needed, and none made.

bl-9300 delivered no source change (its fix had already landed on main as bl-7a3f), so there is no new behaviour for the docs to describe. Checked what already documents this area and found it accurate:

- CHANGELOG.md line 24 records the fix under its delivering ball: "Close gate flakes under parallel-agent load: spawn_retries_past_transient_etxtbsy races two wall clocks [bl-7a3f]".
- src/prompt/tool/tests/etxtbsy.rs carries a module doc stating the rule the file now follows ("Every wait in this file is sized far past what it is waiting on, or injected outright") and cites bl-1c2e / bl-7a3f.
- src/prompt/tool/subprocess.rs documents ETXTBSY_RETRY_BUDGET as injected per-executor via SpawnTool::with_etxtbsy_budget, with the rationale for injection.
- README.md ("A floor of exactly 100% only holds if ...") already states the general rule this class of flake violates.

The only stale pointer was inside the two ball bodies themselves (both cite the pre-move path src/prompt/tool/tests/errors.rs:169; the test now lives in src/prompt/tool/tests/etxtbsy.rs) — corrected in bl-9300's and bl-2061's bodies at close, not in any doc.