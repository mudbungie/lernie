+++
title = "Close gate flakes under parallel-agent load: spawn_retries_past_transient_etxtbsy races two wall clocks"
created = 1784956192
updated = 1784959249
claimant = "Nettle"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Found while reproducing bl-1c2e (which fixed a *different* load flake, in the
advance.rs lease probe). This one is a hard test failure, not a coverage miss,
and it fails the whole close gate for whoever is committing at the time.

## Repro

`make coverage` on a box with a dozen parallel agent tarpaulin runs live
(load average ~40 on 16 cores):

    thread 'prompt::tool::tests::errors::spawn_retries_past_transient_etxtbsy'
    panicked at src/prompt/tool/tests/errors.rs:169:10:
    retry budget covers the ~40ms hold: Spawn { name: "blocked", source: Os {
    code: 26, kind: ExecutableFileBusy, message: "Text file busy" } }

## Why

The test holds the tool binary's write fd open in a sibling thread for a
`thread::sleep(Duration::from_millis(40))`, and relies on that hold ending
inside production's `ETXTBSY_RETRY_BUDGET` (`src/prompt/tool/subprocess.rs:34`,
200 ms wall-clock). Two independent wall clocks under contention: a 40 ms sleep
only has to stretch 5x — routine when the machine is swapping — for the
production budget to expire first and the spawn to surface `ExecError::Spawn`.

Its sibling `spawn_surfaces_etxtbsy_after_budget_exhausted` has the mirror-image
exposure (a hold that must OUTLAST the same 200 ms budget).

## Proposed fix — inject the budget, as the SIGTERM grace already is

`SpawnTool` already parameterizes its other wall-clock, the SIGTERM-to-SIGKILL
grace, with `with_deadline(...)` for exactly this reason. Give the ETXTBSY
retry budget the same treatment and the ratio stops mattering: the
retry-succeeds test sets a budget of seconds against its 40 ms hold, and the
budget-exhausted test sets a near-zero budget against any hold. Both then hold
on any machine, and neither hides a real regression. Production keeps its
200 ms default — the constant stays the single source of the shipped value.

Do not just enlarge the hold or the budget; that only moves the ratio.