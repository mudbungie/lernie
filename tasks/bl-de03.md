+++
title = "Cleanup: retire superseded design-lineage artifacts (LHCS/WDF drafts, ~/dev/harness, harness_stack, lernie-adapters)"
created = 1783465324
updated = 1783466123
claimant = "Sandcastle-de03"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
The v0.4 fold (bl-f739, main 2375d70) supersedes all of these. Operational task — the deletions live outside git/worktree flow; record what was done in the close message.

1. docs/LHCS.md + docs/WDF.md at the lernie repo-root checkout: UNTRACKED May-2026 drafts (LHCS v2.0-draft layered component standard; WDF flat-directory sequence-file format), never committed. Their surviving ideas already live in shipped ARCHITECTURE (adapter layer, tool binary/schema/skill triple, dispatch/skill built-ins, single-writer discipline); their daemon Conductor + UDS/JSON-RPC + sequence-numbered-file ideas were rejected (no-resident-interpreter §6; UI is fs-reads + CLI only §3.5). Preserve copies into the archive tarball (step 2) FIRST, then rm from the working directory. Untracked means there is nothing to deliver via the worktree — the rm at the root checkout is the whole in-repo action.

2. ~/dev/harness (git repo, single commit 23ea6c5, 2026-07-03): fully absorbed — brazen boundary -> v0.6, budgets -> v0.7, WASM sandbox -> v1.1, JSONL-transcript SSOT rejected. Tarball to ~/dev/trash/harness-2026-07.tar.gz (include the LHCS/WDF copies from step 1), optionally leave a tombstone README pointing at lernie docs/ARCHITECTURE.md v0.4, then remove the directory.

3. ~/dev/harness_stack and ~/dev/lernie-adapters (both NOT git repos): earlier fossils of the same lineage (HANGAR/STANDARD docs; adapter.py + rust prototype). Before removing: grep shell config/Makefiles/PATH for live references, and confirm nothing in lernie's install flow points at them. Tarball each into ~/dev/trash, then remove.

Tarball-first is mandatory; nothing is deleted without an archive.