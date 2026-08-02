+++
title = "bound tool output committed to the transcript: head+tail cap with an honest truncation marker"
created = 1785649530
updated = 1785649843
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-ffc5"
on = "claim"
+++
Source: codex comparison survey 2026-08-01 (openai/codex @ 2b5bdcf). VERIFY premises against the tree before editing.

## Verified gap
The bash tool captures and returns stdout/stderr UNBOUNDED (src/prompt/tool/builtin/bash/mod.rs:151-155 — full captured bytes written through). read_file caps at 1MiB by declining, and its module doc names the missing piece: "The auto-dispatch shim that turns oversized output into a [parsing subagent]" — deferred (ARCH §3.3/§11), unbuilt. So one `cat big.log` or a chatty build commits megabytes into the transcript, poisoning every subsequent request on that branch (assembly is append-only; the damage is permanent until compaction deletes it).

## What codex does (the floor, not the shim)
- Middle-out truncation with an honest marker: "Warning: truncated output (original token count: N)\nTotal output lines: M" (codex-rs/utils/output-truncation/src/lib.rs) — the model is TOLD what it lost and how much, so it can re-run with a filter.
- HeadTailBuffer: 50% head + 50% tail of a 1MiB cap, middle dropped with "... N bytes omitted ..." (codex-rs/core/src/unified_exec/head_tail_buffer.rs) — keeps the command banner AND the failure tail, the two parts that matter.

## The lernie-shaped design
The full bytes ALREADY have a home: steps/<agent>/<NNN>/tools/<tool-id>/output.json (ARCH §2.3 diagnostic layer, outside every worktree). So the transcript entry becomes a bounded projection and nothing is lost — transcript = model-facing, steps = audit; single source of truth holds. The marker must state original byte/line counts and where the full record lives. Byte counts only — lernie has no tokenizer; never fabricate token counts.

Cap value and head/tail split are config (workflow.yaml or manifest territory — decide; policy must stay severable). This is the floor; the specced auto-dispatch shim remains a separate, later ball.