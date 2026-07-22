+++
title = "gate: tests"
created = 1784266637
updated = 1784698175
claimant = "Prostheses-a953"
parent = "bl-a953"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-a953"
on = "claim"
+++
gate: tests — PASS. make check green on delivered main (d7cad09): 100% coverage (4228/4228 lines); make schemas regenerates schemas/ with zero drift. The gen-schemas objective was met by bl-9e2d's subtraction (binary removed, schemas driven by the in-crate schemas_golden test), so there is no moved-binary surface to test.