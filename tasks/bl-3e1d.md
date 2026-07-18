+++
title = "task suite (v0.9): 50 machine-checkable tasks, ≥10 per failure category, per-category tagging"
created = 1784336892
updated = 1784336892
parent = "bl-8094"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-26ea"
on = "close"
+++
Deliverable 2 of epic bl-8094. Per §9/v0.9: 50 tasks, each with a machine-checkable success criterion (a checker script, not LLM-as-judge), ≥10 per failure category, tagged by category. OPEN (user, from epic body): suite home — in-repo (tests/suite/) vs own repo. Lean in-repo: the suite versions with the code it measures and adds no new repo mechanism — confirm with user before landing. Task content itself deserves user eyes: which failure categories, what task sources.