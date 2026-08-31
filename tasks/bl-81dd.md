+++
title = "`lernie help <verb>` refuses the four verbs the help text itself documents"
created = 1788138757
updated = 1788151412
claimant = "OrderJoiner2"
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
`lernie help` prints a page for the six gesture serializations (`workspaces`, `conversations`, `transcript`, `follow`, `message`, `nudge`) and answers `lernie: no verb named "X" — \`lernie help\` lists every one` for `start`, `ask`, `entries` and `help` — the four structural verbs that same listing names in its usage block and describes in a paragraph each.

Two things are wrong with that, and the second is the one that costs.

The sentence is a LIE about the surface: `lernie ask` and `lernie start` are the two verbs an operator is most likely to reach for a page on, because they are the ones that are not a one-line shorthand — `ask` is the escape hatch the README calls 'the surface' and `start` is the only verb that is two gestures. Being told they do not exist is the worst available answer.

And a real verb is INDISTINGUISHABLE from a typo: `lernie help ask` and `lernie help bogus` print the same bytes. Help is the one surface whose whole job is to answer with no engine up and nothing provisioned (its own words), so there is no second place to learn the difference from.

The text already exists — the bare `lernie help` listing carries a paragraph for each of the four. What is missing is the per-verb resolution reaching them, so the fix is a lookup that spans both classes rather than new prose.

Found while installing the seat on a laptop against a local engine (yog bl-17f8); everything else on that exercise passed, including `lernie entries`, an `ask` over an §8.2 entry and the window attaching to it.

---

## Re-verified on 0.1.0, and consolidated into bl-6bda

Still exactly true against the current binary: `lernie help start`, `help ask`,
`help entries` and `help help` all answer

    lernie: no verb named "X" — `lernie help` lists every one

at exit 2, byte-identical to `lernie help bogus`, while the seven gesture verbs
answer a page at exit 0. `enroll` has joined the gesture table since this was
filed, so the listing is eleven words with seven pages.

**bl-6bda is the same defect and is the fuller statement** — it carries the
cause (`crate::verbs`'s table is data, and the four doors cannot be rows of it),
both candidate fixes, and a second face this ball does not name: the four doors
have no ARITY refusal either, so `lernie entries x y` answers `unrecognised
argument: entries x y`. Fix them together; this ball is the sighting from the
help surface and closes with that one.