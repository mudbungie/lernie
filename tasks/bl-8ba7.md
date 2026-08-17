+++
title = "brazen 0.0.6 is published while the pin reads =0.0.5: consume it so lernie publishes ahead of its downstream"
created = 1786938066
updated = 1786938066
priority = 5
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
brazen 0.0.6 landed on crates.io carrying a default oauth2 provider row (brazen bl-77fa, operator-ruled). lernie pins the adapter exactly, so a published lernie declaring `brazen =0.0.5` beside a downstream pinning `=0.0.6` resolves TWO brazen crates in one graph — the skew bl-e4ef and bl-143e each paid for once already (USER_STORIES G6, third recurrence).

Operator direction: lernie consumes and publishes FIRST, the downstream follows.

Deliverable: the pin's one home — the `brazen = "=<pin>"` line in the workspace Cargo.toml (ARCH §4.4) — moves to `=0.0.6`, `cargo update -p brazen --precise 0.0.6` refreshes the lockfile, and every README version the pin test holds equal (`src/prompt/tests/pin.rs::every_brazen_version_the_readme_spells_is_the_pin`) is re-spelled. Makefile BRAZEN_PIN, the load-time guard and `lernie --version` all derive from that one line and need no edit. Any lernie-side fix for 0.0.5->0.0.6 surface drift rides here.