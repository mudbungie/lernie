+++
title = "the rule's new segment controls take a Tab and show no ring: the keyboard lands on them invisibly"
created = 1791868221
updated = 1791869959
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Absorbed by bl-d74b: every rule selector and the name are links that wear the brand ring while they hold the keyboard (theme::paint::link), and Enter or Space opens a focused selector; keys::tests::focus::a_tab_lands_on_a_ringed_selector_that_enter_and_space_open proves it.