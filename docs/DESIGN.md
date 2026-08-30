# lernie — DESIGN

**Status: a working seat client, and no window yet.** `lernie ask` opens the
channel a gesture's workspace names — the box's own engine, or one of the
client-side workspaces it holds elsewhere — over a real mTLS handshake with a
real version preface, carries the operator's envelope across, and prints the
reply stream. `lernie entries` says what this box holds without dialling any of
it. The suite proves all of that against a stand-in engine that speaks the
protocol, at 100% coverage.

**What is deliberately not here is the window**, which is what a seat is
*for*. §5 is the ledger of that, ball by ball; nothing in it is a hand-wave and
none of it is claimed to work.

This document states what lernie is, which invariants it inherits rather than
owns, and what it defers. It is a living document: amend it when reality
diverges, and never code around a stale section.

---

## 0. The fence, before anything else

**`lernie` on crates.io carries two eras and the version is the only rule that
separates them.**

- **`lernie` ≤ 0.0.x** was the **agent-loop engine**. That program did not
  retire; it was renamed, and it continues as **`litany`**.
- **`lernie` ≥ 0.1.0** is **this crate**: the seat — the window and its wire
  client, severed from yog (yog's `docs/REMOTE.md` §12, adopted 2026-08-28).

Both READMEs state it, in both directions, because a published record cannot be
corrected in place. Read every `lernie` you meet against that rule: a bare one
names the seat or the split's ruling, and one bound to a `0.0.x` version names
the engine at that release.

Three consequences are structural rather than editorial, and all three are
enforced somewhere in this tree:

1. **0.1.0 is the floor and the manifest says so.** A `0.0.z` release from this
   tree would collide with the engine's own line and destroy the
   disambiguation. `Cargo.toml` carries the reasoning beside the version, and
   `src/cli/tests.rs` asserts the compiled version is on the seat's side of the
   fence — so a hand that edited the number down fails the suite rather than
   the registry.
2. **`LERNIE_HOME` is refused.** That variable named the engine's harness root
   and is `LITANY_HOME` now. Reviving the spelling for a different program would
   make one name mean two things on a box that has both. `src/paths.rs` states
   it; there is no knob.
3. **`$XDG_DATA_HOME/lernie` is taken anyway, and that is deliberate.** A box
   that ran the pre-fence engine may hold that directory from the other era.
   This crate reads exactly two paths under its root — `wire/` and
   `wire/workspaces/` — and the engine's home held neither shape, so the two
   coexist with no file in common. Picking a different directory to dodge a
   directory name would have been the seat conceding the name it was given.

---

## 1. The two authorities, and which owns what

- **yog's `docs/REMOTE.md` is the PROTOCOL authority.** It defines the wire —
  the nouns, the boundary verbs, the identity model, the framing, the
  client-side workspace. It is versioned, and all four components of the split
  implement against it. lernie does not get a vote on the protocol and does not
  restate it: this document cites REMOTE by section and says what lernie does
  about it.
- **This document is lernie's ARCHITECTURE authority** — its module map, its
  local decisions, its own invariants. It governs this component and nothing
  else.

Where lernie's code and REMOTE disagree, one of them is a bug. Where lernie's
code and this document disagree, one of them is a bug. In neither case invent a
third answer; fix the one that is wrong, and if it is the document, fix the
document.

Code style is the house **contained Rust** standard, adopted from birth. The
machine-enforced half lives in `rules/`, `Cargo.toml [lints]`, `clippy.toml`
and `deny.toml`; `README.md` summarises it. The two hard numbers are 300 lines
per source file and a 100% coverage floor.

---

## 2. What lernie is

**The operator's face on a yog server, and nothing else in the harness.**

A seat holds an operator-issued certificate for the box it runs on and dials in
to an engine over mTLS. It asks the boundary's queries, dispatches its actions,
and paints what comes back. REMOTE §2 names it: *"a client connection acting as
an operator face: it asks queries, paints replies, dispatches gestures. A seat
is operator-grade by definition — a face that could not ask is not a face."*

It is one of four components that meet only at the wire — the server (yog), the
seat (this), the engine (litany, beneath the server), and the foot (thrall).
The seat was severed so that the machine holding the conversations is not the
machine an operator sits at, and so that a phone, a laptop and a desk can all
be seats on one world without any of them holding it.

**The extraction moved code, not architecture.** yog's window had already been
a pure wire client of localhost since the ruling REMOTE §1.2 records: the real
socket, the real handshake, the real certificate, everything through the front
door, with the frame's own way into the chokepoint deleted. So there was no
in-process face to unpick — only a boundary to move a crate line across.

## 3. What lernie may never become

Each of these is a thing another component is, and the seat holding it would
undo a severance rather than add a feature.

- **It never holds a world.** Every durable fact of a workspace is the
  server's, on the server's disk, under the file religion (REMOTE §6). A seat
  caching one would be a second authority for it.
- **It never executes.** Tool execution is a foot's, over the same front door
  (REMOTE §12's first invariant, §5.4). A seat that ran a command would be a
  tool host with an operator-grade certificate, which is a thing that exists
  and is not this.
- **It never listens.** A seat dials and is never dialled; the engine never
  speaks first. There is no inbound direction to secure because there is no
  inbound direction.
- **It never mints.** Certificates arrive out of channel, by the operator's
  hand (REMOTE §1.4). There is no enrolment, pairing or bootstrap flow in the
  channel, ever. The suite's own mint is `cfg(test)` and shells to `openssl`,
  which is what an operator does; there is no production caller and there must
  never be one.
- **It adds no wire vocabulary.** REMOTE §3's ban is on a capability that
  exists on the wire and nowhere else. A gesture typed at a seat is the same
  envelope the engine's own inbox carries, answered by the same dispatch.

---

## 4. Inherited invariants, and what this crate does about each

### 4.1 The wire is a transport for the boundary, not a vocabulary (§3)

What crosses is the JSON envelope the boundary already carries — `op` the
discriminant, every parameter a named field — byte for byte, except at the one
place §8.2 says a name is rewritten. `src/envelope.rs` is the whole of what
this crate understands about one, and its module doc is the argument for why
that is three things and not thirty.

### 4.2 The framing (§3)

A big-endian `u32` length, then that many bytes of JSON; a request is one frame,
an answer is N ≥ 1 reply frames followed by a zero-length terminator.
`src/channel/frame.rs`. **The streaming form is not a second form**: every
answer is a stream, so a follow-class read is the general path with more than
one frame in it. `Channel::ask` is written in terms of `Channel::follow` rather
than beside it, so there is one reader and it cannot drift.

### 4.3 The version preface (§3)

Each end writes `{"protocol": <n>}` before it reads the peer's, so neither
waits on the other. A mismatch is fail-closed, names **both** versions, and is
the upgrade prompt: there is no negotiation, no version list and no compat
shim. `src/channel/hello.rs`.

**This is why a separate crate needs it.** Until the split, one crate shipped
both ends of every connection and the wire could not skew. A seat is installed
on the operator's own device and upgraded on that device's schedule while the
engine is upgraded on the server's, so the day the two disagree is a day that
will arrive.

### 4.4 mTLS, and the certificate is the whole authentication story (§1.3, §4)

The engine requires a client certificate chaining to the operator CA; this end
requires the same of the engine and presents its own. No password, no token, no
account — so there is nothing in the channel to phish, rotate or leak, and a
connection that cannot authenticate gets a TLS refusal rather than a reply.
`src/channel/tls.rs`. The provider is named (`ring`) rather than defaulted,
because the default read is a panic path and prod has none.

**The engine's name comes off the address and from nowhere else** (§8): an IP
literal is an IP identity, anything else a DNS name. There is no server-name
knob and so nothing that can disagree with what was dialled.

### 4.5 Material arrives out of channel and is never written (§1.4, §8.2)

`src/channel/material.rs` only ever reads, and answers three ways: nothing
provisioned (an answer, not an error — removing it deletes config, not code),
half provisioned (a refusal naming every gap at once), provisioned. The four
file names are the wire's rather than this crate's, so an operator's act does
not depend on which program was installed.

### 4.6 The client-side workspace (§8.2)

An entry is a directory under `wire/workspaces/<leaf>/` carrying the channel
facts that reach one workspace held elsewhere. `src/channel/entries.rs`.

- **`<leaf>` is the client's name and the fifth file is the host's.** A host's
  namespace is the host's fact and two hosts may both call something `home`;
  the remedy for a collision is a local rename, which is `mv`, never a
  server-side rewrite.
- **Separation is the absence of a mechanism.** Entries share nothing — not
  anchors, not leaves, not addresses — so there is no inheritance and no path
  by which one entry is read through another.
- **A refusal is one entry's, never the set's.** A box serving three engines
  does not lose the two that are fine.

### 4.7 Resolution, and the one place the mapping is spent (§8.2)

`src/seat.rs`. A gesture's workspace name resolves over the entries first; a
name no entry holds — and a gesture naming no workspace — goes to the flat root,
which stays what it has always been: the box's own client relationship, held
without naming it.

**`route` is the only function in this crate that rewrites a name.** One site,
so a gesture cannot cross renamed down one path and unrenamed down another. Two
properties ride with it and the suite pins both: an entry that exists is the
answer to its name **even when it cannot be dialled** (falling through would
send a gesture to the wrong engine on the strength of a missing file), and
where the two names agree the operator's envelope crosses byte for byte.

### 4.8 Port zero is a request, not an address (§8)

A self-provisioning engine writes `127.0.0.1:0` and only the listener knows what
it became — it tells its own in-process window in RAM. A separately installed
seat cannot be told that, so a flat root naming `:0` refuses with the sentence
rather than the raw connect error port zero would otherwise earn.

---

## 5. Module map

| Path | What it is | Cap band |
|---|---|---|
| `src/main.rs` | the process entry: argv in, the environment folded once, a stream and an exit code out. The one `tarpaulin.toml` exclusion, and it is honest because it decides nothing. | small |
| `src/lib.rs` | the crate doc and the module list. | small |
| `src/cli.rs` | the command line as a **pure function**: arguments in, a `Decided` out. No argv, no environment, no streams, no exit. | ~200 |
| `src/paths.rs` | the data root, from two variables and no knob of the seat's own. Neither set is a refusal, never a guess. | ~90 |
| `src/envelope.rs` | the gesture envelope from the seat's side: is it one, which workspace does it name, did the last reply say ok. **One table, not two** — the read answers through the write. | ~150 |
| `src/seat.rs` | which engine a gesture reaches, what it carries there, and what this box says it holds. | ~160 |
| `src/channel.rs` | one wire to one engine: dial, ask, follow. | ~150 |
| `src/channel/frame.rs` | the framing. | ~105 |
| `src/channel/hello.rs` | the version preface. | ~85 |
| `src/channel/tls.rs` | the mTLS configuration. | ~80 |
| `src/channel/material.rs` | what the operator carried here, and what its absence means. | ~110 |
| `src/channel/entries.rs` | the client-side workspaces this box holds elsewhere. | ~165 |
| `src/test_support/mint.rs` | the operator's out-of-channel act, performed by the suite. **The crate's one spawn site.** | ~200 |
| `src/test_support/engine.rs` | the stand-in engine: a real listener, a real handshake, a real preface. | ~150 |

**The three confined files, and two of them do not exist.** A confinement rule
names its one location *before* the first site is written, or the first site
picks the location by being written.

| Kind | Confined to | Rule | State |
|---|---|---|---|
| `unsafe` block or `unsafe fn` | `src/sys.rs` | `unsafe-outside-sys.yml` | **absent, and expected to stay so.** A seat dials a socket, frames JSON and paints; upstream's raw effects are all engine-side. An ask to create this file is an ask to explain what the seat is now doing. |
| `Mutex` / `RwLock` | `src/state.rs` | `locks-outside-state.yml` | **absent, with a tenant already named.** The wire client is one synchronous ask per call and shares nothing across a thread. The off-frame threads (bl-8ed9) are what fill it. |
| Building and forking a child | `src/test_support/mint.rs` | `no-bare-command.yml`, `no-bare-fork.yml` | **occupied, by test scaffolding.** The seat forks nothing in production, so the rule names the one place that does rather than an aspirational file nothing would ever go in. If production ever needs a child, the location moves to `src/spawn.rs` in the commit that writes the first site — never a second entry, because two confined files are two inventories. |

There is **no fork lock**, and that is a fact about this tree rather than a
disagreement with the hazard: ETXTBSY needs a write-then-exec pair, and the
crate's one child is `openssl` off `PATH`, which no test writes. The day a test
writes a program and runs it, the lock lands in the confined file with it — at
the fork, never at the write.

---

## 6. Deferred, with the ball that pays for it

Nothing in this section works. Each row is filed, and each says what it costs.

### 6.1 The window (bl-428f), and the vocabulary it paints from (bl-4174)

**This is the seat's whole reason to exist, and it is the larger half of the
extraction by a wide margin.** It is deferred rather than half-done, and the
reason is a dependency the transport does not have.

A window paints **typed** replies — a roster row, a conversation, a transcript
step, a diff hunk. This crate carries a gesture envelope and reads three things
out of it, which is enough to route and to exit and is not enough to draw
anything. So the reply vocabulary comes first (bl-4174), and it comes
**reimplemented**: yog's REMOTE is the protocol authority, a shared protocol
crate was refused, and the seat implements what it needs exactly as the android
client does. bl-4174's first duty is to answer how much of that vocabulary a
first window actually needs, because most of it belongs to panes that do not
exist yet.

The window itself (bl-428f) brings the largest dependency approval this crate
will ever make, which is why `Cargo.toml` does not pre-grant it: the approval
lands with the ball that links it, and that ball re-derives the licence
allow-list from the lockfile it produces.

It also brings **the paint probe and the rule that governs it**, and that rule
travels for cause. A galley reports the string that went IN, so a label the
toolkit elided to an ellipsis still reads back whole and every assertion against
it is blind to truncation. Upstream found that defect three times before it
became a rule; it arrives here as a rule and not as a memory.

### 6.2 The off-frame threads (bl-8ed9)

A frame that never blocks means no read and no act happens on it: the standing
questions per channel, the acts and their later receipts, the follow lane
holding one connection open on the focused conversation, and the roster composed
as the **union** across channels with every row carrying the channel it came
from — a client-side stamp, so no origin ever crosses the wire.

The transport half is already here: `Channel::follow` hands frames over as they
arrive. What is missing is everything above the socket.

### 6.3 The typed argv gesture (bl-a9eb)

`lernie ask` takes an envelope as JSON today, which is the honest shape for a
transport with no vocabulary. Upstream's seat verb takes a typed argv through
the same reader the depositing seat uses, so the two cannot drift, plus a help
whose subject is the interface and which therefore answers with no engine up.
Gated on the vocabulary, for the window's reason.

**What it must not do is grow a second spelling of a gesture beside the
envelope.** One surface, two serialisations, never two implementations.

### 6.4 The local foot-grade check (bl-f5c2)

REMOTE §4.2's grade is enforced at the engine's chokepoint, fail-closed and in
band, so a seat holding a foot-grade leaf is already refused correctly. Nothing
deferred here is a security property — what is missing is a **diagnosis**, so
that a misconfiguration of this box's own files reads as a sentence about this
box rather than as an authorization refusal from somebody else.

### 6.5 The late half of the disclosure gate (bl-28fb)

The gate here is prevention only: local, and bypassable by whoever runs it. The
standing question — what the tree and the store carry in total — is answered by
a scan of the published ref, and this repository has a published ref from its
founding. The check is simply not written. There is also no CI at all yet, so
`make check` is run by a person or by nobody.

### 6.6 Per-seat UI state (bl-0fba)

REMOTE §7's state never crosses the boundary and is the seat's own. The window
will therefore have durable state on this box for the first time, and the hazard
is written in `src/paths.rs` already: nothing the seat *generates* may sit beside
material the seat cannot replace. This is decided before the window decides it
by being written.

### 6.7 One agreed omission, and it is a finding (bl-4a36)

The envelope's workspace table mirrors yog's typed table exactly — top level, or
one level down inside `prepared` — and the suite pins the agreement. **They
agree on an omission.** The config family carries a workspace inside its
destination and neither table reads it as the gesture's address, so a config act
aimed at a workspace held elsewhere under a local rename resolves to no entry,
goes to this box's own engine, and edits the wrong wall's file. yog's own window
has the same shape, so it is an upstream finding as much as a local one, and the
ball's instruction is to fix both sides or neither: the tables agreeing is worth
more than either answer.

### 6.8 The first publish (bl-11fc)

`publish = false`, and flipping it is not the change. It needs an `include`
allowlist and a guard test over the real packaged file list, and it needs the
publication checklist run by a person — history, other refs, commit messages,
repository text nobody committed, CI logs. Every item is a one-time judgement
whose remedy is destructive, which is why none of it is automated.
