+++
title = "stop cascade pgid discovery can signal the wrong process group — e2e stop tests killed their own gate run under load"
created = 1784957131
updated = 1784957131
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Observed repeatedly during parallel gates: src/prompt/stop/cascade.rs does libc::kill(-pgid, SIGTERM) with a pgid read from /proc; under load a just-spawned executor can still report its PARENT's pgid (setsid not yet effective / read raced), which under make check is the tarpaulin process group — the e2e stop tests SIGTERMed their own coverage run (2x observed). Production framing: the same race means lernie stop could signal the operator's own group if it reads /proc before the detached executor takes its group. Fix the race at the source: only trust a pgid once it equals the executor's own pid (a setsid'd leader's pgid == its pid — cheap invariant check), retry/backoff the /proc read until that holds, and refuse to signal a group the stop process itself belongs to (belt-and-braces guard). Tests: constructed /proc-race fixture if feasible; at minimum the self-group refusal is unit-testable.