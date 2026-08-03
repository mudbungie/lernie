---
name: multi_tool
description: "Run several tool invocations in one round trip. Give it a list of `{name, input}` entries — the same shapes the individual tools take — and they run one after another, strictly in your order, each seeing the side effects of the ones before it. Every entry's result comes back together in this one tool result, labelled `[k/N] <name>: ok|failed|declined|skipped`; nothing streams back early. By default a failed entry aborts the rest (they report as skipped); pass `on_failure: \"run_all\"` to run every entry regardless. Each inner tool is checked against your toolset exactly as if you had invoked it top-level, and `multi_tool` may not contain itself (depth 1). Reach for it when you already know the next several tool invocations and none of them needs a look at the previous result first."
---

# multi_tool

Fans one model round trip into N tool executions (ARCH §3.3 *The
multi-tool*). Each inner invocation is treated exactly like a top-level
one — same toolset check, same output bounding, same diagnostic record —
the only difference is that all the results return together.

## Input

```json
{
  "invocations": [
    { "name": "read_file", "input": { "path": "Cargo.toml" } },
    { "name": "bash", "input": { "command": "ls src" } }
  ],
  "on_failure": "abort"
}
```

- **`invocations`** — the inner tool invocations, in order. `name` is a
  tool from your own toolset; `input` is that tool's own input object
  (omitted means `{}`).
- **`on_failure`** — optional. `"abort"` (default): a failed entry ends
  the envelope, and every later entry is reported `skipped` without
  running. `"run_all"`: every entry runs regardless.

## Execution order and delivery

- **Serial, in your order.** Entries never run in parallel. A later
  entry sees everything an earlier one did: a file `bash` wrote, a
  directory `cd` moved you to.
- **Block-on-all.** The result arrives once, when the last entry has
  resolved. There is no incremental delivery — if you need to read one
  result before choosing the next invocation, issue them as separate
  top-level tool invocations instead.

## Output

A first-line tally, then one section per entry, in order:

```
3 invocations: 2 ok, 1 failed, 0 skipped

=== [1/3] read_file: ok ===
Exit code: 0
...

=== [2/3] bash: failed ===
Exit code: 1
...

=== [3/3] message: ok ===
Exit code: 0
...
```

Each section's body is that invocation's ordinary result envelope
(`Exit code:` first, stderr under its marker), or the decline / skip
reason. Statuses: `ok`, `failed` (ran, `is_error`), `declined` (never
ran: outside your toolset, or a nested `multi_tool`), `skipped` (never
ran: `abort` ended the envelope earlier). The whole result is
`is_error` when any entry failed or was declined.

## When to use

- You already know the next several invocations and no entry's input
  depends on an earlier entry's *result* — N file reads, a
  write-then-test sequence, a fan of `message` deposits.
- Your toolset has grown past `bash` and batching saves round trips.

## When not to use

- The next invocation depends on reading the previous result — use
  separate top-level invocations; this tool never returns early.
- One invocation. The envelope adds framing and saves nothing.
- Nesting: `multi_tool` inside `multi_tool` is declined (depth 1) —
  flatten the list.

## Failure modes

- A malformed envelope (wrong shape, unknown field) is declined whole,
  restating the expected shape; nothing runs.
- An entry naming a tool outside your toolset is `declined` for that
  entry alone, with the same text a top-level refusal carries.
- Under `abort` (default), the first `failed` or `declined` entry stops
  the envelope; later entries report `skipped` and have not run.
