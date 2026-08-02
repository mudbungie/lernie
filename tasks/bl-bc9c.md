+++
title = "partial compaction with rebase-forward: zero-downtime compression of a commit span"
created = 1785650728
updated = 1785650728
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Operator design (2026-08-01, codex-comparison follow-up), verbatim: "this is a place we can accelerate, actually. The nature of lernie allows us to do a partial compaction. We pick a commit, and compress everything before it (or in a span of commits), then as the conversation proceeds ahead, we can rebase the conversation after the compaction point on top of the compaction. this lets us do zero-downtime compaction."

## Reconcile with the existing compactor first
ARCH §2.6-2.7's compactor already runs concurrently off a checkpoint and lands as a filtered merge (summary + deletions only), declining on conflict. VERIFY the shipped mechanism against src (src/prompt/tool/builtin/compaction/, the compaction_merge workflow action) before designing. The deltas this ball adds over that baseline:
- **Span selection** — compress a chosen commit range, not only everything-before-checkpoint.
- **Rebase-forward instead of merge-back** — entries committed after the compaction point replay on top of the compacted base. Transcript entries are one immutable file each with monotonic NNN names (append-or-delete only), so replay should be conflict-free by construction unless a later entry touches a file the compactor deleted — state what happens then (the existing decline-loudly discipline, refs/lernie/conflicted, presumably holds).

## Cache honesty
Record the cost in the doc when this lands: any compaction truncates the provider prompt-cache prefix at the compaction point (ARCH §5.4/§5.5 already prices deletion this way); rebase-forward changes availability, not that price.