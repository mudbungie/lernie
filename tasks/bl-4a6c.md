+++
title = "child terminal deposit does not launch the parent's driver — parent revival needs a manual scan"
created = 1784955704
updated = 1784956116
claimant = "Rushlight"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 6, FIXED. ARCH promised "a running child … revives [the stopped parent] by depositing its result (§2.11)" and "normal operation needs zero scanning", but `deposit_terminal` only wrote the file into the parent's inbox — the parent sat at its old tip until a hand-run `lernie scan`.

**Fix.** `dispatch::terminal::revive_parent`, called from `exit_launch` after the executor releases its own lock: derive the parent from the exiting agent's id (`inbox::parent_of` — the id IS the address) and hand it to `inbox::probe_and_launch`, the *unmodified* seam `lernie message` and `lernie dispatch` already use. No second probe, no second spawn path; a held parent lease launches nothing (its executor drains at its next boundary), a free one gets one detached `lernie advance`.

**Reconciling with the §2.11 exit protocol.** The protocol's sequence names only "spawn a driver at own agent". The parent-side launch is NOT a second protocol bolted on: it is the *deposit's* own launch — the plain "a deposit into a quiescent agent starts a driver" rule (Writer/driver totality: a writer deposits, probes, launches, never drives) applied to the one deposit an executor makes on its way out. So the terminal sequence now reads: terminal event → deposit into the parent's inbox → release own lock → spawn at own agent → probe-and-launch at the parent → exit. It is placed *after* the release for the same no-authority reason the self-directed launch is: from the release onward the exiting process only spawns and exits, so a revived parent that immediately messages or stops its child meets no lingering lease. The single conditional ("does a parent inbox exist") governs both the deposit and this launch, because they are one act.

**Epitaph decision (pin 2) governs BOTH targets** — it does apply at the parent, read one level up: *final-response* → launch (a child reporting work is exactly what a parent should wake for); *stopped* → never (the woken parent would react to — plausibly re-dispatch around — the branch the operator just killed: the stop undone one level up); *budget-exhausted* → never (the ceiling is derived over the whole tree, §6, so the woken parent exhausts on its own next check and deposits again — the same spam cycle, climbing). In both "never" cases the deposit still lands and stays undelivered; the next explicit touch delivers it. Unrelated and unchanged: a *stopped parent* is still revived by a later child final-response (§2.9, PRINCIPLES) — pin 2 keys on the exiting agent's epitaph, not the recipient's history.

**Tests** (`src/prompt/tests/parent_revival.rs`, plus the extended `exit_launch.rs`): a dispatched child's real `lernie advance` terminal revives its parent, whose launched hop delivers the result (transfer + delivery commit) and steps — no scan anywhere; a held parent lease yields no second driver and the result waits in the inbox; stopped and budget-exhausted child terminals deposit and launch nothing at all.

**Docs**: ARCH §2.3 step 5, §2.11 exit protocol + pin 2 + both shipped-state notes; README "The exit protocol and the operator scan".