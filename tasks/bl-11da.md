+++
title = "coverage runs on ptrace, not llvm: tarpaulin.toml silently overrides the Makefile's --engine llvm; set the engine in the toml, delete the dead flag, and cover the 16 lines llvm sees that ptrace never did"
created = 1790734443
updated = 1791438597
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Re-scoped 2026-10-08 after the first attempt measured the original fix and found it cannot work. The original symptom (bl-32d0): tarpaulin reports a phantom uncovered line in `src/state.rs` whenever a non-code line of ours lands on a line number a dependency's same-named `src/state.rs` (atspi-common) has code on; bl-32d0 dodged it by shifting a doc line.

## What was measured on the builder

- With `static WORKED` moved from line 224 to 225 by one doc line, `src/state.rs` 225 reports uncovered; at 224 it does not. Phantom reproduced.
- Adding `exclude-files = ["*/.cargo/registry/*"]` changes nothing. The ptrace loader builds each DWARF row's path as project root + directory entry + file name with no comp_dir, so the dependency's relative `src/state.rs` resolves to OUR path. No glob can separate them; excluding that path excludes our file.
- **Every coverage run has been the ptrace engine.** The Makefile passes `--engine llvm`, but tarpaulin 0.35.2's `Config::merge` never copies `engine` from the command line once a `tarpaulin.toml` exists, and the file's Linux default is ptrace. The builder logs show `process_handling::linux: Launching test`.
- With `engine = "Llvm"` in `tarpaulin.toml`: the phantom is gone (`src/state.rs` 49/49 with the shifted static), the run takes 179 s instead of 417 s, and the two timing-bet failures bl-7b83 chases did not recur. But llvm counts untaken branches ptrace never saw: 99.82%, 16 uncovered lines in 15 files, mostly `let … else { return/continue }` early exits — channel/leaf.rs 149, dht/transport.rs 67, seat/enroll/tests.rs 55, test_support/corpus/editions.rs 66, test_support/engine.rs 126, test_support/roving.rs 184 and 205, ui/act.rs 84, ui/config/edit.rs 63, ui/config.rs 106, ui/keys.rs 168, ui/model/absorb.rs 101, ui/model/fleet.rs 146, ui/model/start.rs 181, ui/queue.rs 114, ui/roster/engine.rs 90. Line numbers are as of 2026-10-08 and will have drifted.

## The work

1. `engine = "Llvm"` in `tarpaulin.toml`, with the reason beside it (the file's header already says a second setting is a decision whose reason belongs there). Delete `--engine llvm` from the Makefile's coverage target: the engine has one home, and the flag was a second spelling that lost silently.
2. Cover every line llvm reports. Each `let … else` early exit is either reachable — write the test — or unreachable by construction — restructure so the branch does not exist (a special case is usually a missing reframe). Do not add `ignore-panics`-style exclusions for them.
3. Prove it both ways on the builder with `bl-remote-run coverage`: 100% under llvm with the static shifted onto a line the foreign file has code at. Mention the measurement in the DESIGN or README sentence that describes the coverage floor if one names the engine.
4. Drop the two tarpaulin entries in the ops papercut log once it holds.

yog carries the same dead flag; that is yog's ball, not this one.