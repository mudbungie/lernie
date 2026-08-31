# lernie — Agent Operating Guide

You are working in **lernie**, a single binary crate: the **seat** of the
four-component split — the operator's face on a yog server. It dials in over
mTLS, asks and acts, and paints what comes back. It holds no world, runs no
agent and executes nothing.

**READ `docs/DESIGN.md` §0 BEFORE ANYTHING ELSE.** The name `lernie` carries two
eras: through 0.0.x it was the agent-loop engine, which continues as `litany`;
from 0.1.0 it is this seat. The version is the only rule separating them, and
three consequences of it are enforced in this tree.

Read `README.md` first for what lernie is and how to build it. This file is the
discipline that surrounds the work.

## Authorities, and they do not overlap

- **yog's `docs/REMOTE.md` is the PROTOCOL authority.** It is versioned and all
  four components implement against it. lernie does not get a vote on the wire.
- **`docs/DESIGN.md` is lernie's ARCHITECTURE authority** — the role, the
  inherited invariants, the module map. Amend it when reality diverges; never
  code around a stale section, and never implement a deviation silently.
- **`README.md` states the code-style rules**, and `rules/`, `Cargo.toml
  [lints]`, `clippy.toml` and `deny.toml` enforce what a machine can.

Where code and an authority disagree, one of them is a bug. Do not invent a
third answer.

## The gate

`make check` is the complete gate: `fmt-check → lint → coverage`, where `lint`
is `line-cap → leak-scan → clippy -D warnings → rules-audit → cargo deny
check`. The pre-commit hook runs the same targets via `scripts/pre-commit`;
neither restates a step the Makefile defines. Run `make install-hooks` once per
clone — it seats `pre-commit` **and** `commit-msg`.

**And CI is that same target, run by a machine rather than by whoever
remembered.** `.github/workflows/ci.yml` installs the three pinned tools the
gate shells to and runs `make ci`, which is `make check` — it spells out no step
of its own, because a CI file with its own list of steps is a second definition
of green and the two drift within a week. It runs on every push to `main` and on
every pull request. A pushed branch exists only to buy a runner verdict; that is
what the pull-request trigger answers, and the branch is deleted and its pull
request closed in the same breath as the `bl close` that lands the work.

**On a push it runs as a job of `release-plz.yml`, not on a trigger of its
own** (bl-459d), and that placement is load-bearing rather than tidy: crates.io
refuses an OIDC token minted under a `workflow_run` trigger, so a release gated
by *observing* CI publishes nothing, quietly, forever. `ci.yml` therefore has a
`workflow_call` entry point and no push arm — the release workflow calls it
inside its own run and `needs:` the result. Consequence: `ci.yml` has no runs of
its own on `main`, so anything reading them must read `release-plz.yml`.

**All tests must pass and coverage must be 100% before anything merges.** It
does not matter who broke the test.

### The disclosure gate

`make leak-scan` is the disclosure half, ported from yog.
`scripts/leak-rules.sh` is the one definition of what may not be committed —
private keys, vendor tokens, credential assignments, routable IPv4/IPv6/MAC
addresses, absolute paths under any home root on any platform (the synthetic
roots `/home/u`, `/home/op`, `/home/x` are the only account names that pass),
email addresses outside the reserved documentation space, dialogue behind a
speaker label, agent-session artifacts, credential-shaped file paths, and
**content no rule can read**. `scripts/leak-scan.sh` is the mechanism;
findings are truncated to 12 characters, because a finding must LOCATE a leak,
never reprint it into a terminal or a log.

Four properties are load-bearing and none is decoration:

- **It reads index BLOBS, not the worktree**, so the bytes scanned are the
  bytes committed — a leak staged and then overwritten with a clean copy on
  disk is still caught.
- **Unreadable is rejected, not skipped.** `grep -I` silently passes binary
  files, which is the class most likely to carry a dump. lernie's allowlist of
  tracked binaries is EMPTY and it stayed empty when the window's icon arrived
  (bl-8f48): the mark is generated arithmetic emitted as a scalable SVG, which
  is text the scanner reads, so the byte-for-byte pinning test exists
  (`mark::tests`) without an allowlist entry existing. A derivation is added to
  the list only with such a test — and only once something genuinely unreadable
  has to be tracked at all.
