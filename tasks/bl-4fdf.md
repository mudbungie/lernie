+++
title = "compaction merge imports the compactor's private transcript into the parent"
created = 1784955704
updated = 1785027675
claimant = "Yarrow"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 12. After a checkpoint compaction, the parent branch gained the compactor's OWN dialog — messages/003-<goal>.md, 004-<model>.json, 005-tool.json (including a failed tool call on a hallucinated path) — and NO summary/001.md; the imported tool entry then wedged the parent's next step on the local provider. The merge lands the child's whole tree. Read ARCH §2.7's current compaction design and decide what the merge is SPECIFIED to land (almost certainly the summary/ artifact, not the compactor's messages/); implement the filter accordingly (cf. the §2.6 work-product transfer, which is already 'filtered to work products' — same principle, maybe same mechanism). Fix ARCH if it is silent on the filter. Tests: post-compaction parent tree gains summary content and zero compactor transcript entries; parent's next step assembles clean.