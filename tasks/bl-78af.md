+++
title = "gate: alignment"
created = 1785459718
updated = 1785459878
claimant = "pedantic-lernie"
parent = "bl-e4ef"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
GREEN. Coherent with the spec docs; the change is a value move inside an invariant the docs already state, and it adds no mechanism.

ARCHITECTURE §4.4 'Version skew is guarded' — verbatim: 'Both readings derive from the one `brazen = "=<pin>"` line in `Cargo.toml` (`crate::prompt::brazen_pin`, above) — never a second hard-coded string.' The bump edits exactly that line. Every derived reader keeps deriving: `src/prompt/pin.rs::parse_brazen_pin` over the embedded manifest, the Makefile's `BRAZEN_PIN` (sed over the same line, feeding BZ_TEST_ROOT, install-bz and CI's cache key via `make brazen-pin`), and the load-time `bz --version` guard in `src/prompt/resolve.rs`. No hard-coded version was introduced anywhere; `src/prompt/tests/pin.rs::the_makefile_derives_the_same_pin` proves the two readers still agree.

PRINCIPLES 'Single source of truth' — upheld and, cross-repo, restored: before this, lernie's declared brazen and the brazen yog pins were two representations of one fact and had drifted. After 0.0.3 publishes with `=0.0.5`, exactly one brazen resolves in yog's graph.

TAXONOMY — no new term of art. 'pin', 'provider adapter', 'version guard' are all already defined; nothing was coined, and no banned term (bare 'call', 'turn', 'session', 'compression') entered the diff.

No architectural deviation to fix in the docs: the docs described this exactly, the number was just stale.