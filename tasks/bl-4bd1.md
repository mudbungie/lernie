+++
title = "The built-in tool set is undiscoverable from the CLI: 'lernie tool --help' shows a bare <NAME> and the unknown-tool decline lists nothing, while load_skill's sibling decline names its whole pool"
created = 1785130201
updated = 1785133157
claimant = "flop-4bd1"
priority = 1
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Repro

```
$ lernie tool --help
In-process built-in tool entry (ARCH §3.3): `tool_use.input` JSON on stdin, bytes on stdout, exit 0/non-zero. Third resolver hop (`<data-root>/tools/lernie-tool-<name>` → PATH → `<lernie> tool …`)

Usage: lernie tool <NAME>

Arguments:
  <NAME>

Options:
  -h, --help  Print help

$ echo '{}' | lernie tool nosuchtool
lernie tool nosuchtool: unknown built-in tool: "nosuchtool"
```

Neither surface names any of the five built-ins (`bash`, `dispatch`,
`load_skill`, `message`, `read_file`).

## The product already has the right idiom, one level down

```
$ echo '{"name":"nosuchskill"}' | LERNIE_CONV_REPO=… LERNIE_CONV_BRANCH=… lernie tool load_skill
lernie tool load_skill: unknown skill "nosuchskill"; available: bash, dispatch, load_skill, message, read_file
```

ARCH §3.3 (quoted in `docs/USER_STORIES.md` US-12) makes that the rule for
`load_skill`: *"an unknown name or a non-single-component name is **declined**
with `is_error`, **naming the available pool** — never fuzzy-matched, never
sanitized."* The tool dispatcher one hop above enumerates from the same closed
set and does not.

README §"Built-in tools" invites the user to type these by hand — *"Try it
directly: `echo '{"path":"README.md"}' | lernie tool read_file`"* — so the CLI
is a documented entry point for them, not an internal seam.

## One-line fix

Render the same list in both places: append `; available: <names>` to the
unknown-tool error, and put the names in the `<NAME>` argument's clap help
(`value_parser` over the known set would give both plus shell completion for
free).

## Related, same shape, lower value

Every verb's positional arguments are documented nowhere except `advance`,
which has them: `lernie advance --help` renders `<WORKSPACE>  Path to the
workspace (conversation repo) root` and `<AGENT>  Agent id (== branch name /
hyphenated descent) to drive`. `new`, `config`, `prompt`, `dispatch`, `stop`,
`message`, `scan`, `bundle`, `replay`, `tool` all render their positionals
blank. Adding doc comments to the `Args` fields is compatible with the US-23
parity checker, which asserts the argument *set*, not its help text.

## Severity

Papercut. Nothing misbehaves; the CLI just cannot answer a question it is
documented to invite, and the fix is one list already computed elsewhere.

## Verified in passing (US-11 / US-12, listed as unchecked in USER_STORIES §12)

Both tool shims were driven as real subprocesses in this pass against the
0.0.1 binary and behaved exactly as US-11/US-12 specify:
`lernie tool load_skill` → `{"status":"loaded","path":"skills/bash"}` then
`{"status":"already_loaded",…}` with the body copied into the agent worktree;
`lernie tool message` → `{"status":"deposited"}`, the deposit self-delivered
via the detached driver (a `transcript 002: <sender-id>` commit landed and the
inbox emptied), and a deposit to a nonexistent recipient came back non-zero
carrying the front door's own refusal. `docs/USER_STORIES.md` §12 lists both
as *"not checked by this pass"*.

Filed by an outside evaluation pass (wharfinger) walking 0.0.1 from the public
docs only; not claimed, not fixed.