- **Both directions, run first.** `--self-test` is the stronger half: every
  rule owns a fixture in `scripts/leak-fixtures/` where every non-comment line
  must be flagged *by that rule* and must carry the `notreal` marker, plus
  near-misses that must NOT be flagged. A leak gate dies by matching nothing
  and passing everything forever; a noisy one dies by being bypassed.
- **`.githooks/commit-msg` runs the same scanner over the commit MESSAGE**,
  which no pre-commit step can see.

Two scopes, because they answer different questions. Bare, it scans the whole
tracked tree — the right question for a commit hook, because the tree IS your
change. `--commit REV` scans what one commit publishes: the blobs it adds or
rewrites plus its message, which is the store gate's question and the only
scope that can read a `-m` note.

**What no hook can promise.** This scans one tree, at the moment somebody runs
it. Old commits, other refs, and anything published elsewhere are outside it —
and a hook is one `--no-verify` from not running at all. That residual is
`.github/workflows/store-scan.yml`'s: a scan of the PUBLISHED store ref, daily,
on dispatch, and whenever the rule table itself changes. It DETECTS rather than
prevents — by the time it runs the material is on the remote and the remedy is a
history rewrite — and what it buys is that the agent writing the ball cannot
switch it off. Prevention is local and bypassable; enforcement is remote and
late; saying both is worth more than a gate that implies otherwise.

### The confinement rules, and the three files they name

`make rules-audit` runs the `rules/` table. Four rules
confine a kind of code to one file, and all four name a file that **does not
exist yet** — which is the point: a confinement rule installed after the first
site is a rule that has to be argued with.

| Kind | Confined to | Rule |
|---|---|---|
| `unsafe` block or `unsafe fn` | `src/sys.rs` | `unsafe-outside-sys.yml` |
| `Mutex` / `RwLock` | `src/state.rs` | `locks-outside-state.yml` |
| Building a child (`Command::new`) | `src/test_support/mint.rs` | `no-bare-command.yml` |
| Forking one (`.spawn/.output/.status/.exec`) | `src/test_support/mint.rs` | `no-bare-fork.yml` |

**Each rule's `ignores` list is the one location authority.** Add a site to the
named file; never add a path to the list. A second confined file is two
inventories, which is no inventory. The three files are rows in DESIGN §5.

Two of the three do not exist, and that is the rule working rather than a gap.
The third names test scaffolding because the seat forks nothing in production
and the crate's one child is the suite performing the operator's own
out-of-channel certificate mint — a rule pointing at an empty `src/spawn.rs`
while the one real site sat elsewhere would not be an inventory. If production
ever needs a child, the location moves in the commit that writes the first site.

The spawn boundary is two rules because building and forking are two contracts:
one decides what environment a child inherits, the other holds the ETXTBSY
window a fork opens on every *other* thread's open write fd. Neither has a test
carve-out — `#[cfg(test)]` is where the fork hazard actually lives.

**The audit is per RULE, not per directory.** It runs each rule alone, by the
id in its own file, against `rules/fixtures/violations.rs`, and fails the one
that flags nothing. That is what makes a rule with nothing in `src` to match
measurable at all: scanning `src` is silent about the four above whether they
work or not. A new rule with no fixture fails on the run that adds it.

## Task tracking

Tasks are `bl` (balls). Run `bl --skill` before using it, and
`bl <command> --skill` before running a command.

- Session start is `bl prime --as YOUR_IDENTITY`, then `bl list`.
- **Claim → work → close, in the worktree.** `bl claim <id> --as ID` prints a
  `work/<id>` worktree; **every edit goes there**, never on `main`. A stray
  edit on `main` is invisible to the squash and is left behind. Always pass
  `--as ID` — never let the model invent a name.
- **The store has a remote and it is the source remote.** Ball bodies publish
  on `balls/tasks` beside the crate, within the same command that writes them.
  Nothing you write in one is private — see below.

