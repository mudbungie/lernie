+++
title = 'print the linked brazen pin in lernie --version (e.g. "lernie 0.0.1 (brazen 0.0.4)")'
created = 1785124352
updated = 1785129843
claimant = "Ferrule"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Upstream ask from yog (recorded in yog DESIGN §16.6 W5, filed by the bl-f69b amendment work). The linked-crate skew class (installed bz vs lernie's compiled brazen pin) is invisible to any read-only probe today: `lernie --version` prints only `lernie 0.0.1`; the only speaker of the mismatch is check_bz_version in src/prompt/resolve.rs, reached exclusively from the mutating `prompt` path. yog's W5 capability gate already spawns `bz --version`; if `lernie --version` also printed the linked pin, the gate could compare the two with an ordinary probe and refuse Start with a cause instead of letting every conversation die post-dispatch. One extra token on an existing verb — no new verb wanted.

SHIPPED. `lernie --version` now prints `lernie 0.0.1 (brazen 0.0.4)`.

Mechanism: `cli_version()` (src/cmd/mod.rs, the Cli struct's `#[command(version = cli_version())]`) formats `"{CARGO_PKG_VERSION} (brazen {pin})"`, where the pin comes from `crate::prompt::brazen_pin()` — the exact function the load-time version guard (`check_bz_version`, src/prompt/resolve.rs, ARCH §4.4) already calls, which itself derives from the one `brazen = "=<pin>"` line in Cargo.toml via an embedded `include_str!`. No second hard-coded string; the CLI reader and the guard reader are a bijection on the same fact. clap's `version` attribute takes the computed expression directly (memoized behind a `OnceLock` since clap wants a `&'static str`).

Tests added (src/cmd/tests/surface.rs::cli_version_pairs_lernie_and_the_brazen_pin): asserts the computed string starts with the crate version, extracts the `(brazen ...)` component and asserts it equals `crate::prompt::brazen_pin()` (the guard's own expected pin — bijection, not duplication), and asserts it is actually wired into `Cli::command().get_version()`.

docs/ARCHITECTURE.md §4.4 "Version skew is guarded" updated with a note that `lernie --version` now surfaces the same pin for read-only probes (an embedding host's capability gate, e.g. yog's), deriving from the same `brazen_pin()`.

Gate: `make check` green — fmt-check, clippy -D warnings, 100% coverage (5007/5007 lines), test-install all passed.

ACTION FOR REQUESTER: yog (the asking repo) should be pinged that this shipped — not done from here, yog is a separate repo and out of scope for this ball.