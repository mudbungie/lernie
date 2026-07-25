+++
title = "Validate agent id at the inbox boundary; lernie message errors on a nonexistent agent"
created = 1784955529
updated = 1784955536
claimant = "Ivory"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Harrow audit findings 1+2 (same root cause, two fixes differing in kind). (a) src/prompt/inbox/mod.rs:52 inbox_dir joins a raw model-controlled agent_id into the workspace path: 'lernie message $W ../../victim/pwned hello' writes outside the workspace, exit 0; an absolute path REPLACES the base (Path::join semantics). Fix: validate at the boundary with the is_component idiom already written in src/prompt/tool/builtin/load_skill/mod.rs:147-154 ('the name is a fact, not a slot to munge') — lift it to a shared helper; or better an agent-id shape check (hyphenated descent per ARCH §2.3). (b) lernie message to a NONEXISTENT agent silently succeeds (exit 0, phantom inbox dir) — silent message loss; lernie bundle already resolves the id against agents/* refs and errors ('no branch matches agent id ...'); message must do the same. Sell as integrity, not threat model (the bash tool already grants the model execution).