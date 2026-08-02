+++
title = "gate: alignment"
created = 1785650143
updated = 1785650421
claimant = "context-home"
parent = "bl-b415"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-b415"
on = "claim"
+++
Discharged for bl-b415.

Checked against docs/ARCHITECTURE.md, docs/PRINCIPLES.md, docs/TAXONOMY.md and the shipped code.

- **Code is authority, docs followed it.** `src/prompt/dispatch/assembler/body.rs::compose` selects via `select(worktree, &rules.pinned, …)` then `select(worktree, &rules.order, …)` over the walked file list — an unmatched path is never read. `assemble` appends the transcript tail unconditionally. `compose` with `rules: None` returns empty, so a role absent from `roles:` is transcript-only. §5.1 now states exactly this.
- **No stale phrasing left.** `grep -rn 'sequencing and budget|no exclusion list|everything in a worktree is context' docs/ src/ README.md` returns nothing.
- **TAXONOMY needed no change** — it never asserted the strong form; its LangChain mapping already reads 'select = manifest inclusion' (ARCH §5.2), which the correction vindicates rather than contradicts.
- **No new terms of art coined** (AGENTS.md terminology discipline): the amendment reuses 'inclusion list', 'pinned', 'order', 'transcript tail', 'system slot', all already defined.
- **One real divergence surfaced and left unfixed by ruling:** the compactor is instructed to read `summary/` and work products that its manifest entry does not compose and its empty grant cannot read. Filed as **bl-2c63**; ARCH §2.7 and `compactor_goal`'s doc comment now name it rather than papering over it.