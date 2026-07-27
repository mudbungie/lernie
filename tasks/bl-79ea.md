+++
title = "lernie config editor-failure decline names neither the $EDITOR value tried nor the fix"
created = 1785133371
updated = 1785133371
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Repro

```
$ EDITOR=/no/such/editor-binary lernie config <ws>
sh: 1: exec: /no/such/editor-binary: not found
lernie config: edit step: editor exited with exit status: 127

$ EDITOR=false lernie config <ws>
lernie config: edit step: editor exited with exit status: 1
```

Source: `src/bin/lernie/cli.rs:28` —
`Err(io::Error::other(format!("editor exited with {status}")))`.

## Why it is a papercut

Not a raw errno or git dump, but it fails the decline standard (name the rule
and the fix in the product's voice): it never says which `$EDITOR` value was
tried, and for a missing binary the shell's own `sh: 1: exec: … not found`
leaks ahead of lernie's line. The user cannot tell whether their `$EDITOR` is
misconfigured or the editor itself failed.

## Fix

Include the editor command and point at the knob, e.g.:

```
lernie config: editor "/no/such/editor-binary" exited with exit status 127 — set $EDITOR to a working editor and retry
```

(127 specifically means the shell could not exec it; if cheap, say "not found
on PATH" for that case.)

## Severity

Papercut. Correct refusal, correct exit code, transient checkout torn down on
this path (verified); only the voice is wrong.

Found by the 2026-07-27 evaluation walk (Flophouses' story walker).