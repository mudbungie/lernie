# lernie

**The seat.** The operator's face on a [yog](https://github.com/mudbungie/yog)
server.

> ## ⚠ The name has two eras, and the version is the fence
>
> **`lernie` through 0.0.x was the agent-loop engine.** That program did not
> retire — it was renamed, and it continues as
> **[`litany`](https://github.com/mudbungie/litany)**. If you are upgrading
> from `lernie` 0.0.x, `litany` is what you want, and its README carries the
> migration: `LERNIE_HOME` becomes `LITANY_HOME`, the XDG harness roots move
> from `.../lernie` to `.../litany`, and the in-workspace mark namespace moves
> from `refs/lernie/*` to `refs/litany/*`.
>
> **`lernie` 0.1.0 and above is this crate**: the seat, severed from yog by the
> four-component split adopted 2026-08-28 (yog's `docs/REMOTE.md` §12).
>
> **The version is the only rule that separates them.** A published record
> cannot be corrected in place, so both READMEs state the fence and both name
> the other era. Read every `lernie` you meet against it: a bare one names the
> seat, and one bound to a `0.0.x` version names the engine at that release.

A seat holds an operator-issued certificate for the machine it runs on and dials
in to an engine somewhere else. It asks the boundary's queries, dispatches its
actions, and paints what comes back.

It holds **no world**, runs **no agent** and executes **nothing**. Every durable
fact of a workspace is the server's; every execution is a foot's. A seat is the
part you look at.

## Why it is a separate program

lernie is one of four components that meet only at the wire — the server
(**yog**), the seat (**lernie**), the agent-loop engine (**litany**, beneath the
server) and the tool-execution foot (**thrall**).

The seat was severed so the machine holding the conversations is not the machine
an operator sits at. A phone, a laptop and a desk can all be seats on one world
without any of them holding it; agents run on the server, in the background,
independent of any seat being attached.

The extraction moved code, not architecture: yog's window had already been a
pure wire client of localhost — real socket, real handshake, real certificate,
everything through the front door.

## Status

**A working seat client, and no window yet.**

```
lernie workspaces                       # every workspace an engine holds
lernie conversations <workspace>        # one workspace's conversations
lernie transcript <workspace> <agent>   # one conversation, entries and tail
lernie follow <workspace> <agent>       # hold the line on the live tail
lernie message <workspace> <agent> <content>
lernie nudge <workspace> <agent>

lernie start <workspace> <goal>         # begin a conversation — two acts, one word

lernie                  # open the window
lernie entries          # every channel this box holds, without dialling any
lernie ask <envelope>   # the same gestures, written out as JSON
lernie help [<verb>]    # what a verb takes — answered with no engine up
```

A verb opens the channel its workspace names — this box's own engine, or one of
the workspaces it participates in elsewhere — over a real mTLS handshake with a
real version preface, carries the envelope across, and prints each reply frame
on its own line. It exits 0 when the last reply says ok.

The verbs are a **serialization** of the envelope, never a second spelling of a
gesture: each builds the object `ask` would have taken and hands it to the same
router. `ask` is the escape hatch for every op the table does not name —
including one this build has never heard of, which the protocol says is not a
version bump.

`start` is a serialization of **two** gestures rather than one, and the only
one: starting is two acts — a `prepare` that stages it and answers the fire's
parameters, then a `prompt` that hands that body straight back with the goal —
so the thing between them is a local, and one word is what holds it. Both reply
streams print; the exit code is the fire's.

**Bare `lernie` opens the window**: the roster grouped by channel, the
conversation list, the chat pane and the composer, painted from a snapshot and
firing gestures through the same doors the command line spends. The composer
speaks to the conversation that is selected and **begins one** where none is,
holding the staged body between the start's two acts — so the window can start a
conversation and not only continue one. Behind it are three threads — the asker
over the standing question set, the poster
draining what a click composed, and the follow lane holding one connection open
on the focused conversation.

**Everything it does is reachable from the keyboard.** Most of that is egui's —
Tab moves focus between the controls and Space fires the focused one — and what
Tab cannot make *usable* is a list, so the arrows walk the roster and the
conversation list, left and right say which of the two they belong to, and
Escape puts a notice down. Moving in a list **selects**, so the cursor and the
selection are one thing and the highlight the pointer paints is where the
keyboard is. Every binding calls the same door the click beneath it calls.

**The frame never dials.** Every read and every act happens off it, and the
frame's whole side is one `settle` at the top of an update: file what landed,
hand over what was composed, publish what to ask next. What to ask is derived
from the window's own state rather than stored, so a click changes the model and
the next question follows from it.

What it paints *from* is the typed reply vocabulary (`src/reply/`,
`docs/DESIGN.md` §4.9), reimplemented off yog's REMOTE rather than shared
through a crate, decoding only what a window renders — eight kinds today. It is
judged by **yog's own generated conformance corpus**, vendored under `corpus/`
by `scripts/refresh-corpus.sh` and replayed both directions, with
`corpus/unreadable/` standing as the ledger of what is not painted yet.

Every assertion about the window reads the **glyphs that reached the glass**
(`src/paint_probe.rs`, `rules/no-hand-rolled-paint-walk.yml`). A galley reports
the string that went in, so a label the toolkit elided to `…` reads back whole
and every assertion against it is blind to truncation. Upstream found that three
times before it became a rule; it arrived here as a rule.

What it reads, all of it put there by the operator's hand and none of it ever
written by lernie:

```
<data root>/wire/                     this box's own channel
<data root>/wire/workspaces/<leaf>/   one channel per workspace held elsewhere
```

where the data root is `$XDG_DATA_HOME/lernie` or `$HOME/.local/share/lernie`.
Each directory holds `ca.pem`, `client.pem`, `client.key` and `address`, plus an
optional `workspace` file naming what that workspace is called on its host.
Certificates arrive out of channel, by the operator's hand; **lernie mints
nothing**, and there is no bootstrap flow and must never be one.

`Cargo.toml` carries `publish = false`. The seat's first release is the
coordinated cutover moment for the whole split and is an operator decision that
has not been made — and `cargo publish` is irreversible, so the first version
under this name at 0.1.0 is the act that fixes the fence in the public record.
The flag is the enforcement rather than a note, because a note is not a gate.

## Build

`make` is the build authority. `make check` is the whole gate and nothing runs a
step it does not:

```
make check     # fmt-check -> lint -> coverage
make build     # debug build
make test      # cargo test
make install   # release build, then the binary into ~/.local/bin
```

`make lint` is `line-cap`, then `leak-scan`, then `cargo clippy --all-targets --
-D warnings`, then `rules-audit`, then `cargo deny check`.

Every tool is pinned, or the gate is not reproducible: rustc 1.95.0
(`rust-toolchain.toml`), ast-grep 0.44.1 (`sgconfig.yml`), cargo-deny 0.20.2
(`deny.toml`), cargo-tarpaulin 0.35.2 (`tarpaulin.toml`).

Run `make install-hooks` once per clone to seat the pre-commit hook.

## The rules

Two are hard and machine-enforced:

- **300 lines** on every source file, inline tests included. Docs and config are
  exempt. `make line-cap` is the one definition of both. 300 is a wall, not a
  target; `make line-cap LINE_CAP=199` lists the pre-split band.
- **100% test coverage.** If it can't be tested, it mustn't be built.

A third is hard and machine-enforced but is not about the code:

- **Nothing discloses.** `make leak-scan` reads the index this commit would
  publish — not the worktree — for credentials, routable addresses, home paths,
  pasted dialogue, session artifacts and content no rule can read.
  `scripts/leak-rules.sh` is the one definition of what counts, and
  `--self-test` proves every rule still bites in both directions before the tree
  is scanned at all. `.githooks/commit-msg` runs it over the commit message,
  which no pre-commit step can see. It scans one tree and promises nothing about
  anything already published — that half is filed, not built.

Beyond those, lernie follows the house **contained Rust** standard from birth:
complexity lives in function bodies, where the compiler catches it, not in type
signatures, where it is viral. No named lifetimes; a `pub fn` returns an owned
concrete type; no generic bounds on a `pub` item; no panic paths outside tests;
no `#[allow]` in prod — policy lives in `Cargo.toml [lints]`, justified in one
place; no `Rc`/`RefCell`. Three more rules **confine** a kind of code to one
file: `unsafe` to `src/sys.rs`, `Mutex`/`RwLock` to `src/state.rs`, and both
building and forking a child process to `src/test_support/mint.rs`. The first
two files do not exist, and that is the discipline working — a confinement rule
names its location before the first site is written, or the first site picks the
location by being written. The third names test scaffolding because the seat
forks nothing in production, and the crate's one child is the suite performing
the operator's own out-of-channel certificate mint.

The `rules/` directory enforces what it can, and `make rules-audit` checks both
directions: the tree clean **and** every rule, run alone by its own id, still
flagging its deliberate violation in `rules/fixtures`. Per rule rather than per
directory, because nine live rules would answer for a tenth dead one forever —
and because the confinement rules have little or nothing in `src` to match, so
their fixture is the only thing proving they work at all.

## Authorities

- **`docs/DESIGN.md`** — lernie's own architecture: the fence, the role, the
  inherited invariants, the module map, and what is deferred.
- **yog's `docs/REMOTE.md`** — the **protocol authority**. It is versioned, and
  every component implements against it. Where this crate and that document
  disagree, one of them is a bug; never invent a third answer.

Tasks are tracked with `bl`. Run `bl --skill` before using it.

## License

MIT.
