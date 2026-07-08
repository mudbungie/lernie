+++
title = "Amend §2.11: writer/driver totality — every branch invocation is either a writer (deposit, exit) or a driver (acquire-or-exit), never both"
created = 1783490442
updated = 1783490442
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surgical follow-on to bl-ed40. The shipped executor-lock paragraph says 'A would-be executor that fails the acquire … deposits whatever it came to deposit and exits', implying an invocation that both deposits and drives. Contradiction with the design's own discipline: lernie message is only ever a writer (its lock try-acquire is a liveness probe, released immediately, never held to drive); lernie prompt/dispatch/advance are only ever drivers (acquire-or-exit as a no-op, nothing to deposit). Fix: state the totality explicitly and rewrite the losing-acquirer sentence so the loser arm is the same code as the uncontended case — no combined path whose arms could drift.