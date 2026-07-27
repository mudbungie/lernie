+++
title = "lernie dispatch skips the shared id/workspace guard: raw git argv on a missing workspace, config-derivation voice on a missing agent, raw sha on an unknown role"
created = 1785133371
updated = 1785133371
priority = 1
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## The claim README makes

README ("Messaging an existing agent directly"): *"The id guard is the same
rule at every verb taking an agent id from outside — `message`, `advance`,
`stop`, `dispatch`, `bundle`."* It does not hold for `dispatch`: `dispatch`
alone skips the shared workspace/agent-existence guard and falls through to
`governing_config` derivation, which shells to git and surfaces failures raw.

## Repro 1 — nonexistent workspace (all five verbs, same input shape)

```
$ lernie bundle  <no-such-ws> someagent <out>
lernie bundle: <path> is not a workspace (no repo.git) — create one with `lernie new` (ARCH §2.2)
$ lernie message <no-such-ws> someagent hi          # same voice
$ lernie advance <no-such-ws> someagent             # same voice
$ lernie stop    <no-such-ws> someagent             # same voice
$ lernie dispatch worker <no-such-ws> someagent --goal hi
lernie dispatch worker: governing config for someagent: git ["for-each-ref", "--format=%(refname)", "refs/heads/config/"] exited with exit status: 128: fatal: cannot change to '<path>/repo.git': No such file or directory
```

Raw git argv as a Rust Debug array, exit status, and the `fatal:` line — the
exact shape filed as bl-55e0, on a more commonly-hit verb.

## Repro 2 — parent agent id does not exist (workspace real)

```
$ lernie dispatch worker <ws> nosuchparent --goal hi
lernie dispatch worker: governing config for nosuchparent: no config/* ancestor for agents/nosuchparent — every agent forks off a config commit (§2.2)
```

vs. the shared voice on the identical condition:

```
lernie message: no agent "nosuchparent" in this workspace — a message is addressed to an existing agent (ARCH §2.11); check the id against the workspace's `agents/*` refs, or start an agent with `lernie prompt` / `lernie dispatch`
```

`dispatch` never says "no agent", leaks the internal `agents/<id>` ref form,
and reasons about governing-config derivation (an implementation detail)
instead of the existence rule.

## Repro 3 — unknown role leaks a raw commit sha

```
$ lernie dispatch verifier <ws> <real-agent-id> --goal hi
lernie dispatch verifier: role "verifier" is not defined in 806e58cd9d2de8b2208bddf010ab79197b117aa3:providers.yaml
```

The 40-hex commit id and `:providers.yaml` git-show syntax are internal
representation, not product voice. Name the source the user knows (the
workspace's providers.yaml as governed by the agent's config lineage) and,
ideally, the roles that ARE defined — the same "name the pool" idiom as
load_skill / bl-4bd1.

## Root cause (confirmed in source)

`src/cmd/dispatch.rs:24` calls only `crate::name::require_agent_id(&args.branch)`
(the shape check) — never the workspace-layout guard nor `agent_exists`, which
`message`, `advance`, `stop`, and `bundle` all call before doing anything else.

## Fix

Apply the same guard sequence to `dispatch` ahead of `governing_config`:
workspace-exists (shared "is not a workspace (no repo.git)" voice), then
parent `agent_exists` (shared `no agent "…" in this workspace` voice). For the
role decline, render the role pool / config source in product voice instead of
`<sha>:providers.yaml`. One shared guard, no per-verb copy — the README
sentence becomes true again.

Found by the 2026-07-27 evaluation walk (Flophouses' story walker) driving the
five verbs with identical inputs against a fresh build of main.