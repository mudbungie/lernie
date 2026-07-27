+++
title = "e2e::advance_cli baton test outruns its 120s evidence-poll bound when the box runs several full suites at once"
created = 1785130071
updated = 1785131039
claimant = "Cotter"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
DELIVERED (Cotter, 2026-07-26). Mechanism landed; **the ball's premise was disproved** and the real defect is filed as **bl-9c8f**.

## The finding: this was never a bound problem

The filer's reading — "a 120s bound was outrun by a slow success" — does not survive the arithmetic it reports. That iteration's whole `cargo test --lib` took **120.97s**, i.e. the entire budget was consumed by this one test while the other ~900 finished inside it. The box was not 20x slow; the chain was **dead**.

Reproduced on the landed tree, unaided, twice in ~17 full `e2e::` runs at load 40-55. Forensics at the stall (undelivered mail, no lock holder, writer probe returned `Busy`) and a 100%-deterministic widening experiment identify it as a **lost wakeup in the ARCH §2.11 deposit-probe-launch protocol**: a driver's last inbox read precedes the deposit, the writer's probe precedes that driver's release, so the writer defers to a holder that has nothing left to look at, and the deposit is stranded until a hand-run `lernie scan`. It is a production defect, not a test artefact. Full proof, reproduction recipe and proposed fix (the *dual* of the deposit rule: whoever releases a lease re-reads the inbox after releasing and launches if mail pends): **bl-9c8f**.

## What landed here

**`src/e2e/poll.rs` — one primitive, bounded by silence rather than by time.** `poll::until(workspace, probe) -> Option<T>` probes until it yields, or until `SILENT_PROBES` (600) consecutive probes at `PROBE_INTERVAL` (100ms) observe **no change anywhere in the workspace tree**. The activity witness is an order-independent sum over (path, length, mtime) of every entry under the workspace, so it moves on any create, append, replace, rename or delete — the whole vocabulary of a live driver's disk activity (delivery commits, transcript entries, streamed `response.json` event lines, worktree churn) — and holds still only when nothing is running. Consequences: an arbitrarily slow box makes the pass path slower and never redder; the residual verdict is genuine silence, which is the hang the bound always existed to catch; and the failure *names the cause* ("nothing is driving it") instead of reporting an expired stopwatch. `until` returns `None` rather than panicking, so the caller keeps its own diagnostics and its mutable borrows (`poll_for_conv_branch_with_diag` drains the prompt's stderr and dumps the ref list after the poll's borrow ends).

Rationale is the repo's own, one level over: ARCH §2.9 already refuses a wall-clock deadline for stop's leader-pgid re-read — *"a retry **count**, not a wall-clock deadline: this race only appears under load, and a deadline measured under load reports the load."* Counting **non-progress** is that argument applied where the work is unbounded but observable.

**Every wall-clock verdict is gone from `src/e2e/`.** Converted, all to the one primitive: `advance_cli::wait_for` and its step-3 terminal-`end` loop; `replay_drive::wait_for` and its 004-entry loop; `stop_common::poll_for_path` and `poll_for_conv_branch_with_diag` (the `deadline` parameter is deleted, so `stop_cli`'s 15s/20s and `stop_children`'s `PATIENCE` are gone with it); `stop_children::poll_until_no_holder`; and the two duplicate process-reap helpers (`stop_children::wait_reap` 30s + `stop_cli::wait_with_timeout` 15s), **subtracted into one** `stop_common::reap`. `PATIENCE` is deleted outright. The only `Duration`s left in `src/e2e/` are the mock-server holds (120s), which are the non-verdict side of the race and out of this class.

Docs: `docs/ARCHITECTURE.md` §9 opens with the rule and its reasoning.

## Evidence

- **Hang-proof.** Widening the §2.11 window (`drain::drain`: hoist `pending()`, sleep 1500ms before the delivery loop) strands the deposit every run. The shipped diagnostic fires in 64s: `"…/messages/003-user.md" never appeared, and /tmp/…/conv went untouched for 60s — nothing is driving it`. Against the old bound the same hang cost 120s and said only `timed out waiting for …`. Widening reverted.
- **Load-proof.** 80 CPU spinners on 16 cores, load **80.6-116.0** every run (bar: 30+): **30 consecutive passes, 0 failures**, of every touched test — 9 tests per run (`advance_cli` x3, `replay_drive` x2, `stop_cli` x2, `stop_children` x2), run against the tree with `main` already merged in. Plus 20/20 of the ball's own test alone at load 43.
- Residual: the `advance_cli` baton test still flakes at roughly 1 full-suite run in 8 under load until **bl-9c8f** lands. That flake is bl-9c8f's, not a bound's, and it is now diagnosed on sight rather than mistaken for machine load.