+++
title = "lernie's brazen pin is =0.0.4 while crates.io has 0.0.5: releasing 0.0.3 as-is locks the skew into yog's dependency graph"
created = 1785459718
updated = 1785459718
priority = 5
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-f3bc"
on = "close"
+++
crates.io: brazen 0.0.5 (2026-07-29), lernie latest 0.0.2 declaring `brazen =0.0.4`. Cargo.toml:47 on main still reads `brazen = "=0.0.4"`.

yog's ball bl-56bd pins `brazen =0.0.5` + `lernie =0.0.3` under a parity requirement, quoted verbatim: "lernie 0.0.3 must declare \`brazen =0.0.5\`, the same exact version yog pins, so exactly ONE brazen resolves."

Release PR #3 (chore: release v0.0.3) as it stands would publish a 0.0.3 pinning brazen 0.0.4 — cementing the skew. So the pin must move to =0.0.5 and land on main BEFORE that PR merges.

Deliverable: Cargo.toml `brazen = "=0.0.5"`, refreshed pin comments naming 0.0.4 verbatim, `cargo update -p brazen --precise 0.0.5` lockfile, and any lernie-side fix for brazen 0.0.4→0.0.5 API drift. ARCH §4.4: the pin has one home (the dependency line); src/prompt/pin.rs and the Makefile's BRAZEN_PIN both derive from it.