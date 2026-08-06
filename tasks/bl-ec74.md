+++
title = "multi_tool: let the envelope assert parallel execution"
created = 1785996502
updated = 1785996502
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
The agent, not the harness, knows whether its inner invocations collide. bl-a690 (closed unbuilt) proposed a harness-side per-tool read/write table; this is the cheaper design that replaces it — the invocation asserts it.

**Established in the design conversation (2026-08-06):**
- The envelope already commits ONCE for all N inner invocations. `commit_tool` has two call sites (src/prompt/dispatch/tool_step.rs:245, settle.rs:65); nothing under tool_step/multi/ calls it. So `git add -A` sweeping the whole worktree is already the right shape for 'N tools wrote, commit the lot' — no commit rework needed, no per-inner attribution to preserve.
- Tools never invoke git; the harness commits post-hoc. So concurrent tools cannot contend on .git/index.lock.
- No per-tool read/write classification. That was a690's cost and it is not needed.

**User ruling (2026-08-06): `cd` inside a parallel envelope stays LEGAL.** It moves refs/lernie/cwd/<agent-id>, read at spawn time by prompt::tool::spawn::Caller::resolve, so it races every sibling invocation's cwd resolution. That is 'not gonna work out great' but is not against any rule — do NOT decline it, do NOT build detection. Guard rails wait until real usage goes over the edge. Same for two concurrent writers to one path: last-write-wins, the -A sweep commits the survivor, and that is the agent's problem.

**Open decision:** where the assertion lives. Either overload `on_failure: run_all` (no new field, but it currently means independent-w.r.t.-FAILURE and would silently come to also mean independent-w.r.t.-SIDE-EFFECTS — two different claims) or a distinct envelope field defaulting to serial. Leaning distinct-and-explicit: the whole point is the agent saying what it means, and overloading makes it say so by accident.

**Known feasibility risk:** prompt::Deps<'a> (src/prompt/mod.rs:93) holds bare &dyn trait objects with no Sync bound; running inner invocations on threads needs Sync on at least tool_executor and git. Probe the ripple before designing around it.

Also settle: what on_failure abort means under parallel (skip-later is incoherent when all start at once), and update schemas/tools/multi_tool.json, skills/multi_tool/SKILL.md, ARCH 3.3 'Serial inside the envelope, deliberately' (which currently states serial as a standing position).