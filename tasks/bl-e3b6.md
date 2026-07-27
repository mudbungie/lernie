+++
title = "gate: docs"
created = 1785125522
updated = 1785130107
claimant = "Halyard2"
parent = "bl-7318"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-7318"
on = "claim"
+++
PASS. Two doc edits landed with bl-7318 (main 6a4a47d), both correcting statements that were factually wrong rather than merely incomplete.

1. `docs/ARCHITECTURE.md` section 2.11, *The executor lock*. The paragraph said the lock is 'kernel state bound to process lifetime: released by the kernel on any death' and stopped there, which left release-by-close reading as sufficient. Added, in the same paragraph: '**A voluntary release is an explicit `flock(fd, LOCK_UN)`, never a bare close.**' followed by why — the lock rides the open file description, closing one fd naming it releases the lease only once every fd naming it is gone, `fork`/`clone` copies the whole fd table and close-on-exec fires at `execve` rather than at the fork, so any spawn anywhere in the process transiently makes more of them; a lease released inside that window stays kernel-held until the unrelated child execs and the next probe reads 'another executor drives this branch' — the one lie this signal must never tell, since it turns a driver into a silent no-op (Writer/driver totality) and a silently-dead agent into a live one (section 8). Closes with the invariant preserved: death is unaffected, the kernel drops the description with its last fd.

2. `docs/ARCHITECTURE.md` section 6, *The exec baton carries the lease*. The old sentence claimed the successor '...restores close-on-exec so the lease never leaks into the tool and adapter subprocesses the hop spawns.' That is false as written: close-on-exec keeps the lease out of those subprocesses' **exec'd images**, not out of their pre-`exec` window, where the fd table copy already carries it. Reworded to say exactly that, and to name the consequence — 'which is why section 2.11 makes release an explicit `LOCK_UN` on the description rather than a close of one fd naming it'.

Code docs carry the same account at its one home: `src/prompt/inbox/lock.rs`'s module header gained a '**Release is explicit `LOCK_UN`, not a bare close**' section, and `ExecutorLock`'s `Drop` and struct docs say what they do and why. No other doc mentions lease release. README and TAXONOMY needed no change: no new term of art was coined ('lease', 'executor lock', 'open file description' are all pre-existing — the first two established in ARCH 2.11 and `lock.rs`, the third POSIX).