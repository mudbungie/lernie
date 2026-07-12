+++
title = "Design: workspace substrate — one repo, config branches, agents as worktrees; merge-back eliminated"
created = 1783819509
updated = 1783820283
claimant = "edits-b09c"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["design-2026-07"]

[[blockers]]
id = "bl-08df"
on = "claim"

[[blockers]]
id = "bl-c904"
on = "claim"

[[blockers]]
id = "bl-ac8b"
on = "close"

[[blockers]]
id = "bl-7467"
on = "close"

[[blockers]]
id = "bl-6848"
on = "close"
+++
Settled in design discussion 2026-07-09..11. Deliverable: living rewrite of ARCH §2.2–§2.6 and §7 (touches §9.2 archival). Written in the bl-08df ladder, on the bl-c904 context model — hence the needs edges.

Settled: ONE REPO PER WORKSPACE, worktrees as the cloning mechanism — explicit rejection of per-agent blast radius; interoperability (inter-agent messaging, forking from any agent's ref) is the point. Isolation relocates to the workspace boundary: separate repos, no cross-workspace mechanism, structural no-cross-talk (e.g. different employers). CONFIG BRANCHES, descriptively named, no main; branch advancement invariant replaces 'Nothing writes to main directly': config branches advance only by user config edits, agent branches only by their executor. STARTING AN AGENT = branch + worktree off a ref — default the chosen config branch's head; any ref legal (fresh = config head; fork-back-in = historical ref; parallel fan = N forks of one ref). The §2.2 frozen-copy bootstrap (cp -r from harness-root profile) DELETES — fork IS the freeze; the agents/<profile>/ pool dissolves into config branches. MERGE-BACK ELIMINATED: child results return as inbox messages ('complicated, doesn't deliver anything useful, breaks caching badly'); merge=ours + alignment-commit + .gitattributes machinery of §2.6 deletes wholesale; await(handle) degenerates to wait-for-message-from-child with derived-status fallback (refs/response.json framing) for children that die silent. MERGE IS RESERVED FOR COMPACTION: compactor forks at checkout C, rewrites only files <= C; live agent only appends new sequence files (bl-c904 immutability); write sets disjoint by construction, so replaying since-checkout commits on top is conflict-free — surgical compaction while the agent keeps stepping. REPROMPT = MESSAGE: user deposit -> delivery commit -> step loop resumes on the same branch; exchange despecialized to a span; the §2.3 roots-merge self-contradiction dissolves (neither arm survives — no branch-per-exchange exists). Inbox deposits are create-only new files, never append/edit (an append is two messages) — lock-free by construction. The §2.11 no-live-executor REFUSAL INVERTS: a deposit into a quiescent agent is the reprompt path — it must succeed and cause a driver to run.

Open questions to resolve in-doc: (1) work-product transfer vehicle — lean: child's result message names its terminal ref; delivery applies diff fork-point..tip (work products only, not transcript) as a commit at the parent's sequence tail — append-only, ordered, declinable. (2) Driver exclusivity post-flock — git's one-branch-one-worktree rule covers most; residual is two processes re-entering one worktree. (3) Archival/replay unit (§9.2): tarball-per-run dies; git bundle per agent branch + steps/<id>/ + inbox/<id>/ slices, vs workspace tarball; retention/GC becomes branch deletion + object pruning in a long-lived repo. (4) Ref namespace for config vs agent branches. (5) Whether goal/soul are sequence item zero (bl-c904 (d)).