+++
title = "0.0.1 ships two binary distribution channels (crates.io, GitHub release tarball) that the README documents no path for: no cargo install line, no bz step, and the tarball carries no docs at all"
created = 1785130186
updated = 1785130728
claimant = "stanchion"
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-0aa9"
on = "close"

[[blockers]]
id = "bl-8507"
on = "close"
+++
## What was walked

Three install routes for lernie 0.0.1, each into an isolated prefix with
isolated XDG homes, then `lernie --version` / `lernie new` / `lernie prompt`
on each:

| | (a) `cargo install lernie --locked --root <p>` | (b) `v0.0.1` release tarball | (c) `make install INSTALL_PREFIX=<p>` |
|---|---|---|---|
| documented in README? | **no** | **no** | yes (§Install) |
| binaries laid down | `lernie` | `lernie` | `lernie`, `agent-eval`, `lernie-eval-agent` |
| other files | — | **none** (tarball is one bare executable) | — |
| installs `bz` | **no** | **no** | yes (`install-bz`, no-op at the pin) |
| runs `lernie prime` | **no** | **no** | yes |
| post-install guidance | cargo's "add to PATH" warning | **none** | full banner: PATH, config/data roots, `bz --dump-config` / `bz --login`, `models.yaml` |
| `lernie --version` | `lernie 0.0.1` | `lernie 0.0.1` | `lernie 0.0.1 (brazen 0.0.4)` |
| `lernie new` | works | works | works |
| `lernie prompt` with no `bz` | `adapter subprocess: No such file or directory (os error 2)` | same | same |

Route (c) is flawless end to end — exit 0, three binaries at `0755`, `bz`
recognised already-at-pin, `prime` run through the verb, `install-verify`
passing, and a genuinely useful closing banner. `docs/USER_STORIES.md` US-01
(status **unverified**) can be marked observed for the `INSTALL_PREFIX` form
on the strength of it, modulo the `lernie-eval-agent` binary US-01's
acceptance clause does not yet list.

## The defect

Routes (a) and (b) exist as published artifacts — `lernie 0.0.1` is live on
crates.io and `lernie-x86_64-unknown-linux-gnu.tar.gz` is attached to the
`v0.0.1` GitHub release — and the README, which *is* the crates.io landing
page (cargo auto-attaches `README.md`; `Cargo.toml`'s publish list keeps it),
documents neither. Its §Install offers only three `make install` spellings,
and §Contributing frames the Makefile as the user path: *"Users installing a
release don't need any of this — `make install` from **Quickstart** is the
user-facing entry point."* But a user installing a release is exactly who has
no Makefile.

The concrete stranding, walked:

```
cargo install lernie --locked        # succeeds, ~22s
lernie new ~/work/chat               # succeeds, prints the path
lernie prompt ~/work/chat 'hello'    # adapter subprocess: No such file or directory (os error 2)
```

Three facts route (c) supplies and (a)/(b) do not: that a second binary `bz`
is required at all; what version it must be; and that `lernie prime` founds
the harness root. Only the third self-heals — README §Layout notes `lernie
new` founds the root itself, and it does.

The pin is *stated* in the README (§Providers: *"lernie links the `brazen`
crate (`brazen = "=0.0.4"`)"*, and *"`make install` installs the pin with
`cargo install brazen --version =0.0.4`"*) — but only inside prose about the
Makefile path, ~850 lines in, and a tarball user has no README at all. Nor
can the released binary be interrogated for it: `lernie --version` prints bare
`lernie 0.0.1` there, since the `(brazen 0.0.4)` suffix (bl-c1b9) landed after
the `v0.0.1` tag.

The `v0.0.1` release body is the auto-generated changelog — 20-odd ball
titles, no install instructions.

## Suggested fix

Two lines and an asset, no new mechanism:

1. A README §Install lead paragraph for the binary routes, since it is the
   crates.io page:

   ```
   cargo install lernie --locked
   cargo install brazen --version =0.0.4 --locked   # the pinned provider adapter (§4.4)
   lernie prime                                     # found the harness root
   ```

   with the brazen version rendered from the same `brazen = "=<pin>"` line
   the Makefile's `BRAZEN_PIN` and `crate::prompt::brazen_pin()` already read
   — so it cannot drift (this is the third recurrence of a stale-pin class:
   bl-143e, bl-8c92).

2. Ship `README.md` and `LICENSE` inside the release tarball beside the
   binary.

Related and separately filed: bl-63c1 (the missing-`bz` error message names
neither `bz` nor the fix) — that one is the runtime half of the same
stranding, and fixing either alone leaves the user a thread to pull.

## Severity

Defect. Two published, publicly reachable distribution channels lead a
first-time user to a hard stop, and the product's own docs contain the remedy
in a section those users are told they do not need.

Filed by an outside evaluation pass (wharfinger) walking 0.0.1 from the public
docs only; not claimed, not fixed.