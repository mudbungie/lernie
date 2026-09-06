+++
title = "every CLI verb prints one line of raw JSON and there is no rendered form: transcript is one line of tens of kilobytes, carrying each entry twice"
created = 1788673691
updated = 1788673691
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
Round-1 daily lane. lernie 0.1.30.

Every read verb prints its reply as one line of raw JSON under a channel heading. There is no rendered form. A day of CLI use is a day of piping into `python3 -m json.tool`.

    $ lernie workspaces
    (this box's own engine)
        {"kind":"workspaces","ok":true,"rows":[{"agents":4,"attention":4,"config_tip":{"oid":"334456a4…","short_oid":"334456a4"},"kind":"named","running":false,"workspace":"home"},{"agents":4,…

    $ lernie transcript <ws> <agent>
        {"kind":"transcript","ok":true,"rows":[{"body":"…","kind":"delivered","name":"001-user.md","raw":"---\nfrom: user\ndeposited_at:…

`transcript` is the worst of them: it carries every entry AND each entry's `raw` bytes, so one conversation is a single line of tens of kilobytes, twice over. `attention` on twelve conversations is one 4,000-character line. Neither can be read in a terminal.

The seat's own sibling tools set the standard. `bl list` renders; `bl list --json` is the machine form. `yog gesture` prints the reply and a verdict. Here the machine form is the only form.

The design note in the help — "the reply stream prints one envelope per line" — is a statement about the WIRE, and it has been taken as the CLI's presentation. They are different questions: `--json` can keep the envelope exactly as it is today while the default renders.

What a render needs, per verb, is small and mostly already decided by the window: `workspaces` is the roster row the window paints; `conversations` is `name [state] age N waiting` plus the preview; `transcript` is the chat pane's own rows; `attention` already carries a human sentence per row in `says` ("came to rest — your turn", "raised a notify mark; has mail queued and no driver taking it") and prints it inside JSON.

Expected: a default render per verb and `--json` for the envelope. `attention`, `conversations`, `workspaces` and `transcript` are the four that matter for a day of work.

Severity p2: it does not break anything, and it is the single largest drag on using the CLI, which is the whole product on a box with no display.