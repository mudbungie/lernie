+++
title = "the session-artifact rule misses the one session id this repository actually published"
created = 1788146189
updated = 1788146740
claimant = "OrderCensor"
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Found by bl-f468's publication checklist, item 4 — *repository text nobody
committed*. The one pull request this repository has ever had carries an
agent-session URL in its body: a conversation id, which is the last class on
AGENTS.md's *What may never enter a ball body* list and is no more publishable
in a pull request than in a ball.

THE RULE DOES NOT SEE IT. `scripts/leak-rules.sh`'s `session-artifact` pattern
matches an alternation of vendor resource prefixes plus four Claude Code
transcript keys. The prefix actually used by the session URL that was published
is not in that alternation, so the scanner reads the text and says clean —
confirmed by running the real scanner over the real body, which reported no
findings.

WHY IT MATTERS EVEN THOUGH THE TEXT IS NOT IN THE TREE. The gate is what an
author is judged by at the moment of writing, and the same id lands in a commit
message, a ball body and a pull request body from the same habit. Two of those
three ARE scanned — `commit-msg` runs the scanner over the message, and the
store gate runs it over the ball — so the hole is not confined to the one place
it was found.

THE FIX IS A ROW AND A FIXTURE LINE, both directions. Add the missing prefix
form to `session-artifact`, and add a line for it to that rule's fixture under
`scripts/leak-fixtures/` carrying `notreal` — `--self-test` requires every
non-comment line of a fixture to be flagged BY THAT RULE, which is what stops a
dead alternative hiding behind the ones still working. Check `clean.txt` does
not start matching.

WHAT NOT TO DO. Do not scrub the published body. GitHub keeps the edit history
of a pull-request body and serves `refs/pull/<n>/head` forever, so an edit buys
the false assurance a history rewrite buys elsewhere — the same reasoning that
made a squashed initial commit the right remedy upstream rather than a filter.
The remedy for what is already out is nothing; the remedy for the next one is
this rule.

THE UPSTREAM HALF. The rule table is a near-byte-identical port of yog's, and
this hole is in both copies. Fix it here and say so upstream; a divergence in
the table is worth less than the fix.