+++
title = "the published-ref scan: the late half of the disclosure gate"
created = 1788068285
updated = 1788068285
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The gate here is PREVENTION only, and it says so in its own comments: `make leak-scan` reads the index this commit would publish, the store plugin reads the op`s own commit, and both are local and both are bypassable by whoever runs them.

The question neither can answer is the standing one — what the tree and the task store carry in TOTAL, including whatever predates the gate or arrived past it. Upstream asks that daily, by a scheduled workflow over the published ref, where a hit`s remedy (a history rewrite) belongs anyway. This repository has a remote from its founding and its store publishes to the same remote, so the question is live here and the check is simply not written.

Two pieces: a CI workflow running the same gate on push and on pull request, and a scheduled scan over the published store ref. Both run the scanner already in the tree — never a second copy of the rules, which drift within a week.