+++
title = "Delete merge/: result-message return replaces rebase-then-merge-back [substrate]"
created = 1783829720
updated = 1783831582
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["code"]

[[blockers]]
id = "bl-65d8"
on = "claim"

[[blockers]]
id = "bl-4298"
on = "claim"

[[blockers]]
id = "bl-cb44"
on = "claim"

[[blockers]]
id = "bl-1129"
on = "claim"
+++
Blocked on bl-65d8 (the doc must land first).

`docs/ARCHITECTURE.md:274` (§2.6 Shipped-state note): *"The shipped harness still implements the v0.3 protocol this section retires — `merge=ours` attributes at scaffold, the alignment step, rebase-then-merge (see `tests/merge_ours.rs`). This section is the target contract; deleting the merge machinery and building the result-message return is implementation work tracked separately."*

It was **not** in fact tracked separately. This is that ball.

Delete `src/prompt/merge/` (`rebase_and_merge`, `align_merge_ours`) and `tests/merge_ours.rs`. Build the return path in its place: lifecycle step 5 deposits a result message (terminal ref + epitaph + terminal response) into the parent’s inbox, and delivery applies the work-product-filtered diff (§2.6).

Note this depends on the inbox existing, which it does not — `grep -rn inbox src/**/*.rs` returns zero hits. The inbox + `message` tool + `lernie message` verb are a prerequisite and are also untracked; file that ball before claiming this one.

Exactly one merge survives in the system: the compaction merge (§2.6).