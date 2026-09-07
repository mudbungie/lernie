+++
title = "a wall's roster line elides its rollups at the column's own width: 'home  (named)  6 conversations  5 waiting  running' loses 'running' at 280 points, so the row says less than the wire does"
created = 1788755364
updated = 1788755364
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r3"]
+++
Seen while restyling (bl-d1ae): theme::paint::row elides one run at the roster's width, and roster::wall::line joins five facts into that run. At the policy's 280-point roster the tail ('running', sometimes 'N waiting') is cut to an ellipsis on a real world (the busy fixture), so the state the rule at the left edge says is the only place it is said. Reframe the line: the kind '(named)' and 'N conversations' are the weakest facts and could go second-line or drop; the rollups that carry a state belong first after the name. Keep line() a pure function and the tests that click by it.