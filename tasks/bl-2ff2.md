+++
title = "Design: message delivery vs blocking await — reminder-shaped children pair with check(), or await grows a timeout (§2.5/§2.11)"
created = 1783490950
updated = 1783490950
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-a277"
on = "close"

[[blockers]]
id = "bl-6306"
on = "close"

[[blockers]]
id = "bl-2868"
on = "close"
+++
Deliverable: tracked doc edit (ARCHITECTURE §2.5 and/or §2.11), no code. Follow-on to bl-ed40/bl-3eea.

Gap: delivery happens only at step boundaries (§2.11), but a parent parked in a *blocking* await(handle) (§2.5) has no step boundary until that child terminates. The motivating recursion — a reminder/babysitter child messaging its still-running parent — works while the parent is stepping, but not while the parent is awaiting the babysitter itself or any other long-lived child. Spec is internally consistent today (nothing promises mid-await delivery) but the workflow guidance is unstated.

Resolve one of:
- Guidance-only: state in §2.5/§2.11 that reminder-shaped children pair with check(handle) polling, never blocking await; a blocked await defers delivery to the boundary after it resolves. Zero mechanism.
- Mechanism: await gains a timeout (or await-any) so a stepping cadence survives long children. New knob — weigh against 'Add scrutiny, subtract mechanism'; only if guidance-only is judged insufficient.

Whichever lands, the delivery-only-at-step-boundaries invariant (§2.11 'the only possible seam') must not weaken.