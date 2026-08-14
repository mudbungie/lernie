+++
title = "Scrub the live operator path from docs and adopt the leak-scan stack"
created = 1786683232
updated = 1786683232
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
docs/USER_STORIES.md line ~452 names a real operator home path in an adapter example (live at main tip, public repo) — swap for /home/u. Then port scripts/leak-rules.sh + leak-scan.sh + leak-fixtures/ from rust-bootstrap bl-2c4e, wire make leak-scan into lint/CI, add store-scan.yml. Store history findings (a real personal email in a release-check User-Agent across 3 revisions of one closed ball, a routable IP in a pasted transcript, operator paths) are on the published balls/tasks ref — rewrite decision tracked in ops bl-bd8f. Do NOT wire bl-tracker until the store is scrubbed; note the three-way divergence between the local store tip, the local balls/tasks ref, and origin's.