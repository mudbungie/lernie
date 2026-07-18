+++
title = "Decouple the GUI: extract lernie-ui-egui to its own repo, strip crate + tethers from lernie"
created = 1784336691
updated = 1784336692
claimant = "Flute"
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
lernie ships as a composable component; the egui frontend interferes with that delivery and is growing toward direct deps on both lernie and balls (which lernie must not take). Extraction plan:

1. New sibling repo ~/dev/lernie-ui-egui seeded via git subtree split (history preserved) from crates/lernie-ui-egui. Standalone package: own Makefile, README, .githooks (300-line + 100% coverage pre-commit), tarpaulin config, bl substrate founded.
2. lernie side (this ball): remove crates/lernie-ui-egui; drop workspace member; Makefile PATH_BINARIES + `ui` target; tests/install.rs bin assertion; tests/prompt_retry.rs doc-comment cross-ref; README GUI sections point at the new repo.

The crate already has no Cargo dep on lernie (drives the CLI, reads git directly) — this is a repo-boundary move, no code seam to cut.