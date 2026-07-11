+++
title = "XDG harness-root split: config/data dirs, LERNIE_HOME collapses both"
created = 1783747997
updated = 1783747997
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["design-2026-07"]
+++
Settled in design discussion 2026-07-09/10. Split the harness root (currently ~/.lernie, src/harness_root.rs) along XDG lifetimes: $XDG_CONFIG_HOME/lernie for configuration (models.yaml; likely workflow templates — implementer decides split with the doc) and $XDG_DATA_HOME/lernie for data (conversations/, and the agents/skills/tools pools). LERNIE_HOME stays as the single override that collapses BOTH to one dir — test isolation (parallel tests, sandboxed replay) keeps working unchanged with one env var. brazen already resolves XDG-style, so this aligns the two projects.

Scope: harness_root.rs resolution + every caller; amend ARCH §2.2 (harness root paragraph) and §4.1/§4.2 config-split table to name the XDG homes. Severability check: policy lives in the resolver, one file today — do it while greenfield.

100% coverage, <=300-line files.