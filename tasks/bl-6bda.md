+++
title = "help promises a page for ten words and has one for six"
created = 1788148157
updated = 1788151289
claimant = "OrderJoiner2"
priority = 4
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
`lernie help` documents its own second form:

    lernie help [<verb>]

and then prints ONE roster in which ten words appear — the six gesture rows,
then `start`, `ask`, `entries` and `help` under them, each with a summary in
the same column. Nothing in that page distinguishes the two groups.

Four of the ten refuse:

    $ lernie help start
    lernie: no verb named "start" — `lernie help` lists every one

and the same for `ask`, `entries`, and `help` itself. The six gesture verbs
answer normally.

**The cause is not a bug and should not be "fixed" by widening the table.**
`src/verbs.rs` is explicit that the table is data — every row a word and its
parameters, *"all of them named strings"* — and that `start` and `ask` cannot
be rows precisely because their arguments are not: *"`prepare` carries a
payload rung and `prompt` carries a prepared body, and a nested object is not
a word an operator types."* The module calls them *"typed doors with no row"*.
That reasoning holds. `src/verbs/help.rs` then resolves `page(word)` through
`find(word)`, which searches that same six-row table, so the four doors are
unreachable by construction.

**The defect is that the help page does not know this about itself.** It
advertises a per-verb page over a list in which four entries have none, so the
promise is made for ten words and kept for six. The operator learns which is
which by being refused — and the two most likely to be asked about are exactly
two of the four: `ask` is described in that very page as *"the surface, and
the verbs are its shorthand"*, and `start` is the composite an operator reaches
for first.

The refusal also misdescribes what happened. *"no verb named `start`"* is
false in the only sense the operator means it: `start` is a verb, it is listed
one screen up, and it runs. What is absent is a help page, not a verb.

Two candidate shapes, and the first looks right:

- **Give the four doors pages.** `Verb`'s row carries `summary` and `detail`
  already; what the doors lack is `params` that are all strings, which only
  `envelope` needs. A help entry that is a word plus a usage line plus prose —
  with no envelope builder behind it — costs no second implementation of a
  gesture, because it builds no gesture. That keeps one roster, one page
  surface, and the six-row envelope table exactly as it is.
- Failing that, at minimum the refusal must stop saying the word is not a verb,
  and the roster must mark which entries have a page. That is strictly worse:
  it documents the seam instead of removing it.

Found by an operator reading the help and typing what it said.

---

## Re-driven against the 0.1.0 binary, and the count has moved

Every claim above still holds, and two things have changed.

**It is eleven words now, and seven pages.** `enroll` joined the gesture table
since this was written, so the bare `lernie help` roster lists

    workspaces  conversations  transcript  follow  message  nudge  enroll

each with a page, and then `start`, `ask`, `entries` and `help` in the same
column, each with a paragraph and none with a page. The same four refuse, in
the same bytes, at exit 2:

    $ lernie help ask
    lernie: no verb named "ask" — `lernie help` lists every one

and `lernie help bogus` is byte-identical to it. Nothing about the fix changes;
the arithmetic in the title does.

**The same lookup gap has a second face, on the argv path.** `crate::cli::run`
sends anything it does not match to `typed()`, which resolves the first word
through the same six-row-now-seven-row `crate::verbs::find`. So the four doors
also have no ARITY refusal, and excess arguments to one are reported as a
single unrecognised argument whose text is the whole argument list:

    $ lernie entries x y
    lernie: unrecognised argument: entries x y
    $ lernie help a b
    lernie: unrecognised argument: help a b
    $ lernie ask '{}' extra
    lernie: unrecognised argument: ask {} extra

against a gesture verb in the table, which refuses properly:

    $ lernie workspaces extra
    lernie: `lernie workspaces` takes 0 argument(s) and got 1 — usage: lernie workspaces

`entries`, `help` and `ask` are all real verbs, all listed one screen up, and
all three are told they are not arguments this binary recognises — the same
sentence a genuine typo earns, and the same indistinguishability this ball
already names. Giving the four doors rows with a usage line, which is this
ball's first candidate shape, closes both faces with one table.

**bl-81dd is this same defect**, filed from the other end (it names the help
refusal as a lie about the surface). Both were re-verified live; keep this one
as the fuller statement.