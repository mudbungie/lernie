+++
title = "sweep wall → workspace now; engine → server once yog's REMOTE §2 ruling lands"
created = 1792036528
updated = 1792304392
claimant = "Swishes"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
# Two renames, one of them held

## wall → workspace (land now)

REMOTE §2 already rules it: "Multi-server adds no noun … A workspace is one
word at both ends." lernie holds no world and never touches the directory yog
DESIGN §3 calls a workspace's wall, so every "wall" in this tree (215 DESIGN
mentions, ~230 src files; `asker::wall`, `Model::wall`, `render/walls.rs`)
means plain workspace. Rename all of it. This is the seat catching up to the
authority, not leading it.

## engine → server (held on yog bl-3390)

REMOTE uses "engine" for the yog program and "server" for the box; lernie's
DESIGN §0 parenthetical "the engine (litany, beneath the server)" contradicts
REMOTE — the window code (engine = the thing a seat dials) does not. The
ruling that retires "engine" and leaves litany with no role noun lands in
REMOTE §2 first; then this tree sweeps engine → server (386 DESIGN mentions,
~391 src files) and corrects §0.

Expect `make line-cap` / `fmt-check` pushback: "workspace" is five characters
longer than "wall".