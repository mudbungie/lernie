+++
title = "tarpaulin maps a dependency's src/state.rs line numbers onto ours, so a static declaration in our state.rs reads as uncovered whenever it lands on a line the dependency has code on; bl-32d0 dodged it by shifting a doc line — configure the exclusion instead"
created = 1790734443
updated = 1791438044
claimant = "Mystical-11da"
priority = 4
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Seen in bl-32d0: tarpaulin reported the 'static WORKED' declaration in src/state.rs uncovered; the cause is a dependency crate that also ships a src/state.rs, whose line map tarpaulin merges with ours. The fix landed was to trim a doc line so the static sits on a line the other file has no code on — fragile, any edit above it brings the phantom back. Do it properly: tarpaulin.toml exclude-files for the registry path (or the crate), or the --exclude-files glob yog uses if it has one; verify with a deliberate edit that moves the static; drop the papercut entry once it holds.