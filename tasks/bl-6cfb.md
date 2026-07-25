+++
title = "Switch crates.io publishing to trusted publishing (OIDC), drop CARGO_REGISTRY_TOKEN"
created = 1784959015
updated = 1784959015
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
The trusted publisher for crate `lernie` is registered on crates.io (owner mudbungie, repo lernie, workflow release-plz.yml, no environment). Move .github/workflows/release-plz.yml off the stored-secret model:

- add job-level `permissions: {contents: write, id-token: write}` to release-plz-release (job-level replaces the workflow default, so contents: write must be restated for tag/Release push)
- delete `CARGO_REGISTRY_TOKEN` from both the release-pr and release jobs; release-plz does the OIDC exchange itself when id-token is granted
- rewrite the 'One-time repo setup' header comment: CARGO_REGISTRY_TOKEN is no longer required; describe the trusted-publisher config instead, and define 'trusted publishing' inline where introduced

RELEASE_PLZ_TOKEN and the 'Allow GitHub Actions to create and approve pull requests' setting are unaffected.