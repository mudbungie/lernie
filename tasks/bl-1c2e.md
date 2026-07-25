+++
title = "Coverage measurement flake: intermittent 1-line miss on unrelated diffs (llvm region attribution)"
created = 1784699263
updated = 1784956625
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
which passed a **20 ms wall-clock deadline**. A dozen tarpaulin runs from parallel agents
were live on the box during the repro (load average 25-80 on 16 cores). Under that load the
first `try_acquire` can return *after* the 20 ms deadline has already expired: the loop
takes the give-up arm on its first pass, the sleep never runs, and the gate reports one
uncovered line on a diff that touched nothing. An immediate retry on an idle moment greens.
The earlier sighting recorded in this ball — "src/prompt/tests/advance.rs at 33/34 once"
during bl-06d5 close prep — is the same line, the same cause.

Class statement: **a line whose execution is gated by a wall-clock deadline has
load-dependent coverage.** The 100% floor is only a property of the code if every line's
reachability is too.

## Fix (landed on this ball's branch)

- `free_within` now takes a retry COUNT and shares `PROBE_RETRIES`
  (`src/prompt/tests/exit_launch.rs`) with the two sibling executor-lock probes in
  `exit_launch`/`exit_race`, which already had the count shape — one budget for the
  fork->exec fd-inheritance window instead of two spellings of it. The sleep moved to the
  head of the loop (`if attempt > 0`), so a lease held for the whole budget sleeps
  `retries - 1` times on any machine; the give-up test passes 2 attempts and reaches both
  arms by construction, with no clock in the path.
- README ("pre-commit gate" section) states the rule: a test's retry budget is a count of
  attempts, never a wall-clock deadline, and why.

## Two things that are NOT this, both filed separately

- **bl-7a3f** — `prompt::tool::tests::errors::spawn_retries_past_transient_etxtbsy` fails
  outright under the same load: it holds the tool binary's write fd for 40 ms and needs
  that hold to end inside production's 200 ms `ETXTBSY_RETRY_BUDGET`. Two wall clocks
  racing; a 5x stretch is enough. Same disease, harder failure, and it fails the close gate
  for whoever is committing at the time. Proposed fix in the ball: inject the budget the
  way `SpawnTool::with_deadline` already injects the SIGTERM grace.
- **bl-7e33** — the `src/prompt/budget/mod.rs:127` sighting that opened this ball has no
  timing component, so the fix above does not explain it. Best remaining candidate, read
  out of the tarpaulin 0.35.2 and llvm-profparser sources: e2e tests spawn instrumented
  `lernie` subprocesses that inherit `LLVM_PROFILE_FILE`, so one test binary's profraw pool
  holds records from two different binaries; the merge is keyed by function NAME
  (`find_record_by_name_mut`), a same-name record of differing counter length is dropped
  silently, and the base record is whichever profraw the directory walk parsed first — an
  order that changes run to run because the filenames carry the pid.

## Ruled out, with evidence

- **The §2.9 stop cascade is not signalling the test harness.** `lernie prompt` makes
  itself a process-group leader in the `become_pgid_leader` prelude before it ever takes
  the inbox lock; `ps -eo pid,ppid,pgid,sid` during the e2e stop tests shows the spawned
  `lernie prompt` running with `pgid == its own pid` while the test binary sits in another
  group, so `kill(-pgid, ...)` structurally cannot reach tarpaulin.
- **The runs that died mid-suite (rc 143 / rc 137) were my own tooling**, not the box and
  not the repo: they line up with the 10-minute cap on this agent's tool calls, which
  terminates with SIGTERM (128+15 = 143). Discount them entirely. The measurement that
  matters — run 7 — completed normally and reported the one-line miss on its own.
- **A stale `bz` blocks measurement outright** (not a flake, a hard stop): `bz 0.0.4`
  installed against `brazen = "=0.0.3"` fails the §4.4 load-time guard and takes all five
  e2e tests with it. Worked around here with a pinned `bz` on PATH; bl-f01f puts
  `make install` (which re-installs `bz` at the pin) into the close gate.

## Residual, deliberately not touched

`bash/mod.rs` and `subprocess.rs` carry production SIGTERM-grace loops of the same
deadline shape, whose inner poll sleep is only reached while the grace has not expired.
Their deadlines are seconds, not 20 ms, and wall-clock is the correct semantics for a real
grace period — so they are noted, not rewritten. If one of them ever shows up as a
one-line miss, the fix is to inject the deadline (bl-7a3f's shape), not to widen it.