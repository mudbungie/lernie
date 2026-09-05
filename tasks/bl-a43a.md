+++
title = "the fleet has no surface: a workspace cannot be run, stopped, watched or scanned, and what its agents changed is unreadable"
created = 1788581509
updated = 1788581509
parent = "bl-4c48"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The other half of bl-4c48, which landed the trail's read and left this. Seven
`control`-classed ops with no interactable in this seat: `fleet`, `disband`,
`arm`, `disarm`, `scan`, `science`, `work-diff`.

## The naming trap, first, because it is the one that costs an afternoon

**`fleet`/`disband` are the fleet LOOP; `arm`/`disarm` are the alignment
MONITOR.** Two families, two carriers, two config entries — and one shared
reply kind, so a seat cannot tell them apart by the answer. Read the `op` back,
never the reply.

## The frames, exact

Requests (every fixture already vendored under `corpus/request/`):

    {"op": "fleet",   "workspace": W, "project": P, "cap": N}   cap is a NUMBER
    {"op": "disband", "workspace": W}
    {"op": "arm",     "workspace": W, "model": M}
    {"op": "disarm",  "workspace": W}
    {"op": "scan",    "workspace": W}
    {"op": "science", "workspace": W}
    {"op": "work-diff", "workspace": W}
    {"op": "work-diff", "workspace": W, "file": {"ball": B, "path": P}}
    {"op": "work-diff", "workspace": W, "file": {"ball": B, "handle": H, "path": P}}

Replies:

    fleet | disband | arm | disarm  ->  {"ok": true, "kind": "armed", "armed": BOOL}
    scan                            ->  {"ok": true, "kind": "outcome", …}   already painted
    science                         ->  {"ok": true, "kind": "science",   "rows": […]}
    work-diff                       ->  {"ok": true, "kind": "work-diff", "rows": […], "patch": …?}

`armed`, `science` and `work-diff` sit in `corpus/unreadable/` today; each moves
to `corpus/answers/` in the breath that paints it.

## Two things that are NOT here, and knowing why saves the search

**There is no `fleet` READ and no `fleet` reply kind.** The loop's state — cap,
count, tick, lease, ceiling, how long since it last acted — rides on the
**board** reply, in an optional top-level `fleet` array that is **absent rather
than empty** when nothing is armed. So the fleet's readable half belongs to the
ball pane (bl-d2af), which is what would decode `board` at all, and this ball's
own reads are `science` and `work-diff`.

**`cap` is a number**, so `fleet` cannot be a row in the verb table — that table
is rows of named strings and refuses to grow an arm for anything else. It is a
typed door, beside the `ops` door bl-4c48 put in `src/verbs/trail.rs` and the
tuning pair's two.

## What it will need beyond the frames

An arming or a stated reason not to, for the two that stop things: `disband`
takes a running fleet down and `disarm` drops the watch. DESIGN §4.20 is the
idiom.

A subject. Every one of the seven addresses a workspace, so unlike the trail
these hang off the **aimed wall** — the tuning pane's shape, not the queue's.

Delete each op's row from `parity.toml` as its control lands.