+++
title = "gate: alignment"
created = 1785124069
updated = 1785124453
claimant = "Capstan"
parent = "bl-d32a"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-d32a"
on = "claim"
+++
PASS, checked against docs/ARCHITECTURE.md, docs/PRINCIPLES.md, docs/TAXONOMY.md.

ARCHITECTURE.md: the change is repo-development infrastructure, not product spec. The only hook the document mentions is the pre-commit one, and only as a runner of `make check` (§ on the command-surface parity check: 'Parity is enforced mechanically under `make check` — hence in the pre-commit hook and in GitHub Actions alike, since both run it'). That claim is untouched: the new hook adds no gate and changes no gate. No architectural invariant is asserted, weakened, or restated.

PRINCIPLES.md: coherent, and specifically supported by two.
- 'Structure over discipline' — 'When a safety property … can be enforced by construction, so the failure it prevents becomes *impossible* rather than merely forbidden, prefer that to a rule maintainers must remember.' Remembering to push after a landing was exactly such a rule, and forgetting it is what left origin/main 82 commits behind for two months. The hook makes the push structural.
- 'Single source of truth' — 'When in doubt, derive don't mirror.' The hook derives what to push from the ref transaction git hands it; it stores nothing, flags nothing, and keeps no record of what has been pushed.
- 'There is no `main`' (One writer per branch) is about the workspace repo model — an agent workspace has no mainline — not about lernie's own git checkout. No conflict.

TAXONOMY.md: no term of art coined. `reference-transaction` is git's own hook name (an external mechanism named, not a coinage); 'auto-push hook' is descriptive prose. The four banned terms from ARCHITECTURE §2.1 — bare 'call', 'turn', 'session', 'compression' in the context-management sense — do not appear anywhere in the added hook, tests, or README text (grepped).

One correction found and fixed under the docs gate (bl-ea25): the hook comment and README justified the mechanism with a census of main's history ('199 of 199 … zero merge commits'), which main invalidated the same day by taking a merge commit. Replaced with the durable fact.