+++
title = "a role write repaints the rule on the next asker pass, seconds later: post the roles read on the write's receipt so the pick costs one round trip"
created = 1793512642
updated = 1793512642
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Operator 2026-10-31: picking a model on the rule, there is a visible delay before the status rule shows it; a config change should read as instant.

Why it is slow today. Model::pick -> post_model -> tune pushes the `model` act to the outbox. On its good Outcome the seat does nothing to the rule (absorb.rs: 'a role write's is the rule's own segment (bl-a7f5): the roles read repaints it on the next beat'). The rule paints Model::roles, filled only by the standing `roles` read, which the asker re-posts every beat (750 ms) and asks LAST in a pass, after the roster fan over every channel ('one pass of roster fans is seconds', asker.rs). So the pick waits: poster beat + write round trip + litany config + the asker's next pass reaching roles + its round trip.

What stays: the server is the authority; the seat never writes its own row back (tuning.rs: a seat that wrote its own row would be painting a claim the server had not made). No optimistic paint.

Fix: on a good Outcome for MODEL, EFFORT or PRIORITY (the three ops `witnessed` names), post the aimed workspace's `roles` read through the outbox at once (Posted::read, as the dropup does on open: 'posts the read that refreshes that list in the same gesture'), so the repaint costs one round trip. Optionally mark the picked selector in-flight with the dropup's Reload::Asked glyph until the roles answer lands, so the pick visibly took. Test: a model Outcome in the inbox leaves a roles read in the outbox; a refusal leaves none. Amend DESIGN 4.42 item 11 / 4.44 where it says the next beat repaints.