+++
title = "Auto-push main to origin on every merge (post-commit hook)"
created = 1785124069
updated = 1785124069
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
DECIDED (Tumult, architect): every merge landed on local main must reach origin automatically — origin/main lagging local main by 82 commits for two months is how the release stalled. Mechanism: a post-commit hook in .githooks/ (the repo's core.hooksPath) that, when HEAD is main and origin exists, pushes origin main. Constraints: push failure must WARN, never block or fail the commit (post-commit exit status is ignored by git anyway, but the hook must not hang — use a short timeout or background the push); quiet when offline. Match how .githooks/pre-commit is structured and tested; keep it a few lines. Do NOT touch release-plz.yml or ci.yml — the remote side (CI -> release-plz -> crates.io publish -> superseded-branch pruning) is already complete on main. This hook is the last link: merge locally -> lands on origin -> CI -> release-plz.