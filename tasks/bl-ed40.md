+++
title = "Design: front-door conversation messaging — queue model (any sender can message any conversation; single step-loop executor per branch)"
created = 1783471203
updated = 1783471203
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-a346"
on = "close"

[[blockers]]
id = "bl-5e34"
on = "close"
+++
Design task. Deliverable: tracked edits to docs/ARCHITECTURE.md (+ TAXONOMY.md entries), no code.

Vision (user): a conversation is exactly two things — recoverable on-disk state, and an optional in-flight execution. Any sender (user or another conversation) uses the same front door to message any existing conversation. Generalizes §2.5 'from the subagent's perspective, its parent is indistinguishable from a user' to: every sender is indistinguishable from a user. Dissolves user-reprompt / parent→child steering / child→parent nagging / user-drops-into-descendant into one primitive.

Agreed direction — queue model, not mutex-abandonment:
- Depositing a message and running the step loop are separate acts. Messages are append-only, sender-namespaced on-disk files (per the steps/<conv-id>/ namespacing pattern, §2.2); writers never collide.
- At most one step-loop executor per branch; incumbent finishes its step, newcomer enqueues. Pending messages enter context at the next step boundary (the only structurally possible delivery point: step read-state is a commit §2.3; provider wire requires tool_use/tool_result pairing §2.5).
- No abandonment: aborting a step wastes a billed attempt and muddies §3.5/§4.4 terminal classification.
- Executor liveness observed, not stored: same /proc fd-scan pattern as §2.9/§3.5 — no lock files to go stale.

Must reconcile:
- §2.5 'Subagent conversations are expected to terminate and merge before their parent branch does' (babysitter child that messages a running parent breaks the wording).
- Message to a quiescent branch = existing reprompt/dispatch shape (new conversation consuming branch state); the only new mechanism is delivery into a live execution.
- Provenance: recorded sender (conv-id or user) on each message; recipient treats all senders uniformly.
- Taxonomy entries for the new terms (e.g. 'message', 'steering' if coined — user approval per terminology discipline; 'turn' and 'session' banned).
- 'One obvious path' (§2.5 ¶ on unification): new primitive must subsume, not sit beside, existing dispatch/reprompt verbs.