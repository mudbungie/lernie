+++
title = "agent naming becomes a first-class fact: --name at prompt/dispatch, stored under the agent; message resolves id-or-unique-name"
created = 1785645958
updated = 1785645958
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Ruled in yog bl-50f3 (cross-repo precedent for this linkage: bl-9391). Operator evidence 2026-08-02: a yog-spawned agent told to message a peer by its display name failed twice — the message tool resolves only agents/* refs (workspace::agent_exists, src/workspace/guard.rs), but the display name every operator and UI surface speaks is yog prose with no machine-readable home. The name is agent identity; agents live in lernie; so the fact lives here.

Feature (the ruled interface — keep it this narrow):
1. `lernie prompt` and `lernie dispatch` grow an optional `--name <name>` (and the dispatch built-in tool an optional `name` input). A name is set at creation and immutable, like the goal.
2. Creation REFUSES a --name equal to a living agent name in the workspace (uniqueness is enforced where the fact lives). Same single-path-component guard as ids; additionally a name that could collide with the id grammar should be considered (a name is two hyphenated words today, ids are timestamp-hex — document the disambiguation rule you pick).
3. Storage: one home, under the agent. RECOMMENDED: a file committed on the dispatch/root commit beside goal.md (e.g. `name`), so the agents/* ref namespace stays the only registry, resolution reads `git show agents/<id>:name`, worktree teardown does not lose it, and retention (ref deletion) recycles names with zero cleanup path. Do NOT add a workspace-root registry file or index — that is the drift-prone shape the yog ruling rejected.
4. `message` (verb + built-in tool) resolves `{agent}`: exact id match first; else a scan of agents/* refs for a unique name match; ambiguous or unknown is a loud error naming the candidates. The other id-taking verbs (`advance`, `stop`, `bundle`, `dispatch` parent addressing) may adopt the same resolution — decide and document; the message path is the mandatory one.
5. Tool description for `message` teaches that `agent` accepts an agent id or a unique display name.
6. ARCHITECTURE.md amendments: §2.11 (addressing needs no registry — now: the ref namespace plus the committed name fact), §3.4 (verb surface), §2.5/§2.8 as touched. TAXONOMY.md: define "name" (display/self-identity discriminator, recyclable) vs "id" (the identifier: branch name, worktree dir, namespace keys — never carries display semantics).

Explicitly out of scope (rejected in the ruling): name derived from id (impossible — the consumer mints via RNG + occupied set and previews the name before the id exists); name as the id (ids must not collide; names lawfully recycle after retention).

Consumer sequencing: yog waits on this landing in a published release (next ball), then bumps its pin and passes --name at fire (yog balls filed under bl-50f3). Nothing here depends on yog; a bare lernie user gets nameable agents for free.