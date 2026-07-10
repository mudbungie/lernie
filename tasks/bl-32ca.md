+++
title = "alignment gate: §6 budget collapse bl-f48c"
created = 1783645923
updated = 1783645923
parent = "bl-f48c"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Check the implementation is coherent against docs/ARCHITECTURE.md (§6, §2.2/§2.3/§2.6 shared steps/), docs/PRINCIPLES.md (Single source of truth — one live whole-tree number, no stored/snapshot copy), docs/TAXONOMY.md. Confirm no new mechanism was added (root_of is a helper, not a stored field); depth/global-ceiling/local-cap remain separable; bl-1067 is closed/repointed with no orphaned 'hand-off' left dangling.