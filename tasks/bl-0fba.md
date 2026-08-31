+++
title = "per-seat UI state: where the seat keeps what is its own"
created = 1788068290
updated = 1788138812
claimant = "OrderPolish"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-428f"
on = "claim"
+++
REMOTE §7 rules that per-seat UI state — focus, scroll, tab selection, drafts — never crosses the boundary and is the seat`s own. The window will therefore have durable state of its own on this box for the first time.

The hazard is already written down in src/paths.rs: everything under this crate`s data root today is operator-provisioned and irreplaceable, so nothing the seat GENERATES may be written beside it — a regenerable subtree under the same root would make a rebuild a revocation. UI state is exactly such a subtree.

So this ball decides where it goes and records the decision, before the window is the thing that decides it by being written. The likely shape is the XDG state root rather than the data root, which is the same separation for the same reason.