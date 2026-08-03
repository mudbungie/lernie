---
name: apply_patch
description: Edit local text files with one structured patch envelope carrying add, delete, update, and rename operations across multiple files, applied atomically — all-or-nothing in a single tool invocation. Prefer this over `bash` (sed, heredocs) for every file edit. Context is matched with bounded fuzz (whitespace and Unicode-punctuation drift are tolerated) but must locate a unique target; a stale, missing, or ambiguous target refuses the whole patch with the exact reason and writes nothing. Files are on the local machine, resolved against your current working directory; only writes inside your worktree are committed.
---

# apply_patch

The structured edit path: one envelope, many files, one atomic
application. Failures are typed refusals naming the file, hunk, and
reason — repair the patch and reinvoke.

## Input

```json
{ "input": "*** Begin Patch\n...\n*** End Patch" }
```

## Grammar

```
*** Begin Patch
*** Add File: path/to/new.txt
+each content line, +-prefixed
*** Delete File: path/to/old.txt
*** Update File: path/to/existing.txt
*** Move to: path/to/renamed.txt
@@ fn enclosing_symbol
 context line (space-prefixed)
-line to remove
+line to add
 context line
*** End Patch
```

- `*** Update File:` carries one or more hunks. `@@` on its own line
  separates hunks; `@@ <text>` names an **anchor** line (an enclosing
  function or class header) that is located first, so a hunk inside a
  repeated block can say which copy it means.
- `*** Move to:` directly after the `*** Update File:` line renames the
  file after the hunks apply. The destination must not already exist.
- `*** End of File` after a hunk pins it to the end of the file — use
  it to append, or to disambiguate a block that recurs earlier.
- A hunk of only `+` lines (a pure insertion) needs an `@@` anchor or
  `*** End of File` to say where it lands.
- Blank lines: inside an update body a bare blank line is an **empty
  context line** — the file must have a blank line there, trailing
  blanks included. In an add section a blank content line is a lone
  `+`; a bare blank line there, or between file sections, refuses the
  patch. Blank lines around the envelope, and directly after
  `*** End of File`, are ignored.
- Paths resolve against your current working directory (the `cd` tool
  moves it); absolute paths are taken as-is, but only worktree writes
  ride the tool commit — edits elsewhere are off the record.
- A destination that is itself a symlink — dangling or not — refuses
  the patch for add, update, and rename targets: the write would land
  through the link, outside the path you named. Name the link's target
  directly, or delete the link first (delete removes the link itself,
  never its target).

## Matching

Each hunk's context is located by descending a fixed ladder: exact
match, then ignoring trailing whitespace, then ignoring edge
whitespace, then Unicode-normalized (smart quotes, non-breaking
spaces, em-dashes folded to ASCII). The first rung with matches wins;
the match must be **unique** from the previous hunk's position onward,
or the patch is refused (`add an @@ anchor or more context`). Copy
context from the file as you last read it — the fuzz absorbs
formatting drift, not content drift; if the file changed underneath
you, the patch refuses rather than overwriting unseen work. Re-read
the file and re-author.

## Output

A JSON report: `{"status":"applied","files":[{"path","op","moved_to?",
"hunks":[{"rung","line","matched?"}]}]}` — per hunk, the ladder rung
that matched and the 1-based line where it landed; `matched` carries
the exact replaced lines whenever a fuzzy rung won.

## When not to use

- Binary or non-UTF-8 files — the tool edits text; use `bash`.
- Reading — use `read_file`; verify before you edit.
