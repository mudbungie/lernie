+++
title = "publish 0.1.0: the act itself, once the registry stops refusing a token"
created = 1788146231
updated = 1788146478
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

---

## The act — DONE. lernie 0.1.0 is published, by the workflow, not by a token.

ROUTE TAKEN: the second one. The registry setting was NOT relaxed. bl-459d
landed `.github/workflows/release-plz.yml` at the filename the crate's
engine-era trusted publisher already names, and the push that delivered that
ball is the push that published — arming and firing are the same act when the
trigger is `push: branches: [main]`, which is what "arm it" means.

WHAT THE RUN DID, five jobs, all green, in one run on the delivery commit
`7884050`:

- `CI / linux` — `make ci` as a CALLED workflow inside this run. Green in 57s.
- `release-plz PR` — answered `release_pr_output: {"prs":[]}`. It proposed
  NOTHING: no PR, no branch, no 0.1.1. The manifest version was already ahead
  of the registry and not yet published, so release-plz read it as bumped
  already and left it alone. The feared "it will propose 0.1.1 because it reads
  0.1.0 as released" did not happen, and the reason it cannot is that
  release-plz compares the manifest against the REGISTRY, not against a tag.
- `release-plz release` — requested a token from
  `crates.io/api/v1/trusted_publishing/tokens`, published, revoked the token,
  and answered
  `{"releases":[{"package_name":"lernie","prs":[],"tag":"v0.1.0","version":"0.1.0"}]}`.
- `macOS artifact (aarch64-apple-darwin)` — see below.
- `prune superseded release branches` — nothing to prune.

## The trifecta, each read back from the public record

**The registry.** `crates.io/api/v1/crates/lernie` answers newest `0.1.0`, max
`0.1.0`, `trustpub_only` still true. The version's own record: created
2026-08-30T17:13:06Z, **`published_by: null`** — no human account published it,
which is the signature of a trusted-publishing release — and `trustpub_data`
naming provider `github`, repository `mudbungie/lernie`, and sha
`788405046ac8b4d0ec9e74114ee1b5babed3a8cf`. That last field is the registry
itself attesting which commit of which repository produced the artifact.

**The tag.** `v0.1.0`, annotated, dereferencing to `7884050` — main's tip.
BARE, not crate-prefixed, and that was a decision (`release-plz.toml` carries
it): the sibling engine crate prefixes because ONE history there holds both
eras of this name, and the engine-era `v0.0.1..v0.0.11` tags are in it. This
repository was founded fresh, holds one crate, had no tags at all, and will
never carry an engine commit — so nothing this tree pushes can collide with
them, and `v<version>` names one thing here and always will.

**The Release.** `v0.1.0`, marked Latest, cut 2026-08-30T17:13:09Z. Its body is
empty, and that is by consequence rather than by oversight: release-plz fills a
release body from the changelog only when it manages the changelog, and
`changelog_update = false` here (`release-plz.toml` states why — every commit
subject in this tree is prose plus a `[bl-xxxx]` trailer, and the sibling
measured what git-cliff's conventional parser does to exactly that shape).

**The rendered page.** Fetched and read, not assumed. The fence banner is the
first thing after the title and renders whole: the two eras, the engine's new
name, the migration it carries, and *"The version is the only rule that
separates them."* The point of the exercise is intact in the published record.

## The macOS artifact — bl-9380's blocker is answered, natively

The `macos-14` job built with the platform SDK already on the image and
`scripts/mac-verify.sh` then READ what came out rather than trusting it:

    Mach-O 64-bit executable, arm64, 6698528 bytes
    platform macOS, minimum OS 11.0.0, built against SDK 14.5.0
    code signature present (ad-hoc; not notarized)
    13 dynamic libraries, all stock

All thirteen are `/usr/lib/*` or `/System/Library/Frameworks/*` — libSystem,
libobjc, libiconv, and the ten frameworks bl-9380 named as the wall a Linux
cross build hits. The five malformed inputs were refused first, in the step
before the build. Nothing in this repository acquired, vendored or pathed to an
SDK. Signing and notarization remain out of scope: the linker's signature is
ad-hoc, and a downloaded artifact's quarantine attribute is a credentialled act
on a mac by whoever publishes.

## What 0.1.0 shipped that cannot be taken back

One README paragraph asserting `Cargo.toml` carries `publish = false` and that
the release decision "has not been made". Stale since bl-f468 flipped the flag,
and missed by two balls that opened the file for other reasons — including this
session's own edit, which corrected the *other* stale paragraph in the same
file and did not grep for the rest.

It is not a disclosure and not a credential: it is a false statement about the
crate's own status, in the copy crates.io serves for 0.1.0, forever. `cargo
publish` is irreversible and a yank does not remove a file. The tree is
corrected and the next version carries the correction; 0.1.0 does not. The
lesson is in AGENTS.md item 6 now: read the WHOLE of a packaged file, not the
paragraph you came for.

## The checklist, and what the tree's movement since bl-f468 invalidated

bl-f468 ran all seven items and its verdicts stand for items 4, 5 and 7. The
three that move with the tree were re-run on the delivery commit:

- **1 (history)** — the two commits added since bl-f468 were scanned by the
  disclosure gate at their own close, and the whole tracked tree was scanned
  again at this delivery: 300 files, no findings, with the rule table's own
  self-test green first.
- **3 (messages)** — both new commit messages went through
  `.githooks/commit-msg`, which runs the same scanner.
- **6 (packaged list)** — unchanged in class and in content. Everything bl-459d
  added is outside the `include` allowlist by construction: two workflow files,
  `release-plz.toml`, `scripts/mac-verify.sh`. `tests/packaged_files.rs` held
  green in both directions through the delivery gate.

Item 2 gained one ref and it is the intended one: the `v0.1.0` tag. No probe
branches, no `release-plz-*` branches — the prune job found nothing because
release-plz pushed nothing.
