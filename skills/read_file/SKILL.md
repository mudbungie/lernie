---
name: read_file
description: Read the entire contents of a file at the given path and return its bytes verbatim. Use when the conversation needs to inspect a specific file by path, before suggesting an edit or making claims about its contents. Files larger than 1 MiB are rejected; reach for `bash` (e.g. `head -n N`) for those.
---

# read_file

Reads a file from disk and returns its bytes. Best for source files,
configs, and short documents whose content the model needs to see in
full.

## Input

```json
{ "path": "<filesystem path>" }
```

`path` is a string. Relative paths resolve against the agent's current
working directory.

## Output

Raw bytes from the file, surfaced as the `content` of the matching
`tool_result` block. Empty files yield an empty `tool_result.content`.

## When to use

- The user references a file by name and you need its contents to
  reason about it.
- Verifying state on disk before suggesting an edit (cheaper and more
  reliable than asking the user to paste it).

## When not to use

- Files larger than 1 MiB — the tool rejects them with a `TooLarge`
  error rather than truncating. Use `bash` with `head`, `tail`, or
  `sed` to scope the read.
- Directory listings or recursive searches — use `bash` (`ls`, `find`)
  instead.

## Failure modes

- Missing file or permission denied → exit non-zero, `is_error: true`,
  message of the form `open <path>: <io error>`.
- Oversize file → exit non-zero, `is_error: true`, message names the
  observed size and the cap.
- Malformed input JSON → exit non-zero, `is_error: true`.

The tool's stderr is concatenated after stdout into the
`tool_result.content` on failure (ARCH §3.3 stdio contract), so the
model sees the exact reason in the next step's request.
