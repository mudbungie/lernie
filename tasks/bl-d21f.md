+++
title = "tests"
created = 1785124428
updated = 1785124428
parent = "bl-6f1b"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-6f1b"
on = "claim"
+++
Ensure test coverage is at 100% and all tests pass — on GitHub CI, not just locally — for the tarpaulin segfault fix. Re-run CI on main several times to confirm the failure rate is zero, given the original was timing-sensitive.