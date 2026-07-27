+++
title = "CI on main is RED: cargo-tarpaulin segfaults somewhere in prompt::tool::builtin::bash::tests (moves between tests)"
created = 1785124427
updated = 1785125073
claimant = "Fathom"
priority = 9
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
ROOT-CAUSED AND FIXED (Fathom, 2026-07-26). Commit 346971f on work/bl-6f1b.

ROOT CAUSE — the Makefile's '--engine llvm' never took effect. cargo-tarpaulin 0.35.2 builds its run configs from tarpaulin.toml ([default]) and Config::merge (src/config/mod.rs:622) never carries 'engine' from the CLI backup config into file-derived configs; the file config's engine defaults to the platform default, which is Ptrace on Linux (TraceEngine derive at src/config/types.rs:43). So since tarpaulin.toml landed (e0609be, 2026-04-21), EVERY run — local gate and CI, green and red — ran the ptrace engine. The engine() fallback is silent here because no fallback fires: the config's engine simply IS Ptrace.

Deterministic reproduction (no stochastic segfault chase needed):
- old tree: cargo tarpaulin --engine llvm --print-rust-flags → '-Clink-dead-code' (ptrace flag set) — the CLI flag is provably discarded;
- fixed tree: same command → '-Cinstrument-coverage' (llvm).
- CI logs concur: both green run 30216420512 and red run 30216944990 show 'INFO cargo_tarpaulin::process_handling::linux: Launching test' (the ptrace launch path; the llvm path logs no such line and statemachine::instrumented instead).

SEGFAULT MECHANISM (ptrace engine): std Command::spawn with a pre_exec closure forks; tarpaulin sets PTRACE_O_TRACEFORK|TRACEVFORK so the shell child is auto-attached, but the PTRACE_EVENT_VFORK handler (statemachine/linux.rs:648) does NOT register the child in pid_map when follow-exec is off (the FORK handler does). The child executes the instrumented pre_exec code in the shared address space; when it traps on a breakpoint, collect_coverage cannot attribute the pid ('Failed to find process for pid'), the breakpoint goes unserviced, and the resumed child runs a clobbered instruction stream → real SIGSEGV in the child → tarpaulin aborts with 'A segfault occurred while executing tests'. Timing-sensitive: reliable on warm 2-core runners, ~never on a 16-core box — matching the observed green(cold)/red(warm) split. The green run had 0 'Failed to find process' warnings; each red run had exactly 1.

FIX (landed):
1. tarpaulin.toml [default] gains engine = "Llvm" — the engine's ONE home, with the mechanism documented in the file. Makefile drops the dead --engine llvm flag.
2. The llvm engine counts precisely and exposed 17 lines ptrace had over-counted via coarse debuginfo line tables (verified with a minimal probe crate: executed continue/map_err lines ARE counted by llvm). All honestly covered now: per-op failing-git stub tests (checkpoint x3, verifier gate-branch-list), blank-line adapter test, steering-skip interpret_pending test, run_tool_calls mixed-content test, non-UTF8-name tests (next_seq, collect_inbox_dirs), discover read_dir per-entry errors dissolved into filter_map(Result::ok), bash setpgid / inbox setsid pre_exec closures extracted to named fns tested in-process (child-side counters die at exec — structural), load_skill single-line literal (llvm attribution quirk precedented in that file).
3. Gate: make check green, 100.00% coverage 4839/4839 under the llvm engine, tarpaulin pin unchanged at 0.35.2 (both homes intact).

NOT touched: the 0.35.2 pin (no bump), test semantics (cascade tests unweakened), no CI retry.

The tarpaulin.toml bullet formerly claiming '--test-threads=1 sidesteps the race' was corrected: serial execution never addressed this race (it is fork-child attribution, not test parallelism).