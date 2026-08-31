+++
title = "publish 0.1.0: the act itself, once the registry stops refusing a token"
created = 1788146231
updated = 1788146401
claimant = "OrderBellman"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The residual of bl-f468, and it is one command behind one owner act.

bl-f468 ran AGENTS.md's *Before a publish* checklist in full, recorded every
verdict, audited the packaged list by content rather than by class, and flipped
`publish = false` to `true`. Then `cargo publish --locked` was answered:

    403 Forbidden: New versions of this crate can only be published using
    Trusted Publishing

The crate `lernie` is not new — it carries the engine era's twelve 0.0.x
releases — and it is configured to accept Trusted Publishing only. That is a
registry-side setting. No manifest edit reaches it, and nothing in this tree is
missing.

TWO ROUTES, and the operator picks one. They are not equivalent.

- **Relax the setting** on the crate and re-run `cargo publish --locked` from a
  checkout at the trunk. Minutes, no code, and it is the route the greenlight
  assumed. It also means the first release under the new name is the same
  hand-run act the sibling crates' first versions were, which is the argument
  for it: a first publish is not a recurring release and does not want a
  pipeline's failure modes on its first outing.
- **Land bl-459d** and let the workflow publish. The trusted publisher is
  already registered on this crate from the engine era and names a fixed
  workflow filename, so this needs a file at that name and no registry
  registration. Slower, and the first release under the new name would also be
  the first exercise of an untested pipeline.

WHAT THIS BALL IS, WHOLE. Take the route, run the act, and verify the result:
the registry serves **lernie 0.1.0** as the newest version, and the rendered
page carries the fence-stating README — which is the point of the whole
exercise. The name changes meaning at this version, the published record cannot
be corrected in place, and the README IS the disambiguation. Verify it renders,
not merely that it uploaded.

DO NOT RE-RUN THE CHECKLIST FROM SCRATCH. It ran, and its verdicts are in
bl-f468. Two are worth re-reading before the act: item 4 found an
agent-session id in this repository's one pull-request body, whose remedy is
bl-9fbe's rule row and NOT a scrub; and item 6 is this ball. If the tree has
moved since, the items that can move with it are 1, 3 and 6 — history, messages
and the packaged list — and those are three commands, not seven judgements.