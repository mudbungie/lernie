+++
title = "inline backticks in a model's prose paint raw"
created = 1791871673
updated = 1791872142
claimant = "FixC"
parent = "bl-957a"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Round 2 (bl-0639): fenced code blocks paint in monospace (bl-beef) but inline `code` spans paint with their backticks. Paint a span at MONO without the ticks, as the fence fix did; headings (##) are a sibling defect noted by bl-c059 and may share the pass.