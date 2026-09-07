+++
title = "steal from hermes: work-diff has no rendering — a diff wants a file header, a totals line and an explicit 'omitted N lines' cut, not a row of JSON"
created = 1788745727
updated = 1788746034
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

---

Two premises moved before this was picked up, and both change what lands.

**A renderer exists.** bl-6ae7 landed `src/render/`, and `work-diff` has an arm
in it (`render/tasks.rs::work`) — one header a ball, one churn line a file. The
`{"kind":"work-diff","ok":true,"rows":[…]}` in the body is the `--json` form.

**This wire is a numstat, never a patch.** yog reads the attempt with
`git diff --numstat` (`src/workdiff/read.rs`) and answers a path plus two counts
a file; `truncated` there is `files.len() > MAX_ENTRIES`, a FILE cut. So there is
no hunk on this surface, no `@@` header, and no diff LINE to omit — items 1 and 4
of the comparator list (an unprompted `review diff`, and a `-`/`+` gutter) have
nothing on this frame to render, and the form that answers one file's patch is
deliberately not composed by this seat (`verbs::WORK_DIFF`'s own detail says so).

What the frame CAN answer, and what landed:

- **The totals line.** `N files  +A -B  M binary`, folded over the churn, first
  line under each row's header. Binaries counted apart — no line count describes
  them, and folding them in understates every total they are part of.
- **The explicit cut.** `truncated` was decoded and printed NOWHERE, so a listing
  the engine stopped early read as the whole change. It is now a line that states
  what it knows and names what it does not: the count dropped is genuinely not on
  this wire. Same rule the worktree listing already held with `(truncated)`.
- **The header's two ends.** The per-file `a/… → b/…` has no referent here (one
  path a churn; a rename is not carried), but the fact it exists to state — which
  two things are being compared — is the ROW's, and `target_oid`/`source_oid`
  were decoded and printed nowhere either. The header now carries `at <t>..<s>`,
  short, and says nothing when only one end resolved.
- **A `diff` row with no files** now says the attempt has changed nothing yet,
  rather than a header with an empty space under it. Shape-read, not word-read:
  `truncated` is written by the `diff` state alone.

Not done, and not filed: **colour**. `render::said` returns a `String` with no
knowledge of whether its sink is a TTY, so theme-resolved colour is a change to
what a rendering IS, not to this diff — it wants its own ball across all forty
renderings, or none.

The rendering split to `src/render/work.rs` at the cap (DESIGN §5); §4.37 gained
the elision rule and the numstat bound.