## What may never enter a ball body

A ball body is markdown on a git branch that publishes with the source. Nothing
you write in one is private, and the source gate has never seen a byte of it —
`make leak-scan` reads the index of *this* tree, and the store is not in it.
The task store gate below closes the mechanical half; the half no regex can
reach is yours. Write the reasoning; leave out the identity, the chronology and
the machine state:

- **Other people's names, handles and addresses.** Third parties, other
  operators, anyone who did not publish themselves. The maintainer's own
  published identity is fine; every other address is a leak.
- **Verbatim transcript prose.** Operator dialogue, model output, an agent's
  own reply pasted back in. Cite the conclusion and the ball it came from — a
  conversation is content somebody said, and quoting it publishes them.
- **Live machine state.** Process ids, load figures, absolute paths under a
  real home, host and device names. Cite the *shape*, not the instance.
- **Provider auth state.** Who is signed in to what, which credential exists.
  "The account cannot run jobs" is the fact; the provider's sentence about it
  is disclosure.
- **Conversation and session ids.** Vendor resource ids, transcript keys, the
  identifiers of a specific run on a specific box.

### The task store gate

`scripts/lernie-leak-gate` is a **balls plugin** that runs
`scripts/leak-scan.sh --commit` over **the op's own store commit** — the same
script and the same rule table as `make leak-scan`, because two copies of the
rules drift within a week — and exits non-zero. A non-zero exit is the balls
protocol's abort: the op is refused and the plugins that already ran roll back
in reverse, so the store commit is un-sealed before anything can publish it.

It hangs at `<op>.post`, not `pre`. A `pre` plugin runs **before** `bl` writes
the task file, so it would scan the previous state and wave through the very
body being added; `post` is the one window in which the ball exists and has not
yet been published.

It scans the op's COMMIT, not the store. A store checkout is shared and
long-lived: a tree scan there judges every agent for every other agent's text,
so one polluted body refuses every `bl` op in the checkout — `create` included,
so the defect about the wedge could not be filed. The author who writes a bad
body is the one who should be told, at the moment of writing.

Wiring is one act per checkout, and this repo cannot perform it — the plugin
schedule lives in the balls landing (`balls/config`), not in lernie's tree:

    bl install --bin lernie-leak-gate=<repo>/scripts/lernie-leak-gate
    for op in create update claim unclaim close drop; do
      bl conf prepend $op.post lernie-leak-gate
    done

`prepend`, never `append`: plugins run in list order and only the irreversible
belongs last, so the gate must sit ahead of whatever publishes and whatever
squashes. Those six ops are exactly the ones a publisher runs on — *the gate
goes immediately before the publisher, everywhere the publisher runs*. It is
severable: `bl conf remove <op>.post lernie-leak-gate` deletes config, not code.

**What it cannot do.** It stops the accident, not the author: the same agent can
`bl conf remove` it, or commit inside the store clone by hand, exactly as
`git commit --no-verify` defeats the source hook. There is no unbypassable
preventive placement to move it to — a git hook inside the store clone is
strictly worse (untracked, per-clone, re-founded by `bl prime`, absent on every
other box and silently so), and GitHub cannot interpose a check on a direct push
to `balls/tasks`: there is no pull request to require a status check on. The
check the author cannot switch off is `store-scan.yml` above.

## Before a publish

**READ THIS FIRST: THE PUBLISH IS NOW AUTOMATIC, AND ITS TRIGGER IS A VERSION
NUMBER LANDING ON `main`** (bl-459d). `.github/workflows/release-plz.yml`
publishes any manifest version crates.io does not already serve, on every push
to `main`, behind the CI gate. There is no publish *command* left to withhold —
the irreversible act is now `bl close` on a ball that edited `version =` in
`Cargo.toml`, and it fires minutes later with nobody in the loop. **So this list
runs BEFORE the version bump lands, not before some later act.** That is the
whole of what changed; nothing below it got easier.

