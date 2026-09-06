+++
title = "spend is tokens and nothing else: every comparator answers in currency and the seat cannot"
created = 1788673907
updated = 1788673907
priority = 4
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 swdev lane, comparator observation rather than a defect.

`lernie agent <ws> <agent>` answers spend as:

    "spend": {"attribution": {"count": 1, "kind": "conversations"},
               "tokens": {"cache_read": 91984, "cache_write": 20596,
                          "input": 112930, "output": 1032, "total": 113962}}

That is more detail than either comparator gives about a single conversation, and it is the right substrate. What is missing is the number an operator actually decides on.

    claude -p --output-format json   ->  "total_cost_usd": 0.1077
    codex exec --json                ->  token counts only, same as here

So one of three comparators states currency and it is the one people reach for. On this lane's numbers the absence is load-bearing: the same TypeScript refactor cost claude 251k prompt tokens and $0.1276, and cost this workspace 5,705,120 prompt tokens across a worker and two compactors. Anyone comparing the two harnesses has one figure on one side and a multiplication to do on the other, and the multiplication needs a per-model price table nothing here carries.

The reason it is p4 and not p2: the price table is a fact about a provider, it changes without warning, and a wrong figure is worse than none — so this is an ask with a real design question behind it, not an oversight. brazen is the component that knows which model a call went to and which row it went through; if a price ever lands anywhere it belongs there, and the seat's job is only to render what it is handed.

Two smaller things in the same surface, noted rather than filed separately:

- The rollup is per conversation and there is no per-workspace or per-day total, so 'what has this workspace cost me' is a sum the operator does by hand across every row `lernie conversations` lists — compactors included, and this lane had five of those.
- `attribution: {count: 1, kind: "conversations"}` reads as though it might roll up children, but a parent's row did not include its compactors' tokens; the three numbers had to be added by hand to get the 5.7M above.