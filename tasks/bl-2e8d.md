+++
title = "Add build-gated release-plz release automation"
created = 1784698905
updated = 1784698905
claimant = "Junctions-lernie"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
lernie had CI only — no release automation at all. This adds the same build-gated release-plz pipeline the other repos use: a release PR is opened/updated on every push to main (queued, never auto-published), the actual release only runs when the CI workflow concludes success on main, and a binaries job in the same run builds the `lernie` binary for x86_64-unknown-linux-gnu and attaches it to the GitHub Release. Version bumps default to patch (0.0.1 -> 0.0.2), with `[minor]`/`[major]` opt-in markers documented in release-plz.toml.

Two new files only: `.github/workflows/release-plz.yml` and `release-plz.toml`. ci.yml is unchanged (already `name: CI`, already triggers on push-to-main and pull_request).

Gates:
- tests: `make ci` passes.
- docs: n/a beyond the inline comments in the two new files, which document the pipeline and the bump markers.
- alignment: CI/release config only; no code, no new terms of art.