**The advantage this list existed to keep is spent: 0.1.0 shipped from this tree
on 2026-08-30** (bl-3be4). Until then nothing had been published from here, so
items 1 and 2 had a deadline that had not arrived; it has arrived, and every
commit and every ref reachable at `v0.1.0` is public and stays public. The flag
stopped being what holds a publish at bl-f468 (`publish = true`, the operator
having taken the cutover decision for the four-component split), and the
registry-side setting of item 6 stopped being what holds it at bl-459d — that
setting was satisfied by the workflow above rather than relaxed. What holds the
NEXT one is this list and the person reading it, and there is nothing else.

**The list was RUN, in full, in bl-f468**, and each item's verdict is recorded
in that ball. Two things it found are worth reading before the next run: the
one item that is *not* clean is item 4, and the one that stopped a publish is
item 6. The notes under each item are the checklist's own evidence that it
works.

Two halves, and only one of them is a gate.

**The half a machine holds.** `Cargo.toml` declares an `include` ALLOWLIST —
the crate's own `src` and the two files crates.io renders — never an `exclude`,
because the failure modes are not symmetric: a missing include entry costs a
build, which is loud and reversible, while a missing exclude entry costs a
publication that cannot be recalled. `tests/packaged_files.rs` reads the real
`cargo package --list` and fails on any path outside those classes, in both
directions. **With no list at all this crate packages 298 files**, including
`scripts/leak-fixtures/`, a directory of deliberately fabricated secrets — and
the sibling crate published exactly that way, its whole tree with operator home
paths in it, because its manifest declared nothing.

**The half a person holds, once.** Each item is a one-time judgement whose
remedy is destructive — a history rewrite, a yank, a rotation — so a green
checkbox would be a worse answer than a person looking. None of it is
automated, on purpose, and a commit hook cannot promise any of it: `make
leak-scan` reads the index it is about to commit, and that is the whole of what
it can say.

1. **History.** The gate has only ever seen the trees somebody ran it on. Sweep
   every reachable commit for the same material before the first publish —
   `git log -p --all` through `scripts/leak-scan.sh`, or a checkout of each
   commit scanned in turn. A hit means a rewrite *and* rotation of whatever it
   named. This tree is young and was gated from its founding, which makes the
   sweep cheap rather than unnecessary.
2. **Other refs.** Tags, and any probe branch that was pushed to buy a runner
   verdict and not deleted. `balls/tasks` and `balls/config` are on this same
   remote and publish with it — the store is a ref, not a private sidecar. Run
   `store-scan` by `workflow_dispatch` rather than trusting the schedule was
   alive, and note that it only ever sees the TIP: the store's own history is
   item 1's problem too, and rewriting it invalidates every existing clone.
3. **Commit messages.** `.githooks/commit-msg` covers messages written with the
   hook seated (`make install-hooks`). Messages written elsewhere are not
   covered, and no message is in any tree.
4. **Repository text nobody committed.** Pull-request titles and bodies, issue
   text, review comments, release notes, and the crates.io description and
   metadata. None of it is in the tree.

   **This is the item that was not clean** (bl-f468). The one pull request this
   repository has ever had carried an agent-session URL in its body — a
   conversation id, which is the last class on the *What may never enter a ball
   body* list above and is no more publishable in a pull request than in a
   ball. Two things follow and neither is a scrub. Editing the body does not
   remove it: GitHub keeps the edit history of a body and serves
   `refs/pull/<n>/head` forever, so an edit here buys the false assurance a
   history rewrite buys elsewhere. And the scanner did not catch it — the
   `session-artifact` rule's prefix alternation did not name that form. **The
   rule half is closed** (bl-9fbe): the rule now carries both the bare
   `session` id prefix and the code-session URL path shape, each with its own
   fixture line. The published body is not, and will not be; it is recorded
   here and left alone.

   **The standing ruling — no agent-session URL in this repository's published
   text, anywhere** (bl-9fbe, operator 2026-08-30: *ban them, no reason to
   allow it*). Pull-request titles and bodies, issue text, review comments and
   release notes never carry a session URL or a conversation identifier. The
   harness convention of appending one to a pull-request body is **overridden
   here**: strip it before you open the PR, because a body cannot be
   un-published afterwards. The scanner now reads both forms, so a commit
   message or a ball body carrying one is refused at the moment of writing —
   but a PR body is in no tree and no gate will ever see it. That half is
   yours.
