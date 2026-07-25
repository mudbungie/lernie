+++
title = "Description-always is partial: a standalone skill's frontmatter composes nowhere"
created = 1784698287
updated = 1784955418
claimant = "Bulwark"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-e0cb. ARCH §3.3 Description-always: "Every available skill's SKILL.md frontmatter (name + description) is committed in the config under descriptions/skills/ … and composed into the context on every model call." Shipped state: `descriptions/skills/<name>.md` reaches the model only as the `description` field of a same-named tool's entry in the request's tools array (`src/prompt/dispatch/tools.rs`). A standalone skill (no tool, §3.3) has its frontmatter snapshotted into the config commit but composed nowhere — the agent cannot discover it to elect `load_skill`. §5.2 assembly (bl-e0cb) deliberately skips `descriptions/**` as body text because its wire home is the tools array; the standalone-skill remainder needs a home (likely: compose descriptions/skills entries with no matching declared tool as head text blocks). Fix §3.3's claim or the code, one or the other.