+++
title = "the first publish: the fence in the public record, and the allowlist that guards it"
created = 1788068290
updated = 1788068374
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The manifest carries `publish = false` and states why: the seat`s first release is the coordinated cutover moment for the whole four-component split, and it is an operator decision that has not been made.

When it is made, flipping the flag is not the change. Three things ride with it.

An `include` ALLOWLIST in the manifest — never an `exclude`, because the two failure modes are not symmetric: a missing include entry costs a build, which is loud and reversible, while a missing exclude entry costs a publication that cannot be recalled. And a guard test over the real packaged file list, in both directions, because the allowlist judges file CLASSES and never content.

The publication checklist, run by a person once: history, other refs, commit messages, repository text nobody committed, CI logs and artifacts. Upstream carries that list in full and this crate inherits it unchanged. Every item is a one-time judgement whose remedy is destructive, which is why none of it is automated.

And the fence itself: 0.1.0 is the first version this crate may ever bear, because a 0.0.z release under this name would collide with the engine`s own line and destroy the one rule that disambiguates the two eras.