+++
title = "tests gate: §6 budget collapse bl-f48c"
created = 1783645923
updated = 1783645923
parent = "bl-f48c"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Verify after the change: (1) 100% coverage, all workspace tests pass, all source files <=300 lines. (2) remaining/clamp and their unit tests are gone; grep confirms no references remain. (3) New test: create two conv-id step dirs under a temp steps/ (root-id and root-id-<child>), each with a response.json Usage line, assert derive::spend(repo, root_id) sums BOTH — the whole-tree live-visibility invariant this refactor rests on. (4) check() derives tokens/wall over root_of(branch) and depth over branch; a subagent branch at depth>max_depth exhausts while its root does not.