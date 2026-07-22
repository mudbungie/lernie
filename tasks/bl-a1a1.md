+++
title = "overflow: summarize is inert at assembly — wire it to trigger the §6 compaction checkpoint"
created = 1784698286
updated = 1784698286
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-e0cb (§5.2 context assembly). The manifest's `overflow: summarize` policy sheds nothing at assembly: assembly is a pure function of the read-state tree (§5.1) and cannot invoke a model, so `src/prompt/dispatch/assembler/body.rs::fit` rides the body whole and documents that the §6 compaction checkpoint is the real shedding mechanism. The deferred piece: a role declaring `summarize` should have its over-budget observation actually *trigger* a compaction dispatch (the §6 `worker_flush` seam / compaction checkpoint clock), so the policy means something operationally. Decide whether that trigger lives in the workflow `compaction:` clock (a budget-pressure condition beside the cadence) or is subtracted from the OverflowPolicy vocabulary; either way, fix ARCH §5.2's note when it lands.