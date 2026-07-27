+++
title = "CI on main is RED: cargo-tarpaulin segfaults somewhere in prompt::tool::builtin::bash::tests (moves between tests)"
created = 1785124427
updated = 1785124465
priority = 9
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
BLOCKS THE 0.0.1 RELEASE (bl-9253) — the release job only fires on a GREEN CI run on main via workflow_run, and main is red. Ops agent Halyard hit this 2026-07-26; ops-only, so filing rather than fixing.

SYMPTOM — reproduced 4/4 on GitHub CI (ubuntu-latest, cargo-tarpaulin 0.35.2)
    WARN cargo_tarpaulin::statemachine::linux: Failed to find process for pid: <pid>
    WARN cargo_tarpaulin::statemachine::linux: Failed to find traces for pid: <pid>
   ERROR cargo_tarpaulin: Failed to run tests: A segfault occurred while executing tests
   Error: "Failed to run tests: A segfault occurred while executing tests"
'make ci' exits 2.

THE KEY FINDING — IT IS NOT ONE TEST. It moves.
Do not chase a single test name; the failure walks around the module.
- Runs 1-3 (commit 0a6a36a, run 30216944990 + two 'gh run rerun --failed'): died at
  prompt::tool::builtin::bash::tests::cascade_kills_child_when_stop_flips (pids 8422 / 8317 / 8432).
- Run 4 (commit 70775ce, run 30217247304): cascade_kills_child_when_stop_flips PASSED, so did
  cascade_kills_descendant_subprocess_tree; the lost pid (8557) landed on
  prompt::tool::builtin::bash::tests::happy_path_returns_zero_and_stdout_bytes instead, and then
  missing_shell_surfaces_spawn and nonzero_exit_propagated_with_stderr_separated reported FAILED
  before tarpaulin aborted.
So the unstable unit is the WHOLE prompt::tool::builtin::bash::tests module (src/prompt/tool/builtin/bash.rs) — every test in it spawns a bash subprocess, and tarpaulin's ptrace state machine loses one of them, then reports a spurious segfault and takes the run down. No assertion is genuinely wrong; the later 'FAILED' lines are collateral from the broken ptrace session.

FAILURE RATE / BISECT DATA (treat as data, not a conclusion)
- 39c10a5 GREEN (run 30216420512, 10m52s, COLD bz + cargo caches)
- 31cb7d0 GREEN (run 30216928513, ~3m)
- 0a6a36a RED 3/3 (~2m each, fully WARM caches)
- 70775ce RED (~2m, warm)
The delta from 31cb7d0 to 0a6a36a is d793c52 [bl-7e33], which is COMMENT-ONLY in tarpaulin.toml: 1 file, +17/-1, entirely inside the leading prose block (the one deleted line is prose replaced by prose). tarpaulin.toml's functional body is byte-identical:
    [default]
    exclude-files = ["src/bin/*", "src/bin/lernie/*", "src/e2e/*", "crates/*/src/main.rs"]
    ignore-panics = true
    timeout = "300s"
    args = ["--test-threads=1"]
An inert diff cannot cause this. The green runs were the SLOW ones (cold caches) and every red run was ~2m on a hot runner — so the real variable is almost certainly machine timing, and the release-drive pushes simply warmed the caches. Whoever takes this should re-run CI on the green commits a few times to get an honest failure rate before blaming any commit.

CONSTRAINTS ON A FIX
- cargo-tarpaulin is PINNED to 0.35.2 in BOTH .github/workflows/ci.yml and tarpaulin.toml's comment, deliberately: 0.35.3 was yanked, and 0.35.4+ silently drops inline '#[cfg(test)] mod tests;' files from the coverable denominator, which would move the coverage floor. Do not bump the pin casually — and if you do, it has two homes today, so check both.
- The repo requires 100% coverage, so excluding bash.rs or #[ignore]-ing the module is not free.
- It reproduces on GitHub's runner, not on the local close gate (d793c52 passed its own local tarpaulin gate). Expect to iterate against CI, not locally.
- 'args = ["--test-threads=1"]' is already set, so this is not test-level parallelism.

RELATED
- bl-9253 — the 0.0.1 release drive; blocked on this AND on bl-53e3.
- bl-53e3 — 'release-plz release' aborts on an agent-eval publish-consistency mismatch. Fully independent; BOTH must land before 0.0.1 can publish.
- Prior art for this class in-repo: 126de9f [bl-7a3f] (close-gate flake under parallel load), 24768d0 [bl-5f0c] (stop-cascade pgid discovery signalling the wrong process group).