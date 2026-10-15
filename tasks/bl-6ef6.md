+++
title = "start mode's hint says no provider yet while the rule beneath it shows a provider and a model"
created = 1792037252
updated = 1792037252
parent = "bl-4446"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: aim a wall, click the bar head's + new (or /new) so the box is in start mode.

Observed: the box's hint reads 'no provider yet — /login or /config, then pick one on the rule' while the rule directly under it shows 'anthropic' and 'claude-sonnet-5' as the worker role. Two surfaces one line apart disagree about whether a provider is chosen. Seen against a fixture world whose wall carries no provider rows; the rule's runs come from the conversation's providers.yaml role and the hint from the wall's provider rows, so each may be right about its own source — but the glass states one fact two ways.

Expected: one answer. If the wall holds no provider row the rule's provider run should say so (or show what the hint names), or the hint should name the role the rule shows and say what is missing (a credential, a wall row). §4.45's principle: state is represented once, and two representations drift.