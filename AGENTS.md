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

## Never

- Never credit AI or tooling in commit messages, code, or docs.
- Never `cargo publish`. `publish = false` is the enforcement. The first release
  under this name at 0.1.0 is what fixes the fence in the public record, it is
  irreversible, and it is an operator decision that has not been made (bl-11fc).
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
