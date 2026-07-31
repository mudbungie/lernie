+++
title = "pooled skill frontmatter is validated only at first lernie prompt: prime/new/config snapshot descriptions/** without parsing, so a malformed SKILL.md poisons workspaces that already exist"
created = 1785473896
updated = 1785473896
priority = 4
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["bug"]
+++
A SKILL.md description: containing ': ' (YAML plain-scalar trap) — or any malformed frontmatter — passes lernie prime, lernie new, and lernie config untouched, because pool snapshotting never parses what it snapshots. It surfaces only at the first lernie prompt: 'tool slack_post skill frontmatter … is malformed: mapping values are not allowed in this context…' — after the workspace, config commit, and agent branch already exist. Fix: parse skill frontmatter (and tool schema JSON) at snapshot time — lernie new and lernie config decline naming the offending pool file; prime validates what it seeds. Keep prompt-time validation as the backstop. Found implementing the fleet demo (fleet/tools/skills/).