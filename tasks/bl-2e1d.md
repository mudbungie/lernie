+++
title = "steal from hermes: work-diff has no rendering — a diff wants a file header, a totals line and an explicit 'omitted N lines' cut, not a row of JSON"
created = 1788745727
updated = 1788745868
claimant = "Cantaloups-S6"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r2"]
+++
Round 2, lane comparator. Steal-list item 2 of 3: the DIFF DISPLAY.

## The comparator, verbatim

`hermes-agent 0.15.2`, same run as the activity-line ball. After the patch tool returned it printed, unprompted:

      ┊ review diff
    a/./calc.py → b/./calc.py
    @@ -1,2 +1,2 @@
    -def add(a,b):
    +def plus(a,b):
         return a+b

Round 1 read the source that produces it (`agent/display.py:426,434-522`) and found two more properties the one-file case does not show:

- colours are resolved from the ACTIVE THEME rather than hardcoded, so the diff is legible on a light terminal;
- a long diff is cut with a line that says it was cut —

      summary = f"… omitted {omitted_lines} diff line(s)"

Four conventions, again separable:

1. **The diff is shown WITHOUT being asked for**, immediately after the edit, headed `review diff` — a two-word imperative that tells the operator what the block is for.
2. **A per-file header on its own line**, `a/path → b/path`, so a multi-file patch does not rely on `+++`/`---` to separate.
3. **An explicit elision line.** A truncation that says nothing reads as a complete diff; this one states the count it dropped.
4. **The gutter marks the harness's line and not the diff's**, so `-`/`+` columns stay at the left margin where every diff tool puts them.

## What the suite prints today

`/work-diff` answers `{"kind":"work-diff","ok":true,"rows":[…]}` and lernie has no renderer for it at all (`src/render/` has no work-diff arm; `lernie work-diff <workspace>` falls through to the generic row printer). Round 1's UX opinion put this first among the daily-user complaints: every read is one dense JSON line and the operator pipes it through a JSON formatter to read it.

Worth recording beside this: on a path-rung conversation this lane drove, the agent edited the bound directory in place and `/work-diff` answered `rows: []` while `/files` listed the CONFIG lineage — so the operator has no gesture that shows what the agent changed. That is a yog question and not this ball; noted so the renderer is not built against an empty source.

## Proposed rendering for lernie

    lernie work-diff <workspace>

    ┊ 2 files changed  +31 −4

    a/roman.py → b/roman.py
    @@ -1,3 +1,18 @@
     def to_roman(n):
         """Convert an integer 1..3999 to a Roman numeral string."""
    -    raise NotImplementedError
    +    values = [
    +        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
    …  omitted 11 diff line(s)

    a/test_roman.py → b/test_roman.py
    @@ -6,2 +6,3 @@
         test_all(); print("ALL PASS")

- A one-line header with the file count and the +/− totals, on the gutter, before anything else.
- One `a/… → b/…` line per file, blank line between files.
- Elision by an explicit `… omitted N diff line(s)` line, never a silent cut. lernie already has the principle in the tree — yog bl-8550 was filed for exactly this defect on the failure clause ("cut at 120 chars with no elision mark, so a truncated remedy reads as a whole sentence") — so this is the same rule applied to the second place it is needed.
- Colour from the terminal's own theme, and the whole thing plain when not a TTY, so `lernie work-diff | patch` is not a trap.
- `--json` keeps the raw frames, as every rendered verb already does (bl-6ae7's ruling: the boundary stays JSON, the seat is the part you look at).

## Severity

p3 rather than p4: bl-6ae7 ruled that every verb renders, and `work-diff` is one of the verbs whose whole content is a shape a row printer cannot show.