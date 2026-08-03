+++
title = "a multi-tool tool: structured tool calls as arguments, with execution metadata"
created = 1785650728
updated = 1785724430
claimant = "Gudgeon"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Operator design (2026-08-01, codex-comparison follow-up), verbatim: "bash overwhelms my tools because my tool list is tiny. It will expand. I think the simple answer is literally a multi-tool tool that takes other structured tool calls as arguments, as well as some execution metadata (return answers as they come or block on all being complete, abort on any failures, that kinda thing). In many cases, though, yes. Bash will do."

## What
One builtin that accepts a list of structured tool calls (the same shapes the individual tools declare) plus execution metadata: streaming vs block-on-all result delivery, abort-on-first-failure vs run-all. One model round trip fans into N tool executions.

## Verified constraints to design against
- Execution today is serial (src/prompt/dispatch/tool_step.rs:76 drives tool_use blocks one at a time) and every side effect is a commit (ARCH §3.3 commit-per-side-effect) — concurrent mutating calls against one worktree must serialize their commits. Codex's answer is a read/write distinction: parallel-safe tools share a read lock, mutating tools take a write lock (codex-rs/core/src/tools/parallel.rs in openai/codex) — the same split works here (read_file/load_skill/message parallel; bash/patch exclusive).
- "Return answers as they come": a tool result is a transcript entry; incremental delivery means multiple result entries for one call envelope — verify what the wire (brazen request encoding) and the assembler tolerate before promising streaming; block-on-all is the safe first rung.
- Recursion: whether multi-tool may contain multi-tool — decide and state it (a depth-1 refusal is fine, say so in the tool description).