5. **Actions logs and artifacts.** A failed gate prints paths and sometimes the
   offending line into a log that survives the run and is public the moment the
   repository is. The scanner truncates findings to 12 characters for exactly
   this reason; nothing else does.
6. **Already-published versions — 0.1.0 from this tree, and twelve under this
   name before it.** `cargo publish` is irreversible: a yanked version stays
   downloadable, so items 1 and 2 no longer have a deadline, they have a
   history. Audit `cargo package --list` before publishing, not after — the
   guard test judges file CLASSES and never content, and every home path the
   sibling crate published lived inside `src`.

   **What 0.1.0 shipped that it should not have, and cannot take back**
   (bl-3be4): one README paragraph asserting the crate carried `publish =
   false` and that the release decision had not been made. Stale since bl-f468
   and missed by two balls that read the file for other reasons — no
   disclosure, no credential, just a false statement about the crate's own
   status in the copy crates.io serves for 0.1.0 forever. The tree is
   corrected; the published record is not, and cannot be. Read the whole of a
   packaged file, not the paragraph you came for.

   **The name is not new, and that is what stopped bl-f468's publish — and how
   it was answered.** The registry crate `lernie` already exists, carrying the
   engine era's 0.0.x line, and it is set to accept **Trusted Publishing
   only**: a token `cargo publish` is answered `403 Forbidden` however the
   manifest is written, from any box, forever. Nothing in this tree can satisfy
   that with a credential, and the setting was NOT relaxed. What satisfied it
   is `.github/workflows/release-plz.yml` (bl-459d), whose filename is the one
   the crate's engine-era trusted publisher already names — so the only route
   to the registry runs through a green CI gate on `main`, and there is no
   hand-run alternative to fall back to. Read the crate's own record before
   assuming a token will do — `crates.io/api/v1/crates/lernie` answers
   `trustpub_only` without authentication.
7. **The fence.** 0.1.0 is the first version this crate may ever bear. A
   `0.0.z` from this tree would collide with the agent-loop engine's own
   published line and destroy the one rule that disambiguates the two eras
   (DESIGN §0). `src/cli/tests.rs` fails the suite if the number is edited down.

## Never

- Never credit AI or tooling in commit messages, code, or docs.
- Never land a `version =` bump outside a ball that has run the section above.
  Since bl-459d that edit IS the publish: `release-plz.yml` releases any
  manifest version the registry does not serve, on the next push to `main`,
  and nothing asks a second time. The flag stopped being the enforcement at
  bl-f468 (`publish = true`) and the registry setting stopped being it at
  bl-459d, so the checklist is, and it is a person's act rather than a gate.
  There is still deliberately no `make publish` and no local publish path: a
  hand-run `cargo publish` here is refused by the registry anyway (the crate
  accepts trusted publishing only), and a convenience target for an
  irreversible act is how the act happens by accident.
- Never rename `.github/workflows/release-plz.yml`. crates.io matches that
  filename literally against the OIDC claim, so renaming it breaks publishing
  until the registry entry is updated to match — silently, at the next release.
- **Never lower the version below 0.1.0.** That is the fence; a `0.0.z` from
  this tree collides with the engine's own published line. `src/cli/tests.rs`
  fails the suite if it happens.
- Never add a dependency that is not on the approved set. **`Cargo.toml`'s
  `[dependencies]` comment is that set**: rustls with `ring` and no default
  features, `serde_json`, `thiserror` — each landing in the manifest with the
  ball that first LINKS it, never in advance. `eframe`/`egui` are DEFERRED and
  deliberately not pre-granted: they are the largest approval this crate will
  make and they arrive with the window (bl-428f), which also re-derives the
  licence allow-list from the lockfile that results. `clap` is deferred pending
  a verb surface that needs it; `tokio` and `rcgen` are refused outright.
