+++
title = "CI on main is RED: cargo-tarpaulin segfaults at prompt::tool::builtin::bash::tests::cascade_kills_child_when_stop_flips"
created = 1785124427
updated = 1785124427
priority = 9
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
BLOCKS THE 0.0.1 RELEASE (bl-9253) — the release job only fires on a GREEN CI run on main, and main's HEAD is red. Ops agent Halyard hit this 2026-07-26; ops-only, so filing rather than fixing.

SYMPTOM (reproduced 3/3 on GitHub CI, run 30216944990 + two reruns, commit 0a6a36a)
    test prompt::tool::builtin::bash::tests::cascade_kills_child_when_stop_flips ...
      WARN cargo_tarpaulin::statemachine::linux: Failed to find process for pid: 8432
      WARN cargo_tarpaulin::statemachine::linux: Failed to find traces for pid: 8422
     ERROR cargo_tarpaulin: Failed to run tests: A segfault occurred while executing tests
    Error: "Failed to run tests: A segfault occurred while executing tests"
'make ci' exits 2. The pid differs each run (8422 / 8317 / 8432); the test and the failure mode do not. No test assertion fails — tarpaulin's ptrace state machine loses the process and reports a segfault. The run aborts there, so nothing after it is measured.

BISECT EVIDENCE (this is the confusing part — please treat it as data, not a conclusion)
- 39c10a5 — CI run 30216420512 GREEN (10m52s, cold bz cache).
- 31cb7d0 (merge of PR #1, CHANGELOG.md only) — CI run 30216928513 GREEN.
- 0a6a36a (= 31cb7d0 merged with d793c52) — RED 3/3, ~2m each.
The ONLY delta from 31cb7d0 to 0a6a36a is d793c52 [bl-7e33], and that commit is COMMENT-ONLY: 1 file, +17/-1, all inside the leading comment block of tarpaulin.toml (verified with git show; the single deleted line is a prose line replaced by prose). tarpaulin.toml's functional body is unchanged:
    [default]
    exclude-files = ["src/bin/*", "src/bin/lernie/*", "src/e2e/*", "crates/*/src/main.rs"]
    ignore-panics = true
    timeout = "300s"
    args = ["--test-threads=1"]
So an inert diff cannot be the cause, yet 0/3 vs 1/1 is the observed split. The likely real story is that this is TIMING-SENSITIVE, not commit-sensitive: the green runs were slow (cold caches, ~11m and ~3m) and the red runs were fast (~2m, fully warm caches), and a hot runner changes the scheduling around the test's kill of its child. Whoever takes this should NOT assume d793c52 is at fault — re-run CI on 31cb7d0 a few times to get a real failure rate before touching anything.

WHAT THE TEST DOES
prompt::tool::builtin::bash::tests::cascade_kills_child_when_stop_flips (src/prompt/tool/builtin/bash.rs, landed in 193e81a [bl-ecf1] 'v0.3 built-in tool: bash') signals a spawned child process group. Tarpaulin ptraces everything; a test that kills processes out from under the ptracer is the classic trigger for 'Failed to find process for pid' + a spurious segfault report. This repo already has a live history of this class: 126de9f [bl-7a3f] (close-gate flake under parallel load) and 24768d0 [bl-5f0c] (stop-cascade pgid discovery signalling the wrong process group).

CONSTRAINTS ON A FIX
- cargo-tarpaulin is PINNED to 0.35.2 by BOTH .github/workflows/ci.yml and tarpaulin.toml's comment, deliberately: 0.35.3 was yanked and 0.35.4+ silently drops inline '#[cfg(test)] mod tests;' files from the coverable denominator. Bumping the pin changes the coverage floor — do not do it casually.
- The repo requires 100% coverage, so simply excluding the file or #[ignore]-ing the test is not free.
- It reproduces on GitHub's ubuntu-latest runner, not (so far) on the local close gate — d793c52 passed its own tarpaulin close gate locally. Expect to iterate on CI.

RELATED
- bl-9253 — the 0.0.1 release drive, blocked on this AND on bl-53e3.
- bl-53e3 — 'release-plz release' aborts on agent-eval publish-consistency. Independent of this; both must land before 0.0.1 can publish.