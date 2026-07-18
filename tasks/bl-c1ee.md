+++
title = "runner (v0.10): agent-eval --config <experiment> --suite <suite> --runs N → pass@1/pass@5 with CIs"
created = 1784336892
updated = 1784336892
parent = "bl-8094"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-3e1d"
on = "claim"

[[blockers]]
id = "bl-0007"
on = "close"

[[blockers]]
id = "bl-e20b"
on = "close"

[[blockers]]
id = "bl-44c7"
on = "close"
+++
Deliverable 3 of epic bl-8094. An evaluation run is (experiment × suite × N): run one workflow-config variant against the fixed task suite N times, machine-check each task, aggregate pass@1 (mean-of-means, Wilson 95% CI) and pass@5. DECIDED (epic recommendation, spec's naming): separate binary crates/agent-eval, not a lernie subcommand. Needs the suite format settled first (bl-3e1d).