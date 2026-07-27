+++
title = 'print the linked brazen pin in lernie --version (e.g. "lernie 0.0.1 (brazen 0.0.4)")'
created = 1785124352
updated = 1785124352
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Upstream ask from yog (recorded in yog DESIGN §16.6 W5, filed by the bl-f69b amendment work). The linked-crate skew class (installed bz vs lernie's compiled brazen pin) is invisible to any read-only probe today: `lernie --version` prints only `lernie 0.0.1`; the only speaker of the mismatch is check_bz_version in src/prompt/resolve.rs, reached exclusively from the mutating `prompt` path. yog's W5 capability gate already spawns `bz --version`; if `lernie --version` also printed the linked pin, the gate could compare the two with an ordinary probe and refuse Start with a cause instead of letting every conversation die post-dispatch. One extra token on an existing verb — no new verb wanted.