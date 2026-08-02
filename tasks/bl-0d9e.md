+++
title = "lernie delete <workspace> <agent> [--children] — agents have no lawful removal verb; yog needs one"
created = 1785645947
updated = 1785646033
claimant = "delete-builder"
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Filed from yog bl-f17a (2026-08-02): the operator wants right-click→delete-agent in yog, but lernie 0.0.3's Command enum (New, Config, Prompt, Dispatch, Stop, Message, Scan, Bundle, Replay, Advance, Tool, Prime) has no delete/remove/prune verb, no removing flag, zero ref-deleting call sites in prod — and yog's I2 invariant rightly forbids yog writing inside a workspace, while lernie owns agent placement, so agent disposal is lernie's (yog DESIGN §3.6's rejected 'upstream delete' argument is scoped to workspaces only). Six requirements, spec'd in yog bl-f17a's body: (1) subtree semantics explicit — bare form refuses if descendants exist, --children takes the subtree; (2) removes every slice: agents/<id> ref + descendants, worktree, steps/<id>/, inbox/<id>/, refs/lernie/*/<id> marks; (3) refuses while the executor lock is held (never reap a live driver); (4) surfaces pending inbox deposit count so a caller's confirm can enumerate what dies; (5) convergent on re-run (crash-safe partial delete); (6) bundle composes in front (bundle-then-delete = archive). Also noted: lernie 0.0.3 ships no retention/GC at all — yog DESIGN §3.6's '30-day branch-level GC' premise is stale; correct whichever doc claims it when this lands.