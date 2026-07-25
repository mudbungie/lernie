+++
title = "Coverage measurement flake: intermittent 1-line miss on unrelated diffs (llvm region attribution)"
created = 1784699263
updated = 1784955888
claimant = "Cinder"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Root cause — REPRODUCED 2026-07-24, and it is not llvm region attribution

`make coverage` run 8x back-to-back on pristine main (worktree bl-1c2e, no source
change between runs): runs 1-5 green at 4351/4351, run 7 RED at 4350/4351 with
exactly one uncovered line — `src/prompt/tests/advance.rs:116`, the retry
`thread::sleep` inside the test helper `free_within`:

    pub(super) fn free_within(ws: &Path, agent: &str, deadline: Duration) -> bool {
        let end = Instant::now() + deadline;
        loop {
            if try_acquire(&inbox_dir(ws, agent)).unwrap().is_some() { return true; }
            if Instant::now() >= end { return false; }
            std::thread::sleep(Duration::from_millis(5));   // <-- line 116
        }
    }

The only test that can execute line 116 is `free_within_gives_up_on_a_genuinely_held_lease`,
which passed a **20 ms wall-clock deadline**. Seven tarpaulin runs from parallel agents
were live on the box during the repro. Under that load the first `try_acquire` can return
*after* the 20 ms deadline has already expired: the loop takes the give-up arm on its first
pass, the sleep never runs, and the gate reports one uncovered line on a diff that touched
nothing. An immediate retry on an idle moment greens. The earlier sighting recorded in this
ball — "src/prompt/tests/advance.rs at 33/34 once" during bl-06d5 close prep — is the same
line, the same cause.

Class statement: **a line whose execution is gated by a wall-clock deadline has
load-dependent coverage.** The 100% floor is only a property of the code if every line's
reachability is too.

## Fix (this ball's branch)

- `free_within` now takes a retry COUNT and shares `PROBE_RETRIES`
  (`src/prompt/tests/exit_launch.rs`) with the two sibling executor-lock probes in
  `exit_launch`/`exit_race`, which already had the count shape — one budget for the
  fork->exec fd-inheritance window instead of two spellings of it. The sleep moved to the
  head of the loop (`if attempt > 0`), so a lease held for the whole budget sleeps
  `retries - 1` times on any machine; the give-up test passes 2 attempts and covers both
  arms by construction, with no clock involved.
- README ("pre-commit gate" section) states the rule: a test's retry budget is a count of
  attempts, never a wall-clock deadline, and why.

Validated: `cargo fmt --check`, `clippy -D warnings`, and `cargo tarpaulin --fail-under 100`
green on the merged state.

## Observed alongside, environmental — not a lernie defect

- Runs 6 and 8 of the repro died outright: rc 143 (SIGTERM) mid-unit-test, rc 137 (SIGKILL)
  inside an unrelated `agent-eval` test binary. NOT the §2.9 stop cascade: `lernie prompt`
  makes itself a process-group leader in the `become_pgid_leader` prelude before it ever
  takes the inbox lock, verified live with `ps -eo pid,ppid,pgid,sid` (the spawned
  `lernie prompt` ran with `pgid == its own pid` while the test binary sat in another
  group), so `kill(-pgid, ...)` structurally cannot reach the test harness. The box had
  systemd-oomd active and 12 GiB of swap in use under seven concurrent tarpaulin runs.
  Parallel coverage is memory-bound; that is the cost to watch, not signal safety.
- The tree cannot be measured at all against a locally installed `bz` newer than the
  Cargo.toml brazen pin: `bz 0.0.4` vs `brazen = "=0.0.3"` fails the §4.4 load-time guard
  and takes all five e2e tests with it. Worked around here with a pinned `bz` on PATH;
  bl-f01f puts `make install` (which re-installs `bz` at the pin) into the close gate.

## Remaining lead — split to bl-7e33

The `src/prompt/budget/mod.rs:127` sighting that opened this ball has no timing component,
so the fix above does not explain it. The best remaining candidate, read out of the
tarpaulin 0.35.2 and llvm-profparser sources: e2e tests spawn instrumented `lernie`
subprocesses that inherit `LLVM_PROFILE_FILE`, so one test binary's profraw pool holds
records from two different binaries; the merge is keyed by function NAME
(`find_record_by_name_mut`), a same-name record of differing counter length is dropped
silently, and the base record is whichever profraw the directory walk parsed first — an
order that changes run to run because the filenames carry the pid. Details, the
verification recipe, and the candidate postures are in bl-7e33.