+++
title = "an apply_patch-class edit tool: atomic multi-file patch with fuzzy context matching"
created = 1785649530
updated = 1785649863
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Source: codex comparison survey 2026-08-01 (openai/codex @ 2b5bdcf). VERIFY premises against the tree before editing.

## Verified gap
The shipped toolset is bash, cd, dispatch, load_skill, message, read_file (src/prompt/tool/builtin/) — every file edit rides bash (sed, heredocs, python -c). A structured patch removes shell quoting/escaping from the edit path and makes failures typed. Whether model familiarity improves solve rate is a hypothesis for agent-eval, not an acceptance premise.

## What codex does
codex-rs/apply-patch:
- One envelope carries add/delete/update/rename across multiple files atomically — one tool call, one round trip.
- Fuzzy context ladder (seek_sequence.rs): exact match, then ignore-trailing-whitespace, then ignore-edge-whitespace, then Unicode normalization (smart quotes, NBSP, em-dash → ASCII) — "mirrors the fuzzy behaviour of `git apply`". A string-replace tool fails hard on one smart quote; this recovers without a retry.
- "@@ <enclosing symbol>" disambiguation for repeated blocks — a bounded recipe for uniqueness instead of "paste more context".

## Fit
Commit-per-side-effect (ARCH §3.3) is untouched: the patch application is the side effect and the per-call `git add -A` commit rides as for bash. Decide builtin vs. external `lernie-tool-*` binary — builtin argues consistency with read_file; external argues extraction discipline (§ layer law: mechanism yes, but the toolset is deliberately thin — this ball must make the case or close won't-fix). Grammar can be codex's (documented, model-familiar) or unified diff; pick what models are trained on, not what is elegant to parse.

## Safety acceptance added by bl-e249

- Refuse a patch against stale source state (version/read guard) rather than overwrite unseen changes.
- Require a unique target after the documented matching ladder; ambiguity is a loud error, never a guessed edit.
- Preserve the pre/post diff and exact failure reason in the ordinary tool record.
- Benchmark retry/solve impact before adding further edit verbs.
