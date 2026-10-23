+++
title = "brazen bl-6feb phase 1: delete the bl-526d dropup filter; start::unsigned fires when the worker's row is absent from the providers table"
created = 1792729118
updated = 1792729118
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["integration"]
+++
Phase 1 of bl-8c03 (see that ball). yog's providers reply is now brazen's routing table, so unfunded presets nobody named are absent rather than listed as bare. Drop the bl-526d dropup filter (every table row is a choice); synthesize the worker's row into the dropup when the table lacks it; make start::unsigned fire when the worker's provider is absent from an answered table; align DESIGN.md, STYLE.md and the providers module doc. Phase 2 (login pane fed by yog's catalog read, corpus re-vendor) stays on bl-8c03.