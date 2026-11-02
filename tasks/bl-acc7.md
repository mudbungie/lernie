+++
title = "model/session selection is global: the model pick should be per-chat, not per-workspace"
created = 1793514250
updated = 1793584821
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui", "model"]
+++
Operator report 2026-11-01: the model/session selection is global. Model picks should be per-chat.

## What the seat does today

The Model selector (status bar dropup, `/model`) posts `verbs::model(workspace, role="worker", provider, model)` — `src/ui/model/dropup.rs` `pick` → `src/ui/model/tuning.rs` `post_model(WORKER, …)` → `src/verbs/tuning.rs`. That write lands in the `providers.yaml` role assignment on the aimed workspace's config lineage (REMOTE §9.4/§9.13), so it rebinds every conversation on that workspace at once; the selector's own help string says it plainly: *"picks the model this workspace's worker runs on"* (`src/ui/status/selectors.rs`). Effort and priority ride the same door and have the same scope.

## What the operator wants

A pick made while a chat is selected should bind that chat only; another chat on the same workspace keeps its model. The selector should read back the selected chat's binding, not the workspace's.

## Open design questions (not decided here)

- The wire has no per-conversation tuning op: `/model` takes `<role> <provider> <model-id>` against a workspace (REMOTE §3). Per-chat scope needs either a conversation-addressed op in yog/litany, or a config lineage per chat (litany retarget, ARCH bl-22a5, already lets a branch move to a new governing config commit). yog's `docs/REMOTE.md` is the authority and lernie does not get a vote on the wire, so this likely begins as a yog/litany task with lernie's half following.
- What the pick means with no chat selected (the start/composer case): workspace default, or refused?
- Whether effort/priority follow the same scope change (they share the door, so presumably yes).

---

Upstream chain filed 2026-11-01: litany bl-246f (per-agent model mark, the primitive) → yog bl-616f (model/effort/priority/roles take an optional agent). This ball is the seat's half and waits on both: post_model passes the selected conversation; the selector reads back the chat's binding (mark vs lineage) and gains a clear; help text splits the two scopes.
