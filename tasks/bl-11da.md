+++
title = "tarpaulin maps a dependency's src/state.rs line numbers onto ours, so a static declaration in our state.rs reads as uncovered whenever it lands on a line the dependency has code on; bl-32d0 dodged it by shifting a doc line — configure the exclusion instead"
created = 1790734443
updated = 1791438572
claimant = "Mystical-11da"
priority = 4
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Seen in bl-32d0: tarpaulin reported the 'static WORKED' declaration in src/state.rs uncovered; the cause is a dependency crate that also ships a src/state.rs, whose line map tarpaulin merges with ours. The fix landed was to trim a doc line so the static sits on a line the other file has no code on — fragile, any edit above it brings the phantom back. Do it properly: tarpaulin.toml exclude-files for the registry path (or the crate), or the --exclude-files glob yog uses if it has one; verify with a deliberate edit that moves the static; drop the papercut entry once it holds.

---

Stopped: the configured fix cannot work, and the working fix is materially larger than this ball.

Measured (coverage-probe runs on the builder, tests filtered to state::): with the static WORKED moved from line 224 to 225 by one doc line, src/state.rs reports 225 uncovered; at 224 it does not. Adding exclude-files "*/.cargo/registry/*" changes nothing - 225 is still reported.

Why exclude-files cannot help: every coverage run is the PTRACE engine, not llvm. The Makefile's --engine llvm is dead: tarpaulin 0.35.2 Config::merge never copies engine from the CLI when a tarpaulin.toml exists, and the file's default engine on Linux is Ptrace (the logs show process_handling::linux 'Launching test'). The ptrace path (test_loader.rs get_addresses_from_program) builds each DWARF line row's path as project root + the row's directory entry + file name, and opens the line program with comp_dir = None. A dependency row whose file is the relative 'src/state.rs' with directory index 0 (the comp dir, here atspi-common's registry root) therefore resolves to the project root's own src/state.rs - literally our path. No glob can tell the two apart; excluding that path excludes our file.

What does fix it: engine = "Llvm" in tarpaulin.toml (the engine the Makefile already says it uses). Measured: state.rs 49/49 with the shifted static, run 179s vs 417s under ptrace, and none of the ptrace run's two test failures (dht frontier silent-router, offframe window real-engine) recurred. But llvm counts untaken branches ptrace never saw: 99.82%, 16 genuinely uncovered lines across 15 files (mostly let-else returns/continues: channel/leaf.rs 149, dht/transport.rs 67, seat/enroll/tests.rs 55, test_support/corpus/editions.rs 66, test_support/engine.rs 126, test_support/roving.rs 184 205, ui/act.rs 84, ui/config/edit.rs 63, ui/config.rs 106, ui/keys.rs 168, ui/model/absorb.rs 101, ui/model/fleet.rs 146, ui/model/start.rs 181, ui/queue.rs 114, ui/roster/engine.rs 90). Switching engines means covering those first. yog carries the same dead --engine llvm flag.
