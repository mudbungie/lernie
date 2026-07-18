+++
title = "Decouple the GUI: extract lernie-ui-egui to its own repo, strip crate + tethers from lernie"
created = 1784336691
updated = 1784336872
claimant = "Flute"
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
lernie ships as a composable component; the egui frontend interferes with that delivery and is growing toward direct deps on both lernie and balls (which lernie must not take). Done:

1. New sibling repo ~/dev/lernie-ui-egui seeded via git subtree split — 23 commits of crate history preserved. Standalone package: own Makefile, README (ported from lernie's UI section), .githooks pre-commit (no-direct-main + 300-line + 100% coverage), tarpaulin.toml (pin 0.35.2, serial tests), LICENSE, Cargo.lock. bl substrate founded; scaffold delivered as bl-69b0; make check green (100%, 747/747). Standalone clippy surfaced two collapsible_if lints (fixed via let-chains) and a dead LogEntry.parent_count field (deleted, %P dropped from the log format). Parallel-test ETXTBSY spawn race filed there as bl-a86c.
2. This ball (lernie side): crates/lernie-ui-egui removed; workspace member dropped; Makefile PATH_BINARIES/ui target/.PHONY; tests/install.rs bin assertion; tests/prompt_retry.rs cross-ref comment; README UI section replaced with a pointer, install/uninstall mentions corrected to lernie+agent-eval.
3. Incidental: lcov.info was a tracked generated artifact (stale egui paths inside) — untracked + gitignored. cargo fmt swept ~26 src/prompt/** files of pre-existing drift; main failed `make check` (fmt-check) before this ball.

make check green post-removal: fmt, clippy, 100.00% coverage 3973/3973.