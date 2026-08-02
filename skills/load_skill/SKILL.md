---
name: load_skill
description: Load a skill's full instructions into your context by name. The always-present skill descriptions tell you which skills exist and when each applies; when one of them fits the task at hand, call load_skill to pull its body in. The harness copies the skill directory into your worktree at skills/<name>/ and it becomes part of your context on the next model call. Load early — a mid-work load is more expensive than one made before you start the task.
---

# load_skill

Brings a skill's full body into your context (ARCH §3.3
*Body-on-demand*). You already see every available skill's short
description on every model call (the progressive-disclosure convention);
those descriptions are the menu. `load_skill` is how you order from it:
when a description matches what you are about to do, load the skill to
get its complete instructions, scripts, and reference files.

## Input

```json
{ "name": "<skill name>" }
```

`name` is one of the skill names from the always-present descriptions. It
must be a single directory name — no slashes, no `..`.

## Output

A JSON object, carried in the result envelope below after its
`Exit code: N` line:

```json
{ "status": "loaded", "path": "skills/<name>" }
```

- `status: loaded` — the skill body was copied into your worktree at
  `path` and will be part of your context from the next model call on.
- `status: already_loaded` — the skill was already present at `path`;
  nothing changed. The copy already in your worktree wins, even if the
  install's skill pool has changed since — it is the snapshot you are
  pinned to. To pick up a newer version, delete `skills/<name>/` (e.g.
  with `bash`) and load again.

## When to use

- A skill's description matches the task you are starting. Load it before
  you begin, so its instructions guide the work from the first step.

## When not to use

- The task does not match any advertised skill — do not load
  speculatively; each loaded body costs context.

## Load early — it is priced

Loading a skill inserts its body into your assembled context. Loading it
**before** you start a stretch of work keeps that body at a stable
position, so the provider's cached context prefix stays warm across your
steps. Loading it **mid-work** inserts into the middle of your context
and flushes the cache from that point, re-charging the tokens after it
(ARCH §5.5). The body is the same either way; the cost is not. Read the
descriptions, decide what you will need, and load it up front.

## Failure modes

- Unknown skill name → exit non-zero, `is_error: true`, message naming
  the skills actually available. The tool declines rather than guessing a
  near match — pick a name from the advertised pool.
- A name with a path separator or `..` → rejected as not a single
  directory name.

Every result is a §3.3 **result envelope**: an `Exit code: N` line
first, then the output described above, then — whenever the tool wrote
any, on success as well as failure — its stderr under a
`--- stderr ---` marker. So the exact reason for a decline reaches you
in the next step's request, and the stated code tells you which decline
it was.
