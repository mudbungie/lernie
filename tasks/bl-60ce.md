+++
title = "0.0.9 release prep: promote the [Unreleased] changelog section and merge the release PR"
created = 1786757686
updated = 1786757686
claimant = "Reneges"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
The tool-injection seam (bl-9001) is on main under [Unreleased]. Promote the changelog to 0.0.9 per the release flow, merge, then merge the release-plz PR so crates.io carries 0.0.9 — the yog consumer bumps its exact pin next (yog step-7 tool hosts).