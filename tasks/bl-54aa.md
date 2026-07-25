+++
title = "lernie config: declined/failed authoring pass leaks the transient checkout and wedges the verb"
created = 1784955704
updated = 1784955720
claimant = "Pintle"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 3. An authoring pass that changes nothing is DOCUMENTED as declined — but the decline path (git commit exit 1) skips teardown: .config-author remains, and every later lernie config fails with 'worktree add … already exists'. A failed --from also leaves a dangling refs/heads/config/<name>. Fix: teardown on every exit path (decline, git failure, editor failure) — RAII/defer-style; the decline itself must exit 0 or a distinct clean 'nothing changed' outcome (decide against ARCH §2.2's authoring contract), with an actionable message, not raw git plumbing. Also: on startup, a leftover .config-author from a crashed pass should be handled per the repo's crash-debris philosophy (§2.11 'the next touch heals') — either auto-remove if safe (git worktree prune semantics) or refuse actionably. Tests: decline→re-author succeeds; failed --from leaves no ref.