# lernie — DESIGN

**Status: a working seat, whole.**
`lernie <verb>` and `lernie ask` open the channel a gesture's workspace names —
the box's own server, or one of the client-side workspaces it holds elsewhere —
over a real mTLS handshake with a real version preface, carry the operator's
envelope across, and print the reply stream. `lernie entries` says what this box
holds without dialling any of it, and `lernie help` says what a verb takes
without dialling anything at all. `src/verbs/` is the gestures typed (§4.10)
and `src/reply/` reads their answers back as the nine kinds a window paints
(§4.9). **`lernie start` begins a conversation** — the start family's two acts,
staged and fired in one process (§4.10). The suite proves all of that against a
stand-in server that speaks the protocol, at 100% coverage.

**Bare `lernie` opens the window** (§4.11): the roster grouped by channel, the
conversation list, the chat pane and the composer — which speaks to the
conversation that is selected and **begins one** where none is, holding the
staged body between the start's two acts — painted from a snapshot and firing
gestures through the same doors `lernie message` and `lernie start` spend. Behind it
are four threads (§4.12) — the asker over the standing question set, the poster
draining what a click composed, and two held lanes: the follow lane on the
focused conversation, and the sign-in lane on the provider row the login pane is
following (§4.24). **Everything the window does is reachable from the
keyboard** (§4.11). **The frame never dials**, and the suite proves the
whole of it in one process: a real listener with a real handshake and a real
preface, the three threads, the settle, and the window, asserted on the glyphs
that reached the glass.

This document states what lernie is, which invariants it inherits rather than
owns, and what it defers. It is a living document: amend it when reality
diverges, and never code around a stale section.

---

## 0. The fence, before anything else

**`lernie` on crates.io carries two eras and the version is the only rule that
separates them.**

- **`lernie` ≤ 0.0.x** was the **agent loop**. That program did not
  retire; it was renamed, and it continues as **`litany`**.
- **`lernie` ≥ 0.1.0** is **this crate**: the seat — the window and its wire
  client, severed from yog (yog's `docs/REMOTE.md` §12, adopted 2026-08-28).

Both READMEs state it, in both directions, because a published record cannot be
corrected in place. Read every `lernie` you meet against that rule: a bare one
names the seat or the split's ruling, and one bound to a `0.0.x` version names
litany at that release.

Three consequences are structural rather than editorial, and all three are
enforced somewhere in this tree:

1. **0.1.0 is the floor and the manifest says so.** A `0.0.z` release from this
   tree would collide with litany's own line and destroy the
   disambiguation. `Cargo.toml` carries the reasoning beside the version, and
   `src/cli/tests.rs` asserts the compiled version is on the seat's side of the
   fence — so a hand that edited the number down fails the suite rather than
   the registry.
2. **`LERNIE_HOME` is refused.** That variable named litany's harness root
   and is `LITANY_HOME` now. Reviving the spelling for a different program would
   make one name mean two things on a box that has both. `src/paths.rs` states
   it; there is no knob.
3. **`$XDG_DATA_HOME/lernie` is taken anyway, and that is deliberate.** A box
   that ran litany under this name may hold that directory from the other era.
   This crate reads exactly two paths under its root — `wire/` and
   `wire/workspaces/` — and litany's home held neither shape, so the two
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
to a server over mTLS. It asks the boundary's queries, dispatches its actions,
and paints what comes back. REMOTE §2 names it: *"a client connection acting as
an operator face: it asks queries, paints replies, dispatches gestures. … A
seat is operator-grade by definition: a face that could not ask is not a face.
lernie is the seat component; the phone app is another."*

It is one of four components — the server (yog), the seat (this), the foot
(thrall), and litany, the agent loop yog runs as a subprocess per verb. The
first three meet only at the wire; litany is not on it, and is named only.
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
  server's, on its box's disk, under the file religion (REMOTE §6). A seat
  caching one would be a second authority for it.
- **It never executes.** Tool execution is a foot's, over the same front door
  (REMOTE §12's first invariant, §5.4). A seat that ran a command would be a
  tool host with an operator-grade certificate, which is a thing that exists
  and is not this.
- **It never listens.** A seat dials and is never dialled; the server never
  speaks first. There is no inbound direction to secure because there is no
  inbound direction.
- **It never mints.** Certificates arrive out of channel, by the operator's
  hand (REMOTE §1.4). The suite's own mint is `cfg(test)` and shells to
  `openssl`, which is what an operator does; there is no production caller and
  there must never be one.

  **This clause used to add "there is no enrollment, pairing or bootstrap flow
  in the channel, ever", and that sentence has been amended** (REMOTE §8.4,
  operator ruling 2026-08-30). Two things were welded together in it and only
  one was the invariant. What may never happen is a **device acquiring its
  identity over its own channel** — a machine holding no material opening a
  connection and being handed some. That stays impossible rather than merely
  forbidden: with no certificate there is no handshake, so an unenrolled box
  performs no channel act of any kind, and this seat still answers nothing and
  still listens to nothing.

  What the second half described was the *surface as it then stood*, and the
  `enroll` act is now on it (§4.13). The seat does not mint it: the mint is the
  server's, on the server's box, over a trust root that already exists. The act
  crosses a channel **already** authenticated at operator grade on material
  that already moved out of channel; the box being enrolled is not a party to
  it and has no channel yet; and what reaches that box reaches it through a
  camera pointed at a screen, which is the same class of act as carrying a file
  on a stick and is mediated by the same operator. The seat asks, and paints.
- **It adds no wire vocabulary.** REMOTE §3's ban is on a capability that
  exists on the wire and nowhere else. A gesture typed at a seat is the same
  envelope the server's own inbox carries, answered by the same dispatch.

---

## 4. Inherited invariants, and what this crate does about each

### 4.1 The wire is a transport for the boundary, not a vocabulary (§3)

What crosses is the JSON envelope the boundary already carries — `op` the
discriminant, every parameter a named field — byte for byte, except at the one
place §8.2 says a name is rewritten. `src/envelope.rs` is the whole of what
this crate understands about one, and its module doc is the argument for why
that is three things and not thirty.

**Where a gesture names its workspace is three holders and one table**
(bl-4a36, and yog's twin bl-523f in the typed table): top level; one level down
inside `prepared` on the two envelopes carrying a prepared start; and one level
down inside `target` on the config family, whose destination *is* its address —
the workspace whose file the act edits. The third row is the one that had to be
argued for: without it a config act aimed at an entry under a local rename
resolves to no entry, falls through to this box's own server, and writes the
wrong workspace's file with nothing painted. The two tables are fixed together or
not at all, because a §8.2 mapping the two ends disagree about is worse than
either answer; `src/verbs/tests/corpus.rs` reads the holders off upstream's own
`shapes.json` as the second opinion.

### 4.2 The framing (§3)

A big-endian `u32` length, then that many bytes of JSON; a request is one frame,
an answer is N ≥ 1 reply frames followed by a zero-length terminator.
`src/channel/frame.rs`. **The streaming form is not a second form**: every
answer is a stream, so a follow-class read is the general path with more than
one frame in it. `Channel::ask` is written in terms of `Channel::follow` rather
than beside it, so there is one reader and it cannot drift.

**A read whose frames REPLACE is whole on its first frame** (REMOTE §14.1,
bl-09e1). Since PROTOCOL 10 a server that can hold answers `attention` as a
sequence — the answer at connect, a frame per change, until a bounded hold
(30 s) ends — and *"frames replace; they never append"*. So `Channel::ask` stops
after the first frame of any op `verbs::queue::REPLACING` names and drops the
line, which is the word that ends the hold. Collecting to the terminator there
cost every one-shot `attention` ask — the window's asker fan, `lernie
attention` — the whole hold, and stalled the asker's single pass behind it. The
list is the one place the knowledge lives; no call site names an op.

### 4.3 The version preface (§3, §3.2)

Each end writes `{"protocol": <n>, "edition": <n>}` before it reads the peer's,
so neither waits on the other. **Only the first key can refuse.** `protocol` is
the MAJOR — it moves on a break and nothing else — and a mismatch is
fail-closed, names **both** versions, and is the upgrade prompt: no
negotiation, no version list, no compat shim. `edition` is what that end can
SPELL, it refuses nothing, and an end that states none is at the floor (§4.9).
`src/channel/hello.rs` — where the number is *included*, never declared: the
repo-root `PROTOCOL` file states it and `build.rs` compiles it in (bl-55b1),
because the release gates that read it are other repositories fetching one path
out of a tree they do not build. The LEDGER — the prose beside the re-export,
saying what each version moved and what this seat did about it — is
`src/channel/hello/ledger.rs`, with the closed 2-to-18 era whole in
`ledger/exact.rs` (bl-c515, bl-aa01). Two homes and neither is a copy of the
other: the file says what the number is, the ledger says why it is that.

**This is why a separate crate needs it.** Until the split, one crate shipped
both ends of every connection and the wire could not skew. A seat is installed
on the operator's own device and upgraded on that device's schedule while the
server is upgraded on its own box's, so the day the two disagree is a day that
will arrive.

### 4.4 mTLS, and the certificate is the whole authentication story (§1.3, §4)

The server requires a client certificate chaining to the operator CA; this end
requires the same of the server and presents its own. No password, no token, no
account — so there is nothing in the channel to phish, rotate or leak, and a
connection that cannot authenticate gets a TLS refusal rather than a reply.
`src/channel/tls.rs`. The provider is named (`ring`) rather than defaulted,
because the default read is a panic path and prod has none.

**The server's name comes off the address and from nowhere else** (§8): an IP
literal is an IP identity, anything else a DNS name. There is no server-name
knob and so nothing that can disagree with what was dialled.

**The grade is read off this box's own leaf before anything is dialled, and it
is a DIAGNOSIS rather than an enforcement** (`src/channel/leaf.rs`, REMOTE
§4.2). A certificate carries one of two grades, spelled as the subject's
organizational unit; a seat is operator grade by definition, and enforcement of
that is the server's, at the chokepoint where the client identity is already
spent for scoping, fail-closed and in band. So a seat holding a foot-grade leaf
is already refused correctly, and nothing here is a security property.

What is missing without it is a **sentence**. That refusal arrives as an
authorization answer from the far end about a fault that is entirely this box's
own configuration, and reading the grade here turns it into a sentence about a
file on this disk. Three consequences follow and the second is the one that
decides the design.

- **It identifies one fault; it never validates.** `Some` only where it
  positively read `OU=foot`; `None` for everything else, bytes that are not a
  certificate included. A walk that refused what it could not read would be a
  second, weaker certificate parser standing in front of rustls, refusing
  leaves the server would have accepted — an outage bought with a diagnostic.
  Default-operator is REMOTE §4.2's own rule, and reading it any other way here
  would be this end inventing a policy the authority does not have.
- **It is a DER walk, not a byte search**, because the structure is the point:
  the issuer carries a common name too and it comes first, so a scan for the
  common-name object identifier answers the operator CA's name for every leaf
  on the box. `subject` is located relative to the serial number rather than at
  a fixed index, so a v1 and a v3 certificate take one path.
- **It reads the leaf `tls::identity` already parsed**, inside the one place a
  channel is built and before a socket exists, so it costs no second read of
  the file and there is no path to a dial that skips it.

The sibling foot component does the mirror of this and fails **closed** —
its obligation is to carry a foot leaf and refuse anything else — which is not
a disagreement: the two ends answer different questions.

### 4.5 Material arrives out of channel and is never written (§1.4, §8.2)

`src/channel/material.rs` only ever reads, and answers three ways: nothing
provisioned (an answer, not an error — removing it deletes config, not code),
half provisioned (a refusal naming every gap at once), provisioned. The four
file names are the wire's rather than this crate's, so an operator's act does
not depend on which program was installed. Two more may stand beside them,
optional as a pair — the rendezvous material, read the same three ways one
level down (§4.40).

**The remedy is two remedies, and which one is the CALLER's fact**
(`material::Whose`, bl-ad7f, bl-5cbe). Every refusal about absent material used
to carry one sentence, and it was the visiting box's: *mint an extra client
leaf on the host that issued it and carry it here*. That is the first thing
this binary ever says to a new operator, and on a single-box install it is
advice that does not work — the leaf already exists, because the server's own
boot minted `ca.pem`, `client.pem`, `client.key` and `address` into its own
wire directory, so the act is a **copy** — while a `WIRE_LEAF` leaf is
registered in no workspace besides. So the flat root and an entry now answer
different sentences, and the fact that decides which is the reader's rather
than the directory's: `channel::entries` is reading a directory somebody made
for another box's server, `seat::route` is reading this box's own channel, and
`material::read_dir` is told.

**And both name the four files.** *Carried here by hand* said what to do and
never what to carry: the filenames appeared in this repository's README and in
no message the binary ever printed, so a user who had not read the README
could not act on the sentence at all. yog's own `wire-certs` output is the
model — it lists each path and says what to rename it to.

**The assignment goes BEFORE the verb** (bl-5cbe). `WIRE_LEAF` is an
environment variable that `yog wire-certs` reads, not an argument it parses, so
the printed spelling `yog wire-certs WIRE_LEAF=<name>` earns an unexpected
argument and issues no leaf. It is the one line a new operator copies.

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
- **A refusal is one entry's, never the set's.** A box serving three servers
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
send a gesture to the wrong server on the strength of a missing file), and
where the two names agree the operator's envelope crosses byte for byte.

**And it answers what this box calls the channel it chose** (`seat::Routed`,
bl-c70d). The name was computed here and discarded, so every caller that files
an answer had to guess at it again — and the poster's guess was the *aim*,
which is where a gesture was composed rather than where it went. The name now
comes out of the one place the choice is made, whether or not anything opened:
a leg that never crossed still has a sentence to paint, and the section it
belongs under is the one it would have crossed on. Nothing re-derives it, which
keeps §8.2's mapping spent exactly once.

**The fallthrough is not silent, and a name with no reader does not take it**
(bl-d574). The flat server's own workspaces are named and held nowhere else, so
a seat cannot know that namespace without asking and §8.2's fallthrough stands
— but two things around it are this seat's own. A gesture whose **op takes no
workspace** is naming a *channel* and nothing else (`lernie ask
'{"op":"workspaces","workspace":"<leaf>"}'` is how an operator asks one entry
for its roster), so a name no entry holds has no downstream reader to refuse it
and falling through would answer `ok` from a channel nobody named; it refuses
here, naming what it looked for. Which ops those are is read off
[`crate::verbs`]'s one table, never listed again. And where a named gesture
*does* fall through to a flat root that holds nothing, the refusal is about the
NAME — not about `wire/`, a directory the operator never asked about, whose
remedy (mint a second leaf) lands in the same wrong place. Both sentences name
the channels this box holds and offer the rename (`mv`) ahead of the mint.

### 4.8 Port zero is a request, not an address (§8)

A self-provisioning server writes `127.0.0.1:0` and only the listener knows what
it became — it tells its own in-process window in RAM. A separately installed
seat cannot be told that, so a flat root naming `:0` refuses with the sentence
rather than the raw connect error port zero would otherwise earn.

### 4.9 The reply vocabulary is reimplemented, it decodes only what it paints, and it grows only (§8, §3.2, §9.7)

`src/reply/`. REMOTE §8 is explicit that *"what the seat reimplements is this
document … no shared protocol crate was created and none should be"* — a shared
crate would make the versioned authority a dependency for one of the four
components and an authority for the other three. So the reply spellings are
read off REMOTE and implemented here, exactly as the android client does.

**Thirty-four kinds, because thirty-four are painted.** The server's reply
surface is
forty-odd variants and most of them belong to panes that do not exist here.
What is carried is the roster (`workspaces`), the conversation list
(`conversations`), one workspace's role tuning (`roles`, §4.17), the
conversation itself (`transcript`), the live tail
(`follow`), the conversation's records pair — the steps its loop took
(`steps`) and what its worktree holds (`files`), both §4.18's — the decision
queue (`attention`) and the receipt that raises a row onto it (`flagged`),
both §4.19's, the window's own three — one server's verb table (`help`), what
a needle found (`search`) and the trail (`ops`, §4.27) — the ball pane's four:
the whole box's binding table (`balls`), the fleet board (`board`), one workspace's
own balls (`workspace-balls`) and the branch it tracks them on (`marks`), all
§4.30's — the login pane's three: what a workspace
can sign in to (`providers`), what one of its rows offers (`models`) and one
sign-in run (`login`), all §4.24's — the machines registered in one
workspace (`clients`, §4.28), the config pane's two — the lineages a workspace
holds (`lineages`) and one file's bytes with the settings its schema found in
them (`config`), both §4.30's — a captured
run (`outcome`), the detached advance's receipt
(`nudged`), the start family's two — the staged body (`prepared`) and the
minted name (`started`) — a new box's material (`enrolled`, §4.15) and
whether the box is wired (`doctor`, §4.21), plus the refusal envelope, which is not a kind at all. A kind nothing renders is a kind nobody has to carry, and the ball that
lands a pane is the ball that adds its kind.

**`roles` is the worked example of that last sentence** (bl-4a2c). It sat in
the ledger directory from the PROTOCOL 6 refresh — a perfectly good frame of a
kind nothing painted — and the commit that moved it to `corpus/answers/` is the
commit that built the pane. The diff of that move is the record of what the
release added, which is exactly what the ledger is for. The records pair
(bl-2cf7) is the pattern's second spend: `steps` and `files` made the same
move in the commit that built §4.18's pane, `help` and `search` made it in the
commit that built §4.21's two (bl-40ec), the sign-in family's three
(`providers`, `models`, `login`) made it in the commit that built §4.24's pane
(bl-e3c5), `clients` made it in the commit that built §4.28's (bl-e53c), the
spine pair (`rail`, `governing`) made it in the commit that built §4.29's half
of the records pane (bl-b52c), `config` and `lineages` made it in the commit
that built §4.30's pane (bl-5c53), the balls family's four (`balls`, `board`,
`workspace-balls`, `marks`) made it in the commit that built §4.31's ball pane
(bl-d2af), the deeper three (`agent`, `step`, `inbox`) made it in the
commit that built §4.32's half of the records pane (bl-3257), the fleet's
three (`armed`, `science`, `work-diff`) made it in the commit that built
§4.33's pane (bl-a43a), the capability boundary's two receipts (`answered`,
`floored`) made it in the commit that built §4.34's controls (bl-bce2), and the
trail's two (`acked`, `trail-cleared`) in the commit that built §4.35's acts
(bl-b8f7) — which between them spend the ledger's last conversation read and
leave the ledger holding only shapes addressed to a different kind of
client. `doctor` arrived after that and made the same move in the commit that
built §4.21's doctor pane (bl-9bbb).

**And a shape can sit in `unpainted/` because it is addressed to a different
KIND OF CLIENT, which is a third reading of that directory** (bl-e53c).
`invocations` is a tool host's own work queue: it is follow-class, it *drains*,
and the queue it answers is the one addressed to the certificate that asked
(REMOTE §5.3). So a seat asking it would learn nothing about any foot — its own
queue is empty by construction — while a box whose seat and foot share one
identity would have that machine's work handed to the window and never
completed, and the foot's own read refused in band as a second reader (§5.1).
`advertise`, `complete`, `invoke` and `capture` are the same class, and the
server's own help table says so: all five are `surface: machine`, so no seat
owes any of them a control. Their ledger lines are not balls waiting to be
built, and `src/verbs/clients.rs` states it where an implementer will look.

**`PROTOCOL` is a MAJOR, and an addition no longer moves it** (yog's
`docs/REMOTE.md` §3.2; yog bl-e598, here bl-aa01). Through 18 the integer moved
for any change to an existing shape, a gained field included, and every client
re-pinned for each one — five bumps in a week, all but two of them additive.
It now moves only on a **break**: a field removed or re-typed, a meaning
changed under a spelling in use, a field the server newly requires. The preface
is unchanged — strict equality, fail-closed, no negotiation — and what changed
is how rarely it can refuse.

**An addition ships under the same major, stamped with an EDITION.** The old
integer line continues as the edition line: `corpus/shapes.json` now stamps
every field PATH with the edition it appeared at, and carries the **floor** —
the edition the current major was cut at. Every path at or below the floor is
required on every server of this major; every later path is optional to READ.
Two numbers, two questions, and only one of them can refuse a connection.

**This seat's copy of the three facts is `src/channel/edition.rs`** — the floor,
this build's edition (the newest stamp in the corpus it vendors) and the ledger
of the paths above the floor. Vendored on `PROTOCOL`'s own precedent rather
than derived at build time, and held equal to `corpus/shapes.json` by its own
tests, both directions: a stamp with no row is as red as a row with no stamp.
The corpus is `cfg(test)` apparatus and is not packaged, so the constants are
what a released binary carries and the test is what makes them true. **The
ledger is EMPTY at the cut**, which is the design and not a gap — the floor is
where the major was cut, so at that instant nothing is above it, and a row is
added by the same commit that re-vendors the corpus which grew the path.

**A fourth fact is vendored beside them and it is a WARNING with a deadline**:
`DEPRECATED`, upstream's list of what it will remove. REMOTE §3.2's window rule
removes a field or a shape only at a major, only after listing it in a
PUBLISHED yog release, and never within three releases of that listing — so a
name arriving in that list is the one moment a consumer can act with three
releases in hand, and a vendored copy is what makes the arrival visible rather
than a line in a generated file nobody diffs. It is `cfg(test)`, because
nothing at runtime consumes a deprecation: a released binary reads the fields
it reads, and the whole product of the list is the diff that carries a name
into it.

**The preface states the edition beside the major** (`src/channel/hello.rs`),
and it refuses nothing. An end that states no edition is at the floor, because
that is the least a server of this major can be; absent, mistyped and out of
range are one answer for that one reason. The server's is kept on the channel,
where `Channel::spells(shape, key)` answers whether it can spell one field —
and a `false` there says *this server cannot say*, which is a different
sentence from the field's default and the only reason to ask. A control whose
fact is stamped above the far end's edition greys rather than painting the
reassuring answer. **Nothing greys today** because nothing is above the floor
today; the first consumer is whatever control lands on the first post-floor
field.

**The reader is GROWS-ONLY by rule and not by luck**, which is what makes the
above safe. Four clauses, each answered where it lives:

- **An unknown key is ignored, structurally.** Every reader indexes by key, so
  a field the server added rides through untouched. Rung 4, and it costs
  nothing.
- **A post-floor key absent reads as its DEFAULT, and the default is the fact
  before the field existed.** Written at the field readers
  (`src/reply/fields.rs`), because that is where a required reading is chosen,
  and asserted by projection below rather than by a promise per field.
- **An unknown WORD in any vocabulary becomes a named catch-all carrying the
  word**, rendered honestly — never the nearest known word, and never a refusal
  of the row. Rung 3, and it is why every vocabulary here is either an enum
  with an `Unknown(String)` arm or a bare `String`: both carry the word and
  paint it as itself.
- **The reply `kind` stays STRICT, by name.** A reader asks only what it
  paints, so a kind nothing renders is refused rather than guessed at, and the
  refusal names it (`crate::reply::read::unpainted`). Rung 2, and the one place
  the opposite rule holds.

**And it is replayed, not merely asserted** (`src/reply/tests/editions.rs`).
The vendored frames are what the NEWEST server writes, which is the one server
a seat never has trouble with; the two it actually meets are made here out of
that frame and the stamps beside it. **Projection** deletes every key stamped
later than `e` and replays at every `e` from the floor to this build's edition
— an older server's frame. **Word mutation** sets every string-typed path
upstream spells, `kind` excepted, to a token no build has heard of — a newer
server's vocabulary. Neither may cost a refusal. Both are vacuous the day a
major is cut, so the walkers are unit-tested against shapes the corpus does not
have (`src/test_support/corpus/editions.rs`): an arm that waits for the first
addition to be exercised is an arm nobody has evidence about.

**A valid frame of a kind nothing paints is a PARITY fact, never an
unreadability**, and that is why `corpus/unpainted/` is split out of
`corpus/unreadable/` (bl-aa01). The two land on one `Read` arm and are opposite
claims: one is a defect — on the wire, or in a decoder — and the other is a pane
nobody has built. The assertion the second owes is therefore not *it refuses*
but *it refuses BY NAME*, so that class asserts the exact rung-2 sentence; and
each direction is structural rather than listed — `unpainted/` holds only
vendored frames, `unreadable/` only frames this repository wrote, upstream's
codec being unable to emit a malformed one. `parity.toml` records what this
seat does not RENDER and never what it refuses.

**The number moves for the wire; the fields move for the panes**, and the two
are decided separately. The closed era is a hundred worked examples of that
sentence and **the ledger is where they stay**: PROTOCOL 2 through 18, one
entry per bump, in `src/channel/hello/ledger/exact.rs` — closed at 18, and
there will never be a nineteenth. This section does not restate them; one fact
has one home, and the copy is what rots. Two of them are rules rather than
entries, and those are kept here:

- **A protocol bump is not a shopping list** (bl-d774, bl-f0ef). PROTOCOL 4 put
  four new facts on the wire and that release consumed exactly one, because one
  pane painted the row it hung on. The other three rode through unread and
  their shapes stayed in the ledger directory until §4.19 landed the queue that
  paints two of them. **The interval between the arrival and the pane is the
  ledger's whole product** — one diff, naming exactly what a release added.
- **A field can be load-bearing while the glass stays exactly as it was**
  (bl-38d4). PROTOCOL 7 gave every `reply/help` row a `surface`, and its
  consumer is a GATE rather than a pane: the roster §4.16's parity assertion
  judges this window against. It does not loosen this section — the vocabulary
  still decodes only what the window renders — and what reads the field is the
  corpus walk, which reads every frame whatever this build does with it.

**A shape whose meaning moves under an unchanged spelling is the one drift no
replay can catch**, and it is now the class the major exists FOR. PROTOCOL 5 is
the plainest case: `oid` stopped naming the fork commit and started naming the
followed lineage's head, so the bytes stay well-formed, the fixture stays in
the class it was already in, and every mechanical check this repository owns
stays green while a future pane paints a plausible number that has been wrong
since the bump. Projection cannot see it, mutation cannot see it and a
signature cannot see it — only prose can, so that trap is written in the
ledger where whoever lands the pane will be reading.

**And a bump reaches the WRITE direction even when it paints nothing.** 6 added
two ops rather than only a field, and a new op is free by §3's rule — the peer
refuses an unknown one in band, by name. Free of a *decode* is not free of an
obligation: `src/verbs/tests/corpus.rs` replays every op in the vocabulary,
including the sixty-odd this seat has no word for, and asserts each routes by
the address upstream's own signature says it carries. That is where a miss
would cost something and would do it silently — a gesture routed by a slot the
seat does not look in goes down the wrong channel — so the request half is
checked on every re-vendor whether or not a pane moved.

**The staged body is carried whole, which is rung 4 read in the WRITE
direction** (`src/reply/start.rs`). `prepared` is the one reply a seat hands
straight back: `prompt` fires the body `prepare` answered. So the reader keeps
the object verbatim beside the two fields it reads — the workspace, because a
fire is addressed, and the goal, because a rung with a prefill composed one —
and a start staged with a work target, a birth lineage or a banner origin this
build paints none of still fires with all three. A seat that re-encoded its own
reading would drop every parameter it had not learned yet, and the dropped
parameter is not a missing badge: it is a conversation born in the wrong
directory off the wrong config. The cost is `Eq` on `Read` and `Reply`, which
nothing in this crate spent.

**The decode policy is four rungs and the module doc is its one statement.**
Shape refuses; an unknown **kind** refuses, naming itself, which is REMOTE §3's
own in-band correction; an unknown **token** inside a
row keeps its word and paints as itself, because refusing there would drop a
whole readable listing to avoid one word; an unknown **field** is ignored
structurally, which is the other half of §3's rule that a new field is not a
protocol bump — and under §3.2 it is not even an edition a reader has to know
about. Rungs 2 to 4 are three of the grows-only clauses above, stated as a
posture rather than as a version contract; the fourth, an absent post-floor key
reading as its default, is the same policy meeting a field that arrived late. Nothing is a panic path and nothing is defaulted to a known
neighbour: a token painted as a word it is not is a lie, where a token painted
as itself is merely unstyled.

**One deliberate divergence from the server's own reader**, recorded at the
site (`src/reply/stream.rs`): a `follow` frame's delta token takes rung 3 where
the server's decoder refuses. The two readers are not doing one job — the
server's reads bytes the server wrote, so a mismatch there means its own codec
drifted, while this one is the last reader of somebody else's answer and
refusing would throw away an accumulated turn while the operator watches the
tail move.

**A follow frame is an APPEND, and the fold is the lane's** (REMOTE §5.5,
PROTOCOL 2; bl-5b07). The rule has no flag and no case — *absorb every frame of
a read, in order, onto an empty fold* — and this seat implements it where a
read begins and ends: `crate::offframe::follow::tick` holds one
`Stream::default()` for the whole of one held connection, absorbs each frame
into it and hands `Model::live` the accumulation. So a read boundary needs no
field, no flag and no representation anywhere; it is a local variable's scope,
and two reads of one conversation cannot run into each other because the second
starts holding nothing. `Stream::absorb` is the server's own operation copied
rather than re-derived, and its contract is the equality
`fold(a).absorb(fold(b)) == fold(a ++ b)`, asserted in
`src/reply/stream/tests.rs` rather than stated in prose.

Two consequences worth naming. The decode of a live frame moved to the lane's
own thread, which is where the crate's one high-rate read already runs, so
`Said::Live` crosses the lock already read; the settle's stale-tail drop is
unchanged, being a comparison on the stamp and not on the content. And **the
wire spelling did not move** — the body is still `{"delta", "text",
"thinking"}` — so no field signature can see this change, which is why the
protocol integer carries it and why this paragraph exists at all.

**The fold paints only in the place of the committed read's `Streaming`
entry** (bl-f6b5; `src/ui/chat/rows.rs` `rows`). The two routes describe one
open response, and the server's streaming entry rides exactly that (REMOTE
§5.5), so a read with no `Streaming` entry is the read saying the response
closed: the fold in `Model::live` then has nothing to be newer than and is not
painted. Nothing clears the field when the lane ends, deliberately — the lane's
end is not the commit (the committed read can land while the lane is still
open), and a fold appended after the committed `model` entry painted the answer
twice for as long as it stood. The cost is one ask cadence: a turn already
streaming that no pull read has yet seen paints its tail from that read on.

**The frame carries the TOOL WINDOW beside the prose** (REMOTE §5.5, PROTOCOL
15; bl-183b): two entries a call, appended by the rule the prose already obeys.
An operator watching an agent administer their machines saw a thinking marker
and two sentences while eight commands ran on two boxes, and that is the
omission this closes — the box is precisely the thing that cannot be inspected
afterwards. Three facts about the shape and one about this seat:

- **`exit_code`'s presence is the status.** Absent is a call in flight, and
  there is no arm for *complete, status unknown*, so two readings cannot
  disagree.
- **The closing entry restates neither name nor input**, being keyed by
  `tool_use`, and **the machine is already in the name**: REMOTE §5.1 presents
  a routed tool as `<client>_<tool>`, always, so this end never takes that
  composition apart and never joins the registry instead — the registry answers
  where a call *would route now*, which is a different question.
- **`tools` is required, empty included**, and this seat reads it strictly:
  absent would make *this build has no window* and *nothing ran* one shape,
  which is the reassuring answer on exactly the build that cannot tell.

**And the window is what makes a fold necessary on the command line too.** The
window pane already held one (`offframe::follow::tick`, above) and paints the
accumulation; the CLI prints frames as they land, because a terminal is already
a fold and re-printing the accumulation would be quadratic. But a line already
printed cannot be amended, so a closing entry rendered alone could only say
`exit 0` about an invocation it can no longer name. `seat::follow::held` now
holds a fold of its own for exactly that: `Stream::appended` absorbs the frame
and answers it back with the append's prose and its calls WHOLE, and
`render::tail` is the one entry point in the rendering that takes state — the
caller's, whose lifetime is one read.

**The corpus is a directory, not a test table** (`corpus/`, with its own
README), and **the frames in it are not this repository's** (bl-6b8e). yog
generates a wire conformance corpus from the boundary that *is* the protocol
authority (yog bl-32cb, REMOTE §3) and commits it; `scripts/refresh-corpus.sh`
vendors it here verbatim. A seat that judged its own decoder by fixtures it
wrote itself would be checking what it already thought of, which is the one
thing a reimplemented vocabulary cannot afford.

**The layout reconciles two shapes and keeps the better half of each.** The
frames and their stamps are upstream's, whole; the **directory is still this
seat's assertion** — `answers/`, `refusals/`, `unpainted/`, `unreadable/` —
because a classification is a decision this end makes and never one copied in.
So the refresh does not classify: a shape already filed goes back where it was,
and a shape yog has grown lands in `unpainted/` as a new file the diff shows. A
silent pass is the outcome the arrangement excludes. There is still no
manifest and no expected-value sidecar; a frame captured off a live server
still drops in as a bare file, which is how the malformed and rung-3 readings
upstream's codec cannot emit are held.

**Vendored, not read from a checkout, and the reason is not taste.** This crate
must build and its suite must pass on a box that has never held a yog checkout,
and a test that reads a path nobody configured is a test skipped everywhere but
one machine. What vendoring costs is staleness, and the **protocol stamp** is
what that is paid with: every fixture carries the version its fields last moved
at, `shapes.json` carries the corpus's own, and a corpus ahead of `PROTOCOL`
fails in a sentence naming both numbers — the same sentence the version preface
owes an operator (§4.3), for the same reason.

**Both directions of the wire, and both directions of the discipline.** The
request half (`corpus/request/`, judged by `src/verbs/tests/corpus.rs`) is
upstream's whole request vocabulary: every frame must decode as a gesture this
seat can **route** — the workspace slot read off `shapes.json`'s own signature
rather than off the seat's rule, so it is a second opinion — and every frame
the seat's encoder can compose must round-trip byte for byte, with the ones it
cannot recorded by count and reason. And the replay fails a directory that
enumerates nothing, a fixture carrying no frame, a file no directory claims and
a vendored file `shapes.json` does not name, the same two-direction discipline
`rules-audit` and `line-cap` hold.

### 4.10 The typed argv surface is a serialization, never a second implementation (§3)

`src/verbs/`. REMOTE §3 is the rule — *"one dispatch surface, N serializations,
never two implementations"* — and this is the second serialization the seat
carries. A verb builds the envelope `src/envelope.rs` already defines and hands
it to the same `seat::ask`; `ask` stays the escape hatch for every op the table
does not name, including one this build has never heard of, which §3 says is not
a protocol bump.

**The table is declarative, and that is what keeps it one implementation.** Each
row is a word and its parameters in order, all of them named strings, so one
builder serves every verb and there is no per-verb arm to drift. A gesture whose
parameters are not all strings is **not** added as a special case — it goes
through `ask` until there is a reframe that keeps the one table, because the arm
that would carry it is precisely the second implementation the rule forbids.

**`flags` is the first of those reframes, and bl-9fd1 is why it was taken.** A
gesture parameter that is a `bool` had no typed spelling at all, and the one
the wire actually carries is `stop`'s `children` — the control an operator
reaches for when something is wrong. Measured: on a conversation with children
the bare stop does not stop it. The root's driver dies, a child's driver
deposits into it, and it is running again five seconds later, having spent
796,522 further tokens; only the cascade holds, and it was reachable solely by
hand-writing the envelope. The seat printed the server's own offer — *the
server offers nudge, stop with its children* — in the records pane, and then
spelled no word that reaches it.

**A flag is a boolean field spelled as its own name, typed after the
parameters.** It is still one table and one builder: `Verb::flags` is a list of
field names, a trailing word that matches one writes `true` under it, and a
trailing word that matches none is refused **by name** rather than by counting
— a tail of the right length and the wrong word is a different mistake from a
tail of the wrong length. **Absent is absent, never `false`**: this seat does
not assert into a field nobody touched, and `src/verbs/tests/corpus/emits.rs`
declines a frame that spells one `false` for the same reason.

**Why a flag and not a fourth door.** `start`, `prepare`, `prompt`, `effort`
and `priority` are doors because each carries a shape the table cannot hold —
a nested object, a string-or-null, a payload rung. A trailing boolean is not
one of those; it is a field whose value is *whether the word is there*, which
is exactly what a table of field names can express. Making `stop` a door would
have moved a row out of the roster to gain nothing, and the next boolean the
wire grows would have moved a second.

**And the window spells it now** (bl-3686). This paragraph used to say what
the flag did not close — *"the window's `stop` controls fire the bare form, and
the records pane still prints the offer beside no control"* — because a cascade
wants an arming and that is a pane's work rather than a serialization's. The
pane built it: §4.32's control sits on the offers line it was printed beside,
armed by the conversation's own name, and it goes through **this** door
(`crate::verbs::stop(.., true)`) rather than a second one. The composer's
`stop` and the row menu's still fire the bare form, and that is the placement
§4.20 rules rather than a gap.

**The table and the roster are not one list, and neither count is written
here** — two were, and both rotted the way every restated count in this suite
rots; `src/verbs/rows.rs`'s `TABLE` is the one home.
The *roster* is the gestures whose replies §4.9 paints, and it grows with the
paint surface: the ball that lands a pane adds its kind and its gesture in the
same breath. The *table* is the subset a word can spell, and it trails the
roster by the gestures that cannot be rows. **Three modules hold them and each
is the same rule applied**, and no count is written here — `src/verbs/`'s own
files are the home, and the count that used to stand in this sentence was
already wrong by one when bl-28a4 read it. `src/verbs/start.rs`: `prepare`
carries a payload rung and `prompt`
carries a prepared body, and a nested object is not a word an operator types, so
what argv types instead is the composite below. `src/verbs/tuning.rs` (§4.17):
`effort` carries a level that is a string **or null** — null being the whole of
what *off* means, so a row would have to send a fifth word the boundary refuses
by name — and `priority` carries a bool. `src/verbs/trail.rs`: `ops` carries a
**number**. Each is exactly the case the paragraph
above refuses to special-case, and each is a typed door with no row.

**A gesture with no row still needs a WORD, and `ops` is where that was
learned** (bl-28a4). Having no row is a statement about the table; it was
being read as a statement about the surface, and `lernie ops` answered
*"unrecognised argument: ops"* — the sentence a typo earns, for a gesture this
binary builds an envelope for and paints an answer to. So the class the
`src/verbs/doors.rs` table holds is **a word whose grammar the gesture table
cannot express**, not merely *a word this binary answers itself*: `start` composes two
gestures, `ask` composes any, `entries` and `help` compose none, and `ops`
composes one whose parameter is a number. All five are a word, a usage line and
prose with no envelope builder in the table behind them, which is the whole of
what kept the doors from being a second implementation.

**And the depth is DEFAULTED, which is the half of this worth stating as a
rule.** The wire refuses an `ops` envelope without `max`, so the word cannot
leave the field out — but *this seat already knows what to ask for*: `DEPTH`
is the figure the window's own trail pane spends, and it is deliberately larger
than a screen because what an operator is looking for is the row before the one
that broke. A word that refused without a number would be making an operator
answer a question the wire asked the SEAT. So `lernie ops` is the ordinary
reading and `lernie ops <max>` is the override, and the two faces ask with one
figure from one constant rather than two that can drift. A depth that is not a
number is read in `cli::run` and refused there, on `ask`'s body's and
`enroll`'s grade's own division: decided entirely by what was typed, the
caller's typo, earns the usage, costs no connection.

**A staged body CAN be re-fired without `start`, and the word for it is `ask`**
(bl-28a4). The complaint was that a prepared prompt has no seat spelling
outside the composite, so a seat holding a staged body could not fire it. It
can: `prompt` is an ordinary envelope, `lernie ask` carries any envelope
verbatim, and it is **routed by the same one table every other gesture is** —
`envelope::slot_mut` reads the workspace one level down inside `prepared`, so a
hand-written fire goes down the channel that body names and is re-addressed
into the host's spelling on the way, exactly as the composite's second act is.
What has no typed WORD is the nested object itself, and that is this section's
opening rule rather than a gap in it.

**What `start` adds is the two things a second process cannot hold.** The
local between the two acts — the staged body, held while the fire is composed —
and the goal JOIN: the server composes a prefill for every rung that has one
(the path rung's §3.3 target preamble), and the composite sends the prefill and
the operator's words as one goal, which is what a seat with a composer does by
editing. A hand-written fire carries whatever goal it is given and no prefill,
so a `prompt` typed by hand off a path-rung `prepare` starts a conversation
without the preamble its rung meant it to have. That is the residual, and it is
a fact about what a one-shot word can hold rather than about the vocabulary.

**The two arguments the seat settles itself are `enroll`'s grade and its
destination** (bl-07b9, bl-1554). The second is `--into <dir>`, the one optional
word in this surface, and it is read in the same place for the same reason: a
directory on THIS box is decided entirely by what was typed. A tail that is
anything else refuses naming that word rather than falling through to the verb
table, whose arity sentence is about three arguments and says nothing about a
fourth. The usage line stays computed from the row's parameters and stores
nothing (`Verb::usage`); the word is stated in the row's own `detail`, which is
what `lernie help enroll` prints.

**And `model` reads before it writes** (`src/seat/model.rs`, bl-1e5a). The
word accepted an id the same seat could prove does not exist, answered `ok`,
wrote it into the workspace's `providers.yaml`, and the workspace was broken until
somebody read a step record — the next deposit died and the failure reached the
operator truncated, on a row, a turn later. The documented rule is the
server's and it is right: *a model id is validated by the wire at the first
live model call*, because a server holding a second opinion about a
provider's catalogue would be a second authority on somebody else's table. It
is not an argument for the seat staying silent when it has just been handed a
word it can check against a list it already knows how to fetch — `models` is a
row in the same table, one round trip, off the same channel.

**It warns and never refuses**, for the reason the server does not validate
early: a provider's list moves under this seat, a cache is stale, a row that
cannot be asked answers nothing. So the assignment goes through and what is
bought is that the operator learns it now, in full. **Every way the read can
fail to answer a listing says nothing at all** — a refusal, a kind this build
cannot read, an empty row, an unreachable channel — because turning a
provider's silence into a refusal would make an unreachable row a reason not to
write a file the server is willing to write.

**The whole offered list is named, not a guess at the nearest.** It is the same
round trip, it is exactly what `lernie models` would have printed, and a
similarity score is a mechanism with no input: an operator knows which one they
meant the moment they see the row.

**The one argument the seat settles itself is `enroll`'s grade** (bl-07b9).
`grade` is a closed set of two words the boundary defines (REMOTE §8.4) and
this binary already holds them — `lernie help enroll` says so — so a typo used
to cost a round trip and come back `unknown grade "OPERATOR"`: true, and naming
neither word that would have worked. It is read in `cli::run` for exactly the
reason a hand-written envelope is: decided entirely by what was typed, the
caller's typo, earns the usage, costs no connection. **It is not a second
authority on grades** — the server still decides what a grade means and whether
this box may ask for one at all, which §8.4 makes unknowable here — and the two
words are read off `ui::Grade`'s own list rather than a second copy of it.

**The structural words are DOORS, and they have pages** (`src/verbs/doors.rs`,
bl-6bda, bl-81dd; `ops` joined them in bl-28a4). `start`, `ask`, `entries` and
`help` cannot be rows of the gesture table for the reason above; the defect
was that the help surface did not know it. The usage listed eleven words,
`lernie help <word>` answered a page for seven, and the other four were refused
with *"no verb named `ask`"* — false in the only sense the operator means it,
byte for byte what `lernie help bogus` earns, on the one surface whose whole job
is to answer with no server up. The same lookup gap had a second face on argv:
`lernie entries x y` and `lernie help a b` fell through to *"unrecognised
argument"*, the sentence a typo gets, about words the usage lists one screen up.
A door is a word, a usage line and prose with **no envelope builder behind it**,
so it costs none of the second implementation the gesture table's own doc
objects to. Two things ride with it. Its arguments are STORED where a gesture's
are computed — a gesture's parameters are envelope fields, so its usage is
derived from what must not drift from it, while a door fills no envelope and its
arguments exist only to be printed, including the shape a list of field names
cannot spell (`help`'s optional `[<verb>]`). And the usage's own door section is
now derived from that table, where it used to carry the four paragraphs by hand
beside a lookup that could reach none of them.

**`lernie follow` is a serialization of a read spent REPEATEDLY, and that is
the third shape on this surface** (`src/seat/follow.rs`, bl-3dca, bl-f076).
Nothing new crosses the wire: what goes out is `agent` and then `follow`, the
boundary's own envelopes, asked as many times as the conversation takes to
come to rest. `follow` stays a row of the gesture table — the window's own lane
spends it and the corpus round-trips it — and what the CLI word adds is the
LOOP, exactly as `start` adds the local between two acts. `enroll` is the
precedent for the arm: a row whose word is answered ahead of the table because
what the seat does with the answer is not printing it.

**Two defects, one cause.** The word was one `ask`: one held connection, its
frames collected, printed at the end, and the exit code read off the last
frame. On a **quiescent** conversation that blocked 31 seconds, printed one
newline and exited 1 — byte for byte what a failed dial, a refused certificate
and a dead server look like, and those are what a first-time user suspects,
when nothing at all was wrong. On a **live** one it returned mid-turn every
time, because the server ends a held read at the STEP boundary (REMOTE §5.1)
and a turn is many steps: three consecutive reads of one working conversation
ended after 3, 8 and 6 seconds. Watching an agent was `while true; do lernie
follow …; done`, invented by the operator and losing whatever landed between
two reads.

**The rule: hold the line until the conversation RESTS, or until the user
quits.** The server's step boundary is not the end of anything an operator
asked about, so a read that ends there is re-asked; what ends the word is the
conversation coming to rest, which is a fact the server already answers
(`agent`'s `state`). **The state read comes first**, which is what fixes the
quiescent case at no cost: a conversation already at rest opens no held
connection at all, so the half-minute of silence is not shortened — it never
happens — and the word exits 0 with a line saying which state it rested in and
what makes it move again.

**An unknown state ENDS the watch.** §4.9's rung 3 paints an unrecognised token
as itself, and the safe reading here is *this seat cannot say that this is
working*: ending and printing the word beats holding a connection open forever
on a state nobody here understands.

**Mail waiting is NOT rest** (bl-87ab, bl-3ecd). `message` (or `start`) then
`follow` is the pair every operator types, and it answered *"nothing more will
arrive until it is nudged or messaged"* in zero seconds — about a conversation
that took a lease twelve seconds later and then ran for another seventy-eight.
The deposit had landed and no driver held the inbox lock yet, so the state read
was right about the instant and the sentence it produced was false: something
more was going to arrive, and nothing further was going to be asked of the
operator. The recovery — sleep, then ask again — is written nowhere.

The server has no field for *about to start*, and inventing a guess here would
be a second opinion about a reading it already takes. What it does have is the
deposit itself, so **the state read alone was never enough**: a rest whose
`inbox` still holds mail is a turn that has not begun, and the watch holds
through it exactly as it holds on a live one. It says once that it is waiting,
because a silent hold is the thing this word was fixed for and a quarter-second
drumbeat of the same line would be worse than the silence; Ctrl-C is the way
out it always was. Three properties keep it honest. It costs **one read on the
path that was wrong and nothing anywhere else** — a conversation genuinely at
rest has an empty inbox and answers as fast as it ever did, and a working one
is never asked. An inbox this seat could **not read is no mail**, on every way
the probe can fail, because the state read has already said rest and holding
somebody's connection open forever on an unanswered question is worse than the
sentence being fixed. And the waiting line **names the state the server gave**
rather than hiding it, since that reading is true and is the one the operator
will see again in `conversations` a second later; what it adds is the fact that
makes the at-rest sentence false.

**A HELD conversation is not at rest either, and its ending is its own**
(bl-3a1f; yog bl-58bb, PROTOCOL 18). The same correction as the mail above, one
noun over and the other way round. The capability control parks a tool call
*before* the executor is entered, so the state read says `quiescent` and the
watch answered *"nothing more will arrive until it is nudged or messaged"* — false
in both halves. Something more arrives the moment the operator answers, and
neither remedy named is that answer; it said so at the exact moment the person
reading the line was the thing the conversation was waiting for, and on a foot
lane every call to a non-shell tool is held, which makes it most of the endings
rather than a corner.

**The fact was one field away the whole time**: the `agent` row this loop
already reads to decide rest carries the hold mark — the same three facts
`reply/follow`'s parked entry carries — so nothing new is asked and nothing new
is computed. A park is a distinct KIND of rest and the difference from the mail
case is *who the conversation is waiting for*: mail nobody has taken holds the
line, because something is coming with nothing asked of the operator, and a
park ENDS it, because nothing is coming until they act. The line names the
call, the control's own reason, and `answer` — spelled with the address the
watch was aimed at, because a remedy an operator has to complete from memory is
one they complete wrongly. It asks no inbox: mail behind a park is still behind
the park.

**Under `--json` this seat narrates nothing** (bl-87ab). `--help` promises *"the
frames exactly as they crossed, one envelope per line, which is what a script
wants"*, and this word broke it on the one line that mattered: every frame of a
watch was an envelope and the line saying the watch was OVER was prose, so a
reader calling `json.loads` per line died on the terminator. One rule now
covers both sentences the word writes for itself — in the machine form neither
is said, and what ENDS the watch is the `agent` frame that ended it, printed as
it crossed. It carries the state in a field, which is what a script wanted from
the sentence. The elapsed is this end's own fact and a script has its own
clock; the rendered form keeps it.

**This is the one place the product is written as it arrives.** A held read
that printed only at the end would not be a watch, so `seat::follow` takes a
sink: `src/main.rs` hands it a printer and the suite hands it a `Vec`. The
decision stays entirely in the library, where a test reads it back, and the
stream stays the entry point's — which is the same division that earns
`main.rs` its place as the coverage floor's one exclusion.

**And the rendering of a follow frame is what LANDED** (§4.37,
`render/workspaces.rs`). A frame is an append (REMOTE §5.5) and a terminal is
already a fold, so the appended text is printed in order, one frame a line, and
folding here to re-print the accumulation each frame would be the same answer
at quadratic cost. A frame that carried no content at all — the
`{"delta": "thinking"}` a reasoning-heavy stretch sends over and over — renders
as the delta alone, which is a heartbeat: printing the frame's own shape there
said the same eight characters forever, and printing nothing is the silence
this word was fixed for.

**Under the prose the same frame prints the TOOL WINDOW** (REMOTE §5.5,
PROTOCOL 15; bl-183b), which is the half an operator watching an agent
administer their machines is actually watching. Each entry is a whole line —
how it ended, what ran and on which box (§5.1's `<client>_<tool>` composition
carries the machine), and the input the server already bounded — because a
terminal cannot go back and amend the line it printed a minute ago. **The rows
ride the gutter** (§4.37's column, bl-293d): what a tool is doing is the seat
narrating the run rather than the model's answer, and on this lane the two
arrive interleaved a line at a time. That is
also why this one read holds a fold where every other rendering is a pure
function of one frame: the closing half of a window restates neither name nor
input, so without the fold a landed call could only be said as `exit 0` about
an id. `render::tail` is the entry point that takes it, and the fold's whole
lifetime is one held read.

**`lernie start <workspace> <goal>` is a serialization of BOTH acts, not a
third gesture** (`src/seat/start.rs`). Nothing new crosses the wire: a
`prepare` goes out, its answer is held in a local, and a `prompt` carrying that
answer goes out after it — which is exactly what the window holds in its model
(§4.11). Both reply streams print, in order, because both are the server
answering; the exit code is the *fire's*, so a stage that answered no staged
body exits non-zero with its own frames on stdout. The one outcome two acts
have and one does not — a stage that landed and a fire that never left this box
— gets its own sentence, because the workspace exists and nothing is running,
and the remedy is to type it again (yog §8.1: the steps are convergent).

**Two rungs of the three, and the third is a picker** (bl-4371). yog's §3.4
gives the payload three: bare, a work directory, a ball. This seat composes
**bare** and **path**; the ball rung needs a project, a picker and the §3.5 join
states, so it is unbuilt rather than unreachable and arrives with the surface
that composes it — a seat that guessed a rung would found a claim nobody asked
for.

**The path rung is a third argument to `start` and no new door.** The rung is
said outright and never inferred, which is upstream's own rule for this
payload, so the seat carries **one `Option<String>`** — the target — and the
rung word follows from it. A second word saying which rung would be a fact the
operator has to keep in agreement with the first.

**A path this seat does not check.** The directory is on the SERVER's box and a
seat may hold no such path at all, so a path that is not there is refused over
the wire in the server's own words. That is the same division §4.10 draws for
`enroll`'s grade in the other direction: the grade is a closed set this binary
holds, and a filesystem on another machine is not.

**What the ball measured is why the rung is worth an argument.** Same model,
same box, same minute, one goal each, the only difference being the rung. The
bare rung named the directory in the goal's prose: the agent's shell obeyed the
prose while its patch tool wrote into the agent worktree, and it thrashed
through three compactors and four million tokens to leave an empty report in
the wrong tree. The path rung took eight steps, spawned nothing, and wrote the
file where it was asked to. A rung sets the driver's working directory; prose
in a goal is a request the tools are free to ignore.

**And the fire carries the prefill.** A rung with a prefill composes a goal —
the path rung's is the §3.3 target preamble, verbatim — and upstream's own help
states the seat's half: *"To fire a prefill with words of your own, send the
two joined as one goal — that text is the reply's `prepared.goal`, and editing
it is exactly what a seat with a composer does."* A one-shot process has no
composer, so `crate::verbs::start::goal` is where the editing would have been:
the prefill, a blank line, then what was typed. The bare rung prefills nothing,
so the ordinary case takes no arm.

**Positional and context-free, unlike the server's own line.** yog's line reader
is terse because a seat with a focus supplies the address; REMOTE §8.5 says a
seat with no selection *"spells its targets out"*, and a one-shot process is
that seat. Copying the line's grammar would mint a selection type that is always
empty. The corollary is the verbatim payload: the line takes a message's content
as its whole tail because a line has no quoting, argv does, so arity here is
exact and the shell is what makes a sentence one argument.

**A refusal points at the page; it does not print it** (`cli::verdict`,
bl-b232). Every refusal used to carry the whole usage under its diagnosis —
the fence, the sixty-odd verbs, all four door pages, about 110 lines — so on a
terminal the one line that mattered scrolled off the top before the operator's
eye reached it. That is backwards for a message whose whole value is the word
or the byte offset it names, and it bites hardest on `ask`: an operator
composing an envelope by hand is in a loop where the malformed case is the
COMMON one, not the exceptional one. The structural half is unchanged and is
what made this a one-line fix — the tail is appended by the constructor and
never at a call site, so a refusal written later cannot forget it — and what
the tail says is now `lernie help` and `lernie help <word>`. The list is what
`help` is for.

**Help's subject is this binary rather than a world**, which is why it is
answered in the seat with no dial, no server and no material — a binary an
operator cannot learn to use until the hard part already works is a poor binary.
There is exactly one help: `lernie help`, `--help` and `-h` are one text, whose
verb section is derived from the table rather than restated, and `lernie help
<verb>` is that verb's page.

**Where the parse of a hand-written envelope now happens.** In `src/cli.rs`,
not in `src/seat.rs`. Whether a body is a gesture is decided entirely by what
the caller typed, so it belongs in the pure function where the refusal is a
value a test reads back and where it costs no connection — and it makes a typed
verb and a hand-written `ask` arrive at the seat as the same value, which is the
property the whole surface exists for.

### 4.11 The window renders a snapshot, and the paint layer can testify about it (§7, §8.2, §9.7)

**Superseded for the LAYOUT by §4.42 (bl-7859): one column, a status rule, a prompt line and overlays. What stands below is the record and the model-side rulings; where a sentence here says where a thing is painted, §4.42 says otherwise.**


`src/ui/`. **Three panes and a notice bar since §4.39 folded the middle column
away** (bl-46e5, built in bl-b9a3): the roster — an accordion of servers, each
open server's workspaces under it and the AIMED workspace's conversations under their own
workspace's row — the selected conversation, and the composer. It was four, the
second being a pane of the aimed workspace's conversations; what it painted is now a
level of the first, and `src/ui/convs.rs` is what a row is rather than a pane
— with seven more that COVER the conversation when they are open
(§4.15's enrollment, §4.17's tuning, §4.18's records, §4.19's decision queue,
§4.20's unmaking and §4.21's two window-level reads), asked as one question
rather than seven (`Model::covered`). Every one of
them is a function of the [`Model`] it is handed.

**The frame never dials.** Nothing under `src/ui/` opens a socket, reads a file,
blocks or waits; the one thing a frame *produces* is `Model::outbox`, the
gestures a click composed, drained by whoever can send them. A frame that posted
its own act would be a frame that waits on one, and a window that waits is the
failure a seat has no excuse for. What fills the model is §4.12.

**It fires through the verb table.** A composed deposit is built by
`crate::verbs` — the same rows §4.10's command line spends, through a door whose
arity is its signature — so a click and a typed command build one object and
there is no second spelling of a gesture anywhere in this crate.

**Below the floor the layout takes a second SHAPE: one column at a time**
(`src/ui/shell/policy.rs`; bl-dfda). The yield below is the whole of what the
policy used to say, and it ends: past the point where the list pane is on its
own floor, nothing yields and the conversation goes under `CHAT_FLOOR`. At a
phone-shaped viewport that was three columns of about 120 points each, with
every line in every one of them wrapped to two or three words, and a band of
the window painted by no panel at all. The answer is not more yielding — there
is none left to do — it is the **covering-pane idiom read across the whole
layout**: below the width at which the yield can still keep the conversation
its floor, the window shows one `Column` and a bar naming both, which is a
surface you navigate to, act in, and come back from exactly as §4.15's
enrollment and §4.17's tuning pane are.

**There are TWO columns and two variants since bl-b9a3** — `Servers` and
`Conversation`. The fold did not add a case to the policy; it took a term out
of it. `widths` yields one list pane rather than two, its worth being `CONVS`'
because its widest row is still a conversation's headline with a preview under
it, and `shape` turns over where that one pane on its own floor can no longer
keep the conversation `CHAT_FLOOR` — 140 points narrower than the turn two
panes made.

Four things follow and each is a decision rather than a mechanism:

- **The line between the shapes is the yield itself**, not a second constant.
  `shape(window)` asks `widths` whether the conversation still gets
  `CHAT_FLOOR`, so a tuning of either number moves both answers together.
- **There is no floor under the narrow shape and that is not an omission.** A
  width policy needs a floor where two things compete for one window; in the
  narrow shape nothing competes. So the policy promises a shape at *every*
  width — which is what let `src/snapshot.rs` delete its `promised` gate rather
  than keep one that now always says yes, and the phone row is judged by all
  four assertions with nothing in the harness switched on.
- **A column's name has ONE home**, and where that home is depends on the
  shape: the bar carries it when one column shows, the layout paints it above
  the pane when both do. So the heading moved out of `roster` and `chat` into
  `src/ui/shell.rs` — two nodes reading `servers` would be two things an
  operator and the accessibility tree both have to tell apart — and bl-e5d2's
  *heading outside the scrolled region* became structural rather than a
  convention each pane keeps. **The heading no longer wears a mark saying whose
  the arrows are** (bl-b9a3): the mark told two list panes apart, one lit and
  one not, and with one list there is nothing to tell apart — a mark on the
  only answer says nothing, and one lit with no alternative reads as a state
  that can change.
- **The narrow shape is navigated explicitly and by nothing else.**
  `Model::column` moves in the bar and under a left/right key, and NOT as a
  side effect of aiming or selecting: those two are one door for a click and a
  keypress (§4.11's own rule), so a walk down the roster would have jumped the
  glass to another column on the first arrow. The bar stands down under a
  covering pane, as the composer does, because a modal's way out is its own
  control. The composer stands down off the conversation's own column, which is
  *"the conversation is not on the glass while a pane covers it"* read
  literally. **And left and right name a column in the narrow shape and
  nothing at all in the broad one** (bl-b9a3): they used to hand the arrows to
  one of two list panes, and with one list there is no sideways place to name
  — so the key gains no second meaning rather than acquiring one. The arrows
  walk the one list in either shape, which is why no pane asks which shape it
  is in.

**What does not fit is scrolled, and the conversation has a floor**
(`src/ui/shell/policy.rs`, `widths`; bl-e5d2). The list panes held lists and
neither had an answer for one longer or wider than its box: the overflow was
cut at the panel edge mid-glyph, with nothing on the glass saying anything had
been cut, and the arrow walk moved the selection onto rows the pane had never
painted — the exact disagreement `roster::aimable` exists to prevent. Two rules
close it.
**The list scrolls** (the heading stays out of the scrolled region: it is the
one thing on the pane that is always painted, and it carries the mark saying
whose the arrows are), and **a keyboard walk brings its selection along**
(`Model::reveal`, set by `keys::walk` and taken once by the pane the arrows
belong to — the keyboard is the only surface that can move a selection out of
view, since a click names a row the operator is looking at and a scroll IS the
operator choosing what to look at). Sideways, the side panels used to keep
their widths and the central panel absorb the whole loss, so at 900 points the
pane the window exists for was a ~140-point strip while the roster kept 280.
The rule is the other way round: the conversation keeps a floor and the list
pane yields to it until it reaches its own floor — past which nothing yields,
because a pane showing nothing buys the chat pane a width it still cannot use.
The policy is a pure function of one number, so it is a value a test reads back
rather than a layout somebody has to look at.

**And the yield keeps going the other way above the width every column is
worth** (`widths`; bl-fef8). The share stopped at 1.0, so every pixel of a
large display landed in the one pane whose content is already prose while the
two navigation columns stayed the width they are worth at a 1020-point window
forever. It is one expression rather than a second regime: the share is the
SMALLER of *this column's proportion of the window* and *what is left once the
conversation keeps its floor*, and the two clauses cross at exactly
`CONVS + CHAT_FLOOR` — the width at which each column gets what it is
worth and nothing is over. So there is no constant for the growth, no hinge to
tune, and every answer at or below that width is the one the yield already
gave.

**The panel's width is the policy's until an operator says otherwise**
(`shell::lists`, `shell::drag`, `shell::policy::edges`; bl-fef8, amended
bl-46e5). egui stores a side panel's state as the rect its CONTENT took and
reads that back as the width on the next pass, so a `default_width` plus a
`max_width` is a request the panel stops honouring after one frame: a pane
narrower than its cap shrinks to its content and can never grow again. A window
opened at 800 points and widened to 1440 kept a 175-point roster with every row
in it wrapped after two words — the policy said 280 and nothing ever asked it a
second time. `exact_width` clamps the stored rect to a point range, so what a
pane took last frame cannot outvote what this window is worth, and that is
still how a pane nobody has dragged is shown.

bl-fef8 concluded that this **cost the drag handle**, and called that
subtraction rather than loss: *a dragged width is a second home for a fact the
policy owns, and the drag never worked — a pane pulled wider than its content
snapped back to the content on the next pass.* **§4.39 reverses both clauses
and this paragraph now states the amended rule** (bl-46e5). The snap-back was
egui flooring the content at the width RANGE's minimum — `ui.set_min_width(
(width_range.min - frame.inner_margin.sum().x).at_least(0.0))`, one line of
`SidePanel::show_inside_dyn` — so a pane whose body's first act is
`ui.set_min_width(ui.available_width())` fills what it was given and the drag
holds; `TopBottomPanel` does that itself, which is why the composer never had
this half of the defect. And the fact is not the policy's: REMOTE §7 makes
per-seat UI state the seat's own, and a width an operator set is its plainest
member.

So **the policy keeps the DEFAULT and the FLOORS, and the seat keeps the
drag**. The shown width is the dragged one where the seat holds one, else the
yield's, clamped on every frame to `SIDE_FLOOR` at the least and, at the most,
what leaves the conversation `CHAT_FLOOR` (`policy::edges::shown`,
`policy::edges::span`). **`span` lost its second argument with the middle
column** (bl-b9a3): it used to be told what the other list pane was shown at,
so that two edges dragged past the frame settled rather than fought over one
window, and there is one edge to settle. **A window narrower than the
drag CLAMPS it and never overwrites it** — what is narrowed is the shown width,
so a window briefly made small costs nothing — and a width the clamp imposed is
never taken back as the operator's, which is why `shell::drag` reads a width
back only while the resize handle is being dragged. **Only a drag is clamped**:
the yield is this window's own answer, and below the width where the two list
panes are on their own floor it already spends the overrun on the conversation,
so clamping it would be the policy arguing with itself. **The narrow shape
consults no drag**, because nothing competes for a width there.

**The composer's top edge is the window's one horizontal drag**, and what it
sets is the field's ROW COUNT with `theme::COMPOSER_ROWS` as the default
(`policy::edges::rows`). Rows and not points, which is §4.38's bound kept
rather than worked around: the panel is handed a height computed from a row
count and never reads one back off its content. One row is the floor, and
`policy::edges::TAIL_FLOOR` is the ceiling — `CHAT_FLOOR` one axis over,
because the drag is written down and a composer dragged over the whole window
would be a seat that opens next time with no transcript at all. The width and
the row count live on the `Model` and are written to the place file with the
aim (§4.13) — **one width key, `list_width`, since bl-b9a3**; the two the fold
retired are keys no build reads, which is this file's own rung-4 rule costing
exactly one drag.

**A list row's second line truncates, exactly as its headline does**
(`convs::row::beneath`; bl-fef8, bl-b3b2's rule one line down). The headline was
made to truncate and the preview under it was left extending, which is the same
defect with the same two costs. A preview ran flush into the next panel with no
`…`, so a reader could not tell the sentence continued — and the wire's own
`preview` field arrives already elided, which made the pane look like it was
eliding when it was being cut. In the layout it pushed the pane's `min_rect`
past the panel's painted frame, and a side panel reserves the CONTENT's right
edge from the layout while painting only its own: the strip between the two is
covered by no panel at all, which is the ~150-point band of bare window surface
the ball measured beside the list. One fix, because they were never two
defects.

**The transcript is anchored to its TAIL, and the anchor is a scroll state per
conversation rather than a gesture** (`src/ui/chat.rs`; bl-83ae). The pane
opened on message 001 and stayed there — a conversation streaming its eleventh
step painted its second, unmoved, and a sixty-entry transcript cost about
thirty screens of scrolling to read what the agent had just said, with no *jump
to latest* control anywhere. Two facts close it and neither is a field on the
model. The scroll area is **salted with the conversation's own id**, so *where
I am in this transcript* is a property of the transcript: a conversation
selected for the first time meets a fresh scroll state, egui's fresh state is
stuck-to-end, and the tail is what lands on the glass — while one scrolled up
in and come back to is where it was left, which is the same rule read again.
And `stick_to_bottom` is the following: while the offset is at the end the pane
rides every append, the follow lane's write cadence included, and the first
scroll away takes the stickiness off because egui re-derives it from *is the
offset at the end* on every frame. So scrolling back down starts it following
again, with no control to press and nothing to get out of step. `reveal`'s
reasoning above is the same one from the other side — a scroll IS the operator
choosing what to look at, so the pane must stop following the moment they do.

**The pane says WHICH conversation it is showing** (`src/ui/chat/subject.rs`;
bl-7b03). The header was the literal word *conversation*, on every
conversation and in every state: a reader scrolling a transcript could not tell
whether what they were reading was still running, what it had cost, or which
model had answered. Every one of those facts was typed, decoded and rendered —
on the records pane, two clicks away.

**The column's NAME is still the shell's and this is its SUBJECT.** §4.11's own
rule (bl-dfda) says a column's name has one home, and it does: *conversation*
is painted over the pane in the broad shape and on the bar in the narrow one.
What this adds is content — the thing the rest of the pane is about — which is
exactly where the records pane puts its own header (§4.32), and it stands
outside the scrolled region for the roster's reason: it is the one thing here
that is always true of what is being read.

**Every line is the server's, composed once.** `crate::ui::records::header`
already assembles the identity, the resting clause, the spend and the context
reading that names the model, so this reads them from there rather than
re-deriving them: two surfaces reading one answer is one home, and two surfaces
composing one sentence is two. The failure clause goes above the costing,
because it is why there is no more of it — *a conversation that died on a bad
model id looked identical to one that finished*.

**Which model answered has two sources and they answer different questions.**
The server's context reading names the model it is holding a window open for —
the one the NEXT turn will use, the better answer, and the one the costing line
already carries. But a reading exists only while there is one to take, and a
**quiescent** conversation has none: that is the state the question is actually
asked in, and it is why *nothing in the window says which model answered* was
true of every conversation that had finished. So the transcript's own last
model turn answers when the server's reading does not — a fact this seat
already holds, about a turn that happened, never a prediction about the next
one — and it stands down the moment the server has a reading, rather than being
joined beside it.

**So the `agent` read moved out of the records pane's standing set and into the
conversation's own** (`asker::workspace`). It is one read MOVED and never a second
copy: the pane still paints off `Model::records.agent`, and a header that is
only true while a pane two clicks away is open is a header that is usually not
there.

**A machine's answer is folded and a person's is not** (`src/ui/chat/fold.rs`,
`chat::folded`; bl-90d0). litany bounds a tool stream at 16 KiB head plus 16
KiB tail before it reaches a transcript, so up to 32 KiB per call arrived here
and the pane painted all of it: one `bash` running `ls -la && git log` filled
the pane and continued below it, one `bl --help` was six screens, a `find` over
a home tree arrived essentially whole. The call above each of them is one
compact line, which is what makes the asymmetry sharp — the question is a line
and its answer is six screens.

**The fold is by KIND and never by size**, which is the distinction that
matters: a seven-hundred-line `bl --help` is not a long answer, it is
bookkeeping in front of one, while a long prose answer is the thing the pane is
opened for. So `Row::fold` is `Some` only for a tool result, and only for one
with something to hide — an expander over four lines is a control that costs a
gesture and saves nothing. What stands is bounded twice, because a result
overruns in two shapes and either bound alone lets the other through: six
lines, and five hundred characters of them, since a `git log` is six hundred
short lines and a JSON blob is thirty kilobytes on ONE line, which a pane that
wraps paints as three hundred rows of glass.

**Whether one row is open is the toolkit's memory, keyed on the row**, on
bl-83ae's own reasoning one noun over: it is a per-row view state no other fact
can be asked for and no gesture on the wire corresponds to, so putting it on
the model would be plumbing a click through a snapshot to get back to the
widget that took it. The control names the act it fires and carries the size of
what is hidden — *show all 700 lines, 32985 bytes*, then *fold* — on the pin
pair's rule (§4.25): two words, never one that toggles.

**What hangs UNDER a conversation is under it, and the rule is about descent
rather than about compactors** (`src/ui/model/subtree.rs`, `convs::subtree`;
bl-00f5). One session of ordinary work answered twelve conversations, of which
five were compactors the operator never started — each with a minted name as
memorable as the work's, each at the same weight, four of the five belonging to
ONE conversation. The rows were not wrong: `depth` is 1 and the preview says
what they are. But an index whose majority is machinery is an index people
learn to stop reading.

**The wire carries no kind, and it should not.** A conversation row spells its
address, its badge, its preview and how far it hangs under its root; nothing on
it says *this one is a compactor*, and a fork's candidate is a depth-1 row the
operator DID start — so a seat that guessed from a preview would be reading
prose to make a structural decision. What the server does give is the descent,
and the descent is the whole answer: the list is the conversations, a root is
always on the glass, and its subtree is one gesture away with the count on the
control. That satisfies the ball's own first remedy — *compactors fold under
the conversation they compact* — with no rule about compactors, and folds a
fork's candidates on the same terms, which is right for the same reason: both
are what that conversation is made of.

**The fold is applied to `Model::rows`, which is the one list the pointer
paints and the keyboard walks** (§4.11's own rule, `crate::ui::model::claim`).
Applied in the pane it would have made a row a click could not reach and a key
could — the second surface `crate::ui::keys` exists in order not to have.
`Model::opened` is a set of what is OPEN rather than of what is folded, so the
default costs no state at all, and it is a navigation the operator performed,
which is `Model::column`'s own standing.

**And the same fold reaches the typed word, which is the surface a headless
operator has** (`src/render/workspaces/convs.rs`, bl-96cd). The fix above landed on
`Model::rows` alone — the window's list — and the typed `lernie conversations`
kept answering the flat forest: one lane of ordinary work later it printed
twelve rows of which SEVEN were compactors, three of them under one
conversation, worse than the five this ball was filed for on the same amount of
work. That is the surface reached over `ssh`, from a script, and by this
campaign's own test lanes.

**The subtree is NAMED there rather than opened**, and that is the one place
the two faces differ. The window folds against a set of what is open, which is
a navigation the operator performed; a command has no such state and should not
grow one — a flag on the verb would cross the wire (`Verb::flags` writes
envelope fields, and yog's REMOTE §8.5 keeps folds off the boundary on purpose:
*views gain no boundary representation*), and a second rendered form beside
`--json` would be a third §4.37 `Form`. So a root says how many hang under it
and what they are CALLED, on one line, elided at a preview's width. That is the
CLI's version of *one gesture away*: every name printed is the address `agent`,
`transcript` and `follow` already take, so nothing is out of reach and nothing
costs a second round trip. `--json` still answers every row, as it must — and a
deep row with no root above it is printed as its own row rather than dropped,
because a listing that quietly loses rows is a worse failure than one that
indents oddly.

**What the attention rows say is not this seat's half.** Every compactor in
that session earned an `attention` row reading *came to rest — your turn*,
which is false of a finished compaction; that is a fact the server answers, and
yog's own ball owns it.

**A section header names the address it dials** (`ui::Channel::dials`,
bl-77df). Two entries naming one address are two trust relationships that
happen to terminate at one listener (§8.2), which is lawful — and an entry
naming the address this box's own server listens on is the same thing by
accident, which paints every workspace of that server twice, under two headers,
with nothing on either saying they are the same server. `lernie entries` prints
the address under every row and the window dropped it. The seat is the only
thing that can see the duplication and it can see it for free, so the address
goes on the header and a duplicate is self-evident. **Amended by bl-13b9: the
duplicate is folded, not merely labelled.** Self-evident was not enough — the
seat card listed the server twice and counted its waiting twice. The window's unit is the server, so `Model::folded`
(`src/ui/model/fold.rs`) groups the channels by the address they dial before
anything is counted or painted: one row, the address said once, every other
name beside it (*also held as …*), each workspace once, and a gesture at a workspace goes
down the first of the server's channels that can address it — this box's own
server leading, because it addresses every workspace by its own name. The roster
itself stays one chunk a channel, because a reply is stamped with the channel
it came down. **The decision queue folds on the same terms** (bl-52d2): the
`attention` fan asks every channel, so both names answer the server's whole
queue; `Model::sections` keys the pane's sections by server and keeps a row
once by `(workspace, agent)` — the host's spelling, which both answers carry
alike — beside the first channel that can address its workspace. The unaddressable row's own
sentence was reworded in the same breath: *"this seat holds no name for it"*
fires on a perfectly correct provisioning and reads as an error about the row
above it, where the fact is structural — an entry directory names one
workspace, the channel enumerates every workspace that client is registered in,
and the extras have no entry of their own, so no envelope this seat can write
reaches them.

**An unreachable channel says so on its own section, never in the shell's bar**
(`Model::unreachable`, bl-e620). REMOTE §8.2 rules it: *"a channel that cannot
be dialled is that channel's workspaces painted unreachable, never the whole
shell, which stays reserved for the one wire the window cannot exist without."*
The bar is for what a **server said** about a gesture, and for a frame this
seat could not read — facts about an exchange. An unreachable channel is a fact
about a **relationship**, and routing it to the bar cost three things, all
driven live: the sentence named no subject at all (*"this seat could not reach
**it**"*), a seat holding two dead channels heard about exactly one of them
forever, because there is one bar and the last writer wins, and the bar's
dismiss was inert — a relationship that is down is down on every beat, so it
re-posted faster than a hand could clear it. A row's state is not something one
dismisses. The bar is kept only for a channel this box holds no section for,
and it names the channel, because a fact with no home still has a subject.

**And a failed handshake carries a remedy** (`Channel::wrote`). rustls performs
the handshake inside the first write, so a wrong trust root surfaced as *"send:
invalid peer certificate: UnknownIssuer"* — rustls' own wording, naming no file
and no act, on the one failure that is always about material this box carried by
hand. The certificate class is read off the typed `rustls::Error` rather than
off its wording, so a rewritten message cannot silently stop matching, and every
other write error is still said in the transport's own words.

**Every empty section says WHICH emptiness it is** (`src/ui/roster.rs`,
`crate::ui::Held`; bl-08b6). The pane carried one sentence — *no channel
provisioned — material arrives by hand* — under `roster.is_empty()`, and **an
empty roster is unreachable**: `seat::channels` answers a section for this
box's own flat slot whether or not anything is provisioned in it, which is
right and is what `lernie entries` prints. So the guard was false on exactly
the box the sentence was written for, and a first run got a section header over
a blank while the other three panes said what they were waiting for. The
reframe is one level down: a section is either a list of workspaces or a **reason
there is none**, and the reason is three facts and not one — nothing has
answered down this channel yet (the `convs` pane's own doctrine, one noun
over), this box cannot dial it and knows why off its own files (the sentence
`crate::channel` already computes and `entries` already prints — one home, both
surfaces), or the server answered and holds no workspace. `seat::channels`
seeds the second at boot; the first roster answer spends whichever was standing.

**Amended by bl-693d: four facts, not three.** The server can answer the
roster read with a **refusal** — *this certificate is registered in no
workspace on this server* is what a fresh server says to a fresh seat — and
that is none of the three: the server is up and has answered, so *waiting to
hear* is false and *will not dial* is false. It went to the bar as any refusal
does, re-posted every beat, and stood there beside a running conversation after
a founding (§4.42 item 7's `+ new workspace`) had made it untrue, until Esc. So
every frame the off-frame lanes file now carries the op it answers
(`offframe::file`, `Said::Receipt` — there is no bare frame left), and a
refusal of `workspaces` is filed as `Held::Refused` on that channel's section
(`Model::unseated`): painted where `Unheld`'s sentence stands, in the bar's
own words for a refusal, never counted as a server that is down, and spent by
the next roster answer like every other state — so the founding that registers
the client takes it off the glass on the beat it lands. `+ new workspace`
stands under a refusing server, because founding is the remedy it names.

**The notice wraps** (`src/ui/shell.rs`, bl-3d0f). It was a `ui.horizontal`
holding the dismiss and one label, and a horizontal layout lays its label on
one line however long it is — so the panel cut the sentence at the window's
right edge, with no ellipsis, because the galley was never truncated and so
never had one added. Every refusal this seat paints puts the fact first and the
remedy last, so **the half that was cut was always the half that says what to
do**, and the sharpest case is the first run of a seat on an unprovisioned box:
the notice is the only thing on the window carrying an instruction, and the
instruction is the part that was off the glass. A second line in a bar already
sized to its content costs nothing that matters. Its regression asserts the
RECTS rather than the glyphs: a galley's rows carry no newline where the wrap
broke them, so a wrapped run and a run laid past the frame read back as the
same string — the paint probe's own division, where geometry is unaffected and
it is the text that lies.

**The enrollment is a modal, so it behaves like one** (`src/ui/enroll.rs`,
bl-7574). Its argument for covering the conversation is about the WINDOW — a
private key on a screen, the act is *look at this now and close it*, and a
conversation legible behind it invites the long life on a display the material
must not have — and it only ever covered a PANEL. Two things close the gap and
neither widens what it covers. **Escape closes it** (`Model::escape`), doing
exactly what `done — forget it` does, material included, because the control
that closes this pane is the control that forgets; reachable by Tab is not the
same as operable, and this is the one pane whose purpose is *close it quickly*.
**Nothing live paints under it**: the composer is a bottom panel and so outside
what a central panel covers, which left a live `start` control — firing a
conversation on the very workspace being enrolled into — beneath the symbol, so the
shell stands the composer down while an enrollment is open. The arrows stand
down with it (a modal owns them: the lists behind it are not the subject of
anything, and the name box keeps its own cursor keys). Escape typed into that
name box cancels rather than leaving the box, deliberately: egui clears a box's
focus while processing the same Escape, so the keyboard gate cannot see it, and
cancel is what Escape means on a modal — nothing is minted until `mint`, so it
costs a name retyped. The roster and the conversation list stay legible, which
is where the operator looks rather than what they act with.

**An empty pane says WHICH emptiness it is** (`src/ui/convs.rs`, bl-f780). The
list had two sentences — nothing aimed at, and *"no conversations here"* — so a
workspace this window had not been ANSWERED about was painted with the sentence
reserved for a workspace that answered nothing. That is a definite fact about a workspace
nobody looked at, which is exactly what `convs::UNCERTAIN` refuses to state one
level down about a conversation nobody could take a reading of; the ruling is
the same one level up. Four states now: no aim, **not answered yet**
(`Model::answered` rides beside the rows and is retired with them by
`aim_at`, because emptiness cannot tell "zero conversations" from "nobody
asked"), answered-and-empty, and **an aim on a channel this seat does not
hold** — which is permanent rather than transient, because `Standing::aimed`
finds no channel by that name and so nothing is ever asked. `crate::place`
restores a saved aim without checking it on the ground that a stale one is
inert; inert is right about the dialling and wrong about the paint, and the
refusal that was silent is now the pane's third sentence.

**A row is addressed by what the channel resolves, never by the name it wears**
(`ui::Channel::address`, §8.2 read from the seat's side). This box's own server
rewrites nothing, so a row is addressed by its own name; an entry resolves by
its **leaf** and by nothing else, so the leaf is the address of the one
workspace it names. The third case is real and is painted rather than hidden: an
entry's server may answer a workspace the entry does not name, which no envelope
this seat can write will reach. Dropping it would hide a workspace the operator
has, and addressing it by the leaf would aim a gesture at a different workspace, so
it is shown and said to be unreachable.

**One composer with two subjects, and a start is where the third pane's
refusal went.** A workspace with a conversation on it is spoken to; a workspace with
none is where a conversation is *begun*. That second case used to be half of a
refusal — *pick a workspace and a conversation* — and the refusal was the
start's own case wearing a sentence. A second box beside this one would be the
same box twice, each with its own Enter, on a face that is four panes wide.
What is left refusing is the one case that is neither: no workspace aimed at, so
there is nowhere for either.

**Two rows of verbs under the one box, split by what each does to the turn**
(`src/ui/composer/acts.rs`, bl-213c). The first row *advances* it — `send`,
`interrupt`, `nudge` — and each of the three ends with a driver running; the
first two spend the box, because *what to say instead* is the same question as
*what to say* and a second box for it would be the same box twice. The second
row does not advance it: `stop` kills the driver, `retarget` marks the
conversation for its lineage's head, `delete` unmakes it. Two decisions ride
with that. **Enter is the deposit's and no other verb's** — a key an operator
reaches for by muscle memory must not kill a running driver, which is exactly
what the cut adds to the deposit. And **the deletion's arming is a parameter,
not an enablement**: the wire's `typed` is empty for the bare form and the
conversation's own name for the cascade, so the box beside `delete` fills the
gesture's third field and is *not spent* on firing — a delete is refused
outright while the conversation is live, and charging a retype for the server's
*no* is a toll on the safe path. That is one half of §4.20's rule rather than an
exception to it: `delete-workspace`'s `typed` is an ARMING with one accepted
value, so there it is an enablement — the seat reads which of the two a `typed`
is off the wire's own grammar, and invents no policy of its own.

**A start is the one thing this window holds across a round trip**
(`src/ui/model/start.rs`). Starting is two acts and the second is composed from
the first's answer, so the frame that *absorbs* the staged body is the frame
that composes the fire — which is the only shape that keeps the rule above: a
frame composes into the outbox and never waits on a socket. Three facts ride
with it. The address is **held**, not re-read at fire time, because an operator
may aim elsewhere while the first act is in flight and a fire that followed the
aim would prompt a workspace nothing staged. The goal is held too, so the
operator's text has a painted representation for the whole round trip — a start
that shows nothing between Enter and the answer reads as a window that did
nothing. And **a start in flight refuses a second start** by there being no
control: while one is outstanding the composer paints the sentence instead of
the box. That is upstream's own finding (yog §3.4): two starts chained through
one composer spend one goal on two conversations and leave the first unfinished.
**And a start the server refuses is retired by that refusal, with the goal back
in the box** — §4.26, which is where the one gesture held across a round trip
learns which receipt is its own.

**A start focuses what it started, and the claim is what makes that safe**
(`src/ui/model/claim.rs`; upstream's rule is yog's `docs/DESIGN.md` §3.4).
Selecting the minted name outright does not work: the name is a barrier — every
gesture after the receipt may address it — but only once the detached driver has
written the conversation's branch, which is upstream's own finding (yog bl-56c6:
*"until the driver writes its branch the minted name resolves nowhere"*). So the
selection is taken and three things ride with it, none of them a second pending
concept — the claim **is** the held `Start` in its `Started` phase, read against
what is selected.

- **Nothing is asked about it.** `Model::asked` is what §4.12's standing set
  publishes as its third question and a claimed name is not it. The alternative
  is three refusals per pass against an address the server cannot resolve, and
  the operator's own new conversation painted as unknown for the whole of a
  healthy start. An empty conversation pane is what the world honestly holds.
- **A row stands where the conversation will be.** `Model::rows` is the list
  every surface paints and walks — the pointer's and the keyboard's one list, so
  the row is not a case each of them carries. It is what nothing observed: no
  lock, no completed step, flagged uncertain, which is exactly what the server's
  own classifier answers for a conversation it cannot probe. **It must not read
  `live`**, which would claim a driver this seat has never seen. It carries the
  operator's goal, because a start that shows nothing between Enter and the
  driver's first write reads as a window that did nothing.
- **It is spent where it was made.** `Model::resolve` retires the claim on the
  answer that makes the conversation addressable — a listing row carrying that
  `name`, which the server hands over exactly when a stored fact backs it — and
  moves the selection **only while the selection is still the name the claim put
  there**. A start can take a minute, and an operator who read something else in
  that minute is not yarded back. No claim survives being spent, and a claim
  whose row never arrives is inert: the name stays selected, the row stays
  faded, and one arrow key leaves it.

**And the composer stands down while it stands.** A deposit or an advance
composed against a claimed name is a gesture this end already knows the server
will refuse, so the start's own sentence stands where the box was — the same
shape as the second-start refusal above, and the same reason: not a disarmed
control, but no control.

**Nothing that arrives is dropped.** `Model::absorb` is the one door: an answer
is filed, and a refusal or an unreadable frame becomes the notice bar, standing
where that content would have been. That is §4.9's rung 2 honoured on the glass
rather than only in the type — and the two read differently, because a refusal
is the server's sentence and an unreadable frame is a statement about this seat,
of which only the second is fixed by an upgrade. It is a **bar and not a
modal**: the server refusing a deposit says nothing about the roster beside it.

**The paint probe and its rule travel together, and the rule is why.**
`crate::paint_probe` is the one walk over a finished frame, and
`rules/no-hand-rolled-paint-walk.yml` forbids reading `Galley::text()` anywhere
else. A galley reports the string that went IN, so a label the toolkit elided to
`…` reads back whole and every assertion against it is blind to truncation — the
one defect the paint layer is the only witness for. Upstream found that three
times before it became a rule: in the probe itself, where 1815 tests passed
while covering no truncation at all; in two copies of the walk; and in a third
copy that aimed every pointer test's **click** by input text. So a click here is
aimed by painted glyphs too, and the probe's own suite pins the elision case.

**Everything the window does is reachable from the keyboard** (`src/ui/keys.rs`;
yog's QUALITY F1, inherited as a standing rule rather than a feature). A face an
operator has to leave the keyboard for, once per selection, is a face they use
through the command line instead. Four rulings hold it together and each
subtracts rather than adds.

- **Most of it is egui's, and the suite proves that rather than assuming it.**
  Every control here is a button or a text box; egui moves focus with Tab and
  fires a focused control with Space or Enter. So Send, Nudge, Start and the
  notice's dismiss want no binding of their own, and the beat that presses Tab
  and Space into a real window is what keeps that true.
- **The cursor IS the selection.** A list cursor beside a selection is two
  highlights, two things to paint and two ways to disagree. Moving in a list
  *selects*, so the highlight the pointer already paints is where the keyboard
  is, and the reads that follow a selection follow a keypress for free — the
  standing set is derived (§4.12), so nothing had to be told. What is left to
  paint is only which list the arrows belong to, and it is marked on that pane's
  own heading: a focus that cannot be seen is a focus nobody can use.
- **A binding names a control that already exists.** The three acts a click
  makes moved out of the `if …clicked()` that made them into `src/ui/model/
  acts.rs`, and both surfaces call the same door. The roster walk asks the same
  question the roster's own paint asks (`roster::aimable`), so a row no pointer
  can aim at is a row no key can aim at either. **A binding that could fire
  something a click cannot is a second surface.**
- **The focused thing is visibly the focused thing, and Tab agrees with the
  arrows** (bl-2d6b). Round one measured two costs: a Tab focus that drew
  nothing an operator could see on the dark ground, and a Tab focus that was
  not the selection while an arrow's was — one keyboard with two models of
  *where I am*. Whatever holds the keyboard wears the brand ring, the one
  stroke the language leaves on the glass (`docs/STYLE.md` §1; egui paints a
  focused control with its `active` visuals, which is where the ring is
  installed, and `theme::paint::row` rings itself). And a Tab that lands on a
  list's row hands the arrows to that list, so the mark on the heading moves
  with the ring; Space on the ringed row does what a click does. The paint
  probe projects strokes for exactly this (`paint_probe::strokes_of`) — a
  ring is a stroke, and a focus that cannot be asserted is a focus that
  will silently stop painting.
- **The gate is the composer's box by name, not egui's "is anything focused".**
  `wants_keyboard_input` answers *is any widget focused*, buttons included, so
  tabbing to Send would have turned the arrows off. The box wears one id and the
  gate compares against it — which is also what lets Escape mean *leave the box*
  inside it and *dismiss the notice* outside it, two contexts that never
  overlap.

**The mark is arithmetic, and where it is SEEN is not where a client sets it**
(`src/mark.rs`). A window that names no icon wears the generic one every
unnamed client gets, and the seat's own mark is three shapes — a ring for the
server held elsewhere, a filled disc for the seat that is here, and the one
wire between them, which is what `crate::channel` is.

- **One geometry, two emissions.** `svg` writes the checked-in vector source
  and `rgba` rasterizes the same shape list for the toolkit's icon call, over
  one list in one order. `assets/lernie.svg` is a **derivation** — `make icon`
  emits it and the suite pins it byte for byte — never a hand-edit.
- **Every number is an integer**, in thousandths of the canvas, with coverage a
  count of subsamples rather than a distance through a curve. That is what
  makes a byte-for-byte pin honest (float formatting can differ by target),
  it removes the `f32 as u8` a rasterizer otherwise ends in — a cast the lint
  set denies with no home for a suppression but the manifest — and it means no
  image or vector crate is linked for a picture the seat can compute.
- **Wayland has no protocol for a client to set its own window icon.** The
  toolkit's call reaches X11 and nothing else; a compositor resolves a window's
  mark by matching its **application id** against an installed desktop entry
  and reading that entry's `Icon=` by name through the hicolor theme. So three
  spellings must agree — `mark::APP_ID`, `assets/lernie.desktop`'s
  `StartupWMClass`, and the installed SVG's basename — nothing at runtime can
  check them, and the suite pins the agreement. `make icon-seats` lays the last
  two down and `make install` runs it, because a rebuilt binary alone can never
  refresh a mark it does not own.
- **The installed entry names an absolute binary and the tracked one never
  does**, which is the one place an asset and its installed copy differ on
  purpose. `Exec=` is resolved by the desktop environment out of the session's
  environment rather than a login shell's, so the tracked `Exec=lernie` starts
  nothing at all wherever the binary's directory reaches `PATH` through a shell
  profile — and it fails silently, with no window and no message. `make
  icon-seats` substitutes the path at seat time: `$(INSTALL_BIN)/lernie` first
  (the defect IS that directory being off a `PATH`, `make`'s included), then
  `command -v`, and a refusal if neither resolves absolutely — an entry that
  resolves nowhere is the defect, not a fallback. It cannot be the tracked
  file's business: that file is one repository's source for every box, and a
  real path in it would be a disclosure as well as a lie everywhere else.
- **And no PNG, so the tree still tracks no binary.** Upstream laid sized
  rasters beside the scalable source and paid for them with the only entry its
  disclosure gate's `BINARY_ALLOWED` ever held. hicolor's `scalable/` is read
  by every desktop this seat installs on, so a sized raster is a derivation
  nobody consumes — and an allowlist entry for a file nothing needs is a hole
  in the gate bought for nothing.

**The native boot is `src/main.rs`, which decides nothing.** `ui::render` takes
an `egui::Context` and the model, so every assertion in this crate runs the real
window on an offscreen context. What lives in the excluded entry point is the
event loop and the `eframe::App` impl that forwards one call — process state,
exactly as argv and the environment are.

### 4.12 What fills the model: four threads, a derived question set, and one lock (§7, §8.2)

`src/state.rs`, `src/state/traffic.rs`, `src/offframe.rs` and the four passes
under it. §4.11 says the frame never dials; this is where the dialling happens,
and the whole of what it may say to the window.

**The frame owns the model and no worker touches it.** What crosses the lock is
small and one-directional in each half: frames that **landed**, gestures to
**send**, and the standing question set. [`Model`] is the frame's alone, so no
worker can be mid-write in one while a frame reads it, and there is no shared
structure to keep consistent. `Link::settle` is the frame's entire side of it,
called once at the top of an update: it files what landed, hands over what was
composed, and publishes what to ask next. Nothing in it can block — the lock is
held across a drain and three moves, and no worker holds it across a socket.

**Four threads, because they wait for different things.**

- **The asker** (`src/offframe/asker.rs`) goes round the standing set at human
  cadence: every channel's roster, the aimed workspace's conversations, the selected
  conversation's transcript, and whichever covering pane's read is up. The
  questions **nest** — the aim is asked only where there is one, the
  conversation only where one is selected — and a channel that will not answer
  costs only itself, which is REMOTE §8.2's *"a refusal is one entry's, never
  the set's"* one layer above the file it was written about.

  **The pass's order: the selection first, at every leg** (bl-1f22). A pass
  is legs — one per channel's fan, then the aimed workspace's reads — and before
  each leg it re-reads the standing set and asks the selected conversation's
  reads (transcript, row, and the records pane's when it is open) if they
  have not been asked about that selection yet this pass, down the one
  channel the selection names. A pass's first leg is that rule with nothing
  asked, so the selection leads every pass; a click made mid-pass is answered
  one leg later rather than after the rest of the fan and a beat. That is the
  whole remedy for a selection painting an empty pane: `Model::select`
  empties the transcript, and over a dialed wire the fan alone is seconds.
  **The seat does not fill the gap with what it sent** — the server's read is
  the transcript's one authority, so the remedy is to ask sooner, never to
  predict the answer. No leg is skipped for it: the sweep still visits every
  channel, every pass. The beat between passes is not woken, and needs no
  wake: it is one beat, and the next pass leads with the selection.
- **The poster** (`src/offframe/poster.rs`) sends what a click composed, on its
  own thread because an act must not wait behind a read that is mid-pass. It is
  also where a gesture naming no workspace is **fanned** over every channel
  rather than routed (§4.21), and where a failed leg earns its sentence: a
  read's failure is a *relationship* and goes to the channel's roster section,
  an act's is an *exchange* and goes to the bar (§4.22).
- **The follow lane** (`src/offframe/follow.rs`) holds one connection open on
  the focused conversation and is answered as the tail moves — a read that
  deliberately never finishes, and therefore one that must never be in the
  serial pass.
- **The sign-in lane** (`src/offframe/signin.rs`) is that one noun over
  (§4.24): one connection on the provider row the login pane is following,
  answered at a human-in-a-browser's pace.

**Both held lanes stamp every frame with what it is about, and the FRAME
decides whether it is still wanted.** The server was asked about one
conversation, or one provider row, and answers about that one — so only this end
knows the operator has moved on, and only the frame knows what it is looking at
right now. The guard is therefore a pure comparison at the settle rather than a
poll racing the socket the lane is parked on. Each lane's fold lives for exactly
one `tick`, which is how REMOTE §5.5's *"onto an empty fold"* is implemented
with no flag and no field: a read boundary is a local variable's scope.
**So do the selection's own standing reads** (bl-19b8): the transcript, the
conversation's row and the records pane's five are stamped with the
conversation the asker asked about (`Said::About`, the follow lane's own
variant), because a transcript names no conversation and a read in flight when
`+ new` or a switch moved the selection used to be filed under whatever was
selected when it landed — the old conversation's model on an empty start line,
or its words under another's panel for a beat.

**The standing set is a QUERY, never stored** (`Standing::of`). It is derived
from the model on every settle — every channel's roster, the aim, the selection,
and which covering pane is open — so there is nothing to invalidate and nothing
that can disagree with the focus: a click changes the model, and what is asked
next follows from it. Two consequences are worth stating because both were
decisions.

- **A pane's read is keyed on the PANE, not on the aim** (§4.17). A seat with no
  configuration surface open asks nothing about a file nobody is looking at; the
  reads are cheap, and a standing question nobody has a use for is still one the
  server answers on every beat, forever. `Standing::open` is **one field rather
  than a flag per pane** (§4.24), because no two covering panes ever stand
  together and four bools would make three-at-once representable.
- **Not every selection is a question.** A conversation this window has just
  started is selected under a name the server resolves nowhere until its driver
  writes the branch, so `Model::asked` leaves it out and the standing set never
  asks about it (§4.11's claim).

**The one lock, and the rule named the file before there was anything in it**
(`rules/locks-outside-state.yml`; §5's confinement table). `src/state.rs` holds
the link, and it holds one more thing on the same terms: what worked for an
entry this run — the punched lines held between asks, which every one of these
threads may take up and give back (§4.40). Everything above the socket is a
pure function of what it is handed; everything below it used to be one thread
on one connection, and a held connection is the one fact that outlives the
thread that made it. A poisoned lock is recovered rather than propagated — a worker that
panicked mid-pass left the queues consistent, because they are vectors of
finished values.

**An act is sent exactly once, and there is no retry anywhere in this crate.**
`Link::compose` is a take, the drained envelopes live in the poster's own loop,
and no arm re-queues one. The only repetition the crate owns is `offframe::pump`'s
cadence, which re-derives the standing READS and never touches that queue — which
is REMOTE §3's *"A lost reply leaves an act IN DOUBT, and the recovery is a read
— never a resend"* implemented as an absence rather than as a rule somebody
keeps.

**Every pass is a function a test calls directly.** Each worker's body is
`tick`, and a thread is `pump` around one — so the suite drives a real pass
against the stand-in server with no thread at all, and the one end-to-end beat
is about the threading rather than about what the passes do. **A stop is seen
between passes, not during one**: a held lane is parked on a socket read, so it
learns of a stop when the server writes, when the connection closes, or when the
transport's own read timeout expires. That is not a leak — closing the window
ends the process — and the alternative is a second signal path into a blocking
read, which is a mechanism for a case with no consequence.

### 4.13 The seat's own durable state lives under a DIFFERENT root (§7)

`src/place.rs`, and the decision it records is `src/paths.rs`'s. REMOTE §7 rules
that per-seat UI state — focus, scroll, tab selection, drafts — never crosses
the boundary and is the seat's own, so the window is the first thing this crate
holds that this box **generates** rather than receives.

**Two roots, and the split is a hazard rather than tidiness.** Everything under
the data root is operator-provisioned and irreplaceable by anything here:
deleting it is a revocation, and re-minting is an act on another machine by
another hand (§1.4). Everything under the state root can be deleted at any time
for the cost of a forgotten selection. A regenerable subtree beside
irreplaceable material makes the second look like the first — an operator
clearing the seat's saved place would be one `rm` from clearing the certificate
that gets them back in. XDG draws exactly this line already, so the separation
costs one variable and no invention.

**It may never become a way for the seat to fail to start.** A file that is
absent, unreadable, truncated or written by a build that spelled things
differently has one answer: no place, which is the window a first run opens. A
stale aim needs no validation either — a workspace that has gone is a workspace no channel
resolves, and §4.12's standing set already declines to ask about one, so
checking it here would be a second answer to a settled question.

**It is a JSON object and not two lines**, so the next fact §7 names is a key
beside this one rather than a format: an unknown key is ignored and a missing
key is absence, which is §4.9's rungs 3 and 4 applied to this box's own file.
Three subjects live there today — the aim, the accordion's arrangement
(`open`, `opened`, `aimed`) and the edges the operator dragged (`list_width`,
`composer_rows`), both of the last two §4.39's — and **absence is spelled per
FIELD**, so a file this build can half read is half a place kept rather than
none. The dragged edges were three keys and are two since bl-b9a3 folded the
middle column away: `roster_width` and `convs_width` are read by nothing, and
a file that carries them is a seat that has dragged nothing.

**And it costs the frame nothing.** The place is a **projection of the model**
(`place::Place::of`), published by the settle that already publishes the
standing set — the same lock acquisition, on the same terms, derived so there
is nothing to invalidate. It is read once at boot and written once after the
event loop returns, which is where it has to be written: by then `eframe` owns
the model and `src/main.rs` does not, so the crossing is what makes the write
possible rather than a cost the aim avoided. No fourth thread, no write on a
frame, and no second copy of anything (amended by bl-46e5, which added the
dragged edges beside the aim — §4.39).

### 4.14 The gate is run by a machine, and the store is judged on the ref

`.github/workflows/`. Two workflows, and they answer two different questions.

- **`ci.yml` runs `make ci`, which is `make check`**, on every push to `main`
  and every pull request. It spells out no step of its own: a CI file with its
  own list of steps is a second definition of green, and two definitions drift
  within a week. What it does carry is the three pinned tools the gate shells
  to, each cached under its exact version so a bump is a cache miss rather than
  a stale binary — the key IS the pin.
- **`store-scan.yml` scans the published `balls/tasks` ref** with the source's
  own `scripts/leak-scan.sh` and rule table — the scanner scans the tree it is
  run in, so there is no second copy of the rules to drift. Daily, on dispatch,
  and whenever the rule table itself changes, because a new rule re-judges a
  store that has not moved.

**Prevention is local and enforcement is remote and late, and both halves have
to be said.** `make leak-scan` and `scripts/lernie-leak-gate` run before a push
and are one `--no-verify` or one `bl conf remove` from not running at all.
`store-scan.yml` cannot be switched off by the agent writing the ball, and it
cannot prevent anything either: by the time it runs the material is on the
remote and the remedy is a history rewrite. It is not a `push` trigger on the
store branch, and cannot be — for a push event GitHub resolves workflows from
the pushed ref's own tree, and `balls/tasks` holds `tasks/*.md` and nothing
else.

**A pushed branch is never how work lands.** Work lands on `main` by `bl close`
squashing a claim worktree; the only reason to push one is to buy a runner
verdict on a defect that exists only on a runner, which is what the
pull-request trigger answers. Read the verdict, land through `bl close`, then
delete the branch and close its pull request in the same breath.

---

### 4.15 Enrollment: the one reply this seat draws and refuses to keep (REMOTE §8.4)

`src/reply/enrolled.rs`, `src/seat/enroll.rs`, `src/seat/enroll/entry.rs`,
`src/verbs/enroll.rs`, `src/cli/enroll.rs`, `src/ui/enroll.rs`, `src/qr/`.

**What the act is.** `enroll` names a workspace, a name and a grade, and the
server mints that box's leaf on its own CA, seats the registration, answers the
material and shreds the key. The seat sends it over an entry it already holds
at operator grade and shows the answer as a QR symbol. §3 above records why
this does not lift REMOTE §1.4, and REMOTE §8.4 is the authority for the wire.

**The reply is not the product, and that is the one place this seat departs
from its own shape.** Every other verb hands its reply stream to stdout —
`seat::ask` is the whole of it. This one must not, because the product is not
the frame: it is REMOTE §8.4's **envelope**, which is the same fields with two
of them removed under a marker, and a seat that handed on the frame would leave
the operator to reimplement the payload contract. So `enroll` has an arm of its
own all the way up to `cli::Decided`. `lernie ask` still prints the raw frame,
which is correct — an operator who spells the envelope by hand has asked for
the stream, and nothing here is a boundary against the operator.

**And `--json` is the envelope ALONE** (bl-ac76). The three renderings below
are what a PERSON is offered; a script is offered one. Under `--json` stdout
carries the envelope on one line and nothing else — no caption, no symbol, no
colour, no receipt. It printed the whole human rendering under the flag, so a
line-oriented capture (`head`, a `read`, a pipe into a formatter) took an
ANSI-coloured block and left the material behind. That is not an ordinary
rendering defect here: the server has already minted and shredded by then and a
second enrollment under one name is refused, so what the script kept was a
picture of a key it could no longer ask for, and the name was spent. §4.37's
rule — *every verb renders by default and `--json` is the machine form* —
reaches the one verb that had no machine form at all; what differs is only
WHICH object the machine form is, and that is already this act's whole
exception (above): the frame is not the product, the envelope is.

**The machine form and bl-768a's withholding compose into one sentence**:
under `--json`, stdout carries the envelope exactly when the material was not
written down. A run that FILED withholds it — the files are the copy the
operator chose — so stdout is empty and the receipt naming them goes to stderr,
because a receipt is prose about a file and not a frame. A run whose filing
FAILED says the envelope, for bl-768a's own reason: the screen is then the only
place the material can be. And the sentence explaining a destination that would
not take the files is a diagnosis on stderr in both forms, which is what this
section already required — *the answer is said on stdout whichever way the
filing went, and only the exit code reports a directory that would not take
it*. `enroll` therefore takes a warning sink exactly as the role assignment
does, and `src/main.rs` hands it `eprintln!`. The four words the gesture is
made of ride as one value (`cli::Asking`) rather than as four parameters, so
the act's signature separates what is being ASKED from what this box does with
the answer — and argv's reading of those words and the act's parameters are one
object rather than two lists in agreement.

**One artifact, three renderings, and the operator picks which they can take**
(bl-1554, bl-a8fd). The envelope is one line of compact JSON and everything
else is a rendering of it: the **QR symbol** for a camera, the **line itself**
for the paste box an android seat offers — *one line of JSON beginning
`{"yog-enroll": 1`*, in that screen's own words — and `--into <dir>` for a box
with neither, which lays the envelope down as the four files §4.6's entry is —
six, from a server holding rendezvous material (below). There is one place the
envelope is built (`Enrolled::envelope`) and the renderings are of that, so no
two of them can disagree.

**The envelope carries the rendezvous pair when the reply does** (bl-5378, on
yog bl-9043, edition 20). REMOTE §8.4's reply grew two optional keys beside the
six facts, `rendezvous_pub` and `pairing_salt` — the server's public rendezvous
key and the pairing salt, 32 bytes of hex each, present together or not at all
— and a phone reads its roving material (REMOTE §13.3) from exactly this
envelope. `Enrolled` re-said a fixed six, so the symbol this seat drew for a
phone dropped both and the phone could never find a moved server; and `--into`
wrote the four entry files and not the two `crate::channel::rendezvous` reads,
so a foot filed that way could not rove either. Now `Enrolled` carries the pair
as one `Option`, the reader refuses one key without the other, the envelope
writes both when present (still `serde_json`'s sorted order, pinned byte for
byte against the server's own corpus fixture), and `--into` lays down
`rendezvous.pub` (`0644`, as the server mints it) and `pairing.salt` (`0600`)
beside the entry. A loopback server sends neither and everything here is what it
was. The caption stays the three facts that are not material — the salt is
shared with the server, so it is material.

**The route the NEW box will dial is stated, not assumed** (bl-971c, on yog
bl-fec6's `--at`). REMOTE §8.4's request carries an optional `address`, and it
is a fact about the device being enrolled rather than a setting on the server:
the box being provisioned is by definition not the box the server runs on, so
the route it reaches that server by need not be the one the server wrote for
itself — an emulator reaches its host through the emulator's alias, a phone
over the LAN, an overlay peer by name. Unstated is the server's own
`wire/address`, which is right only for a device sharing this box's view of it.

The seat could not state it, and **the failure was silent at both ends**: the
four files `--into` wrote were correct in every byte except `address`, so the
foot came up, dialled an address it cannot reach, printed nothing and waited,
while the server held the registration and the roster showed the client absent.
Nothing anywhere said why. The only workaround was `yog gesture` on the
server's own box with the reply taken apart by hand — which is available to
every operator except the one a seat is for.

So `lernie enroll <workspace> <name> <grade> [--at <host>:<port>] [--into
<dir>]`: **two optional words, independent and in either order.** They address
different things — `--at` the far box, `--into` this terminal — so neither
implies the other and one refusal teaches both. What the seat settles is only
that a word was given a value; whether the value names an endpoint a device can
dial is the server's judgement, which it makes on its own address and a stated
one alike and answers with the remedy it names. A second copy of that rule here
would be a second authority on addresses that could disagree with the first.
`address` is the field on the wire and `--at` is the word argv spells it with —
the same word yog's own line reader takes, so an operator who learned it at one
face has learned it at both. **The window states none**, and that is a control
it owes rather than a `None` to fill in: it has no field for a route, and an
operator sitting at a window is on a box that dialled this server successfully.

**Picking one is picking one** (bl-768a). `--into` says *this box has neither a
camera nor a paste box; write it down for me*, so once the four files exist the
symbol and the line are a copy nobody asked for — and it is the copy this seat
can never reach afterwards: a terminal scrollback, a `tmux` buffer, a
`script`/`asciinema` capture, the job log of an unattended run. A run that
files therefore says the caption and the receipt and nothing else, and the
receipt is what names where the key is. The rule is not a second custody
policy; it is the one below, read on the one path where the operator has
already named the copy they want.

**It printed the picture alone until bl-1554, and that closed the act to two of
the four components.** A **foot** is by definition a box the operator is not
sitting at — no screen, no camera — so the one component the act exists to
provision was the one it could not reach; and the **phone** asks for the line
verbatim, which the seat was drawing a picture of and discarding. Both had the
same workaround — spell the envelope by hand through `lernie ask` and reassemble
it against §8.4 — and the material was spent either way, because the server
shreds the key and a second enrollment under one name is refused.

**The custody argument does not reach the text, and that is why this is not a
relaxation.** What the rule below rules out is a copy *nobody chose*: a cache, a
log, a temporary file. The envelope was on the screen either way, drawn as a
picture of itself, and a photograph of a private key is a private key.

**Nothing is written UNASKED, and it is asserted over the tree.** No file, no
cache, no log line, no temporary anything, on either face: with no `--into` the
command line holds the material in locals, the window holds it in
`Model::enroll` and drops it with the pane, and closing that pane is a control
whose whole product is the forgetting. `seat::enroll`'s suite walks a throwaway
root before and after the act and compares — over the **tree** rather than over
the paths the code happens to know about, because a defect here is precisely a
path nobody thought of. A destination the operator NAMED is walked the same way
and holds exactly the four files (six with the rendezvous pair).

**And a destination never costs the material.** The server has already minted
and shredded by the time any of this runs, so a filing that FAILED says
everything on stdout — the screen is then the only place the material can be —
and only the exit code reports a directory that would not take it. That is the
same fact the paragraph above rests on, read the other way: what is withheld
beside a receipt is withheld because the files hold it, so no files means
nothing withheld. Two more rules follow from it and neither is a convenience: a
**symbol that will not fit** no longer refuses — it is one of
three renderings and the other two still work, where refusing would burn a name
over a drawing — and a **file already present** is never written over, because
re-issuing distrusts nothing and a clobbered `client.pem` leaves two live
certificates under one identity with the surviving key belonging to neither.
The files are created with the mode on the `open` rather than corrected
afterwards, so the key is never readable off the account even for an instant.

**The QR encoder is this crate's own** (`src/qr/`), byte mode at correction
level M, no new dependency. A QR symbol is a fully specified algorithm rather
than a research problem, and the manifest's dependency set is an approved list
a ball has to argue its way onto. REMOTE §8.4 measures the envelope at 1567
bytes from a loopback server and ~1730 with the rendezvous pair, and states the
rule as *PEM as minted, at level M or lower*; level M carries 2331, and the pair
is what took the envelope past level Q's 1663 — a level this encoder never
emitted, so nothing moved here, and `reply::enrolled`'s suite builds an envelope
at the measured sizes and encodes it. The encoder's own module doc holds the rulings, and its suite
pins three whole symbols and all forty versions against an **independent**
implementation — because an encoder that agrees with itself proves nothing, and
two reference implementations disagreeing with each other is how two of this
one's choices were settled.

**And the drawing is a scale rule, not a constant** (`src/ui/enroll/symbol.rs`,
bl-5e0e). The symbol's whole job is to be read by a camera once, before the
material is forgotten, so two things about how it is painted are properties of
the act rather than of taste. It is drawn **as large as the pane allows** — the
module pitch is what the available square divides into, where a four-point
constant put a realistic §8.4 envelope's 153-module symbol in a 644-point
square that overflowed the central panel at every window size this seat is
photographed at, with no zoom and no way to make it bigger. And the pitch is a
**whole number of device pixels**, drawn as **one mesh** rather than a filled
rectangle per module: egui feathers every fill by a device pixel, half of it
proud of the edge, so at a fractional origin — the only kind a layout offers —
a module's own edge pixels come out grey and the contrast a decoder needs is
spent on anti-aliasing. The suite renders and reads every module back off the
pixels at one and two device pixels per point, which is the only altitude that
can see either defect.

**The symbol is geometry, so the assertions are geometry.** A QR symbol has no
glyphs, so `crate::paint_probe` — the one walk over painted *text* — has
nothing to say about one. The pane's words go through the probe; the picture is
asserted as the module matrix, which is the thing that is right or wrong. A
symbol drawn at the wrong scale carries the same bytes; a symbol with one wrong
module does not.

**And it is the one pane that covers another.** The shell's rule is a bar
rather than a modal, because a refusal about one pane must not stop the
operator reading the other three. The enrollment earns the exception: what it
holds is a private key on a screen, the whole act is *look at this now and
close it*, and a conversation legible behind it would invite the one thing the
material must not have, which is a long life on a display.

### 4.16 Interface parity: a control that fires an op says which op (yog's docs/PARITY.md)

**The authority is yog's `docs/PARITY.md`, and this crate does not get a vote
on it** — the same relation `docs/REMOTE.md` has to the wire. The operator's
requirement is that this seat and the android client have interaction parity:
*not identical placement, but if something is interactable in one it must
exist in the other*, and drift must be caught mechanically rather than noticed
by hand.

**The ledger is now EMPTY, and that is the state it was written to reach**
(bl-8ffa). Every op the roster classes `control` carries one here; what
`parity.toml` holds is the record of how each row left. The test over the
committed file used to assert it was non-empty as a vacuity guard, which made
the finished state unreachable for the one file that matters — the guard the
reader needs is two-directional over text the suite controls, and it already
had one (`src/snapshot/parity/tests.rs`). A row returns the moment upstream
adds a control-classed op this seat has no surface for.

Four facts follow, and each is a decision this tree implements rather than one
it makes.

- **The subject is an op, never a widget.** What is judged is exactly what
  crosses the §8.5 control boundary. An interactable that crosses no wire — a
  focus, a scroll, a selection, a pane that closes — is a view, and views are
  out of contract. So the roster's unit is the wire's `op` token, which is
  already the help row's key, the envelope's discriminant and the corpus
  filename: one name, seen four times, and no translation table.
- **Neither client is judged against the other.** Components meet at the
  interface and never pairwise. The one authority for *which ops owe a seat a
  control* is yog's help table, published through the `surface` field on the
  vendored corpus's `reply/help` rows — `control`, meaning every seat-class
  client owes it a discoverable interactable, or `machine`, spoken by programs
  and owed nothing. Raising the bar is an edit at yog and a corpus refresh
  here; this repository maintains no list of what it owes.
- **The tag is machine metadata and the label stays a human word.**
  `src/ui/act.rs` writes `act:<op>` into the AccessKit node's `author_id`,
  which exists for exactly this. It is written where the accessibility tree
  exists — the suite, by the dev-only ruling in `Cargo.toml` — and the CALL is
  unconditional, so the decision about which ops a control fires is recorded at
  the control and nowhere else.
- **A deliberate absence is config, cited, and loud.** `parity.toml` records
  one line per control-classed op this seat does not surface, each citing the
  ball that will build it; `src/snapshot/parity` prints the whole roster on
  every run and fails on a line that has gone stale (the op is surfaced now) or
  rotted (the roster no longer classes it a control). Deleting a line
  re-reddens the gate and changes no code, which is the severability test.

**And the ledger's first line since it emptied was `doctor`, and it left the
way the rule says a line leaves** (PROTOCOL 17, yog bl-28f4; bl-183b filed it,
bl-9bbb paid it). It is round-1 ruling 7 — *one gesture answers "is this box
wired": wire material, address, workspace rows, role credentials, git identity,
enrolled feet, protocol pins* — whose server half landed upstream first. Its
reply kind sat in `corpus/unpainted/` for §4.9's reason, so the decode and the
pane arrived in one commit and that commit deleted the line: `doctor` is tagged
on the roster strip's control (§4.21), and the fixture moved to
`corpus/answers/` beside it. The two halves of a protocol bump are decided
separately: the number moves for the wire, the paint moves for the panes.

**And PROTOCOL 18 added two more, which is the ledger doing the thing it was
written to do** (bl-c515). `proposals` and `proposal` are the learning loop's
operator half and both are classed `control`, so both owe this seat an
interactable and neither has one — the reply is decoded, both gestures are
composed and the command line paints them, and the GLASS is bl-a1d6's. The
lines said so and cited it, which is the difference between a gap and a debt
with an owner — and bl-a1d6 paid it on the config pane (§4.30), deleting both.
Note what does NOT put a line here: `answer` gained a `scope` at the
same bump and stays tagged, because the roster's unit is the op and the op is
surfaced — how RICH a surfaced control is, is not this ledger's question and
must not become one, or the file stops being a list of absences.

**The inventory instrument is §4.11's harness and never a second one**, for the
reason a second paint walk was refused: two instruments disagreeing about what
is on the window is a defect about the instruments. Presence is the whole
claim — a tag on a dead button passes, and driving the tagged node to assert
the envelope it emits is a later rung. Unproven is red: a control on a screen
the walk never visits fails honestly, so the walk's world set is part of the
instrument.

**The walk visits every width as well as every world, and takes the union**
(bl-dfda). In the narrow shape one column is on the glass, so a control on
another column is not in that width's tree — it is one navigation away rather
than absent from the seat, and *how far away* is `src/snapshot/reach.rs`'s
question and stays there. Union, because presence is what this asserts;
intersection would be reachability asked by the wrong instrument.

### 4.17 Role tuning: the settings pane, and a standing read keyed on a pane (bl-4a2c)

*The seat's own preferences are §4.46 item 4's pane (`/settings`); this one stays the workspace's.*

`src/ui/tuning.rs`. **The seat's second covering pane, and the first that is a
place rather than a moment.** §4.15's enrollment covers the conversation because
what it holds is a private key and the act is *look at this now and close it*.
This one covers it for a plainer reason: it is a surface an operator navigates
to, acts in, and comes back from — which is what §4.11's reach assertion was
written about and recorded that this seat did not have. It does now, and the
walk covers both panes.

**Nothing on it is a preference.** Every row is a fact about the WORKSPACE, held in
that workspace's config on the server, and every control is one gesture across
the §8.5 boundary. A seat that kept its own copy would be a second authority for
a file it does not own — so the pane holds exactly one piece of state, the draft
of an assignment, because two words typed into two boxes are not a fact about
anything until they are sent.

**The read is standing, and it is keyed on the PANE rather than on the aim.**
`Standing` gains a flag rather than a second aim: the pane is about whatever the
window is aimed at, and closes when that moves, so a second address would be a
second authority for one focus. Keying it on the pane is what keeps a seat with
no configuration surface open from asking about a file nobody is looking at on
every beat, forever. What that buys is the property every control here has:
**a write lands and the next answer says so.** Nothing on the pane is ever this
end's prediction about what a `litany config` did, and that matters because it
can refuse.

**The absence is the fourth choice, and it is not a word.** `effort` offers
three levels and *off*, where off is the wire's `null` — the line removed, the
provider's own default governing. The seat's list is therefore `Option`s and not
strings, so the seat a control paints hands back exactly the value the envelope
carries and no translation table exists to drift. A level the four seats do not
name — a config written by a hand, or by a newer server — is §4.9's rung 3 on
the glass: painted as itself, because four unselected seats would otherwise say
the level is *off*, which is a different claim from the one the file makes.

**What each row's provider offers stands beside the roles** (bl-b7d9;
`src/ui/tuning/offers.rs`). A role left on a model id its provider does not
offer used to be painted like every healthy row, and the editor that mends it
was two bare text boxes — while `lernie model` (`src/seat/model.rs`) already
asked the row what it offers before writing. The pane now spends that same
read, the `models` op and its `Reply::Models` decode: while it is open, every
provider row its roles and its draft name is asked on the roles read's own
standing (`Open::Tuning` carries the rows), and each answer is stamped with
the row it was asked of (`Said::Offering`), because the reply names none. It
stands rather than being posted as §4.24 posts it, because the rows to ask
about ARE the standing roles answer: the question set is derived on every
settle and no ledger of who was asked exists to drift. Two things follow on
the glass. **A row whose provider's listing lacks its model says
`<provider> does not offer <id>`, on that row, in the error accent**; a
provider with no listing — a refusal, no answer, an empty list — marks
nothing, the CLI's rule verbatim: absence is not evidence. And **the editor
offers the draft provider's ids as seats that fill the model field**, which
stays free text, because a list can be stale and the CLI's own stance is a
warning, never a refusal.

### 4.18 The records pane: the conversation's own ledger, on the tuning pane's standing (bl-2cf7)

`src/ui/records.rs`. **The third covering pane, and §4.17's shape one noun
over**: the subject is the SELECTED CONVERSATION where tuning's is the aimed
workspace, and every joint carries across. bl-213c built the conversation's ACTS —
each answers a captured run, a kind this seat already painted — and left its
records, each answering a kind nothing decoded. This pane lands the two an
operator actually reaches for: **what the loop did** (`steps` — each step's
framing, cost, retries, wound and sign-in want, under the view-level
orphaned-tail banner) and **what it touched** (`files` — the walked worktree,
whose ABSENCE is a different claim from an empty listing and paints as a
different sentence, plus where the work lands when the listing does not reach
it).

**The reads are standing and keyed on the PANE** (`Standing::records`), for
§4.17's reason verbatim: nothing on the glass is ever this end's prediction,
and a seat with no records surface open asks nothing on any beat. It goes down
when its subject moves: selecting another conversation or aiming at another
workspace retires the pane and every answer under it, because a pane about *the
selected conversation* left standing over a new selection would paint one
conversation's records under another's name for a beat.

**It held no state at all when it was two reads, and it holds one draft now
that it has an act** (§4.29). That sentence used to read *"not even a draft,
which is one less than tuning holds"*, and it was true of a pane whose whole
content was answers. The spine's `fork` is the thing that changed it, and the
change is §4.17's shape arrived at rather than departed from: one piece of
state, the draft of the gesture, because two words typed into two boxes are
not a fact about anything until they are sent.

**One control opens it and carries every standing read's tag** (`act:steps
act:files act:rail act:governing act:agent act:inbox`), leading the composer's
second row — the composer being the pane that already acts on the selected
conversation, and the chat pane being a pure projection no control may be the
first exception to. The pane's seventh read is not among them: `step` is
addressed at one row rather than at the pane, so its control hangs on the row
(§4.32). Nothing about this pane is left in the exemption ledger.

**A step row says its money** (yog REMOTE §9.23; bl-9111). Each row carries
that step's own bill, priced by its own `(provider, model)`, and the row's
headline paints it beside the count — `N tokens — $x.yz`, the count alone where
the server has no price table, and `at least $x.yz` where some of the step's
tokens no rate priced (`crate::reply::spend::priced`, the one sentence every
surface says a count in; §4.41).

### 4.19 The decision queue: the pane about no focus (bl-f0ef)

`src/ui/queue.rs`. **The fourth covering pane, and the first whose subject is
neither the aimed workspace nor the selected conversation.** REMOTE §6 is the
server's *what needs you* — a flattened roster across every enumerated
workspace, filtered to the rows that are asking — and `attention` names no
workspace, so the read **fans** and the pane is the union across channels, the
way the roster is (§4.7, REMOTE §8.2). Three ops land with it: `attention` is
the read the opening control fires, `seen` answers one row, and `flag` raises
one.

**The flag is why the pane exists, and it was the sharpest case of a field held
for no glass.** REMOTE §9.11 put it on the wire at PROTOCOL 4 — *when* somebody
raised a flag on a conversation and *why*, in the raiser's own words — and the
point of a flag is that a second party asks the operator to look. A seat that
carries the field and paints it nowhere is the one place that ask goes to die,
which is what bl-d774 recorded and this ball spends. So it leads a row's detail,
above the failure clause and the parked invocation: it is the only line on a row
that somebody *wrote* rather than something the server observed.

**Why a row asks is the server's sentence, not its tokens** (bl-5507). Every
queue row has carried `says` since PROTOCOL 12 — the signals read out with the
remedy, *was cut off at its output cap — nothing was committed; raise the
role's max_output_tokens…* — and the pane, the ask box (§4.42 item 8) and
`lernie attention` all paint it in place of the signal list. The signals stand
in only where `says` is absent or empty, and `QueueRow::why` is the one place
that choice is made.

Five decisions, and each is a choice rather than a mechanism:

- **Its control hangs off the roster, above the channels and off no row.** The
  roster is the one pane that is already the union across channels, which is
  what the queue is; and because `attention` addresses nothing, the control
  needs no aim and is offered on a seat that has aimed at nothing — which is the
  seat most likely to be asking the question. It sits outside the roster's
  scrolled region for the reason the heading does, and stands down under a
  covering pane exactly as the two per-workspace controls do.
- **Nothing on the glass can retire it.** §4.17's pane goes down when the aim
  moves and §4.18's when the selection does, because each is *about* one of
  them. This one is about everything, so aiming and selecting leave it standing
  — which is the same rule read on a pane whose subject nothing on the glass can
  move, not an exception to it. The read stands on the PANE
  (`Standing::queue`), and there the argument is sharper than tuning's: a fan
  costs one round trip per channel per beat, so a seat with nothing open pays
  none of them.
- **The read is one-shot even against a server that holds** (§4.2, bl-09e1):
  the asker takes the first frame of each channel's answer and hangs up, so a
  held `attention` lane costs a pass one round trip per channel and never its
  30 s hold.
- **A fan's answer replaces its OWN channel's section**, leaving every other
  standing — §8.2's *"a refusal is one entry's, never the set's"*, the shape the
  roster already keeps.
- **A row is addressed off the ROSTER, within the channel its section came
  down.** A queue row names its workspace as *its host* names it, which is not
  what a gesture from this box carries when an entry renames. The window's half
  of that mapping has one home — a roster row and `ui::Channel::address` — so
  `Model::workspace(channel, workspace)` asks it there, **in that channel's roster
  chunk alone** (bl-666f): workspace names are per server and `home` is every fresh
  server's bootstrap workspace, so a walk of the whole roster aimed a second server's
  rows at the first's `home`. Every door — *go to it*, `seen`, `answer`, and the
  ask box (§4.42 item 8, which also requires the row's workspace to resolve to the
  aim, not merely its id to match the selection) — carries the section's
  channel beside the row. Resolving within the stamp is sound because the
  stamp is the channel that ANSWERED: it once was the *aimed* channel instead
  (`src/offframe/poster.rs`), which is why this bullet used to call it a
  display fact and resolve without it, and **that is spent** (bl-c70d) —
  `seat::route` answers the seat-side name it chose (§4.7) and the poster
  stamps it, so the aim is not read on that pass at all. And a row for a workspace this seat
  holds no name for is honestly unaddressable, and says so in the roster's own
  sentence, rather than being aimed at a different workspace by a guess.
- **`flag` is on the composer's second row and not on this pane**, which is the
  one placement decision the ball made outside it. A flag is *somebody asking
  the operator to look at this conversation*, so it is raised while looking at
  it — and this pane covers the conversation, so a control here would flag
  something the operator cannot see. The queue is where a flag is READ; the
  composer, which is already the pane that acts on the selected conversation
  (§4.11), is where one is raised. It sits in the row that does not advance the
  turn, beside `stop` and `retarget`, because a flag *"changes nothing else — it
  does not stop, message or touch the conversation"*. Its reason box is a
  required parameter rather than an arming, so the control is **disabled** until
  there are words (the tuning pane's `set`, not the composer's second start) and
  the box is **spent** on firing, the way a deposit is.

**What was painted and not actionable, and now is both.** `held` — the
invocation parked at the conversation's capability boundary — was read and
shown before anything could answer it, because saying what is parked is worth
more than saying nothing. This paragraph used to record that the control which
releases or declines it *"belongs to the tool-host surface this seat does not
have"*; §4.34 built it here instead, and the three verdicts are on this row —
the pane whose whole question is *what is waiting on you* being the pane a
parked call belongs to.

**And one control on it crosses no wire.** *go to it* aims at the row's workspace and
selects its conversation — the two doors a click on the roster and a click on
the list already spend — and stands the pane down. It carries no `act:` token
because §4.16 puts a view out of the parity contract: the ledger's unit is an
op, and an aim is not one.

### 4.20 Destructive acts: the arming is an enablement, and an unmaking is a place (bl-48fa)

`src/ui/unmake.rs`, `src/ui/model/unmake.rs`, `src/verbs/workspace.rs`. **The
seat's fifth covering pane, and the first control in it that destroys
something.** Everything else this window affords adds or changes and is undone
by doing the other thing — aiming, selecting, depositing, nudging, starting,
enrolling, and the four tuning gestures. `delete-workspace` is the first that
is not, and bl-4a2c refused to build it on exactly that ground: the first
destructive control must not invent an arming, a confirmation shape, a wording
convention and a test scaffolding in the same commit that also happens to unmake
a workspace. This section is those conventions, and the op is their first
customer.

**The idiom, and every later destructive control follows it.**

- **An unmaking is a PLACE, not a control you pass.** It gets a covering pane of
  its own, opened by a control on the surface that names its subject, and the
  pane holds nothing else. Three things follow from that and each is why. A
  covering pane is the **only placement identical in both layout shapes**
  (§4.11): the composer stands down off the conversation's own column in the
  narrow shape, and a list row is aimed by a click with the next row one pixel
  away. It is already inside the two instruments — `src/snapshot/reach.rs`'s
  bounded walk and the parity walk's world set — so a new one costs a row in
  each rather than a new kind of assertion. And a routine pane is a surface an
  operator moves through quickly; a mis-aimed click there must not be able to
  land on this.
- **The pane states its subject before it offers the act** — the address a
  gesture carries and the channel it goes down, in the roster's own two facts —
  because what is being unmade is the one thing a confirmation is about, and no
  control has room to say it.
- **It holds that subject rather than following the aim.** The roster stays
  live and clickable under a covering pane, so an unmaking that re-read
  `Model::aim` when it fired would unmake a workspace the operator armed a different
  one for. This is the opposite of what the tuning pane does (§4.17) and it is
  the opposite for the right reason: a tuning write that lands on the wrong workspace
  is undone by writing the old value back.
- **The way out comes first**, in the layout and therefore in the tab order, and
  it says what it keeps rather than what it abandons (`keep it`, not `cancel` —
  `cancel` names the destruction as the thing in progress, and nothing is in
  progress). The control an operator reaches for by reflex must be the one that
  changes nothing. Escape reaches it too, on `Model::escape`'s ladder, and can
  do nothing else: a key that could arm or spend one would be the second surface
  §4.11 refuses to be.
- **The arming is the subject's own name, and where the wire makes it an
  arming the seat makes it an ENABLEMENT.** yog refuses `delete-workspace`
  unless the workspace is the server's own, nothing in it is live, and the typed
  name matches exactly — so the act is **disabled** until the box holds that
  name, and **disabled rather than absent**, which is the tuning pane's `set`:
  the parameter is missing, not the subject, so the control stays on the glass
  saying what would fill it. The refusal is spelled out beside it, because a
  greyed control says a thing is not live and nothing about what would make it
  live.
- **Where the wire makes `typed` a PARAMETER it stays one.** `delete-agent`'s
  box is the other case and does not move: an empty one deletes the one
  conversation and its name typed back is what admits the descendants, so both
  values are gestures somebody meant and its control is never disabled
  (§4.11, `src/ui/composer/acts.rs`). **The seat reads which of the two a
  `typed` is off the wire's own grammar and invents no policy.** That is why
  bl-213c's composer row is not the precedent this section generalizes — it is
  the other half of the same distinction.
- **The arming is never spent on firing.** A refusal is the COMMON case here:
  the server declines while anything in the workspace is live, which is exactly
  the state an operator reaches for this control in. Clearing the box would
  charge a retype for the server's *no*, a toll on the safe path — where
  clearing a sent message costs nothing, because it was sent. Firing is said
  instead (`asked — waiting for the server`), for the reason the enrollment says
  `minting…`: the outbox is drained within the beat, so nothing else on the
  glass can answer *has this been asked*.
- **The act carries `act:<op>` and the pane's own controls carry none** (§4.16).
  The opening control crosses no wire — it opens a pane — so it is tagged with
  nothing, the same division `enroll a box…` keeps with `mint`.

**What this does not close.** `reply/deleted` is `{kind, ok}` and this build
decodes no such kind, so a successful unmaking answers with §4.9's rung-3
notice rather than with the pane standing down — which is exactly what
`delete-agent` has done since bl-213c, and is a fact about the reply vocabulary
rather than about this idiom. The pane says what it asked and leaves the answer
to the notice bar.

**Amended: what earns an arming is SCOPE, not destruction** (bl-3686). This
section was written with two destructive customers and read as though
destruction were the test. It is not, and yog's own §3.6 doctrine already says
what is — *"typed-name confirm iff the verb destroys objects beyond the one
named on screen. A leaf agent is the row under the pointer … retyping its name
proves nothing the dialog does not already show"*. The operative half of that
sentence is **beyond the one named on screen**; `destroys` is what its two
customers happened to do. The cascade (§4.32) is the first act that is beyond
the row and leaves everything standing: `stop children` kills the drivers of
conversations the control does not name, and `nudge` starts each again. It
takes an arming, and it takes it for the same reason a subtree delete does —
not because what it does is permanent, but because the operator has to be able
to say they meant the ones they cannot see. Undoing it is `nudge` **per
conversation**, which is why *"undone by doing the other thing"* is not the
answer here either.

**And where the wire offers no `typed` at all, the arming is the seat's.** The
two customers above read their shape off the wire's grammar — `delete-agent`
carries a `typed` parameter, `delete-workspace` is refused without a matching
name — and the rule *"the seat … invents no policy"* was written against those
two. `stop` carries neither: its cascade is a boolean flag the server takes on
trust. So the arming there is an **enablement** this seat holds and never
sends, compared against the conversation's own display name by the same rule
the server applies to `delete-agent`'s (surrounding whitespace forgiven,
nothing else). What the seat still invents no policy about is the *act*: the
flag it sends is exactly what the operator asked for, and the server remains
the only thing that can refuse.

### 4.21 The window's own reads: the three ops that name no workspace (bl-40ec)

`src/ui/roster/acts.rs`, `src/ui/commands.rs`, `src/ui/find.rs`,
`src/ui/model/window.rs`, `src/verbs/window.rs`. **The sixth and seventh
covering panes, and the one control that opens nothing.** §4.19's queue was the
first surface whose subject is neither the aimed workspace nor the selected
conversation; these are the rest of that family, and the family has a
mechanical definition rather than a taste: **an op with no `workspace`
parameter cannot name a channel, so its subject is every channel this box
holds** (`crate::verbs::Verb::addresses_a_workspace`). Four ops were in it
when this ball landed — `attention`, `workspaces`, `help`, `search` — and three
of the four reached no gesture at all. The trail's `ops` joined them at §4.27,
which is the definition working: nothing about that ball was a decision to make
the trail channel-wide.

**The mechanism all three needed was one, and it was in the poster.**
`crate::seat::route` resolves an envelope's workspace over this box's entries
and falls through to the flat root when there is none. That is right for
`lernie ask`, where an operator named one channel by hand, and wrong for a
frame: a window that composed `workspaces` would have asked this box's own
server and said nothing about the rest, which is bl-0d54's defect one surface
over. `src/offframe/poster.rs` now reads the envelope — a gesture naming no
workspace is **fanned** over every channel the standing set holds, each answer
stamped with the channel it came down and each failure reported against it.
`crate::seat::fan` is that rule on argv and `crate::cli::Decided::Fanned` is
it in the command line, so all three surfaces read one predicate.

**The bare channel is enumerated when it is provisioned, or when it is the
only thing there is** (bl-b858). The fan asks *every channel this box holds*,
and a flat root with no material at all is not one — it is an ABSENCE. On a
seat for a server somewhere else, provisioned exactly as `enroll --into`
instructs, it was enumerated anyway, so a paragraph of refusal beginning *"if
a server runs on this box"* stood above every working channel on every roster
read, on a box where none does and none is meant to; the read had worked and
its answer was underneath. The two arms are one sentence read twice rather than
a case and its exception: a box that holds nothing at all still gets the
refusal and its whole recipe, because that is exactly the box the recipe is
for, and silence there would answer an empty fan with nothing.

**Provisioned is not reachable, and only the first is asked.** Half-provisioned
material, a `:0` address, a channel that will not open — each is a fault about a
channel this box *does* have and each still says so in its own section. What is
dropped is the one reading that is not about a channel at all.

**And `lernie entries` keeps it, which is the same rule and not an
inconsistency.** `crate::seat::holds::listing` answers *what does this box
hold*, and for that question the absence IS the answer — a row saying the bare
channel holds nothing is what was asked for. The fan answers a question about
workspaces, where the same sentence is noise above the answer. One surface prints an
absence because it was asked about it; the other does not because it was not.

**§4.30 later narrowed what the poster infers from that predicate, and the
narrowing belongs here** (bl-4855). *Names no workspace* establishes that a
gesture cannot name a channel by its workspace; *therefore every channel is its
subject* is a second step, true of these three reads and false of a `config`
write about one server's own file. The poster no longer takes it: a gesture
carries the channel it is addressed to, and the fan is what addressing it to
none means. Nothing about these three changed — they name none, and every
channel is what they mean. The leg
itself has one spelling, `crate::offframe::down`, shared by the asker and the
poster.

**`workspaces` gets a control and not a pane, and the pane it belongs to is
the roster.** yog's `docs/PARITY.md` §2 — *the interactable a query owes a seat
is the affordance that reaches the view it populates* — puts it there and
nowhere else: the roster IS that view. What it adds over the standing read's
cadence is the other half of the catch. A channel that cannot be reached says
so under its own header (`Model::unreachable`, §8.2, bl-e620), and until this
control existed that sentence appeared only on a beat an operator cannot see
and could not ask for again. The refresh is the ask.

**The two panes are §4.19's shape one noun over**, and every joint carries:
their controls hang off the roster above the channels rather than off a row,
they are offered with nothing aimed at, they are sectioned per channel with the
roster's own header, and an answer replaces its own channel's section leaving
every other standing.

Four decisions are theirs rather than the queue's:

- **The reads are POSTED, not standing.** The queue stands on its pane because
  *what is asking* changes under the operator while they look at it. Neither of
  these does: a verb table is fixed for the life of a server build, and a
  search answers a needle somebody typed. A standing read here would spend a
  round trip per channel per beat forever on an answer that cannot have
  changed, and in the search's case would re-scan every store on the box while
  the operator is still typing. So each is composed into the outbox by the
  control that asks it, and `Standing` grows no flag.
- **One field holds both panes, and it is not a pair of bools.** No two
  covering panes ever stand together, so two flags would make *both open* a
  representable state that only the paint order resolves — two representations
  of one fact. `Model::lookup` is `Option<Lookup>`, one door closes either, and
  Escape's ladder gains one arm rather than two.
- **The server's `help` is not `lernie help`, and only one of them has an argv
  row.** `crate::verbs::help` answers *what does this BINARY take*, from a
  table compiled into it, with nothing provisioned and no server up; the wire's
  `help` answers *what does that SERVER offer*. Two subjects, one word, and
  argv has one namespace — so the wire op is reached from the window, and
  `lernie ask '{"op":"help"}'` stays the door argv already has for every op with
  no row. It is the roster-without-a-row shape `prepare`, `prompt`, `effort`
  and `priority` already occupy, taken for a fourth reason. `search` is a plain
  row and `lernie search <text>` fans like `lernie workspaces`.
- **And the commands pane is the table this seat is JUDGED by.**
  `src/snapshot/parity/roster.rs` reads the `surface` field off the vendored
  fixture of this same shape, because it is the one home for *which ops owe
  every seat a discoverable interactable* (§4.16). So the pane an operator
  reads and the ledger that reddens for a missing control come off one answer.
  The classification is painted rather than hidden: an op marked `machine` is
  one nothing here owes a control, and saying so is the difference between a
  short pane and an incomplete one. It rides verbatim on the glass while the
  roster refuses a word it has no reading for — the pane says what an op is
  for, and the roster decides what this seat OWES, where a guess would quietly
  shrink the obligation.

**What this does not close, and it is upstream's** (yog bl-ef16, the recurrence
of yog bl-22ab). A `search` hit spells its workspace and its project as the
SERVER'S OWN ABSOLUTE PATH while every gesture this box composes carries a name
(REMOTE §8, *"paths never cross the wire"*). So the keys a row spells are the
keys the acts take and the values are not: feed one back and it earns `unknown
workspace`. The find pane therefore offers **no control that spends a hit**,
and says why on the glass rather than in a comment — a *go to it* here would
have to guess a name off a path, which is the mis-aim `Model::workspace` exists to
refuse one layer down, and a control that silently did nothing would be worse.
That is §4.19's decision about a parked invocation read one noun over: saying
what was found is worth more than saying nothing while the address it names
cannot be spent. **It is not in `parity.toml`** — the op is surfaced, and that
file records absences.

**The doctor joined the strip on the verb table's terms, with one difference
the aim decides** (bl-9bbb; yog REMOTE, *"And one read asks all of it at
once"*). `src/ui/doctor.rs`, `src/ui/model/doctor.rs`, `src/verbs/doctor.rs`.
Bare, `doctor` names no workspace, so it is this family by the mechanical
definition: it fans, each channel answers for its own box, and the pane is
sectioned per channel with the roster's header. Its workspace is OPTIONAL —
upstream's only workspace-addressed read where it is, because the box it is for
may hold none — and naming one adds that workspace's two checks and routes the read
down that workspace's channel alone. So the one control asks the deeper question
when the window is aimed and the bare one when it is not, rather than a second
control asking the second; argv spells both as `lernie doctor [<workspace>]`, a
door in `crate::verbs::doors` because an optional parameter is not a row. The
read is POSTED, `help`'s cadence, and the held sections are cleared on asking,
which `help`'s are not: two asks may have two subjects, and a bare ask's
sections left under an aimed one would say every channel had answered the
narrower question. Every line on a row is the server's — the check, the fact,
the remedy verbatim — and the one thing the pane composes is whether a row
holds, read off the row's own `ok`. The frame's top-level `ok` is not a verdict
over the rows and is never read as one: upstream carries no tally, and the
command line's head is a count of the rows that fail.

### 4.22 A lost reply leaves an act in doubt, and the recovery is a read (§3, bl-3969)

`src/channel/reach.rs`, `src/ui/model/posted.rs`, `src/offframe/poster.rs`,
`src/ui/model/notice.rs`. yog's REMOTE §3 states the contract and yog bl-d1f1
gave the disk bus its mechanical half; the wire has no reply slot to answer
into, so **this end's half is behaviour**:

> A lost reply leaves an act IN DOUBT, and the recovery is a read — never a
> resend … a client whose act earned a transport error instead of a reply
> paints the failure and consults the world, which is the durable record …
> **Asks are the opposite case and re-ask freely.**

**Nothing wire-visible moved.** `PROTOCOL` stands, the corpus is unchanged,
and no envelope grew a field: an idempotency token is exactly what §3 refuses
to mint. What landed is four decisions.

**Sent exactly once was already true, and is now asserted rather than
observed.** There is no retry, no backoff and no reconnect-and-replay anywhere
in this crate; `Link::compose` is a `mem::take`, the drained envelopes live in
the poster's own loop, and no arm puts one back. The crate's one repetition is
`offframe::pump`'s cadence, which re-derives the standing READS from the model
and never touches that queue. A property nothing enforces is a property that
lasts until somebody adds a helpful retry, so the beat that proves it scripts
a server to hang up once and answer the second dial, and asserts the far end
heard the gesture **once** across both.

**The transport says whether the request crossed, and the seam is one it
already had.** `Channel::ask` answers `Reach::Unsent` or `Reach::Unanswered`
rather than a bare string, and the line between them is `Channel::dial`
returning — whose own doc already called what it hands back *"a socket with a
request on it and no answer yet read"*. So the classification is structural, in
the way `Channel::wrote` reads a typed `rustls::Error` rather than its wording,
and not a judgement about a message. **Both arms are one sentence to a read**
(`Reach::said`), because a standing question is answered in place and asking
twice is asking once — the asker and the follow lane carry no arm for a fact
they would do nothing with.

**A version mismatch is on the Unsent side and an unreadable preface is not**,
which is the one place this ball edited an existing collapse. `hello.rs` folded
four ways of stating no version into one sentence, rightly: none of them can be
served. A **fifth** case was folded in with them — a peer this end could not
read at all — and that one is not about a version. The other four are a peer
*speaking*, so REMOTE §3's *"a request of a version this build does not speak
is never adjudicated"* applies and nothing crossed; an unreadable preface is a
broken connection with this end's request already on it, because the request
goes out in the same breath as the preface. One sentence per outcome still
holds; there are now two outcomes.

**Which gesture is an act is recorded at the control** (`Posted`), and that was
checked rather than assumed. The tempting derivation is the poster's own
branch — a gesture naming no workspace is fanned, and §4.21's three
window-level reads are exactly the nameless ones this window composes. It is
false of the vocabulary: the vendored corpus carries request shapes with no
workspace slot that plainly change the world — `create`, `close`, `complete`,
`deliver`, `update`, `retire` among them — so the rule holds by coincidence and
the next control breaks it in silence. `posted::tests` asserts that against the
corpus rather than stating a count here. The other derivation — a table of
which ops are acts — is the second implementation `src/envelope.rs` exists to
refuse. The composing control already knows, so it says so, exactly as it
already says which op it fires for §4.16's parity tag.

**An act's failure is an EXCHANGE, so it goes to the bar.** That is bl-e620's
own division applied rather than bent: *a refusal is an exchange; an
unreachable channel is a relationship*. A gesture an operator made is an
exchange whatever went wrong with it, and two of the section's own reasons
inverted say so. The section is the slot a `workspaces` answer overwrites —
`Model::seat` sets `Held::Heard` on every one, and the asker answers
`workspaces` on every beat — so an act's sentence written there was **erased
within one beat by a read that succeeded**, which is the wrong outcome for the
one fact on this window that nothing will ever say again. And the bar's dismiss
is live here rather than inert: an act is an event, not a state, so it does not
re-post on a beat, and *I have looked* is a real thing to say about one. A
READ's failure is unchanged and still lands on its channel's section.

**Two sentences, because the remedies are opposite.** `Notice::Unsent` says the
act never left this seat, so nothing happened and doing it again is safe.
`Notice::InDoubt` says it reached the server and may have run, so this seat
never resends one — the fifth notice arm and **the first whose remedy is to do
nothing**. Both name the op, because one bar serves the whole window and a
sentence about an unnamed act is one nobody can act on.

**The read the in-doubt paint leads to is already running, which is why there
is no control.** The contract's recovery is a read of the world, and this
window's reads never stopped: the standing set is re-derived from the focus on
every settle, so the conversation, the roster and whichever pane is open are
being asked again while the sentence stands. A button on the banner would be a
second spelling of the beat, and there is no act-to-read mapping to build for
the same reason. **Nothing here is tagged for parity** — §4.16 judges controls
that cross the boundary, and a banner is state.

**Argv has the same contract and one place it was breaking it.**
`seat/start.rs` told an operator to type the start again when the fire failed,
which is right for a fire that never left (the stage's steps are convergent)
and exactly wrong for one that crossed: a second `lernie start` is a second
conversation. It now says which happened, and the in-doubt sentence names
`lernie conversations` rather than a retype. `seat/enroll.rs` gained the same
split for the sharper case — its product is the one reply this seat never
keeps, so a lost answer leaves a registration whose material exists nowhere,
and `enroll again` would mint a second over a first nobody can see.

**The test seam is the finding upstream recorded not having.** yog bl-d1f1:
*"The wire path's window is not drivable without a way to drop a connection
mid-answer, which is itself a finding — the path with the worst recovery story
is the one with no test seam for it."* The stand-in server now takes
`Answer::Hangup`: it completes the handshake, reads the request, and closes
without a frame or a terminator, which is the in-doubt window exactly. A plain
`Vec<Value>` still converts into the ordinary arm, so no existing script
changed.

**One residual, stated rather than closed.** An answer whose frames landed and
whose terminator did not is painted in doubt though it is not: `Channel::ask`
collects into a `Vec` and drops it on a later read error, so a receipt that
arrived is discarded. Closing it means `ask` handing back a partial stream
beside the failure, which is a second shape for an answer — and the paint it
would buy is a *narrower* doubt, never a wrong one. The honest over-caution
costs an operator one read they were being told to make anyway.

### 4.23 The row's own menu: one gesture, two platforms, and nothing on it destroys (bl-dbc9)

`src/ui/convs/menu.rs`, `src/ui/model/fill.rs`. The operator's requirement was
conversation management reachable from the **conversation itself** — a
right-click on the desktop, a long-press on the android client. The unifying
fact is the toolkit's: **egui synthesizes a secondary click from a touch
long-press**, so the two clients implement one design — `Response::context_menu`
on the conversation row — and the platform-native trigger falls out with nothing
written for it.

**It is the composer's second row, on the row, and that seam already existed.**
`src/ui/composer/acts.rs` is *the acts that spend no words*, and those are
exactly the acts a list row can offer. `send` and `interrupt` stay off the menu
**together**: both advance the turn, and both spend the composer's ONE draft
box, so an item pointing at a box shared by two verbs would name neither. The
menu is a second gesture path to controls that already exist — it duplicates no
plumbing, calls the same `crate::verbs` doors, and the parity walk unions the
tags (§4.16), so two controls per op is lawful and is what this is.

Six items, and the separator is where the one rule is:

| Item | What it does |
|---|---|
| `stop` | fires `stop` on the row |
| `retarget` | fires `retarget` on the row |
| `seen` | fires `seen` on the row — **offered only where the row is asking** |
| `records…` | selects the row and opens §4.18's pane on it |
| `flag…` | selects the row and puts the cursor in the composer's reason box |
| `delete…` | selects the row and puts the cursor in its arming box |

**The rule: an item whose act takes only the workspace and the conversation FIRES;
an item whose act takes a third parameter goes to the box that fills it.** That
is what dissolves the question the ball asked — *how does an arming work inside
a menu* — rather than answering it. A menu cannot hold either box: the composer
is a bottom panel that stands down off the conversation's own column in the
narrow shape, so a box in a list row's menu would be a parameter reachable at
one width and not the other. And a control that fired without one would compose
a gesture from a parameter nobody was asked for.

**So nothing on this menu can destroy anything, which is §4.20 read on a
control instead of on a pane.** That section's warning is exact about this
surface — *"a routine pane is a surface an operator moves through quickly; a
mis-aimed click there must not be able to land on this"* — and a list row's menu
is the sharpest case of it: the next row is one pixel away and the menu opens
under the pointer. `delete…` therefore **opens the arming** and does not fire.
It also does not invent a second arming, a second confirmation shape or a second
place: §4.20's own ruling is that the seat reads which of the two a `typed` is
off the wire's grammar and invents no policy, and `delete-agent`'s `typed` is a
**parameter** whose box is on the composer. This item goes there.

Four decisions ride with it, and each is an existing ruling read one surface
over rather than a new one.

- **The subject is the ROW, and only the items that lead somewhere select it.**
  This is §4.19's division exactly: a queue row's `seen` answers that row
  without selecting it, and only *go to it* moves the focus. A fired act here
  carries `(aim.address, row.root_id)` outright, so it needs no selection and
  takes none — a secondary click that kills a driver must not throw away the
  transcript the operator was reading. The three that lead somewhere DO select,
  because each opens a surface whose subject is the *selected* conversation.
- **The two navigations carry no `act:` token** (§4.16), because an aim, a
  selection and a focus cross no wire and views are out of the parity contract
  — the same division *go to it* keeps, and the same one §4.20's opening
  control keeps. The acts themselves stay tagged where they fire.
- **`seen` is the one convenience the menu adds over the composer's second
  row**, and it rides only on a row carrying attention: the op answers *what
  this conversation is currently asking about*, so on a row with nothing waiting
  it is an act with no subject. It is offered where the row's own headline says
  it applies.
- **The request for a cursor is a field, taken once** (`Model::fill`), which is
  `Model::reveal`'s shape one noun over. In the broad shape the composer paints
  later in the same frame as the list, so the cursor lands on the click; in the
  narrow shape the list is the central panel and the composer is already behind
  it, so it lands on the next frame. That is why it is a field rather than a
  call.

**The keyboard story is whole with no binding added, and that is the honest
answer rather than a deferral** (yog's QUALITY F1, §4.11's standing rule). Every
item on this menu names a control that is already Tab-reachable: `stop`,
`retarget`, `records…`, `flag` and `delete` are the composer's second row, `seen`
is the decision queue's row control, and the two navigations do what an arrow
walk and the column bar already do. **A binding that opened the menu would be a
second surface for controls that already exist** — §4.11's *"a binding names a
control that already exists"* read in the direction it forbids — and egui
offers no keyboard affordance for a context menu to take cheaply. What the menu
adds is reach for a pointer, not an op; the keyboard route is the composer and
the panes, and it is unchanged. Escape closes the menu, which is egui's own and
means what `Model::escape` means: put the thing down.

**One thing the menu DID cost the keyboard, and it is closed here.** The arrow
gate named one box — the composer's draft — because until now Tab was the only
way into the other two. This menu *lands* the cursor in the reason box and in
the arming box, and an arrow taken from inside either would have walked the
conversation list under a half-typed reason and then flagged, or armed a
deletion on, the row it landed on. `crate::ui::keys::BOXES` is now the whole
list of boxes that take text and the gate compares against all of them. It is
still a comparison rather than egui's `wants_keyboard_input`, which answers *is
anything focused* with every button included; a fourth box joins the list in the
commit that paints it.

**There is no menu-open world in the snapshot matrix, and the reason is the
instrument** (§4.11's `src/snapshot/blank.rs`). That detector's subject is *a
rectangle the layout claimed, read off the glass*, and a floating overlay makes
every rectangle beneath it flat **by construction** — so a menu-open shot would
redden the one detector that caught a slab painted over the window, and
exempting it would be blinding that detector to buy a photograph. The menu's
evidence is the two instruments that can carry it instead: the paint probe
drives a real secondary click and reads back what the menu put on the glass, and
the parity walk's own AccessKit tree is driven the same way and counts the
`act:` tokens the items gained — `seen` from nothing to one, `stop` and `steps`
from the composer's one to two. Counted rather than present, because a set
cannot tell one control from two.

### 4.24 The login pane: the sign-in is an act on the boundary (REMOTE §8.3, bl-e3c5)

`src/ui/login.rs`, `src/ui/login/run.rs`, `src/ui/model/login.rs`,
`src/reply/providers.rs`, `src/reply/login.rs`, `src/verbs/login.rs`,
`src/offframe/signin.rs`. **The eighth covering pane, and the second whose
subject is the aimed workspace.** §4.17's tuning pane is what a workspace's roles are set
to; this is what that workspace can sign in to, and the act that signs it in.

**The act is the SERVER'S and this crate never spawns one.** `bz --login` runs
on the server, inside the named workspace's directory, so the credential lands where
the agents that read it run and nothing credential-shaped crosses this wire
(REMOTE §8.3). That is the whole reason the surface is portable at all: the
seat asks, posts and paints, and the run holder, its one-run-per-pair
replacement and its hour sweep are all upstream's. **Closing the pane
terminates nothing** — a run with no lane still settles and still writes its
`ops.jsonl` row — which is a property this side gets for free and must not
re-implement.

**All three cadences meet on one pane, and each is the rule it belongs to.**
This is the only surface in the seat where they do, which is why they are
stated together:

- **`providers` STANDS while the pane is open** (§4.17's rule, keyed on the
  pane). Here the standing buys something the tuning pane's does not: a
  credential lands on the SERVER while the operator is looking at the table, so
  a row that said *no credential* says otherwise on the next beat with nothing
  asked again.
- **`models` is POSTED** (§4.21's rule). What a row offers is fixed for the
  life of the server's own answer, so a standing read would spend a round trip
  a beat forever on something that cannot change under the operator. It is a
  read and not an act: asking twice is asking once.
- **`login-tail` is HELD, on a fourth thread** (`src/offframe/signin.rs`). It
  is answered at the provider's pace — minutes of a human's attention — so it
  is the read in this seat whose N takes longest to finish, and putting it in
  the serial pass would stall the roster and the transcript behind a browser
  nobody has got to yet. It is the follow lane's shape one noun over, stamp for
  stamp: every frame says which provider row it is about, and the guard is a
  pure comparison at the settle where what the pane is following is known for
  certain.

**One control fires two ops.** `sign in to this row` carries
`act:login act:login-tail`, because starting the run is what stands the lane up
— §4.16's *"one control may fire more than one op"*, and the start control's own
shape (`prepare` then `prompt`). A seat that tagged only the act would be
claiming `login-tail` is reachable nowhere.

**Which sphere the sign-in is for, said in the surface, and never as an
address.** The workspace's name and the channel it came down, and — for a workspace held
elsewhere — that it is held elsewhere, read off the roster's own client-side
channel stamp (`ui::Channel::named_there`, §8.2). Two entries naming one
listener are still two trust relationships, and an address in this sentence
would be this end telling the operator where a credential is about to land.

**The loopback remedy is STATED and never built** (REMOTE §8.3). A row whose
flow wants a browser at the server's own loopback completes only from a browser
on that box, or through a port-forward — *an operator act on boxes the operator
administers*, which upstream rules is a remedy and not a channel feature.
**It is said for every row on a workspace held elsewhere rather than for the
browser-only ones**, and that is a limit rather than a choice: the `device`
column is on yog's provider ROW and not on the view that crosses the wire
(bl-7c9f), so this seat cannot tell the two apart. A stated remedy that is
sometimes unnecessary beats a silence that is sometimes wrong. If the column
ever reaches the view, the sentence narrows to the rows that need it and
nothing else here moves.

**The run-by-hand fallback is the server's sentence with one of ours beside
it.** `fallback` is composed by the end that knows the workspace and arrives only on
a non-zero exit. The original surface (yog bl-1ddb) spelled a local workspace's
fallback as an `exec` on this box and an entry-held workspace's as something else;
there is no `exec` in this crate and both are the same case now — **an act the
operator runs on the box that HOLDS the workspace** — so what the pane adds is one
sentence saying so, naming the channel and no address.

**The pane holds two questions and no answer.** `Login { following, asking }`
is a struct of two options and not §4.17's enum, because neither excludes the
other: a sign-in being followed and a row that was asked what it offers are two
independent facts about two possibly different rows, and every combination is a
window an operator really reaches. The state that would mean nothing — a
subject with no pane — is unrepresentable because both live *inside* the pane's
own option. The three answers are `Model`'s, filed whether or not the pane is
open, for §4.17's reason verbatim. `asking` exists because the `models` reply
carries no provider name: the question is the only thing that can say which row
a listing answers.

**And the pane goes with the workspace.** `Model::aim_at` retires it and all three
answers, which is §4.17's rule in its sharper form: the act signs a credential
into ONE workspace's store, so a pane left standing over a new aim would offer to
sign the operator in somewhere they are no longer looking.

**The standing set stopped being flags** (`state::Open`). Four covering panes
now stand reads up, and four bools would make *three of them open at once* a
representable state that only the derivation order resolves — `ui::Lookup`'s
reframe one layer down, and the one clippy's `struct_excessive_bools` asks for
by name. `Standing::open` is one field, the followed provider row rides inside
the login arm because there is no sign-in to follow while the pane is down, and
`Open::of`'s ladder is a total function over a state space `Model::covered`
already promises the window cannot reach.

**What this does not close, and it is upstream's.** A browser-only row signed
in from a seat with no shell — a phone — has no paved path: the port-forward
remedy assumes a terminal somewhere. REMOTE §8.3 states it and refuses a
paste-back arm, so the pane says the remedy rather than offering a verb that
cannot finish. **It is not in `parity.toml`**, because the ops are all
surfaced and that file records absences.

### 4.25 The pin: an assertion about the world, not an arrangement of one screen (bl-7782)

`src/verbs/workspace.rs`, `src/ui/roster/workspace.rs`, `src/ui/model/acts.rs`. The
strip has ordered pinned workspaces first since the roster could read a rank
(`roster::ordered`), and until this ball nothing on the glass could set one.

**The pin crosses the boundary, and that is the point.** A seat-local sort would
have been cheaper and would have been a different thing: yog's own help row says
*"a pin is an assertion about the world, not an arrangement of one screen, so it
is the same list on every seat and it survives a restart."* So it is the server's
list, this seat asserts into it, and the rank comes back on the roster answer
every beat already carries.

**Two ops, and the control is not a toggle.** Upstream spells the pair as
assertions and states why: *"says what it means rather than flipping whatever it
found: unpinning one that is not pinned leaves the list alone, which is what lets
two seats send it at once and agree."* A seat that composed one op from a flag it
read a beat ago would be re-introducing exactly the race that wording removes. So
**the control's word and its `act:` token both follow the row's own rank** —
`pin to the front` on an unpinned row, `unpin` on a pinned one — which is a
reading of what is on the glass rather than a state this end holds. It is the
fifth per-workspace control, hanging off the aimed row and off no other, standing down
under a covering pane with the other four.

**Both answer the workspace listing**, so nothing new is decoded and the next
answer is what says the act landed — §4.17's *a write lands and the next answer
says so*, reached here for free because the roster read is already standing.

**It costs the parity instrument a world rather than a pane** (§4.16, §4.11's
matrix). Each row carries exactly the one act that is not already true of it, so
the window at work can only ever put `pin` on the glass. `pinned` is the
`seated` world with its aimed workspace ranked, and it is the only screen `unpin`
exists on — *unproven is red*, and a control that lives on a screen the walk
never visits fails honestly.

### 4.26 The start's refusal: the server's rung, and the seat's refund (bl-b180)

`src/ui/model/start.rs`, `src/ui/model/absorb.rs`, `src/state/traffic.rs`,
`src/offframe/poster.rs`. The operator ruling this closes is yog's bl-1fd0: *on
a workspace holding no usable provider credential, typing a goal and hitting Enter
will work zero percent of the time — the conversation is born, immediately dies
on no-models, and the operator learns it from a dead row (or from nothing at
all).* Both first goals typed that evening were lost. The ball arrived with
three open questions, and each is answered by a house principle rather than a
preference.

**1. The server decides, and the seat re-derives nothing.** The predicate is
brazen's `credential` column plus one clause about the config's own role
assignments, and it was written and tested on the server once (yog `StartGate`)
before being deleted for having no caller — the doc that survived it said *if
a seat ever needs yog to state the gate, it files against yog and the carrier
is designed then*. This seat filed (yog bl-2291). A seat re-judging
`Query::Providers` rows would be a second implementation of a settled decision
(VISION §8), and with the phone beside this window it would be a third. So the
rung lives at the server's `Prompt` door, in the order blank goal → **sign in**
→ confinement → ceiling, and **the carrier is the refusal every act already
has**: `{"ok": false, "error": "sign in first: … — /login <provider> (rows: …)"}`.
No new verb, no new shape, `PROTOCOL` unmoved, corpus unchanged (yog REMOTE
§9.19). This side paints the server's sentence and never learns the predicate.

**2. A one-line refusal at the moment a goal would be spent, not a standing
band.** The face is four panes wide and the composer is one control with two
subjects; a band that stood for as long as the workspace was unsigned would be paid
for by the pane beside it, on every workspace, whether or not the operator meant to
begin anything. The refusal is instead the **start's own receipt**, painted
where the start's sentence already stands — `not started in <workspace>: sign in
first …` over the box — with the goal back in the box under it, because a
refused fire spends nothing and what was typed is the operator's again. The
standing fact about the workspace belongs to the Login pane (bl-e3c5), whose rows
say *not signed in* for as long as it is true; this surface says it once, when
it matters, and the sentence stands until the next start replaces it.

**3. The waiting question dissolves.** The ball asked what happens to a start
typed before a readiness answer is back. There is no readiness answer to wait
on: the goal is posted as always and the server's reply is the name or the
refusal. A special case was a missing reframe.

**What the seat had to learn to do that — and it is not about the workspace.** A
refusal wears no `kind`, so `Model::absorb` could not tell *whose* refusal it
was filing, and the one gesture this window holds across two acts stayed
outstanding under any refusal at all: a blank goal, the spend ceiling, a
confinement refusal — each already a server-side refusal of `prompt` — left
the composer painting *starting…* with no box, forever. That is the seat-side
half of the ruling, and it is a plain defect this ball found on its path:

- **The poster stamps a routed reply with its op** (`Said::Receipt { op, frame
  }`), because it is the one place that still knows, for the same reason it
  already stamps an unanswered act (`Said::Acted`). Every routed gesture is
  stamped rather than only an act, since the stamp is consulted for exactly
  two ops and a second arm for reads nobody routes would be a branch with no
  beat; the fanned path and the standing reads stay bare, because a question
  answered in place has nothing held against it.
- **`Model::receipt` is `absorb` with that one fact.** An answer files as any
  frame. A refusal of `prepare` or `prompt` while a start is outstanding retires
  it into `Phase::Refused` and refunds the goal; any other op's refusal is the
  bar's sentence and leaves the start alone, and so does a refusal arriving
  after the start landed. A frame this seat cannot read, and no reply at all
  (`Model::acted`), take the start back the same way — the bar keeps its own
  sentence about *that*, and the goal is the operator's — because a start
  nothing will ever answer is a composer with no box.
- **The refund never overwrites a draft typed elsewhere.** The composer is one
  box with two subjects, so a deposit drafted on another workspace while the start
  was out is newer text and is kept; the refused start's own goal is then held
  only in the retired `Start`, which is the one narrow residual and is recorded
  rather than papered over.

**A DEPOSIT that never left this box is refunded on the same rule, and only
that one** (`Model::acted`, `crate::envelope::said`; bl-e85f). The notice for
an unsent act reads, in this seat's own words, *nothing happened — it is safe
to do it again*, and the seat then threw away the one thing *again* needs: the
composer showed its placeholder and the typed text was gone, so *do it again*
meant retyping a paragraph-long goal. It is the one case where the composed
text is provably still needed, because the notice's own claim is that the act
did not happen.

Three things make it a rule rather than a patch:

- **The words ride WITH the act**, on `Said::Acted`, exactly as its `op` does
  and for that field's own reason (`crate::state::traffic`). A seat that kept
  them in a field would be holding a second copy of something already in
  flight, and would have nothing to say about which act's copy had come back.
  `crate::envelope::said` reads them off the envelope by the one key the
  vocabulary has — `message` and `interrupt` are the two gestures a composer
  fires and both spell their body `content`; every other envelope this window
  composes is built from what is on the glass and can be composed again by
  looking at it, so there is nothing to give back.
- **Only where nothing crossed**, which is the whole of the asymmetry with the
  start above. IN DOUBT means the server had the gesture and may have run it,
  and its sentence says so — words back under a live `send` would invite
  exactly the resend REMOTE §3 forbids.
- **One refund, shared.** `Model::refund` is the start's, and the guard that
  makes it safe for either caller is the same: words go back only into an empty
  box, so a draft typed since is never clobbered.

**Not built, with the reason.** No `Query::Providers` read from the start box
(the seat would hold a prediction the door re-derives anyway, and the read
races the typing); no `sign_in_required` flag on the `prepared` body (the same
prediction one frame earlier); no second painted seat for the sentence (the bar
already has a dismiss, the composer's slot already has the receipt, and one
sentence in two places is two sentences).

### 4.27 The trail: the server's own record, and the classification that crosses (bl-4c48)

`src/reply/ops.rs`, `src/verbs/trail.rs`, `src/ui/trail.rs`,
`src/ui/model/trail.rs`. **The ninth covering pane, and the fourth whose
subject is every channel rather than a focus.** It answers a question this seat
carried the wire for and painted nowhere: *what has this box been doing*. The
server keeps one row per child process its boundary spawned — the argv, where
it ran, how it ended and where its alarm stands — and until this ball the only
place that record could be read was yog's own line surface.

**It is §4.19's shape, one noun over, and every joint is the queue's.** `ops`
names no workspace, so its subject is every channel this box holds
(`crate::verbs::Verb::addresses_a_workspace`, §4.21's mechanical definition);
the read fans and the pane is the union; its control hangs off the roster
rather than off a row, because the roster is the pane that is already the union
across channels; and no aim or selection can invalidate it, so it survives both.

**But the read STANDS, where §4.21's other two are posted once.** That is the
one place it parts from the window's other channel-wide panes and it is a fact
about the subject rather than a preference: a verb table is fixed for the life
of a server build and a search answers a needle somebody typed, so re-asking
either on every beat would spend a round trip per channel forever on an answer
that cannot have changed. A trail is neither. Every act this seat spends
appends a row to it, and an alarm goes up and comes down under an operator who
is looking at it. So it is keyed on the pane exactly as the queue's is
(`crate::state::Open::Trail`) and asked only while somebody is looking.

**The seat reads the server's words and classifies nothing** (REMOTE §9.17).
The reply carries `failed`, `exit_label` and `standing` precisely so that a
client need not re-implement the sentinel table, the `128 + n` signal reading,
the retirement key, the ack scan and the origin grouping — *"five derivations
this document names one home for apiece, and whose failure mode is a seat
quietly disagreeing rather than failing to build"*. So the pane paints the
words and never looks at the exit integer, which rides in the detail because it
is the next thing an operator asks for. `standing` is a `String` and not an
enum, on §4.9's rung 3: this seat knows exactly one of the five words
(`clean`), and knows it only in order to say **nothing** — a badge on every
ordinary run would bury the rows the pane exists for — so a vocabulary that
grew upstream costs a badge and not a decode.

**And the row says WHO, on both faces** (REMOTE §9.20, PROTOCOL 16; bl-183b).
`client` is the identity that made the act — a connection's certificate common
name, or the reserved `local` for the window, the gestures inbox and yog's own
loops — and it is read strictly, exactly as `standing` is. It is the one fact
on a row that cannot be recovered later: presence is a point-in-time
observation by design, so two seats depositing into one conversation within the
same second left two rows spelling `argv`, `cwd`, `origin` and `exit` and
nothing that told them apart. The pane carries it in the provenance line beside
the origin and the CLI carries it beside the stamp, because on both surfaces it
is what makes two identical-looking rows two facts.

**The depth is the seat's own answer to a required field.** `ops` carries
`max`, and the wire refuses an envelope without it; the *"defaults to the last
50"* an operator reads belongs to yog's line grammar, which is a different
surface with a different reader. So `crate::verbs::trail::DEPTH` is stated
once, and it is deliberately deeper than a screen: an operator reading a trail
is looking for the row before the one that broke, and a depth that stopped at
the visible rows would make *scroll down* answer nothing. `max` being a NUMBER
is also why this is a typed door rather than a row in the verb table, which is
rows of named strings and refuses to grow an arm for anything else (§4.10).

**What this ball did NOT land, and where each half went.** bl-4c48 named ten
ops and this is the read half of one of its two. The trail's two acts — `ack`,
which is the operator's watermark and the second of the two ways an alarm comes
down, and `clear-trail`, which truncates — were bl-b8f7: both name no
workspace, so both take the poster's fanned path, and *one control truncating
every server's trail* was a question to settle before writing it rather than
after. Both have since landed, and both answers are §4.35's.
The fleet family — `fleet`, `disband`, `arm`, `disarm`, `scan`, `science`,
`work-diff` — was bl-a43a, and it addresses a workspace, so it is the tuning
pane's shape rather than this one's; it has since landed as §4.33. **The fleet has no read of its own**: its
loop state (cap, count, tick, lease, ceiling) rides on the `board` reply, in an
array that is *absent rather than empty* when nothing is armed — so the
readable half of the fleet belongs to the ball pane, which is what would decode
`board` at all. That pane has since landed and it does (§4.28, bl-d2af); what
bl-a43a was left holding is the fleet's controls and its own two reads, and
that is §4.33.

### 4.28 The clients pane: which machines may execute for this workspace (REMOTE §5, §5.1; bl-e53c)

`src/ui/clients.rs`, `src/ui/model/clients.rs`, `src/reply/clients.rs`,
`src/verbs/clients.rs`. **The ninth covering pane, and the third whose subject
is the aimed workspace** — §4.17's is what its roles are set to, §4.24's is what it
can sign in to, and this is which machines may run a tool for it.

**Two lifetimes on one row, and the pane says both.** `present` is an
observation — true at the instant the server answered and recorded nowhere on
either end — and the advertised set is a statement the machine last made, which
stands whether or not it is connected. A row therefore reads *not connected*
beside a full set as the ordinary thing: a tool host holds its connection only
while it is waiting for work (REMOTE §5), so a busy machine and an absent one
are indistinguishable from here, and the sentence says so rather than leaving an
operator to read *absent* as *broken*.

**And a third fact, whose ABSENCE is the reading** (REMOTE §5, PROTOCOL 14;
bl-183b). `last_seen` is the unix second a client last spoke, durable where
presence is not, and this seat paints the half that decides something: a row
with no stamp at all has **never dialled** — an enrolment minted and abandoned,
which is the row an operator can safely remove. The other half, *how long ago*
a machine that has dialled last spoke, is an age, and an age needs a `now`.
This crate holds no clock and every other age on this surface is one the server
computed and sent (`age_secs`), so painting one here would be this seat
asserting a time nobody handed it. **The command line is where this matters
most**: `present` reads false for every row there, because every verb opens and
closes its own connection, so the stamp's absence is the only thing on a
terminal roster that chooses.

**The consent is stated on every tool, present or absent.** `subject_cwd` is
what yog's worktree lane routes on (REMOTE §5.1) — the advertising box
consenting to run that tool at a directory the invocation names — and it is the
fact the ball that landed this pane was filed about: *"an operator with no way
to see which entries consent cannot tell a foot that will take a subject from
one that will refuse it."* A line that appeared only when true would make its
absence ambiguous, so both answers are painted.

**The read STANDS while the pane is open**, on §4.17's rule and for the sharper
form of §4.24's reason: presence is true only at the instant it was answered, so
a row saying a foot is connected is worth nothing unless it is asked again.

**A tool's `input_schema` is not decoded.** It is the host's statement to a
model — yog carries it verbatim and never validates it — and an operator reading
a roster of machines is asking what a box can do, not what shape its arguments
take. §4.9's rule, applied: the commit that paints a schema is the commit that
decodes one.

**The pane carries no control, and that is the surface being honest.** Every
other op in REMOTE §5 is a MACHINE's (§4.9's third reading of the ledger), so
there is nothing here for a seat to fire. What an operator *does* about a tool
call happens on the conversation that is making it, not on the machine that
would run it — the capability boundary's three ops (`answer`, `revoke`,
`restore`) are conversation-addressed, they are cited in `parity.toml` to
bl-bce2, and their frames are in that ball.

**The three listing panes are one field** (`src/ui/model/listing.rs`). This pane
was the fourth `bool` on `Model` and the count is what named the reframe:
`Lookup` had already drawn it for the window's own two — *one field rather than
two flags, because no two panes ever stand together and a pair of bools would
make **both** representable* — and the records pane, the decision queue and this
one hold nothing of their own, so *which one is standing* is their entire state.
Clippy's `struct_excessive_bools` asks for exactly that refactor by name; a
manifest allow would have kept three representations of one fact to avoid
writing one word.

### 4.29 The spine: what a conversation's history is anchored to, and the one act composed off it (bl-b52c)

`src/ui/records/spine.rs`, `src/ui/model/spine.rs`, `src/verbs/spine.rs`,
`src/reply/rail.rs`, `src/reply/governing.rs`. **The records pane's other
half**: §4.18 landed what the loop DID and what it TOUCHED and left what it is
anchored TO — the operable commits, the config commit governing them, and the
fork whose `from` is one of the first.

**It is in that pane and not a pane of its own**, because the subject is the
same noun. A second covering pane over the selected conversation would be a
second place to look for one thing, and the two reads stand on the same
`Standing::records` the other two do — so opening the pane is a fixed set of
questions per beat and closing it is none.

**The act is here because its argument is only discoverable here.** `fork`
takes a `from`, which is a **ref**, and upstream refuses an empty one — *"a
fork with no ref is a different gesture"*. The refs a conversation offers are
exactly this half's notches. That is what `parity.toml`'s line for `fork` said
in those words until this ball deleted it: *a control demanding two strings
nothing here can discover only looks actionable*. So there is one control per
**operable** notch — a notch with a commit, which is upstream's own test for a
notch a gesture can reach — it carries that notch's own commit, and a notch
that recorded none carries no control at all.

**The other string is still not discoverable, and the control says so rather
than pretending.** A role names an entry litany resolves against the fork
point's own governing config, so the honest control would offer the names that
config declares — and the read that lists them is the config-file pane's,
which this seat does not have (bl-5c53). A picker over a list this seat cannot
obtain would be capability theatre; a bare box with no sentence would read as
a free string. So it is a box with a sentence under it, and no line in the
ledger: the op is surfaced, and a stated limit on a surfaced op is not an
absence.

**`fork` is a door and can never be a row.** `src/verbs`' table is rows of
named strings and `fork` carries `skills`, an array — the case §4.10's own
module doc refuses to special-case. So it joins `effort`, `priority`, `ops`
and the start family's two as a typed door with no row, and the seat composes
the empty list: a skill set is a choice off the same config the role is, and
the pane that would offer one does not exist.

**The goal is spent on firing and the role is kept**, which is the composer's
own split read one surface over (§4.11): a flag's reason is taken because what
a flag says is said, and an unmaking's arming is cloned because clearing it
would charge a retype for the server's *no*. A fork's goal is the first — it
was said, to a child that now exists — and its role is the second, being the
same name the next attempt off the same spine wants.

**`reply/governing` is the first shape this seat reads whose MEANING moved
under an unchanged spelling** (§4.9, REMOTE §9.12). `oid` named the fork
commit at PROTOCOL 4 and names the followed lineage's head at 5, and no
mechanical check this repository owns could have caught the difference — so
the trap is restated at the decoder, where whoever paints the number is
reading, and the sentence painted beside it is upstream's own
`GoverningConfig::label()` rather than a third wording composed here (REMOTE
§9.7 rules that directly).

**The workflow mark rides the governing read, and its two acts hang beside it**
(REMOTE §9.24, edition 19; bl-ed20, `src/ui/records/workflow.rs`).
`reply/governing` grew `workflow_mark`: `null` where the followed tip's
`workflow.yaml` governs, else `{holder, oid, short_oid, lineage}`. It sits
**beside** the commit, never instead of it — a mark moves `workflow.yaml` alone,
so `oid` still names the tip — and the seat decodes it into
`Governing::workflow_mark` with absent read as `null`, *"a reader that never
heard of the key reads the answer it always did"*. Three renderings, each a
pure function of the mark: `null` paints nothing, which is the absence it is; a
named lineage paints *workflow.yaml marked to config/<lineage> at <short>, on
<holder>*; and a `lineage: null` beside an oid paints the pinned older commit
its lineage has moved past — a reading, never a refusal.

`workflow <lineage>` and `clear-workflow` are rows of `src/verbs/conversation.rs`
beside `retarget`, on the pin pair's precedent: the op token is the direction,
so a clear cannot read as a set that lost its field. **Their controls are here,
not on the composer's strip beside `retarget`, for the fork's reason above**:
`workflow`'s `config` is a lineage name, §9.24 rules that what a conversation
can be marked to *is* `request/lineages`, and that listing is a read — so the
control is one button per lineage the workspace holds, each carrying its own name,
on the half that already paints which mark answers. The listing therefore
stands on `Standing::records` as a sixth read, landing on the one
`Model::lineages` the config pane fills (§4.30); with none answered nothing is
offered. `clear-workflow` is offered only where a mark is read, the pin pair's
rule (§4.25) — a mark inherited from an ancestor still gets it, because the
holder is a descent id this seat does not map to an address, and the server's
own outcome says whose mark it was.

**Neither is §4.20's unmaking.** A mark is a ref litany writes and deletes; the
conversation, its history and the marked commit all stand either way, and
marking again is the other thing. The one thing a clear leaves unrecoverable
from this seat is a mark pinned to a commit its lineage has moved past — a mark
can only be set to a lineage's head — and that is the same loss `retarget`
already accepts without an arming. So both are plain buttons, as `retarget` is.

**A notch says its money** (yog REMOTE §9.23; bl-9111). The notch's figure is a
rollup as of that step, and the server prices the same rollup and sends it
beside it — so the notch's headline says `N tokens — $x.yz` on §4.18's rule,
the count alone where unpriced and `at least` where the rollup holds tokens no
rate covered. A child's card carries no cost on this wire and so paints its
count alone; a seat that summed one would be a second author of a price.

### 4.30 The config pane: the files a workspace's policy is written in (REMOTE §9, §9.18, §9.22; bl-5c53, bl-4855, bl-a1d6)

`src/ui/config.rs`, `src/ui/config/settings.rs`, `src/ui/config/edit.rs`,
`src/ui/config/proposals.rs`, `src/ui/model/config.rs`,
`src/ui/model/config/draft.rs`, `src/ui/model/config/proposal.rs`,
`src/reply/config.rs`, `src/reply/lineages.rs`, `src/verbs/config.rs`,
`src/ui/model/posted.rs`, `src/offframe/poster.rs`. **The eleventh covering pane,
and the fourth whose subject is the aimed workspace** — §4.17's is what its roles are
set to, §4.24's what it can sign in to, §4.28's which machines may execute for
it, and this is the files all of that is written in.

**Two views of one read, and never two reads.** `reply/config` carries the
destination's `text` and, beside it, that file's schema applied to *those very
bytes* (REMOTE §9.18). Upstream refused a `ConfigSchema` query for the reason
this pane inherits: a schema alone does not say what the values ARE, so a seat
would always join it against a byte read, and two reads of one file have
staleness between them. What is on the glass is one moment.

**The judgement is the server's.** A setting's `fault` is the same call yog's
own pick gate makes, so what this pane paints as wrong is what the far end says
is wrong — and its absence is *nothing is wrong with this value*, not *nobody
looked*. A seat that derived one would be a second authority across a boundary.
The bounds on a control are decoded and stated for the same reason and enforced
by nothing yet: this build paints values and does not edit them.

**A file with no schema answers an empty array, and that is a reading.** §9.5's
raw-text destinations are *"the general path with empty input rather than a
branch"*; the pane says so in its own sentence rather than showing an empty
list.

**Two destinations name a workspace and three name a SERVER, so the pane
addresses two different things.** `brazen` and a lineage's `branch` carry
`workspace` inside `target`, which `crate::envelope` already reads as the
gesture's address — §8.2's routing and its rename apply untouched (bl-4a36 put
that in place). litany's global models file, its workflows and yog's own
`cadence.yaml` belong to the server and name no workspace at all, so what they
address is a **channel**: the pane asks them down the channel the window is
aimed at, by name, exactly as a roster read is asked. Letting them fall through
`seat::route` would have asked this box's own server about a file on somebody
else's.

**It writes, and the fan was never a rule about workspace-less gestures**
(bl-4855). `config` is one op that is a read or a write depending on whether it
carries `text` at all (upstream's discriminator), so the editor is one field
away — and the field was never the problem. §4.21's poster fanned every gesture
naming no workspace, so an ACT composed for one server's `cadence.yaml` would
have been written to **every channel this box holds**, putting one operator's
policy onto every server they are a client of.

The fix is not an arm saying *do not fan this one*. §4.21's predicate is
`crate::verbs::Verb::addresses_a_workspace`, and what it actually establishes is
that such a gesture **cannot name a channel by its workspace** — from which the
poster inferred *therefore every channel is its subject*. That inference is
true of a window-level READ, whose subject genuinely is the box's whole set, and
false of a `config` write about one server's own file. So the address stops
being inferred: `crate::ui::Posted` carries the channel a gesture is addressed
to, `crate::offframe::poster::legs` sends it down that one, and **the fan is
what addressing it to no channel in particular means** — the general path with
empty input rather than a branch. It is recorded at the control for the reason
`Posted::act` already is (§4.22, *"the composing control already knows, for
free"*) and for the poster's own (bl-c70d): the aim is not read on the send pass
at all, because an operator may compose one gesture aimed at a workspace on one
channel and fire another at a row on a second before either leaves.

**So the write is addressed exactly as the read is**, and the branch is the same
one in both halves: a destination naming a workspace is routed by it — which is
what performs an entry's rename at the channel boundary, and what naming a
channel would bypass — and one naming the SERVER goes down the channel the
window is aimed at, by name. `crate::offframe::asker::workspace` for the read,
`Model::write_config` for the write, one predicate
(`Where::addresses_a_workspace`) read twice.

**It is an ordinary act, and the box is what makes it one.** §4.20's idiom — a
PLACE, a covering pane holding nothing else — is for an act whose subject
ceases to exist. This one's subject is a file that still exists afterwards, and
the control cannot be reached by a mis-aimed click because reaching it means
having authored the file's whole text. So it takes §4.20's other half, the
ENABLEMENT: `write` and `revert` are dark until the box differs from what the
server last answered, and the refusal is spelled beside them, *"because a
greyed control says a thing is not live and nothing about what would make it
live"*. `revert` is what makes §4.20's own test — *undone by doing the other
thing* — true rather than claimed: the other thing is on the glass beside it.

**What is NOT ordinary about it is the guard the wire dropped, and the seat
answers it with a reading rather than a refusal.** Upstream's own editors carry
a hash guard because *"a long-lived RAM draft"* can be written over a file that
moved under it; a `config` act *"carries no hash guard, and needs none"* because
a gesture states its whole text in one atomic instruction (REMOTE §9.18). That
is true of the gesture and false of the operator, who types for minutes while a
standing read replaces the answer underneath them. `Draft::seed` — the bytes the
box and the server last agreed on — is the guard restated as a sentence:
the file is neither what the box holds nor what the two last agreed on, which
leaves exactly one reading, another writer. Our own write in flight is excluded
because the answer is still the anchor, and our own write landed because the
answer IS the box. The server remains the only thing that judges.

**Nothing records that a write is in flight, and that is the enablement working
rather than a gap.** The control asks *is there anything to write*, which is a
fact about the world and stays correct while a write crosses. A flag saying
*asked* would be this seat holding a second opinion about a question the
standing read answers every beat, and the worst it prevents is writing the same
bytes twice, which is the same file. A successful write answers
`{"ok":true,"kind":"applied"}`, which this build decodes as no kind and paints
as §4.9's rung-3 notice — exactly what `delete-workspace` has done since §4.20,
and a fact about the reply vocabulary rather than about this pane.

**`litany-workflow` is a box, because the box is the listing.** It is addressed
by a name and no read this seat has says what workflow names exist — and none
can, because upstream mints the file on a name it has not seen. A box is
therefore the honest shape of *say which*, not a control asking the operator to
remember something a listing could have told them. Empty names no destination
and the control beside it is dark, which is the tuning pane's `set`: the
parameter is missing, not the subject. It brings `crate::ui::keys::BOXES` to
five.

**`parity.toml` still says nothing about any of this, and that is still worth
knowing.** The ledger is per-op and cannot express *half an op*; `config`
carried a control from the read half onward. This section is the record that
the other half now exists.

**The learning loop's candidates hang on this pane, not beside it** (bl-a1d6;
REMOTE §9.22). A reviewer stages what it learned as a real config commit on
`proposal/<reviewer-id>`, a branch no lineage points at until somebody says so
— the same subject this pane browses, one branch away. So it is one more
standing read on the pane's own terms rather than a pane of its own, and the
two verdicts hang on its row:

- **The listing stands while the pane does**, and a row is the id, the
  standing, the lineage it would move, the diffstat and the reviewer's subject.
  `fresh` and `stale` are the server's own reading and never inferred here off
  an empty `lineages` (§9.4): only the server read the two in one pass.
- **Naming a row deepens the one read; it does not post a second.**
  `proposals` is one op at two depths, so the standing read is asked with the
  named id and its answer carries the listing and that proposal whole. The
  depth is a derivation — the named id **only while the last listing still
  carries it** (`Model::proposing`) — because a settled proposal leaves the
  listing and asking for it whole after that would earn a refusal every beat.
  Naming another row drops the last one's whole: the answer carries no id, the
  rule `read_config` keeps one read over.
- **The whole is painted verbatim** — one monospace run, never wrapped, a long
  line scrolling sideways — because a diff a seat reflowed is a diff nobody can
  apply.
- **Both verdicts are §4.20 acts, armed by the proposal's own id.** The wire
  takes no `typed`, so by §4.20's amendment the arming is an enablement this
  seat holds and never sends (surrounding whitespace forgiven), and it is
  earned by scope: accept fast-forwards a lineage every conversation on it
  resolves at its next step, and reject throws a reviewer's work away. The
  opened row states its subject and what each verdict does before offering
  them, the way out (`leave it staged`) comes first, the arming is not spent
  on firing — a stale accept refuses in litany's own words — and firing is
  said (`asked — waiting for the server`). The subject is held for §4.20's
  reason at no cost: the id is the pane's, and the workspace is the aim's,
  which cannot move under this pane because the act that moves the aim
  retires it.
- **The answer to a verdict is `reply/outcome`**, filed like every §8.2
  receipt: a refusal becomes the notice, and a success shows on the next
  listing, where the row is gone.

The row control carries `act:proposals` and each verdict `act:proposal`, so
both lines left `parity.toml` in the commit that painted them.

### 4.31 The ball pane: the board, the bindings, and what a workspace holds (bl-d2af)

`src/reply/balls.rs`, `src/reply/board.rs`, `src/reply/spend.rs`,
`src/verbs/balls.rs`, `src/ui/board.rs`, `src/ui/board/rows.rs`,
`src/ui/board/workspace.rs`, `src/ui/model/board.rs`. **The twelfth covering
pane**,
and the one that answers what this window carried the wire for and painted
nowhere: the task store the conversations are working. Until it landed an
operator seated here could watch work happen and could not see the ball it was
happening on.

**It is the first pane whose subject is TWO WIDTHS, and the wire is what
decided that.** Four ops, and they split on the one predicate §4.21 already
defines mechanically (`crate::verbs::Verb::addresses_a_workspace`). `balls` —
the whole box's ball⇄workspace binding table — and `board` — that table folded
into columns — name no workspace, so they fan and their sections are the union
across channels, which is §4.19's shape exactly. `workspace-balls` and `marks`
name one, so they are §4.17's: asked of the aimed workspace, and retired when the
aim moves. Splitting them into two panes would put *the board* and *what this
workspace holds* on two screens, and they are one question asked at two widths; so
they are one pane with one section that says which width it is about
(`src/ui/board/workspace.rs`).

**The pane survives an aim and its workspace half does not.** That is not an
exception to §4.17's retirement rule, it is that rule applied to each half of
the subject: a pane about every channel's board does not stop being about them
because the operator aimed elsewhere, and painting the old workspace's balls under
the new workspace's name is exactly what the retirement prevents. So
`Model::aim_at` drops `holding` and `marks` and leaves `Lookup::Board`
standing.

**All four reads STAND, on the trail's terms** (§4.27). A board is what is
happening: a drone starts, a ball is claimed, a loop ticks, and every one of
those changes the pane under an operator who is looking at it. So the set is
keyed on the pane (`crate::state::Open::Board`) and asked only while somebody
is looking — never posted once, which is what the window's other channel-wide
reads get and what a verb table and a search deserve.

**The fleet's readable half is here because there is nowhere else.** There is
no `fleet` read on this wire and no reply kind for one; what an armed loop is
doing rides on the `board` answer, in an array that is **absent rather than
empty** when nothing is armed. That is the one case where a `Vec` and an
`Option<Vec>` are not two claims, so `Board::fleet` is a plain `Vec`. The
loop's rendered line rides too and this seat paints it rather than composing
one: `label` is upstream's own sentence about the cap, the count, the tick and
the lease, and a seat re-deriving it from the numbers beside it would be a
second opinion about a loop it does not run. The fleet's CONTROLS are §4.33's.

**The money is upstream's rendering and nothing here computes one**
(`src/reply/spend.rs`). `usd` was derived on the box that holds the price
table; a seat multiplying tokens by a rate of its own would disagree with it
quietly, which is §4.27's own failure mode one noun over. Its absence is a
fact and not a zero. The attribution rides as both the classification and the
clause, because a figure over one stamped conversation renders as no clause at
all and the clause alone cannot tell that from *workspace-wide*. The four
counters are `crate::reply::steps::Spend` read from where they already live,
never restated — one shape on this wire, one encoder upstream, one reading
here. **The money half is one reader too** (bl-9111): `Cost` — the server's
`usd` and how many tokens no rate priced — is read by `spend::money` wherever
it rides, inline on a figure or under its own key on the four shapes REMOTE
§9.23 gave one, and `unpriced_tokens` above zero makes it a floor the seat says
as `at least` (§4.41).

**One control carries four tokens**, which is PARITY §3's rule that the
ledger's unit is the op rather than the widget, spent on the widest control
this window has. Opening the pane stands all four reads up: two of every
channel this box holds, and — when the window is aimed — two of that workspace. The
two aimed ones are asked by that click and by no other, so that is where their
token belongs (`src/ui/roster/acts.rs`).

**What this ball did NOT land.** bl-d2af named nine ops and this is the read
half. The five acts — `create`, `close`, `update`, `assign`, `release` — were
held back as **bl-f7ae** because they asked two questions worth settling before
a control existed rather than after: not one of them names a workspace, so all
five would FAN, and each carries a `name` to act as that this seat held
nowhere. §4.35 is those two answers and the controls that follow from them —
the fan is refused, each act is addressed down the channel its row came down,
and the name turns out to be the workspace's own rather than an operator's.

### 4.32 The deeper records: the conversation's own row, one step's drill-in, and its undelivered mail (bl-3257)

`src/ui/records/header.rs`, `src/ui/records/drill.rs`, `src/ui/records/mail.rs`,
`src/ui/model/deep.rs`, `src/reply/agent.rs`, `src/reply/step.rs`,
`src/reply/inbox.rs`. **The last of what bl-2cf7 left**, and the ball that
takes the exemption ledger's conversation half to nothing.

**The seven answers are one value on the model** (`Records`, in
`src/ui/model/records.rs`), not seven fields. They are one pane's questions
about one subject and they are retired together the moment that subject moves,
so seven fields would be seven places to remember — and forgetting one paints
one conversation's records under another's name. It is `super::listing`'s
reframe one layer over.

**Six of the pane's seven reads stand and the seventh is addressed.** The
others are about *the selected conversation*, a subject the window already
holds, so they ride `Standing::records` and cost nothing while the pane is
shut. `step` is about **one step of it**, and the window holds no such
selection — a standing read would have to invent one and then hold it, which
is a second authority for a row an operator clicked. So the control on a steps
row posts it, exactly as §4.24's `models` is posted off a provider row.

**The answer says which row it belongs to, so the model remembers nothing.**
`reply/step` echoes back the `seq` it was asked by, and the paint asks the
model per row (`Model::drilled_into`). Two things follow and both are the
point: a reply that lands after the operator clicked another row cannot paint
under the wrong one, and there is no second name for *which step is open* to
drift from the answer.

**`agent` is the pane's header rather than a row in its scroll**, because it
is the subject every other half is about. It is the largest shape on the reply
surface — twenty-one fields over five nested objects — and the reader takes
all of it but one: the `value` beside every `step` record's `raw` is **not
decoded**, because this seat paints the bytes and §4.9 does not carry a field
for no glass. That is the ledger's own rule spent inside a shape rather than
across one.

**Three of the facts are the server's own rendering and none is recomputed.**
The strip's characteristics are prose one derivation assembles with per-segment
omission rules; the money is the server's spelling of the integer beside it;
and the context percent is its rounding, deliberately **unclamped**, so a
context that has outgrown its window reads as `140%`. A seat that divided the
two figures itself would be re-taking a decision upstream took on purpose.

**The priced figure is §4.31's reader, not a second one.** yog writes the
`agent` answer's `spend` with the same encoder it writes a board row's, so this
seat reads it with the same decoder (`src/reply/spend.rs`) — and what that
costs is three fields nothing here paints, which ride through unread by rung 4.
That is the ledger's rule spent the other way round from the `step` record's
tree: one shape, one reader, and the fields no glass spends are the price of
not having two.

**The cascade is the pane's one act on the conversation itself** (bl-3686),
and it is here because this is the pane that says what is under there. The
header's descent line says what the conversation hangs under, the spine's cards
say what was dispatched off it, and the offers line is the server's own
sentence about whether `stop children` is available at all — so the control
that fires it sits on that line, and its label is [`word`]'s own string for
`Offer::Children` rather than a second spelling of it. It is **absent** where
the server offers no cascade and **disabled** until the arming box holds the
conversation's name, which is §4.20's two states read one after the other: what
is missing in the first case is the subject, and in the second a parameter. The
composer's `stop` and the row menu's stay the BARE form, because both are
routine surfaces and §4.20 keeps a subtree off one.

**The boundary field has one site in this window, and retiring it deletes one
line.** litany bl-3114 made every litany stop take its children, so yog retires
`stop`'s `children` field at its next PROTOCOL bump (yog bl-6efc). The only
place in this seat that means *yes* is `Model::post_cascade`'s `true`, and that
literal is what the bump deletes. What the bump does not settle is where the
arming goes afterwards: once every stop is this act, the composer's control and
the row menu's become it too, and the arming has to move to them — a seat ball
off the bump, recorded here rather than left for an operator to discover by
stopping a subtree from a list row.

**The pane is DENSE now, and the density is a constraint rather than a taste.**
Seven halves ride under one scroll, and `crate::snapshot::clipped` fails the
whole matrix over a single control laid out past the frame — which is what a
long pane produces, because a scrolled widget is still laid out where it would
be. So a heading shares its line with its first fact, a step's headline shares
its line with the control that drills into it, and the header says several
facts per line joined by a separator. **Nothing was dropped to make room**;
what was dropped is the line breaks — and bl-3686 spent the last two of them
that were going spare. The header's cascade did not fit: at the narrow shape
the pane had under four points of slack, and adding one wrapped row put a
control off the bottom of the window. So the `steps` heading and the governing
half's file list were swept to the rule the other halves already keep — a
heading shares its line with its first fact, and three facts about one commit
share one wrapped row. **That is the measurement, not an estimate**: the matrix
reddened, and it is why a control cannot be added to this pane without one.
The alternative — showing one half at a
time behind a section selector — is a view this pane may still grow, and it
would dissolve the constraint rather than pay it.


### 4.33 The fleet: running a workspace's balls, watching them, and what they changed (bl-a43a)

`src/reply/diff.rs`, `src/reply/science.rs`, `src/verbs/fleet.rs`,
`src/ui/fleet.rs`, `src/ui/fleet/rows.rs`, `src/ui/model/fleet.rs`. **The
thirteenth covering pane, and the fifth whose subject is the aimed workspace.** It
is the other half of bl-4c48, whose trail half is §4.27 and whose board half is
§4.31: seven ops, all of them carrying a workspace, and until this landed a
workspace could be watched from this seat and never run.

**The naming trap is the first fact about it, and it costs an afternoon to
rediscover.** `fleet` and `disband` are the **loop** — claim this project's top
ready ball and start a drone on it, up to a cap. `arm` and `disarm` are the
**alignment monitor** — a cheap model reads each commit against its goal and
records a verdict on the trail. Two families, two settings, two carriers, and
**one shared reply kind**: all four answer `{"kind": "armed", "armed": BOOL}`,
so no reader can tell which family an answer belongs to by looking at it. Every
reader here reads the **op** back instead. The poster already stamps a routed
receipt with the op it answers (§4.26's `Said::Receipt`), so
`Model::receipt` joins the two before the pane ever sees them and
`crate::ui::model::Armed` carries both. A seat that classified off the reply
would be guessing between two settings.

**Neither stopping act is armed, and that is an argument rather than an
omission** — the one bl-a43a asked for by name. §4.20's arming is for the
**unmaking**: an act whose product is that its subject is gone. Neither of
these is that. `disband` *"stops nothing that is running; everything already
running is untouched and keeps its ball"*, and `disarm` leaves *"every verdict
already recorded on the trail"*. Each is undone by doing the other thing — one
`fleet` re-arms the loop, one `arm` re-raises the watch — and an arming on an
act a click reverses teaches an operator to type through armings, which is the
one thing that would make the unmaking's arming worthless. What they get
instead is the unmaking's ORDER: the acts that start things first, the acts
that stop them under.

**It holds the workspace it was opened on rather than following the aim**, which is
§4.20's unmaking arrangement and here for a sharper form of its reason: these
acts start drones and spend money in ONE workspace, so a gesture composed
against whatever the aim happened to be at the moment of the click would be a
control that moved under the operator. The pane is retired when the aim moves,
so the two can never disagree — and holding the workspace is what makes that a fact
rather than a promise.

**Three boxes, because three acts carry a word nothing on the glass can
derive.** A cap is a number — which is also why `fleet` is a typed door rather
than a row in the verb table (§4.10) — a project is a name, and a monitor's
model is a name. Each control stands down until its own box has something in
it, which is §4.20's enablement rule with the parameter missing rather than the
subject. The cap opens at **one** and will not go below it: a cap of zero is a
loop that spawns nothing and still reaps, which upstream refuses to spell as a
cap at all, so it is not a value this box can send.

**The pane holds no loop STATE and that is deliberate.** Whether a loop is
running, how full it is and when it last acted are on the `board` answer, which
the ball pane paints (§4.31) — one fact, one home. What this pane holds is the
last receipt, which is an event rather than a state: what happened when the
operator pressed the control, said once, in the op's own name.

**Both reads STAND, on the trail's terms.** `science` and `work-diff` are
**derived when you ask** — nothing behind either is stored — so the same row a
minute later is a statement about the world a minute later, and a posted-once
read would paint a moment that had passed. They are keyed on the pane
(`crate::state::Open::Fleet`) and asked only while somebody is looking. Their
`act:` tokens ride on the control that OPENS the pane, exactly as the tuning
and login controls carry theirs, because opening it is what asks them.

**One diff row, two answers.** A science attempt's `diff` object *is* a
work-diff row — upstream encodes both with one encoder *"so an attempt's
identity has one spelling anywhere"* — so `src/reply/diff.rs` is one type read
by both, and its `state` rides verbatim with every field that state does not
write as an option. Binary is read off the SHAPE rather than a token, because
upstream writes counts or it writes `binary`, never both.

**The patch is not read, and the ledger says so.** `work-diff` answers a
`patch` beside its rows when the gesture names a ball and a path. This seat
composes the bare listing only, exactly as it composes the bare `files` read,
so that field can never arrive here and a decoder for it would be a reader
nothing exercises; it rides through under §4.9's rung 4, and
`src/verbs/tests/corpus/emits.rs` records the two file forms by count and
reason. `parity.toml` has nothing to say about it because that ledger is per
OP, and `work-diff` now carries a control.

### 4.34 The capability boundary: the parked call, and the floor (REMOTE §5, §9.11; bl-bce2)

`src/verbs/capability.rs`, `src/ui/queue.rs`, `src/ui/composer/acts.rs`,
`src/ui/model/notice.rs`. **Three ops on one subject and two homes**, because
the subject divides where the panes already do: `answer` is about the one
invocation waiting right now, and `revoke`/`restore` are standing policy about
the conversation an operator is looking at.

**`answer` is on the decision queue's row, and the row is why.** §4.19 records
that the queue paints `held` — the invocation parked at a conversation's
boundary — and says outright that *"the control that releases or declines it is
`answer`, and that belongs to the tool-host surface this seat does not have"*.
It has one now, and it is not a tool-host surface: the pane whose whole
question is *what is waiting on you* is the pane a parked tool call belongs to,
and it is the one place in the window that already says what is parked. Three
controls, one per verdict, offered on a row that is holding a call and on no
other — the wire's own three words, painted as themselves for
`crate::verbs::tuning::levels`'s reason — and, under their own line, `pass` at
each wider reach the held row's `scopes` lists (§4.42 item 8; bl-da16): the
pane paints the ask's own `choices`, one list for both surfaces.

**Nothing names the invocation, and that is the gesture's own shape.** The
server reads the held `tool_use` off the conversation's hold mark at fire time,
so the envelope carries a verdict and an address and nothing else. A seat that
carried an id would be able to answer a call that is no longer parked.

**`revoke` and `restore` are on the composer's second row**, beside `stop` and
`retarget`, because a floor changes nothing about the turn and everything about
the conversation — which is exactly what that row is for (§4.11).

**Both are offered, always, and that is a decision this ball took against its
own filing.** The pin pair's rule (§4.25) says a row is offered the assertion
that is not already true of it, and the ball expected to read the rank off
`reply/agent`. It is not there: the `agent` answer §4.32 decodes in full
carries no floor field, and `floored`'s `standing` is the receipt of an act
just performed rather than a standing fact. What dissolves it is that
**neither act can be refused**. A floor is one row appended to the server's
trail and the reply is re-derived from that trail afterwards, so `restore` on a
conversation nobody floored is not an error, and `restore` under a
still-floored ancestor leaves the floor standing and *says so*. There is no act
here the far end may have already decided against, which is the only thing the
pane doctrine was written to forbid.

**Both receipts carry a fact, so both become the bar's one line** — the sixth
kind of `Notice`, and the first that is not a failure. `answered` says which call it landed on and whether the conversation
is running again (`hold` launches nothing, which is the operator saying *stay
parked*); `floored` says whether one stands over the conversation now. Neither
has a row to sit under and neither fills a pane, and the window has exactly one
place for a one-line statement about an act just performed. Being there is also
what makes each dismissible: an act is an event and not a state, so it does not
re-post on a beat.

### 4.35 The trail's two acts: the fan is the reading, and the cut is a place (bl-b8f7)

`src/verbs/trail.rs`, `src/ui/trail.rs`, `src/ui/clear.rs`,
`src/ui/model/trail.rs`. §4.27 landed the trail's read and left its two acts,
and bl-b8f7 filed both of the questions they raise as questions to settle
**before** a control exists rather than after. Here are the two answers.

**Both fan, and the fan is the right reading rather than a shape to work
around.** Neither `ack` nor `clear-trail` carries a field of any kind — the
envelopes are `{"op": "ack"}` and `{"op": "clear-trail"}` — so
`crate::verbs::Verb::addresses_a_workspace` is false for both and the poster
puts each down every channel this box holds (§4.21). The alternative bl-b8f7
named was growing a channel on `crate::ui::Posted` and offering each act per
section. It is refused, for three reasons that are one reason:

- **The pane IS the union.** `ops` names no workspace, so the trail this seat
  paints is the union across channels (§4.27). An act offered per section would
  have a subject narrower than the pane it stands on, which is a control whose
  scope an operator has to reconstruct from where they clicked.
- **`ack` is the operator's watermark, and the operator is one.** It records
  *I have looked at what is standing*. Somebody looking at the union has looked
  at every section; a per-server watermark would be bookkeeping this seat
  invented and the operator would have to keep.
- **A channel on `Posted` would be a second addressing table.** This crate
  routes by the envelope's own `workspace` field and nowhere else — that is
  `crate::envelope`'s *"one table, not two"* — and a gesture addressed by a
  field beside the envelope would be a second mechanism for the one question
  the first exists to answer.

What the seat owes instead is **saying so**, which is why the cut's pane names
the servers it reaches one to a line rather than counting them, and why its
word is `cut every trail…`.

**The cut is an unmaking, so it is a PLACE** (§4.20) — a covering pane of its
own (`src/ui/clear.rs`), opened from the trail, holding nothing else, with the
way out first and the act last. All three of §4.20's reasons carry across
untouched: a covering pane is the only placement identical in both layout
shapes, it is already inside the reach walk's and the parity walk's
instruments, and a pane an operator moves through must not be able to take a
mis-aimed click that cuts something. It is a member of `Lookup` rather than a
field of its own, which makes *the trail and its cut standing together*
unrepresentable rather than merely unreached.

**And there is no arming, because there is no name to type back.** §4.20's
arming is **the subject's own name**, and it is an enablement there because the
WIRE makes it one: `delete-workspace` carries a `typed` field and refuses
unless it matches. `clear-trail` carries no field at all, and that same section
rules that *"the seat reads which of the two a `typed` is off the wire's own
grammar and invents no policy"*. A box this seat invented would be an arming
the server does not have on an act it does not gate — and a second confirmation
shape beside §4.20's, which is exactly what that section was written to
prevent. What the arming was FOR is bought by the place instead: the subject is
stated before the act is offered, and the act is not a control an operator
passes.

**Two departures from the unmaking, each with its own reason.** Its pane stays
up saying *asked — waiting for the server*, because a refusal is the common
case for `delete-workspace` and the arming has to survive it. Nothing refuses a
truncation, so this pane stands down onto the trail as it fires: the place to
be standing when the answer arrives is the pane that answers, and the trail's
read is standing. And its way out returns to the trail rather than to no pane
at all, for the same reason — that is where the gesture came from.

**Neither receipt is filed** (`acked`, `trail-cleared`). Both carry nothing,
and what each changed is on the trail: the next standing read answers `acked`
on the rows that were standing, and a truncated trail's first row is the cut
itself. They join `nudged` and `flagged` in the one arm that predicts no
listing (§4.9).

**Amended by bl-9760: none of the four is silent any more.** Predicting no
listing still holds; painting nothing did not — a `/nudge` with nothing to
advance and a `/retarget` with nothing to retarget changed nothing any read
would show, so the window could not tell an act that did nothing from one that
never fired. Each receipt, and a successful captured run, now stands in the
state slot (§4.42 item 2) as `Notice::Said` in the Annotation ink, carrying the
CLI's own sentence (`crate::render::receipt`, one source for both seats), until
Esc or the next notice. The one exception is a deposit (`message`,
`interrupt`): its receipt is the operator's words on the transcript, and an
`ok` standing over the conversation's state after every send would hide the one
fact the slot is for.

**Amended again by bl-a7f5: a receipt is about the selection it was spent
on.** Two holes the round-2 drive found. The rule's selectors (§4.42 item 11)
spend `model`, `effort` and `priority`, whose success is a captured run with
nothing to say, so every pick stood a bare `ok` over the state word until Esc
— those three join the deposits, because their receipt is the segment the
roles read repaints on the next beat. And a receipt outlived the conversation
it was about: `seen` on one, then `+ new` and a whole turn of another, read
`seen <the first> — off the queue` where `live · 3s` belonged. So moving the
selection (`Model::select` to another conversation, `Model::aim_at` off every
one) retires a standing `Notice::Said` — and only that arm: a refusal is the
operator's to read wherever they went.

### 4.35 The ball pane's five acts: the stamp that is not an identity, and the one act with no undoing (bl-f7ae)

`src/verbs/balls.rs`, `src/verbs/balls/edit.rs`, `src/ui/board/acts.rs`,
`src/ui/model/board/acts.rs`, `src/ui/model/board/block.rs`. **The act half of
§4.31**, which landed the read: `create`, `assign`, `release`, `update` and
`close` are what CHANGE the store, and until they landed an operator could
watch a board and could not touch it.

**The fan question, settled: none of the five is fanned.** All five carry a
project and the `--as` name and NO workspace, so `crate::verbs::Verb::
addresses_a_workspace` is false for every one of them and the poster's own
rule would spread each over every channel this box holds — one click filing
the same ball on five servers, or asking five servers to deliver a ball one of
them holds. That rule is right about the window's own reads and wrong here,
and the envelope cannot tell the two apart because on the wire they are the
same shape. **The composing control can, for free**, which is bl-4855's
answer for `config` and is now this family's: every control here hangs on a
row that came down ONE channel, so each says which
(`crate::ui::Posted::down`). Nothing new was built for it — the address was
already there, and the question was whether these five were allowed to use it.

**The `--as` name is the workspace's, so this seat needs no identity.** The
ball that filed this asked where a seat-side operator name would come from — a
box, a setting, or the aimed workspace's `owner`. It comes from none of them,
because it is not an operator name at all: yog spells the field *"the `--as`
stamp every `bl` verb carries (§3.2): the ball's bound workspace name, never
the operator `$USER`"*, and `projects::join::owner_name` binds a ball to a
workspace on exactly that equality. So the stamp is the aimed workspace's name and
the whole question dissolves — a seat that invented an operator name would
break the binding it was making.

**It is read off the CHANNEL and not off the aim**, which is the one subtlety
in it. `crate::seat::route` rewrites a `workspace` field to the host's spelling
where an entry renames, and none of these five has such a field; so an aim's
`address` — this box's leaf — would cross as a claimant nobody on that server
is. `Model::stamp` reads `Channel::named_there`, which is that same rename
seen from the other end, and falls back to the address for the channel that
does not rename.

**Where each hangs is where its fields already are.** `assign` is on a board
row, because a ball nobody holds IS a row of that section and the workspace it
would be claimed for is the aim; it is offered off the row's own `claimant`
rather than off the column's word, so a `gated` row — upstream's *"a ball you
could claim but could not deliver"* — offers it and a claimed one does not. The
other four are in an authoring **block** the aimed workspace's section opens, which
is where the ball's project, its id and the name its verbs stamp all already
are. One block with two subjects — a ball that does not exist yet, and one this
workspace holds — because authoring a ball's words is one act; upstream folds them
the same way and says why, *"one vocabulary, so a fact balls learns is added in
one place instead of in the roster, the codec's field list and a second struct
beside them"*.

**Two of the five are doors and can never be rows** (`src/verbs/balls/
edit.rs`). §4.10's table is rows of named strings written by one builder that
writes every parameter it names, and `create`'s body and each of `update`'s
three may be **absent** — absence being a value here, since an empty string
asks upstream to blank a field nobody touched. Same rule as `effort`'s
string-or-null and `fork`'s list, third application. `assign`, `release` and
`close` are three named strings apiece and are rows.

**`close` is armed and the other three are not.** §4.20 divides on one test —
*undone by doing the other thing* — and three pass it: a filing is undone by
releasing or closing what it filed, an amendment by writing the old words back
(and a `note` is an append that claims nothing), a release by the `assign` on
the board one section up. `close` folds `main` into the worktree, squashes the
work onto the branch and removes the worktree, and there is no verb that
reverses it. So it takes §4.20's **enablement**: dark until the box holds the
ball's own id, with the refusal spelled beside it.

**It does not take §4.20's other half — a pane of its own — and that is an
argument rather than an omission.** That pane exists because
`delete-workspace`'s subject is a row in a routine surface an operator moves
through quickly, *"and a mis-aimed click there must not be able to land on
this"*. This one is already two deliberate gestures deep — a covering pane,
then a control on one of its rows — and the arming is a third. **The arming
names its own subject**, which is why nothing records which ball is armed: the
box holds an id and the control it enables is the one on the ball with that id.
And it is never spent on firing, for §4.20's reason: a refusal is the common
case here — the server declines while the gate fails — and clearing the box
would charge a retype for the server's *no*.

**Nothing new is decoded and no receipt is invented.** All five answer with a
captured run, which §4.9 already paints: a refusal arrives in the child's own
words on the bar, and a success says nothing (*amended by bl-9760: a success
now says the CLI's own sentence in the state slot — §4.34's amendment*). That silence was this pane's
standing reads paying for themselves (§4.31: all four STAND while somebody is
looking) — a ball filed appears in the ready column within the beat, a claim
moves a row, a close takes one away. A `Notice::Said` receipt here would be a
second, worse statement of what the pane itself is about to show.

**What this did NOT land.** Neither authoring door composes the four
scheduling facts — a priority, a tag, a parent, a blocker — because `fields` is
an array of objects and each is a picker this pane does not have;
`src/verbs/tests/corpus/emits.rs` records both by count and reason. `marks`'s
amending form stays unbuilt beside them: a tracking branch is not a ball.

### 4.36 The n-candidate path: spread one obligation, accept one, release the rest (yog's VISION §4.10; bl-8ffa)

`src/verbs/candidates.rs`, `src/ui/model/start/spread.rs`,
`src/ui/fleet/candidates.rs`, `src/ui/model/fleet.rs`, `src/reply/start.rs`,
`src/reply/kinds.rs`, `src/ui/model/notice.rs`. **Not a pane** — three acts on
a listing §4.33 already paints, which is the whole of what makes them cheap.

**The row says which act it earns, and upstream's encoder is what decides.** A
work-diff row carries a `handle` or it does not: with one it is a **candidate**
on `attempt/<handle>` waiting to be accepted or released, without one it is the
ball's **own claim**, whose delivery obligation is the thing a fan spreads. So
the three controls are not a mode this pane holds; they are what the row IS,
and every value the acts take — the project, the ball, the handle — is already
on it. A typed handle box would have been a chance to name a candidate that is
not on the glass.

**A fan is the start with n in the middle** (§4.26 read over n), not a second
start path. `crate::ui::model::Start` gains one field — the obligation, where
there is one — and everything that value already carries applies unchanged:
one is outstanding at a time, a refusal retires it with the goal back in the
box, an unread answer takes it back. What a spread changes is exactly two
things: the staging receipt composes `fan` rather than `prompt`, and the fan's
own answer composes one ordinary `prompt` per candidate.

**The gate is that two `prepare` acts in flight cannot be told apart**, because
the receipt carries no correlation with the gesture that earned it. That is
already `Start`'s rule for a second start and it is why the fan is held there
rather than beside it: a second field would have made *both outstanding* a
representable state that only the absorb order resolves.

**The frame fires the candidates and no control does**, which is §4.26's
argument sharpened. A `fan` MATERIALIZES n attempt worktrees, so a candidate
prepared and never fired is a worktree balls made for nothing — firing them is
the completion of the act rather than a convenience. **What that forfeits is
stated rather than hidden**: upstream's terminal fires each itself *"with
whatever variation you want between them"*, and this window fires n with one
goal. Per-candidate variation is a surface — n boxes, or one edited n times —
and it arrives with the ball that builds it; what it is not is a reason to
leave n worktrees empty.

**A spread's receipts select nothing.** *A start focuses what it started* is a
sentence about one conversation; n `started` receipts land one after another
and the focus would be whichever arrived last, which is a fact about the
network rather than about the operator.

**`deliver` and `retire` are §4.30's first customer**, and without that ruling
this ball could not have landed. Neither names a workspace anywhere in its
envelope — their subject is a ball in a project on ONE server — so the poster
would have fanned them over every channel this box holds, accepting one
candidate on every server the operator is a client of. `Model::post_candidate`
addresses them down the channel the pane stands on. `fan` needs none of that:
it carries its workspace inside the prepared body, which `crate::envelope`
already reads.

**They are doors and not rows for the same reason.** A row is a word an
operator TYPES, and argv has no channel selector — `lernie deliver …` would be
the fan hazard on the other surface. `lernie ask` stays the door argv has.

**Nothing here is armed, and that is §4.33's argument one noun over.** §4.20 is
for an act whose product is that its subject is gone. `deliver` advances a ref
by the ordinary recursive delivery — git holds what it moved, the ball is not
closed, and what its close delivers is unchanged. `retire` releases a worktree
and *"changes no delivery target, ever"*; whether the source ref goes with it
is this project's own declared retention, acting on a schedule the operator set
elsewhere, and the reply says which way it went. So the seat PAINTS that answer
(`Notice::retired`) rather than predicting a policy it has not read. There is
no reject control because there is no reject op: a rejection is the *absence*
of a delivery.

**Two of `delivered`'s four identities are optional and each absence is a
fact** — no `source` is a source ref that was not there, no `commit` is a
delivery that landed nothing. Painting either as a blank would report a
delivery that did not happen.

**The count is a stepper floored at two.** Upstream reads 1 and 0 as
*materialize nothing and hand back the ordinary claim binding*, which is a
start and this pane already has one. Its buttons carry the step (`−1`, `+1`)
where the cap's carry a bare glyph, because two steppers on one pane spelled
the same way are one control an operator cannot tell from another.

**The ball is always named.** All three take an optional `ball` upstream, and
omitting it means *the server's own focused ball* — a focus a seat does not
have, since its gestures are composed off a row that already names one. The
three ball-less frames are recorded in `src/verbs/tests/corpus/emits.rs` by
count and reason.

**A candidate says what it cost** (yog REMOTE §9.23; bl-9111). Each delivery
attempt carries `cost` beside its four counters, and the fleet pane's attempt
lines paint it after them on §4.18's rule — which is what an operator weighs
candidates by besides the diff: the same obligation, spread n ways, at n
prices.


### 4.37 The rendering: the seat is the part you look at (bl-6ae7, bl-b00f)

`src/render/`. **Every verb prints a human rendering by default, and `--json`
is the raw frame stream.** That is the round-1 ruling and it reverses what this
crate shipped through 0.1.30: the boundary's own machine form was the ONLY
form, so a day of CLI use was a day of piping into a JSON formatter, and
`transcript` — which carries every entry AND that entry's `raw` bytes — was one
line of tens of kilobytes, each conversation twice over.

**The rendering is per reply KIND, never per verb**, and that is what keeps it
one implementation rather than a second table beside §4.10's. A verb is a
serialization of a *gesture*; what comes back is a kind of
`crate::reply::Reply`, and several verbs answer with one kind — `fleet`, `arm`, `disarm` and `disband` all answer
`Armed`. A table keyed on the verb would hold four spellings of one rendering
and they would disagree within a week. So the census §4.9 keeps is read in the
other direction here: a kind added there is a match arm missing in
`render/answer.rs`, which is the compiler saying the rendering half of a pane
was not built. (`seen` answers `acknowledged`, its own kind since PROTOCOL 12 —
the item acted on plus the queue that remains — and its rendering is that line
over `attention`'s own listing, one rendering reused rather than spelled twice;
bl-6802.)

**The design note that was taken for a ruling.** The help said *"the reply
stream prints one envelope per line"* and that sentence was read as a statement
about the CLI's presentation. It is a statement about the WIRE, and the two are
different questions — which is exactly why `--json` can keep the envelope byte
for byte while the default renders.

**A rendering is a reading, and the frame is always one act away.** It paints
what a person scans a listing for and elides the rest — an entry's `raw`, a
preview's third paragraph, a step's whole captured document. That loss is
deliberate and it is safe *because* it is not the only form. What must never
happen is the other direction: a rendering that states something the frame does
not.

**The branches live in the vocabulary, not at each field** (`render/parts.rs`).
Every optional field on this surface is rendered by handing it to `clause`, and
every flag to `when`, so *present* and *absent* are written once and executed by
whatever frame carries each. Forty-one renderings each with its own `match`
would be forty-one pairs of arms the corpus — one fixture a kind — could never
reach both of.

**The empty arm is a parameter of a listing and never a default**, and that is
where bl-b00f is answered. A fresh world answers `rows: []` correctly and
terminally; the one fact a person needs at that exact moment is that a world
bootstraps its first workspace when a conversation is started in it (yog DESIGN
§3.1, the fixed name `home`), so the roster's empty arm names the act rather
than printing an empty listing and stopping.

**`--json` is read off the FRONT of argv and nowhere else** (`crate::cli::JSON`).
Every gesture parameter on this surface is a verbatim string — a goal, a
message's content, a ball's title — so a flag scanned out of the middle or the
tail would silently eat an operator's own text and `lernie message w a --json`
would stop being a way to send those seven characters. Leading, everything
after it is the gesture byte for byte. The cost is that the spelling everybody
tries first is the trailing one, and that is paid where it is made: an arity
refusal whose arguments contain `--json` says where the word goes.

**A rendering elides, and it says where it did.** That is the reading rule
above with a name on it, and the work diff is where it was broken: the frame
carries `truncated` — the server stopped listing files at its own bound — and
the rendering decoded the field and printed nothing, so a cut listing read as
the whole change (bl-2e1d). The worktree listing had held the same rule with
its own `(truncated)` since bl-6ae7; the diff now holds it with a line naming
what it knows and what it does not, because the count the server dropped is
genuinely not on this wire.

**And that diff is a NUMSTAT, which bounds what a diff rendering here can be.**
yog answers `work-diff` off `git diff --numstat` — a path and two counts a
file — so there is no hunk on this surface, no `@@` header and no diff LINE to
omit; the named form that answers one file's patch is deliberately not composed
by this seat (`crate::verbs::WORK_DIFF`). What a per-file `a/… → b/…` header
says elsewhere is said here by the ROW, because the two ends being compared are
the row's two refs and the two commits they resolved to — decoded since the
vocabulary was written and printed nowhere until bl-2e1d.

**The follow lane has two writers and one column tells them apart** (bl-293d).
It is the one surface where the model's prose and the seat's own narration
arrive interleaved, a line at a time, as the turn is written — and an operator
watching an agent administer two boxes read a thinking marker and two sentences
while eight commands ran. So the model's answer keeps the left margin, unmarked,
and **everything this seat says about the run rides behind `┊`**
(`render/parts.rs`'s `GUTTER`): the heartbeat, the line the watch ends on, and
the tool window's activity rows when the frame carries one (bl-183b, which is
where `tools` is decoded — until it lands there is no call on this frame to
count, which is why the closing line carries an elapsed and not yet a tally of
calls). **One glyph and no locale branch**: a terminal that cannot draw `┊`
cannot draw the `…` every elision on this surface ends in nor the `→` half the
rows carry, so a fallback here would be one line's exception to a bet the whole
rendering already makes.

**One place, three surfaces.** `crate::render::said` is what
`crate::seat::lines` used to be — the one place this seat's product is written
— so the rendering reaches one gesture, the fan across channels and the
composite start's two streams at once, and there is still exactly one spelling
of *what a seat printed*.


### 4.38 The look: the phone's visual language, and the desktop's deltas (bl-73d2)

**Superseded for the LAYOUT by §4.42 (bl-7859): one column, a status rule, a prompt line and overlays. What stands below is the record and the model-side rulings; where a sentence here says where a thing is painted, §4.42 says otherwise.**


`docs/STYLE.md`, `src/ui/theme.rs`, `src/ui/theme/visuals.rs`,
`rules/no-literal-colour.yml`. **This document says what a pane IS; `docs/STYLE.md`
says what it LOOKS like, and it is normative** — a pane that departs from it
is a defect, and the fix is in the style file or the token module, never on
the pane. The language's home is the phone seat's own `docs/STYLE.md`
(yog-android bl-549b): six state colours and no seventh (operator ruling
2026-09-06 — green asks for you, blue works, purple is the model generating,
orange annotates, grey is done, red is an error), no outlines, and one module
every colour and size is read from. This seat adopts the palette byte for
byte and carries only its deltas, each named beside its token with its
reason: a smaller body and heading, a 28-point row where the phone's is a
48-point touch target, and the column shape the width policy of §4.11
already answers for (two columns since §4.39).

**The adapter installs the tokens at the top of every frame**
(`theme::install`, called by `ui::render`), so the offscreen frame every
assertion reads is painted in the same style the operator sees. The one
rule added is the phone's own: `Color32` is constructed in the theme, in the
paint probe that reads one back, and nowhere else. The ink a run reached the
glass in is already on the paint walk (`paint_probe::Seen::ink`), which is
what makes *the asking row is green* an assertion rather than a picture.

**The anatomy is paint, written once** (`src/ui/theme/paint.rs`; bl-d1ae).
The window it replaces had a `selectable_label` here, a `Button` there and a
fill painted under a run somewhere else, each with its own box, which is how
a face ends up with no hierarchy. Five shapes now cover every pane and no pane
draws its own: the **row** (full width, `ROW` tall, one elided run, a tint
under the pointer, `RAISED` when chosen, and a `RULE`-wide accent at its left
edge — the brand when chosen, else the row's state), the **rail** and the
**elbow** (the phone's L-shaped threading, in faint ink), the **ruled block**
(the transcript's shape: a `RULE` in the speaker's weight beside the block),
the **section** (`L` of air, a hairline, a word in weak ink) and the **field**
(`SURFACE`, the brand ring while it holds the caret, and a tint where the pane
has something to say — the composer glows in the attention accent while its
conversation is asking). A row is ONE run so a click aimed by painted glyphs
lands on exactly what the pane composed, and the accessibility node it
registers is labelled with the same words — which is what keeps the snapshot
walk and the parity ledger reading the restyled window unchanged.

**What the eye lands on is green, and only green.** A workspace or a conversation
that is asking wears the attention rule; the queue's entry in the roster's
band wears the attention accent while anything waits; the composer glows. A
window with nothing waiting has nothing green on it, which is the same fact
read the other way. Every other accent is a state on the six-colour ruling,
read off the wire's own words through `theme::state_of` and
`theme::tone_ink` — this seat never infers a state from prose.

**The composer is the conversation pane's focal element** (bl-f251;
`src/ui/composer.rs`, `src/ui/composer/offers.rs`, `src/ui/composer/acts.rs`,
`src/ui/theme/paint/field.rs`). The first pass left it a one-line bar over a
row of nine chips, which read as a form under a mostly empty pane. It is now
a field `theme::COMPOSER_ROWS` lines tall with the send inside it, and its
height is **allocated from the rows rather than left to the layout**: a
bottom panel lays each frame out in the rect its content took last frame,
and a bottom-aligned layout given the whole of that rect reported it whole,
so the panel grew by a row a frame and the transcript's tail walked off the
glass — the bound is what makes the composer's height a fact of its content.
Enter sends and Shift+Enter breaks a line; the field consumes only the
shifted key, and the bare one is read beside the act it fires, off the
event's own modifiers — **one read, shared by both modes**
(`composer::entered`), because the second mode below is the same box and two
spellings of Enter would be two boxes again. Everything in this paragraph is
said of the deposit and holds of the START as well since §4.39's ruling
landed: that mode is the same field at the same rows with `start` inside it
where this one says `send`, and the sentences below about what the row
offers are the deposit's alone — what the start's row carries is §4.39's.
Under the field, **the row offers what the server
offers**: `interrupt` and `stop` while the conversation's own row says its
driver may be stopped, `nudge` while it says an advance may be started, and
all three until the server has answered about it at all — an absent reading
is not a refusal. That reverses §4.32's ruling that the composer's controls
do not read the four gates; what stands here is the server's own answer read
off the same row the header paints, not a prediction, and a `nudge` offered
on a streaming conversation was a control whose every press was a refusal.
The acts on the conversation as an object stand behind one `…` control
(`composer::MORE`), in §4.20's order with the deletion last; the strip's
open state is the toolkit's memory, as a folded tool result's is, and a row
menu's *flag…* or *delete…* opens it on the way to the box it named
(`crate::ui::model::fill`) — so the parity walk has a world in which it
stands open (`snapshot::worlds::filling`), because a control on a screen
the walk never visits is unproven. Each compact control is a glyph and a
word (`theme::glyph`, `theme::worded`), and every glyph is asserted to be in
the body font.

**The header over the transcript is the pane's name** (bl-f251, item 2;
`src/ui/chat/subject.rs`). The first pass arranged the subject as three runs
on one body-size line, which read as the first row of a list. It now stands
at `HEADING` size with the state word beside it, the model and the spend on
one weak line under it, and the subtree path under that where the row's
`ancestors` says it hangs under anything — every clause still the server's,
composed by `records::header` and read from there. The size is asserted off
the glass as a height: the name's laid rect is taller than the state word's.

**The type scale grew a point and prose grew a leading** (bl-f251, item 3;
`src/ui/theme.rs`, `src/ui/theme/paint.rs`, `src/ui/chat.rs`). The body is
14 where the first pass set 13, and the transcript's bodies are laid through
`theme::paint::prose` at `type_scale::LEADING` — the one run on the glass
with a line height of its own, because prose is read line after line where
a list is scanned. Entries are parted by `M` of air and no line: the rule
beside a block is the speaker's weight, and a divider between blocks would
have made it read as a separator. Asserted off the glass as heights and
gaps, never as a picture.

**The two bands are compact strips** (bl-f251, item 4; `src/ui/roster/acts.rs`,
`src/ui/roster/workspace.rs`). The window's acts were six full-width rows of prose
at the top of the roster and the workspace's a wrapped band of eight sentences
under its row; both are now one wrapped row of compact controls a short verb
each — the `OPEN` word of every pane shortened to its noun and ellipsis, the
pin to `pin` — asserted at the roster's own width as a count of the lines
the verbs stand on. The words changed and nothing else did: every control
keeps its `act:` token, the reach walk and the parity gate read the
constants, and the queue's control still wears the attention accent while
anything waits.

**Every empty state names the next act in one weak line** (bl-f251, item 5;
`theme::paint::empty`). *Pick a conversation* said what was missing and
nothing about where to get one; each pane's empty sentence now ends in the
act that fills it — the `+` on the server's row, the box below, the strip's
`refresh`, a fleet run — and every one is painted through one shape in
`INK_WEAK`, wrapped, so a pane with nothing in it reads as a pointer rather
than a blank with a caption. The words are still the constants every test
and the reach walk read; what changed is the sentence and its ink.

**A label off the window is a layout question** (`src/snapshot/clipped.rs`;
bl-d1ae). The assertion's own doc drew that line and its filter did not: egui
marks selectable prose as clickable so a pointer can select it, so every
sentence of a covering pane was judged as a control, and a pane could not
grow a word of prose past the window even inside its own scroll — the records
pane at the narrow shape was 89 points of wrapped prose over, with every
control inside. Labels are now outside the set the walk judges, with the
other-direction beat beside it; §4.32's density constraint still holds for
every CONTROL, which is the reachability the assertion exists for.

### 4.39 The two-column window: servers, their conversations beneath, and edges an operator can drag (bl-46e5)

**Superseded for the LAYOUT by §4.42 (bl-7859): one column, a status rule, a prompt line and overlays. What stands below is the record and the model-side rulings; where a sentence here says where a thing is painted, §4.42 says otherwise.**


Operator ruling 2026-09-17, and it reverses two settled things at once: the
three-column shape §4.11 and `docs/STYLE.md` §2 stand on, and bl-fef8's
ruling in §4.11 that a dragged width is *subtraction rather than loss*. The
ruling in the operator's own terms: the three columns cannot be dragged and
every pane edge should be; *channels* at the top left means nothing; the
pane is the servers this seat reaches, by name, as a list where the last-used
one is open and pinned to the top, at most one is ever open, the rest are
ordered by when each was last opened, and the open one shows its conversations
beneath it; a `+` to the right of the server's name begins a conversation;
and while no conversation is selected the same box stands at the bottom with
the same options. Four parts, each gated on this section and each its own
ball, and the order they land in is stated at the end.

**The word on the glass is *servers*, and the crate's word stays *channel*.**
A channel is what this crate calls the client-side entry that reaches one
server (§4.6: the `<leaf>` is the client's name, the fifth file the host's),
and that is the right word for a thing in `wire/workspaces/`. It is the wrong
word over a list an operator reads, because what they are looking at is yogs:
the box's own and the ones it dials, each by the name this box gave it. So
`roster::HEADING` becomes `servers`, the narrow bar's word follows it for
free (§4.11: a column's name has ONE home), and nothing under `src/channel/`
or `src/ui/model/channel.rs` is renamed — the rename is of a word a person
reads, and one home for it is the whole point of §4.11's rule.

**The roster is an accordion of servers, and it is the window's one list.**
Each server is a row — `roster::header`'s words, the host's name beside it
where the two differ and its dial state after — carrying one control at its
right edge, the `+` below. At most one server is *open*. The open one is
painted first, above the rest; the rest follow in the order each was last
opened on this seat, most recent first, and the name breaks a tie. Opening
one closes the other, so the arrangement is a single fact — *which* — and
not a set of flags to keep consistent. **What the fold hides is the WORKSPACES,
never what a server says about itself** (bl-cff1): §4.7's ruling is that a
channel that cannot be dialled says so under its own header and never in the
shell-wide bar, which holds one sentence and keeps the last writer's — so a
seat with two unreachable servers could discover only one of them from the
glass if a closed row went silent. A closed server paints its row and its
own sentences; its workspaces are what standing under an open one buys. Under the
open server stand its workspaces, and under each workspace row the conversations that workspace holds: the rows
§4.11's middle pane painted, `convs::row::conversation` with its rails,
elbows and menu (§4.23) carried across unchanged, because a threaded row was
never a fact about which column it sat in. The middle column is gone. What
it painted is now beneath the workspace it belonged to, which is where the
operator's sentence put it — *conversations are pinned under the server* —
and a two-level tree under one heading is one scroll region rather than two
panes disagreeing about what is selected.

**The workspace stays, as a heading and not a level to navigate to.** The
operator's sentence names servers and conversations and no workspace, and
the interpretation is stated here rather than taken silently: a conversation
is begun IN a workspace (yog DESIGN §3.1 — a world bootstraps `home` when its
first conversation is started), a server that holds two workspaces holds two
lists, and a seat that flattened them would have to write the workspace's name on
every row to say what a `+` would begin in. So the workspace is a row between the
server and its conversations, `ordered` as §4.25 already orders it, wearing
its own state rule and its five controls when it is the aimed one — and a
server with one workspace, which is the common case, reads as a server with its
conversations under it and one weak line between. Clicking a workspace aims it;
clicking a conversation selects it and aims its workspace in the same act, which
is §4.11's *one door for a click and a keypress* with the second list folded
into the first. Opening a server aims the workspace this seat last aimed in it,
and its first workspace by `ordered` where it has never aimed one, so an open
server is never open over nothing.

**What opens and in what order is the seat's own, and that is not the pin's
exception.** §4.25 rules that a pin crosses the boundary because it is *an
assertion about the world, not an arrangement of one screen*. This is the
other case, and the line between them is §4.6's: a channel is a client-side
fact, named by this box and held by no other, so the order this box shows
its channels in can be nobody else's and there is no server to assert it
into. It goes in the place file (§4.13), as keys beside `aim`: the open
server's name, what each server's last opening RANKS as, and the workspace last
aimed under each — unknown keys ignored, a missing key absence, a first run
opening the aim's own server and, with no aim, the first by name. *Last
used* means last OPENED on this seat, by the click on its row or by the `+`;
a beat that answered down a channel is not a use, or the list would reorder
itself under the pointer.

**A rank and not a clock reading** (bl-cff1, amending the sentence above,
which said *instant*). Ordering is the only question ever asked of the
number: an opening writes one past the highest any server holds, so the
server just opened outranks every opening before it by construction. Two
servers opened inside one tick of a wall clock would TIE, and the tie-break
is the name — which would order them alphabetically at the exact moment the
operator's two most recent choices are what the list is supposed to be
about. A counter cannot tie, needs no clock injected into a pure model, and
round-trips through the place file as the integer it already is.

**The `+` is the start, reached without leaving the list.** It clears the
selection, keeps the aim on the server's aimed workspace, opens the server if it
was closed, and puts the caret in the box — which is the composer's own
start mode (§4.38, `src/ui/composer/start.rs`: *a workspace, no conversation →
the start*), reached by a control rather than by finding a way to deselect.
It crosses no wire: `prepare` and `prompt` still fire from the `start`
control the box carries and stay tagged there (§4.16), so the parity ledger
does not move. A `+` on a closed server opens it because a start is a use,
and a server that just began a conversation is the one an operator is
using.

**The keyboard walks one list, in paint order.** The two cursor tracks
(`keys::Pane::Roster` over `roster::aimable`, `Pane::Conversations` over
`Model::rows`) were two because the lists were two panes; there is one pane,
so there is one track — server rows, workspace rows and conversation rows in the
order the glass paints them, derived from the same rows and the same order
the paint uses, which is the rule `roster::aimable` already keeps. Landing
on a workspace aims it and landing on a conversation selects it, exactly as the
pointer does; landing on a server row stands there, and Enter or Space
opens it, since the walk moving through a closed server must not open every
server it passes. **The two begins are stops of that kind too** (bl-f991):
`+ new` under each workspace and `+ new workspace` at the own server's foot (§4.42
item 7) are on the track where the glass paints them, the walk stands on
them without beginning anything, and Enter or Space fires each as its click
does — one `Model::standing` (`Stand`: a server's row or a begin) for every
row the cursor stands on without selecting, never a second track. Left and
right keep their narrow-shape meaning (a column sideways) and gain none. yog's QUALITY F1 stands: every act the pointer
reaches here, a key reaches.

**Two columns, and the policy loses a term rather than gaining a case.**
`shell::policy::widths` yields one list pane to the conversation's floor
instead of two, the list's worth being `CONVS`'s (its widest row is a
conversation's headline and preview; the server and workspace rows are shorter),
`Column` has two variants, the bar names two, and the narrow shape's floor
is where the one list can no longer keep `CHAT_FLOOR` beside it. `SIDE_FLOOR`
stays what it is. `convs.rs`'s pane-level render and its `HEADING` go; its
`headline`, `age`, `continues` and the row module stay where they are and
are called from the roster, because they are facts about a row and its
thread. `snapshot::worlds` and `snapshot::reach` are re-read against the
two-column matrix, and the parity walk is what says the fold lost no
control.

**Every edge an operator can see is one they can drag, and bl-fef8's ruling
is reversed in both of its halves.** §4.11 says *it costs the drag handle,
which is subtraction rather than loss: a dragged width is a second home for
a fact the policy owns, and the drag never worked — a pane pulled wider than
its content snapped back to the content on the next pass.* Both clauses are
wrong and each for its own reason. The drag never worked because of egui and
not because of the design: `SidePanel::show_inside_dyn` floors its content
at the RANGE's minimum and stores the content's rect —

    ui.set_min_width((width_range.min - frame.inner_margin.sum().x).at_least(0.0));

(egui 0.30, `containers/panel.rs`) — so a pane whose rows were narrower than
the width it was pulled to reported the rows' width and shrank to it. A pane
whose body's first act is `ui.set_min_width(ui.available_width())` fills
what it was given, the stored rect is the dragged one, and the drag holds;
`TopBottomPanel` does exactly that itself, one function down in the same
file, which is why the composer never had the defect. And the fact is not
the policy's. REMOTE §7 rules that per-seat UI state — *focus, scroll, tab
selection, drafts* — is the seat's own, and a width an operator set is the
plainest member of that list. The policy owns the DEFAULT and the FLOORS:
the width a pane gets until somebody drags it, the least it may be while it
stands beside another, and the most it may be while the conversation keeps
`CHAT_FLOOR`. So the shown width is the dragged one where the seat holds
one, else the policy's, clamped to the policy's range on every frame — a
window narrower than the stored width clamps it and does not overwrite it,
so a drag survives a window that was briefly small — and the narrow shape
consults no drag, since nothing competes there. Two edges exist in the broad
shape and both drag: the list's edge against the conversation, and the
composer's top edge, whose dragged height sets how many rows the field is
(`theme::COMPOSER_ROWS` becomes the default), keeping §4.38's bound that the
panel is handed a height rather than reading one back off its content. A
covering pane has no edge, and the notice bar and the narrow bar are not
panes. Both values go in the place file with the rest, written once after
the event loop returns (§4.13: nothing new crosses the lock on a frame).

**One composer, in both of its modes.** The start mode paints a bare field
and a `start` button (`composer/start.rs`) where the deposit paints the
composer field with the send inside it, the offers row and the `…` strip
(§4.38); the two are one control with two subjects and did not look like
one. The ruling is that they do: the start is laid through
`theme::paint::composer` at the same rows, the act inside the field worded
`start` rather than `send`, and under it the same row of controls, offering
what applies to a conversation that does not exist yet. `records…`,
`interrupt`, `stop` and `nudge` are acts on a conversation's turn and there
is no turn — they are absent, not greyed, by the offers row's own rule that
only what is offered is on the row. What the row carries in the start mode
is the start's own parameter: the role the conversation is born on
(`verbs::start::ROLE`, PROTOCOL 18), which the seat states between the stage
and the fire and which today only plan mode sets. The `…` strip's acts are
acts on a conversation as an object, and stand down with it. A `+` lands the
caret in this box.

**Built in bl-b3c3, and the role control's shape is the one decision that
ball made.** The mode is laid through `theme::paint::composer` at
`theme::COMPOSER_ROWS`, `composer/start.rs` keeps both `act:` tokens on the
one control the walk can see (§4.16 unmoved — the row's control carries
none, because it fires no op: it states a field of the `prompt` the start
control is already tagged for, and a second token would be a second control
claiming one op), and the row is `composer::offers::starting`. The control
is **a seat worded `plan`** — a glyph and a word, as every compact control
on this row is — and not a picker over the workspace's roles and not a box. The
reasoning is `ui/records/spine.rs`'s own, one noun over: a role resolves
against the governing config, this seat holds no standing read that lists
what that config declares, and *a picker over a list this seat cannot obtain
would be capability theatre*. What it can offer honestly is the one role
choice the WIRE names — REMOTE §9.21: *"the seat deposits the body back as
`/prompt` with `"role": "planner"`"* — which enumerates nothing and is the
choice an operator already has a word for. A workspace whose config declares no
such role is refused by litany before the fork, in its own words, which
REMOTE §9.21 rules is the right division. The day a standing read lists a
workspace's roles, this seat is where they go and the fold is one control wide.
The choice is held as `Model::plan`, because the field is written by the
frame that absorbs `prepare`'s receipt (`ui/model/start.rs::fire`) and that
frame reads the window rather than the gesture; a fan's candidates still
carry no role, the spread being fired from the fleet pane where this control
is not on the glass.

**Built in bl-b9a3, and eleven things are stated here because reality differs
from the prediction above.**

1. **A workspace that is not aimed paints NOTHING beneath it** — not its
   conversations, and not an emptiness either. The prediction left this open
   (*"paint what the model holds and say in the doc what a non-aimed workspace
   shows"*), and what the model holds about a workspace it has not aimed at is
   nothing: §4.12's standing set asks about the AIMED workspace, and `Model::aim_at`
   retires the rows when the aim moves. A *waiting to hear about this workspace*
   under every other workspace would be a column of sentences about questions
   nobody asked, and *no conversations here* would be a definite fact this seat
   has no evidence for — the same thing `convs::UNCERTAIN` refuses one level
   down. So the rows appear under the workspace you aim at, and aiming is the act
   that asks.
2. **`keys::Pane` is deleted, not narrowed.** The ball says
   `Pane::Conversations` goes; what is left is a one-variant enum, a
   `Model::focus` that is always `Roster`, a `revealing(pane)` whose argument
   can only be one value, and a mark on a heading saying which of one list the
   arrows belong to. All of it goes: the field, the enum, `keys::HERE`,
   `keys::heading` and the mark. An answer with no alternative is not an
   answer worth holding, and a heading lit with nothing to contrast against
   reads as a state that can change.
3. **Left and right name nothing in the broad shape**, which is §4.39's own
   *"keep their narrow-shape meaning and gain none"* read strictly. They used
   to name one of two list panes; there is one, so the key has no place to
   point at and points at none.
4. **Clicking a conversation SELECTS, and the aim is already its workspace's.** The
   ruling asks for *"selects it and aims its workspace in the same act"*; because a
   row stands only under the aimed workspace (1), the aim half is true before the
   click. A branch for the other case would be a branch no surface can reach.
5. **So `Model::aim_at` is idempotent, and that is a real change** — the one
   the fold forced. The single cursor crosses the aimed workspace's own row every
   time it moves between the server above it and the conversations beneath it,
   and `aim_at` retired the rows unconditionally: an arrow key would have
   emptied the list under the cursor and put *waiting to hear about this workspace*
   where the rows had been, which is bl-f780's transient defect made reachable
   by a keypress. An aim always puts the cursor on the workspace and drops the
   selection under it; it retires the workspace's answers and the panes about the
   workspace **only when it moved**.
6. **`src/ui/convs.rs` stayed where it is.** §4.39 left the move under
   `roster/` open. The roster is its only caller but not its only reader:
   `crate::ui::queue` spells a row's age and its uncertainty mark with
   `convs::age` and `convs::UNCERTAIN`. A vocabulary two panes share is a
   module beside them, not inside one of them.
7. **A conversation row stands one `STEP` in from its workspace**, which is the
   fold making a level of what used to be a column: `convs::row::indent` starts
   at one step rather than at zero, and every connector, elbow and second line
   moves with it because all four read that one function.
8. **The `+` stands down under a covering pane**, which is the per-workspace
   controls' own rule one row kind over. What it does is put the caret in the
   composer's box; the composer stands down under every covering pane, so a `+`
   offered there would drop the selection and ask for a caret in a box that is
   not on the glass. It also keeps the window from carrying two controls
   reading `+` at once — the other being the fleet pane's cap.
9. **The caret is asked for through `Model::fill`**, which gains a third
   variant (`Fill::Goal`) rather than the model gaining a field. The `+` is a
   navigation exactly as the row menu's *flag…* and *delete…* are — it names
   where to go and asks for the box there — and `composer/start.rs` takes the
   request in the frame that paints the box.
10. **`Model::aimed_at` is deleted.** It asked whether a row was the aimed one,
    for a pane that painted rows about a workspace it was not standing under. A
    workspace's row now builds the `Aim` its own click composes and compares that,
    which is the same question asked with the value the click already needs.
11. **The place file's two width keys collapse to one, `list_width`**, and
    `roster_width`/`convs_width` are read by nothing. A file written by an
    older build is a seat that has dragged nothing, which is §4.13's rung-4
    rule costing exactly one drag.

**The order the four land in.** The drag (edges, the place file's width
keys) and the accordion (the heading, the server rows, the order and the
open state, the place file's server keys) share nothing but the place file
and land in parallel; the composer's second mode shares nothing with either.
The fold — the middle column deleted, the conversations beneath the workspace,
the `+`, the one cursor track, the two-variant policy — lands last, gated on
the first two, so the policy is reshaped once with the drag already in it
and the rows are folded under an accordion that already exists.


### 4.40 The punched wire: an entry that finds its server, and the ladder a dial climbs (REMOTE §13.2–§13.4; yog bl-653a, bl-65ee)

REMOTE §13 puts the server behind a residential NAT with **no publicly
dialable listener at all** (its ruling 2), so an entry's `address` stops being
the whole of how a seat reaches it. Discovery rides the BitTorrent mainline DHT
as a commons, the data path is a TCP simultaneous open both ends arrange, and
the connection that costs seconds to make is held. The server's end landed in
yog (bl-4263); this is the seat's, and the server's `rendezvous/{item,
material, punch}.rs` are the byte-for-byte reference.

**The material grows by two files, optional as a pair** (`src/channel/
rendezvous.rs`; §4.5's three answers, one level down). `rendezvous.pub` is
the server's 32-byte ed25519 rendezvous public key and `pairing.salt` the
32-byte secret the pair shares, both hex, both carried into the entry by hand
exactly as `ca.pem` is — the seat reads and writes nothing (REMOTE §1.4).
Neither present is an entry with no roving: dialled at its address and
nowhere else, which is every entry that existed before this section. One
present is the half-provisioned refusal a missing leaf earns. Everything else
both ends need is HKDF-SHA256 over the salt under `yog rendezvous`, one label
each — `presence salt`, `inbox salt`, `seal key`, and `inbox key`, an ed25519
SEED — so the seat mints no keypair of its own to write the inbox and the
server holds no client key to poll it. Every derivation, the sealed-item
format (`nonce(12) ‖ ChaCha20-Poly1305 ‖ tag(16)` over a count byte, a family
byte, the address and a big-endian port; a call prefixes an 8-byte nonce) and
a signed BEP 44 item are pinned in the suite to bytes the server's own code
produced, so cross-implementation compatibility is asserted rather than
assumed.

**The seat carries its own DHT client** (`src/dht/`; REMOTE §13.7 ruling 2,
*reimplement per component*, on yog's measurement of 859 lines). Bencode,
the KRPC query shapes, BEP 44's signed mutable item and the two verbs over an
iterative walk; a pure client over std UDP — no node, no listener, nothing
for the commons to dial back, which keeps §3's *a seat dials and is never
dialled* true of the rendezvous itself. It links `ring` directly, the
provider rustls already brought, and nothing new. The bare `find_node` walk
the server keeps is not here: the two BEP 44 verbs are all a rendezvous asks.

**The walk is the server's, rule for rule, as rebuilt on live measurement**
(bl-e437; yog bl-f6e1, bl-9408, bl-d00f, bl-d9c1 — the four measurement
tables are yog's `docs/REMOTE.md` §13.7 ruling 3). The copy this section
first landed predated all four and carried every defect they fixed:

- **The bootstrap is a door, never a result.** The mainline's routers answer
  `find_node` and never BEP 44's `get`, so a walk that asked them `get` was
  dark in one round. They are asked `find_node` whatever the verb; every node
  they open onto is asked the verb; nothing they say is in the outcome, so a
  walk whose learned nodes were all silent is the dark-commons `Err`, not an
  empty `Ok`.
- **The walk converges on a frontier, not on the pool:** the K closest nodes
  that replied, are not yet asked, or are still in the air. A node silent
  past its deadline, one that answered only an error (live, a `get` refused
  as an unknown query), and one this socket cannot send to (a v6 address
  from a v4 socket — skipped before a query is spent) all leave it, so dead
  nodes never hold the slots a live one past them needs. A node is one
  `(id, address)`, so a router naming one node eight times names it once.
- **The door is re-asked whenever the frontier runs dry** short of K
  replies, as long as it has named anyone; `max_queries` bounds it, and the
  door's queries hold no window slot. Measured, the one router that answers
  names a single random node per query, so its first seeds are both silent
  about one walk in three and a re-ask draws fresh ones. **A re-ask knocks
  only where the door answered** (yog bl-f519): a bootstrap address silent
  past its first deadline leaves the door for the rest of the walk, as a
  silent node leaves the frontier. From the deployed server box the roster
  resolves to five addresses of which three never answer, and re-asking all
  five spent three of every five door queries on nothing — in the walks that
  ended dark, most of the cap.
- **`put` names which zero it met** (yog bl-f519). A walk that reached no
  token holder sends nothing and fails *no DHT node near T offered a write
  token*; *no DHT node stored the item* is kept for holders that were sent
  the item and none stored it. The two used to read as one.
- **A sliding window, not lockstep rounds:** up to α queries in the air, each
  with its own deadline, the next node asked the moment any answers or times
  out — a silent node costs its own slot and never delays an answer beside
  it. `put` spends every holder's token in one flight at once.

Defaults α 8, K 8, a 1 s deadline per query (`Config::deadline`, formerly a
2 s `round`) and 64 queries; the bootstrap is the server's four mainline
hosts. **The server's vote on BEP 42's `ip` is ported** (bl-53be, from yog
bl-efae): a reply's `ip` — six bytes or eighteen, anything else is no claim —
is one node's word on where it saw the query come from, the walk keeps one
per answering node (the door's included), and `Dht::observed` answers per
family the endpoint named more often than any other, nothing on a tie, and
nothing after a dark walk. A seat publishes no presence, but its call is an
endpoint list too, and on a hotspot or a carrier NAT the observed address is
the only one the server can reach (yog's `docs/REMOTE.md` §13.8). What yog measured after all four: lookup
10/10 at a median of ~6.5 s, `put` 10/10 and `get` 9/10 at ~7.5 s.

**A dial climbs a ladder, cheapest rung first** (`src/channel/ladder.rs`),
and nothing above `Channel::dial` knows which rung answered:

1. **A live held line** — a punched connection kept from an earlier ask.
2. **The direct address** — a LAN, a stable box, loopback. An entry with
   no material stops here, byte for byte as before: the connect is unbounded
   and its refusal is the sentence. With material below it the connect is
   bounded (`Roving::direct`), because a NAT that drops the SYN would
   otherwise cost minutes before the next rung.
3. **A re-call from cached presence** (yog bl-278f) — the server's
   endpoints at its punch port, as the last call that LANDED found them:
   write a fresh call naming this box (its route-local addresses and the
   addresses the last walk observed, since a re-call writes before it walks)
   and punch. One DHT walk, not two. **It replaced a client-only re-punch at
   the same endpoints, which cannot land against a server behind a NAT**:
   measured 2026-09-29, the server's home NAT holds no mapping once its
   served stream ends and the server sends no SYN outside a call's window,
   so a re-punch's SYNs reach nothing. The held line's pings are the only
   thing that keeps a mapping alive, so a dropped line is always a call. A
   call that expires unanswered — re-call or full — clears the cache, so the
   climb falls to presence and the next climb reads it again.
4. **The full rendezvous** — `get` presence under `(server key, presence
   salt)`, open it, `put` a sealed call under the derived inbox keypair
   naming this box's route-local addresses, then every address the `get`
   walk observed that is not one of them — all at its punch port, never the
   observed port, which is the DHT socket's UDP mapping (yog's
   `docs/REMOTE.md` §13.2) — with a fresh nonce and a rising sequence, then punch. **The DHT is touched by rungs 3
   and 4 and nowhere else**: a seat pays nothing for the machinery when idle, and only
   the server polls.

Whichever rung produced the stream, `Channel::line` runs the same inner mTLS
over it against the same `ca.pem`, verifying the same server name off the
same `address` (§4.4) — the ladder decides how a socket is obtained and
nothing about what is trusted on it. **The address therefore stays required**
even where the material would let the seat find the server without one: it is
the one home of the name the certificate is verified against (REMOTE §8), and
an entry naming none is the deferred question this section leaves open.

**The punch** (`src/channel/rendezvous/punch.rs`) binds one port twice over —
a listener per family — and connects toward every server endpoint from a
third socket bound to the same port, v6 first, one SYN per two seconds
inside the window; `socket2` sets `SO_REUSEADDR`/`SO_REUSEPORT`, the one
approved new dependency (operator ruling 2026-09-23, on the server's terms).
The seat keeps the **first** stream that lands and the server serves every
one, which dissolves any negotiation over which to keep. That listener is the
one a seat has, and it is not a contradiction of §3: it accepts only during
a punch this end started toward a server it just called, and what it
accepts is spoken TO as a client over the same mTLS, never served.

**A punched line is held between asks, and a direct one is not**
(`src/channel/line.rs`; REMOTE §13.4 meets §10's criterion in its own terms).
The server pings the silence with `{"ping":true}` every 25 seconds and hangs
up after two minutes of silence in all; the seat discards the ping wherever a
frame is read — it is never the start of a reply stream — and judges a held
line gone **one ping before the bound**, the arithmetic the server's own
`pings_at` runs, so a line is never reused at the moment the far end is
closing it. Time is the injected `Clock` (`src/channel/clock.rs`), so the
suite crosses the bound by hand. Before a held line is spoken on it is asked
whether it is still open: whatever the socket holds is drawn into rustls
without consuming a byte of plaintext — a FIN, a `close_notify` that rode in
behind the last answer, a reset, bytes that are not TLS, or nothing but
pings still to read — and a dropped line re-enters the ladder at the next
ask instead of turning it into an act IN DOUBT. A held line owes no preface;
the request alone goes down it. A follower that stops early drops its line,
which was always the word to the server (§4.2), and a line with an answer
still on it is never reused.

**What worked stays RAM for the run and never disk** (`state::worked`; the
runtime half of REMOTE §8's `:0` discipline). The held lines, the cached
presence a landed call answered to and where the commons last saw this end, the punch port bound once so the server's peer sees one
port, and the inbox sequence. Keyed by the entry's directory, because entries
share nothing (§4.6). It is the **second tenant of `src/state.rs`** and it
belongs there on the rule's own terms: a channel is opened per gesture and
the four off-frame threads each open their own (§4.12), so the fact that a
connection exists outlives every channel and is shared across every thread —
exactly the cross-thread state the chokepoint exists to inventory. §4.12's
*there should not be a second* was written about the frame and the workers,
and this is not a third party to that conversation; it is the socket half of
it.

**Every rung is said, on stderr, as families and counts** (`src/channel/say.rs`;
the seat's half of yog's REMOTE §13.4 "The operator's view of the loop",
bl-355c on the server and bl-3e5b here). Before it, a live dial could be
watched only from socket state on the server. On an entry with rendezvous
material each climb says what it saw, in the server's shape under this
binary's name:

    lernie: rendezvous: held line taken up — no dial
    lernie: rendezvous: held line dropped — past the silence bound | closed at the far end
    lernie: rendezvous: direct rung answered | refused — <io error class> | timed out after <d>
    lernie: rendezvous: no bootstrap node resolved — no rendezvous
    lernie: rendezvous: presence not read — the DHT walk failed (reason withheld: it names nodes)
    lernie: rendezvous: no presence is published under this pairing
    lernie: rendezvous: presence seq <s> did not open under this pairing's seal key
    lernie: rendezvous: presence read — seq <s>, <k> endpoint(s) (<families>)
    lernie: rendezvous: every address the call carries (<families>) is in an overlay or carrier-NAT range — a server outside that network cannot reach it
    lernie: rendezvous: re-call from cached presence — nonce <n>, seq <s>, <k> endpoint(s) (<families>), <a> ack(s)
    lernie: rendezvous: call nonce <n> written — seq <s>, <k> endpoint(s) (<families>), <a> ack(s)
    lernie: rendezvous: call nonce <n> not written — the DHT walk failed (reason withheld: it names nodes)
    lernie: rendezvous: punch for nonce <n> started — <k> endpoint(s) (<families>), window <d>
    lernie: rendezvous: punch for nonce <n> landed (<family>) | expired after <d> with no stream
    lernie: rendezvous: punched line held between asks
    lernie: rendezvous: held line ping discarded

A line never carries an address, a key, a salt or a sealed byte — the direct
rung's refusal is said by its error CLASS because the message names the
address, and a DHT failure without its reason because the walk's refusals
name nodes; the sentence the dial hands its caller is unchanged. **A repeated
outcome is said once**: a climb's lines are said together, and only when they
differ from the last climb's for the same entry (the memory is `Worked`'s, so
every thread dialling that entry shares it) — a window asking every beat over
a held line says `held line taken up` once, and a rendezvous, carrying a fresh
nonce, is always said. Pings discarded while an answer was read are a second
transcript, judged the same way. An entry with no rendezvous material says
nothing: its refusal is already the whole sentence. The overlay line (RFC
6598's shared space, 100.64/10, or IPv6 unique-local fc00::/7, for every address the
call carries) is said before the window is spent and refuses nothing — whether
such a rendezvous should be spent at all is yog bl-f612's to decide. The sink
is a knob of `Roving`, stderr by default, so the suite reads every line off a
captured one.

**What this leaves for the server, and for a later ball.** The server's
`wire-certs` does not yet lay `rendezvous.pub` beside a client leaf — the
public key of its `rendezvous.key` — so an operator derives it by hand until
it does (named to yog on bl-653a). The call carries the observed ADDRESS
beside the route-local ones and trusts port preservation; a carrier that
rewrites the port (yog's `docs/REMOTE.md` §13.8) is the case it does not
reach, and stays open. The enrollment envelope
(§4.15) carries no rendezvous material until the server's reply does. The
cadence defaults — a five-second direct bound, a forty-second punch window
covering the server's fifteen-second poll and its own twenty-second window —
are stated so they can be wrong in public (§13.7 ruling 3).


### 4.41 The prices pane and the ledger column: what the world is charged (yog REMOTE §9.23; bl-9111)

`src/reply/prices.rs`, `src/reply/spend.rs`, `src/verbs/prices.rs`,
`src/verbs/doors.rs`, `src/cli/prices.rs`, `src/seat/aimed.rs`, `src/ui/prices.rs`,
`src/ui/model/pricing.rs`, `src/ui/roster/workspace.rs`. **A token count is never
painted without the cost the server put beside it** (yog DESIGN §3.5), and this
section is that rule's seat half: one sentence for every count, a column on the
roster, and one window-level pane over the two money facts.

**One sentence, said everywhere a count is.** `crate::reply::spend::priced` is
`N tokens — $x.yz` where the server priced the count, the count alone where it
has no price table — an absent `cost` is UNPRICED, never zero, and a `$0.00` is
a priced subscription — and `at least $x.yz` where `unpriced_tokens` is above
zero, because then the figure is a floor. The step row (§4.18), the notch
(§4.29), a candidate's attempt (§4.36), a ball's figure (§4.31) and the
conversation's own spend all say it, on the glass and on the command line. The
money is upstream's rendering, verbatim; nothing here multiplies.

**The roster is a ledger.** `reply/workspaces` rows carry `spend`, the workspace's
whole priced figure, and the row's line paints it after the state rollups and
before the weak facts — money is the second thing an operator scans a roster
for, and the line is elided from the right (§4.39). `lernie workspaces` prints
it as `spent $x`. A workspace the server has no table for says nothing, which is the
fact.

**The pane is §4.21's shape, one noun over.** `prices` names no workspace, so
its subject is every channel this box holds: its control hangs off the roster's
strip, it opens from an unaimed seat, it is sectioned per channel under the
roster's header, and it shares `Lookup` with the window-level family. Each
section paints the table — one row per `(provider, model)`, a `*` model as
itself, the four rates as the operator wrote them — then the bound and the
world's spend against it, and what the last ceiling act woke.

Four decisions are its own:

- **The read is POSTED, not standing.** The table moves only when an operator
  writes it, and both writes answer with it re-derived, so the section an act
  fired from is replaced by its own receipt. The spend line is the one fact
  that drifts under the operator; opening the pane re-asks it. A standing read
  would spend a round trip a beat on an answer that almost never moved.
- **Each act goes down ONE channel.** Neither `price` nor `ceiling` names a
  workspace, so the poster would fan either over every server this box is a
  client of — §4.30's hazard. Each control fires from its section and is
  addressed down that channel (`Posted::down`). On argv the two are doors
  (`lernie price <provider> <model> <input> <output> [<cache_read>
  [<cache_write>]] | off [--on <channel>]`, `lernie ceiling <usd> | off [--on
  <channel>]`) and each goes down ONE channel too (bl-1bb9; `seat::aimed`):
  the one `--on` names, opened by `seat::dial` — the same name-to-channel step
  the window's control takes, since the act names no workspace and §4.7's
  mapping has nothing to resolve — or, unnamed, the one channel the box holds,
  counted off the fan's own enumeration. A box holding more refuses an unnamed
  write, naming its channels; a name it holds no channel by refuses naming
  them too, and never falls through to the flat root. They used to fan on
  `lernie ack`'s terms, which wrote one typed number into every world the box
  is a client of; an idempotent acknowledgement means every server, a price
  means one. `lernie prices`, the read, still fans. They are doors and not rows
  because their arguments are numbers that may be absent, which the gesture
  table's rule refuses (§4.10); `prices` takes nothing and is a row.
- **A rate is a JSON number, never an `f64`.** The server reads rates back
  exactly as written, so `15` must go out as `15` and not `15.0` — the corpus
  round-trip holds it. A word that is not a number is refused at the seat; a
  negative number is a number, and the server's in-band refusal names it.
- **The provider is checked against the roster the seat already holds.** Where
  the window is aimed at a workspace on the section's channel and that workspace has
  answered `providers` (opening the pane asks it, which is why the opening
  control carries `act:prices act:providers`), a provider it does not name
  disables the price control with the reason beside it. Anywhere else — no aim,
  another channel, `lernie price` — the server's own refusal names it.

**`ceiling` is not armed, and §4.20 is why.** A bound set under the world's
spend PARKS every conversation over it — conversations no control on the pane
names, which is the amended test's first half (*beyond the one named on
screen*). The second half is what an arming protects against: an act whose
undoing is per-object. The cascade stop is armed because its undoing is a
`nudge` per conversation. A ceiling's undoing is ONE act — raising the bound or
lifting it drives every conversation it parked, and the receipt's `released`
says how many woke — and it sits on the same pane, beside a spend line that
says what the bound is about to be measured against. A typed-name arming here
would charge a retype for an act whose whole mistake is reversed by the next
click. So the controls carry the enablement rule (disabled until the box holds
a number or `off`, the reason beside it) and nothing more. `price … off` is not
armed for the plainer reason that it names its row.

The tags are `act:prices` on the opening control, `act:price` on each
section's write and on every row's delete, `act:ceiling` on the set and the
lift; the three `parity.toml` lines are gone.

### 4.42 The Hermes shape: one column, a rule, a prompt, and everything else an overlay (bl-7859)

Operator ruling 2026-10-12, in the operator's own terms: the window is
*borderline unusable*, iterating toward a functional one has not worked, so
*skip some steps and ape another* — the Hermes Agent terminal UI (Nous
Research, `ui-tui/`), which is functional, is the baseline; clone its
interface design more or less, and adapt what is unique about this seat onto
it. This section is that adaptation. **Where it contradicts §4.11, §4.38,
§4.39 and `docs/STYLE.md` §2, this section wins**, and each of those carries a
line at its head saying so; their paragraphs stand as the record of what was
tried and why, and the model-side rulings in them (the frame never dials, the
verb table, the `act:` tag, the snapshot and parity instruments) are untouched.

**What is aped, named part by part.** The baseline was driven on this box
against a local model and read in its own source; the parts below are its
parts, in its vertical order, each with what it becomes here.

1. *Amended by §4.44: a left bar stands beside the column, and it is the one side column there is.* **One column.** The transcript fills the window; the composer stands at
   the bottom; a one-line **status rule** stands between them. There is no
   side column, no column bar, no narrow and broad shape and no dragged edge:
   §4.11's two shapes, §4.39's two columns and both drags, `shell/policy.rs`,
   `shell/policy/edges.rs`, `shell/drag.rs`, `Column`, the headings and the
   place file's two layout keys all go. A width buys wrapping and nothing
   else, and the phone-shaped viewport is the same window narrower — the
   snapshot matrix keeps its three sizes and the policy question it used to
   ask is gone rather than always answered yes.

2. *Superseded by item 11 for the segments; the notice's slot, the long
   notice's place above the rule and the rule standing down under an overlay
   stand as written; §4.45 item 3 (bl-4792) moves the rule under the box,
   at the window's bottom edge.* **The status rule** (`src/ui/status.rs`) is one line, left to right:
   `─ <state> · <elapsed> │ <model> │ <spend> │ <N waiting> │ <N servers> ─
   <name>`. The state is the wire's own word for the selected conversation
   in its state's ink (`theme::state_of`), `ready` in weak ink at rest, and
   **the notice stands in the state slot** in its own ink while there is one
   — §4.21's notice bar is this slot, and the dismiss is Esc. **A notice
   is never cut and never costs a fact** (bl-7e3e; bl-3d0f's ruling, that
   the half a cut sentence loses is the remedy): it holds the slot only
   while it fits beside every fact and the name, and otherwise stands whole
   on its own lines directly above the rule, wrapped, in its ink, until Esc
   — the rule keeping its one line, its slot saying what it says with no
   notice (`status::calm`). The segments
   are the chat header's facts (§4.38) moved onto the rule: the model that
   answered, what the conversation has spent (§4.41's words, never `$0`), how
   many conversations across every server are asking (in the attention
   accent while it is non-zero — the one green on a resting window), and how
   many servers this seat reaches. **Reaches, not holds** (bl-4cac): the
   segment reads `N servers` while every channel has answered and `R of N
   servers` while any has not — in the error accent where one will not dial,
   weak while one is merely unheard — and the waiting count leaves out a
   channel that will not dial, because a green read off workspaces nobody can
   reach is a count nobody can act on. The right end is the selected
   conversation's name, the aimed workspace's where none is selected, and the
   word `servers` where nothing is aimed. Segments drop from the right as the
   width runs out, the name last. While `/plan` is on, `plan` in the brand
   stands first among the facts (bl-6f3f): the box's hint that also says so is
   gone at the first character typed. The rule replaces the conversation heading,
   the subject block and the notice bar; nothing above the transcript names
   the pane, because there is one pane. **It stands down under an overlay**
   with the prompt (bl-eb29): every pane is built to fit the narrowest shape
   the layout promises (§4.32), the rule's line is what the tallest of them
   needs there, and the overlay's own name line says where you are meanwhile.
   **The model and the name are doors** (item 10).

3. **The composer is a prompt line** (`src/ui/composer.rs`). **Amended by
   §4.45 item 3 (bl-4792): the box stands directly under the transcript,
   and the rule under the box.** The glyph `›`
   in the brand (the baseline's `❯` has no glyph in the body font, and a
   tofu box is a control with no word on it — `theme::tests`), the draft after it, growing a line per line typed, with the
   hint in weak ink while it is empty. There is no `send` control: **the field
   is the control**, it wears the `message` tag, Enter fires it and
   Shift+Enter breaks a line (§4.38's one read, `composer::entered`, kept).
   The start is the same line with the same glyph and a hint naming the workspace
   a conversation would begin on — **or, where the workspace's worker role
   resolves a provider row that holds no credential, `no provider yet —
   /login or /config, then pick one on the rule`** (bl-7a77,
   `start::unsigned`): the goal's question is the wrong one to ask first on
   a workspace whose start the server will refuse. *Holds no credential* is read
   off the providers read's own words — the row's fact ends `not signed in`
   or `no credential stored` (`ProviderRow::bare`), because the wire carries
   that fact as words and no flag; a fact this build cannot read claims
   nothing; `✎ plan` becomes `/plan`, a toggle shown on
   the rule while it is on. **With nothing aimed it is the same line again**
   (bl-2cb0), never a sentence where a field was: the field is where every
   slash verb is typed, and the window's own — `/attention`, `/help`,
   `/doctor`, `/prices`, `/board`, `/trail`, `/find`, `/refresh` — need no
   aim, so the seat that has aimed at nothing (§4.19's seat *most likely to be
   asking the question*) can still ask it. The hint says nothing is aimed and
   names `/` and Ctrl+X; prose has nowhere to go, so Enter on it deposits
   nothing, wears no tag and leaves the draft standing. The workspace's and the
   conversation's verbs are not on its menu. The row of offers under the field goes: `stop`,
   `interrupt <words>` — the same cut with something to say instead —
   `nudge`, `records…`, `…` and everything that strip opened are slash verbs
   (5), listed only while the server offers them. **The baseline's Ctrl+C
   is not bound** (bl-3fd6): in egui's box the chord is *copy*, and a chord
   that copied with a selection and killed a driver without one would be
   two acts under one key, which `crate::ui::keys` refuses; the verbs are
   typed, and the hint over the box says so. The draft's glow while the
   conversation is asking stays, and the ask itself stands above the field
   (6).

4. **The transcript idiom** (`src/ui/chat.rs`; bl-a40c). **Amended by
   §4.45 items 4, 5 and 7 (bl-4792): every block stands on a bubble by
   role, a trail folds itself once its turn settled, and a message this
   seat sent stands at the end in the pending ink until the server's row
   replaces it.** An operator's turn
   is a row with the prompt's glyph in the gutter in the brand and its words
   in body ink; the model's prose carries `¦` in the gutter in weak ink (the
   baseline's `┊` has no glyph in the body font, and its stems `├─ └─` are
   drawn as the roster draws its threading rather than typed); a tool call and its result are
   one row of a **trail tree** under a fold (met by the tool-use id both
   halves carry, which never reaches the glass, and counted by calls —
   bl-eb4e) — `▾ tool calls (3)` with `├─ ●
   Bash("cargo test") ✓` a call, `└─ ✗` a failed one in the error accent, the
   result folded to its head and unfolded in place — and `└─ response` parts
   the trail from the prose that follows it; turns are parted by `───` in the
   hairline; blank lines fall only where the kind of block changes. **A wound
   ends the transcript in the error accent** (bl-d34f): the server's virtual
   `wounded` entry (PROTOCOL 9 — the class `refused`, `no_response` or
   `output_limit`, the adapter's words, the provider row a refusal routed
   to) is decoded rather than left to the unknown-entry fallback, and
   `reply::transcript::wounded` projects it into one sentence naming the verb
   that mends it — `/login` or `/roles` for a refusal, `/nudge` for a step
   that produced nothing — which the command line's rendering prints too. It
   wore *«wound» (wounded, which this seat cannot read)* before, which read a
   provider refusal as a seat bug. A fenced code block in what was said reads
   as code (bl-beef): monospace on the surface, its fences unpainted, its
   lines never wrapped but scrolled sideways inside the column; an unclosed
   fence (a turn still streaming) runs to the end. **The prose between
   fences reads two inline shapes and no others** (bl-f5a7,
   `chat::code::inline`): a `` `code` `` span on one line paints in the
   monospace family at `MONO` on the surface without its ticks, and a line
   opening with one to six `#` and a space paints without its marks at the
   `HEADING` size — STYLE §1's *a heading is a size, not a hue*, there
   being no bold face in the language to set one in. An unpaired tick and a
   `#` with no space after it stay as typed. Both painted raw before. The
   ruled block of §4.38 goes with its side rules: who spoke is the gutter
   glyph, and weight is still the speaker's (`theme::speaker`). At the head of
   a transcript stands the **conversation panel**: a framed card carrying the
   conversation's facts — name, server and workspace, model, spend, where it hangs
   (§4.29's spine) — as one line each and the long ones behind `▸` folds; a
   window with nothing selected paints the **seat panel** there instead: the
   servers this seat reaches, one row each with its dial state, workspaces and
   counts — and **the dial state is read, not assumed** (bl-4cac): a channel
   that will not dial says the wire's own sentence (`Held::Unheld`, the one
   `lernie entries` prints — on a first run, *nothing provisioned*) whole
   and wrapped in the error accent in place of its counts, as does a server
   that refused the roster (`Held::Refused`, bl-693d), and one not heard
   from yet says *waiting to hear from this server* in weak ink; the
   version, and the ways in — `/help for commands · Ctrl+X for a
   conversation · Ctrl+O for the model · /login for a provider` (item 10).

5. **The way in is a slash verb, and the slash verbs ARE §4.10's words**
   (`src/ui/slash.rs`). `/` at the head of the draft opens the **completion
   menu** — a framed list of verb and summary, read off `crate::verbs::table`
   the way `lernie help` reads it, narrowed as the operator types, ↑↓ moving,
   Tab or Enter taking the row. A row is a control and wears its verb's tag,
   so the parity walk reads the menu where it read the strips; the menu's
   open state is a world (`snapshot::worlds`). The two strips of §4.38 — the
   window's and the aimed workspace's — and the conversation's `…` strip go: every
   pane they opened is `/word` — **and the word is the word the control
   carried, minus its ellipsis** (bl-e016): a pane's noun opens the pane and
   a verb's word fires the verb, so a hand that reached for `records…` types
   `/records` and nothing is renamed. The nouns: `/records`, `/attention`,
   `/help`, `/doctor`, `/trail`, `/board`, `/prices`, `/find`, `/enroll`,
   `/roles` (and `/model`, the same pane by the word an operator reaches for
   — item 10), `/login`, `/clients`, `/config`, `/fleet`, `/unmake` — and **`/help` answers with what this window takes
   first**, the menu's own rows in its order and the keys, then the servers'
   op tables headed as each server's own spelling, because most of those
   words are not the box's (bl-55ed). The verbs:
   `/refresh`, `/pin` or `/unpin` by the workspace's rank, `/interrupt`, `/stop`,
   `/nudge`, `/revoke`, `/restore`, `/retarget`, `/flag`, `/delete`, and
   `/plan` in the start's mode. Only what the server offers on the selected
   conversation is listed, which is §4.38's *the row offers what the server
   offers* with the row deleted; the summary beside each wire verb is the
   verb table's own. A verb that takes words takes them on the same line —
   `/flag <reason>`, `/interrupt <words>`, `/delete <name>` is the arming,
   `/find <needle>` spends it; one with its parameter missing fires nothing,
   as the disabled control it replaces fired nothing, **and the menu says
   why** under its keys until the draft moves (bl-df70). **`/delete` is
   armed by the conversation's own name and by nothing else**: the wire's
   typed name arms only the subtree and deletes a leaf on any word, an empty
   one included, so the seat composes no envelope until the word is the name
   the server compares, and the refusal states that name (§4.20: an
   unmaking states its subject before it offers the act). **The bar's
   trashcan is the second arming** (bl-e837; §4.44 item 1): a row under the
   pointer offers it, and the seat types the row's own name into the one
   envelope `/delete` sends (`Model::post_delete`), behind a confirmation
   unless Ctrl is held. The menu lays a window
   of rows around the chosen one and never scrolls, which is the baseline's
   shape and the one the matrix admits — every control laid is on the glass
   whole at every size.

6. **An overlay, not a covering pane** (`src/ui/overlay.rs`). Every pane that
   covered the conversation — records, trail, board, fleet, clients, config,
   tuning, prices, login, enrollment, doctor, find, verbs, unmaking, clearing
   and the queue — keeps its content and loses its frame: it stands in a
   **framed box over the transcript**, as tall as the pane less a margin at
   the top (every pane is built to fit the narrowest shape the layout
   promises, §4.32, so a box that took a share of it would be a pane that
   fits nowhere), with its name on the first line and *Esc closes* at that
   line's other end in weak ink; Esc closes it and one stands at a time. The
   enrollment keeps §4.15's whole-window cover, because what it holds must
   not be legible beside anything. Each pane's `done` control goes with the
   frame — the reach walk types the word and counts the key (bl-e016). It is
   a child region of the pane rather than an `egui::Area`, because a fresh
   area paints nothing on its first frame and every single-frame assertion
   in this crate would read a blank where the box stands. **A pane longer
   than the box scrolls in it, and §4.32's density constraint is read
   accordingly** (bl-eb29): a control scrolled past the box's clip is
   reachable by scrolling and is not a layout fault; one laid past the
   window with nothing to scroll still is. The box writes where it clips
   to the toolkit's memory and the matrix reads it there
   (`overlay::BOX`, `snapshot::clipped::scrolled`). The constraint was never
   met at the phone's width by the records pane in truth: the window it
   replaced let a heading row overflow the column sideways, so the pane was
   judged at a width it did not have.

7. *Amended by §4.44: the bar is the pointer's door to one workspace's rows; this stays the keyboard's and the whole seat's.* **The roster is the switcher** (`src/ui/switcher.rs`, **Ctrl+X** or
   `/conversations`): an overlay listing every server as a heading row, each
   workspace under it, and each conversation under its workspace as one row — state
   glyph in its state's ink (`▶` working, `?` asking in the attention
   accent, `✔` at rest — the body font has no `✓`, and `theme::tests`
   refuses a tofu box — `■` stopped, a trailing `?` where nothing observed
   the state), the name, its age and what waits under it, every count worded by the one
   helper `render::counted` (`1 conversation`, bl-967f) — with `+ new`
   under every workspace, which is §4.39's `+` moved off the server's row onto
   the workspace it begins on (bl-d357: a bare `+` at the far edge of another
   row was the begin a first-time operator did not find). The decision
   queue's rows lead with the same glyph from the same spelling
   (`convs::glyph`, bl-6e59) — always the asking `?` there — where they
   said a bracketed state word. **The model is
   not on the row**: the wire's conversation row carries none, and the
   switcher does not invent one; the rule says it for the selected
   conversation. Enter selects, which is §4.11's
   one door for a click and a keypress, and closes the overlay; `+ new` puts
   the caret in the field in start mode. **`+ new workspace` stands last
   under this box's own server** (bl-673e): a fresh server holds no workspace
   this client is registered in, so there is no `+ new` to pick, and the
   server's own remedy is *a workspace this client founds itself registers
   it too* (REMOTE §4). It puts the caret in the field in a founding mode
   whose hint asks for `<name> <what it is for>` on one line; Enter stages
   the ordinary start on the named workspace, whose `prepare` founds the
   workspace on the host (REMOTE §4.1's raise), and **the workspace is aimed when
   that `prepare` is answered** (`Model::founded`, bl-693d) — aimed at
   Enter, its standing reads went out ahead of the act that registers the
   client and were refused into the bar — one line rather than a
   two-step, because a name held between two Enters would be a second state
   to put down. Esc, an aim or a selection puts the mode down. It stands
   under no entry's server: a name no entry holds routes to the flat root
   (REMOTE §8.2), so it would found the workspace on a different server than the
   one it was offered under. The seat's card names it among the ways in, and
   the field with nothing aimed says *Ctrl+X for a conversation or a new
   workspace*. **A selection the server no longer knows is put
   down** (bl-6dfd): the selected conversation's own row is read every beat,
   and a refusal of that `agent` read — deleted from another seat, or a row
   aimed at the wrong server — drops the selection back to the workspace (the
   start's hint, the seat's card) and stands in the bar once; held, it
   re-posted every beat faster than Esc put it down. Which op a read about
   the selection answered rides with it (`state::Said::About`), because a
   refusal cannot say so itself. **Opening it puts down whatever overlay stood** (bl-339d): one stands at a time, and a switcher stacked over a pane handed the operator back to the pane after they chose. The accordion's order, open server
   and ranks (§4.39, `model/servers.rs`) are what the overlay paints, so the
   place file keeps those keys and loses the two layout ones.

8. **The ask stands above the field** (`src/ui/ask.rs`; bl-dc3d): while the
   selected conversation is asking, a framed box in the attention accent
   stands between the transcript and the rule carrying the queue row's own
   words — the parked call, the flag, why it asks (the server's own `says`
   sentence, carrying its remedy, and the raw signal words only where the
   server wrote none; bl-5507), the failure, the last words — and its choices as numbered rows: `1. seen`, and where a call is
   parked `2. pass · 3. pass for this conversation · 4. pass for this
   workspace · 5. refuse · 6. hold` — the three scopes being the baseline's
   *once*, *this session* and *always* in this wire's own words, §4.19's
   acts and no others; `go to it` is not among them because the operator is
   there. **The `pass` rows are the reaches the held row lists** (`scopes`,
   yog bl-0da6; bl-da16): a secret or destructive call lists `call` alone
   and is offered `2. pass · 3. refuse · 4. hold`, since the server refuses
   every wider answer to it and the seat never re-derives the class from the
   reason's prose; a row that lists none is an older server and offers all
   three; and the queue pane paints this same list (`ask::choices`), so the
   two surfaces cannot drift. **The prompt keeps the caret while it asks** (bl-1252; STYLE §1 —
   a focus that is not where the words go is worse than none): a digit
   picks, ↑↓ walk the choices and Enter answers the chosen row **only while
   the draft is empty**, each taken out of the frame before the box reads
   it (`ask::keys`); with words in the draft every key is the box's and
   Enter sends them, or fires the slash line — typed words never go
   nowhere. The field's hint while it asks says both: *it is asking — 1–6
   answers, or say it — Enter sends* (`ask::hint`; `1` where only `seen`
   is offered, no digits before the queue's row lands). Round 2 found the
   caret held out of the box so a digit could answer, which sent typed
   words nowhere and spent Enter on the question. A click on a row is the
   same answer. **While the aimed server will not dial nothing is answered**
   (bl-8d6d): the choices go faint and take no click, digit or Enter, the
   box's key line goes, and the field's hint is the server's own sentence
   (`diagnostics::down`, bl-4cac's reading) — a box saying *Enter sends*
   over an answer that cannot land was a promise the seat could not keep. It is §4.19's
   queue entry for the one conversation on the glass, painted where the
   answer is typed; the queue itself (`/attention`) is the same rows for
   every conversation, and the count on the rule is its door. **The queue's
   read stands while the selected conversation asks** (`Standing::asking`),
   **on a lane of its own** (`offframe::queue`, bl-5439: a server can take
   half a minute to answer `attention`, and the serial pass that asked it
   starved every other pane and the transcript for as long),
   so the box is answered on the frame the row lands — the roster's own
   word says it asks before the queue has said what. **And it goes down on
   the frame the answer lands** (bl-6802): `seen` answers `acknowledged`,
   the item acted on plus the whole queue that remains on that server, and
   it is filed as that channel's queue answer (`Model::asking`) with a
   one-line receipt on the bar saying what was seen — by the name the panel,
   the rule and the switcher paint, read off its queue row or its workspace's row
   rather than the id the receipt carries (bl-c887) — and whether it is still
   asking — read off the remaining rows, never predicted.

9. *Amended by §4.46 item 4: whether it stands unfolded when the window opens is the `dock` knob.* **The live dock** (`src/ui/dock.rs`; bl-e661) stands above the rule —
   **which §4.45 item 3 (bl-4792) puts under the box, so the dock stands
   between the rule and the box** —
   while anything runs under the aimed workspace — its conversations whose
   drivers are live or in flight, by the server's own word, read off the
   rows the roster already carries so the dock asks nothing of its own —
   one folded line, `+ 2 running · <the first's name> · Ctrl+T`, unfolded
   with **Ctrl+T** or a click into one row apiece, name, state and age, a
   click on which selects; whether it is unfolded is the toolkit's memory,
   as a fold's is. The fleet's runs (§4.33) join it when their read
   stands outside the fleet pane; until then they are the pane's.

10. *Superseded by item 11 for the segments: the model and the waiting
    count are no longer doors here, and `/model` and Ctrl+O open the model's
    dropup rather than the role tuning; the name's door stands as written.*
    **What you talk to is changed from the glass, not only typed**
    (bl-4e55). On the first run of this window the operator could not find
    how to change the workspace, the provider or the model: the ways existed
    — Ctrl+X, `/login`, `/roles` — and nothing on the glass named them. So
    three runs of the rule are **doors**: the name at the right end opens the
    switcher, the waiting count opens the queue (item 8's *the count on the
    rule is its door*, made true by bl-2cb0 — and on a window aimed at
    nothing, because `/attention` needs no aim), and the model segment opens
    the role tuning — where a workspace's
    model per role is read and set — and a workspace whose transcript has named
    no model says the worker role's model there where the workspace's roles are
    in hand, and `model…` in weak ink where they are not (bl-0966: `no
    model` read as a misconfiguration the workspace did not have), so the door
    stands before anything has answered. Each door **is a completion-menu row
    painted elsewhere** (`status::door`): it is a door exactly while the
    menu offers its row, it wears that row's `act:` tags, and a click fires
    that row's word — `/conversations`, `/attention`, `/model` — so the rule opens nothing
    the menu would not. A door is a run, not a button (`theme::paint::link`):
    bare at rest, the hover wash under the pointer (§4.44 item 5), the pointing hand over it —
    the rule stays one line of facts. `/model` is **an alias row**, not a
    focus: it opens the same pane `/roles` does, because the pane opens on a
    standing read and a focus on a row that has not answered yet would be a
    focus on nothing; the word exists so *change the model* is one word.
    **Ctrl+O fires it** — the baseline's model picker — and is admitted by
    the keyboard's rule because a click on the rule's model does the same
    and nothing else. The provider is not a door on the rule: the rule
    carries no provider fact, and a sign-in is the workspace's (`/login`), so it
    is named where the ways in are named instead — the seat's card and the
    prompt's hint, which drops its trailing ways one by one where the line
    has no room for them, as the rule drops its segments.

11. *Amended by §4.44: the workspace selector leaves the rule for the bar's foot (operator ruling 2026-10-13: a chat does not change workspace); the line is `— provider ⏶ | model ⏶ | effort ⏶ | priority ⏶ | diagnostics — name`.* **The rule is the conversation's line** (bl-d74b). Operator ruling
    2026-10-12, verbatim: *"The selectors and interfaces are all full-ish
    screen popovers. They should just be dropdowns (or dropups, since
    they're usually near the bottom). Click it; it shows a cached list, and
    triggers a reload. A little yellow reload icon until it happens, which
    turns into a green checkbox. Leftmost should show the workspace, then
    provider, then model, then context, then (if the API allows) priority.
    After that, all the other diagnostics: tokens out and in, context window
    usage, etc. The only thing along the bottom next to the chat bar should
    be stuff about that conversation, so things like the 12 waiting
    shouldn't be there."*

    **The line, left to right**: `— workspace ⏶ | provider ⏶ | model ⏶ |
    effort ⏶ | priority ⏶ | <state> · <elapsed> | <in> in · <out> out |
    cache <r> read · <w> write | <cost> | context <N>% — <name>`. The five
    are **selectors** (`status/selectors.rs`); the rest are this
    conversation's **diagnostics** in weak ink (`status/diagnostics.rs`) —
    the state slot first, in its state's ink, the notice in it while there
    is one (item 2's rule, bl-7e3e's long notice above the rule, kept); the
    tokens, the cache where either half is non-zero, the money in §4.41's
    words and nothing where no rate priced it, and the context's own
    percent, all off the selected conversation's `agent` row. The name at
    the right end stays the switcher's door (item 10). **`N waiting` and
    `N servers` leave the rule**: they are seat-wide. The waiting count —
    still leaving out a channel that will not dial (bl-4cac) — is the
    queue's door on the seat panel and at the switcher's head
    (`status/waiting.rs`), beside `/attention`; the servers' dial state is
    the seat panel's server rows (bl-4cac's sentence in the error accent).
    **The one dial fact that stays is this conversation's**: while the
    aimed server will not dial, the state slot says its sentence in the
    error accent (`diagnostics::down`), under any notice.
    **A notice that names a menu word carries its door** (bl-7a77): each
    `/word` in its sentence that is a row the menu offers right now stands
    after the slot as a run in the brand firing that row, once each
    (`diagnostics::named`) — so the server's *sign in first … write its key
    into this workspace with /config brazen* is a click on `/config` as well as
    a sentence. The server words the remedy and the menu decides what is a
    door; the seat names no verb of its own. A refused start's sentence,
    which stands over the box rather than in the slot (`composer::start`),
    carries the same doors after it.

    **The frame around the server's sentence, and the seat's own door**
    (bl-8a4b; operator 2026-10-13: *a ton of error message … is opaque to a
    user that doesn't deeply understand the internals*, and a refusal naming
    *the Config editors* with no way named to reach them). `the server
    refused:` is **kept**: it says the act did not happen and whose words
    follow, so the nouns in them read as the server's and not as this
    window's — the seat never rewrites them, and a server sentence that is
    opaque is filed against the server. Where the seat **knows** the door
    because it knows the op it sent, it appends its own sentence after an
    em dash: a refused `model`, `effort` or `priority` ends *— /config opens
    the settings editors* (`Notice::refused`), which `named` then makes a
    run like any other. The signal is the op the receipt answers, never the
    server's prose. A dial that will not open leads with *cannot reach this
    server:* before the wire's own sentence (`Held::said`, `UNHELD`) on the
    card, the slot and the ask's hint alike, because that sentence is
    `lernie entries`' too and names files and transports. The seat's own
    sentences follow one test: a person who has never read the code knows
    what happened and what to do next, the next thing named as the slash
    word or key that does it — so an act in doubt or never sent names the
    act as what it was (*your message*, *the new conversation*, *the
    nudge*), never as `/word`, which would be a door inviting the resend
    REMOTE §3 forbids; an unreadable answer says updating lernie may fix it;
    and *seat*, *channel*, *lineage*, *op* and *boundary* do not
    reach the glass in this window's own words.

    **Three readings of the ruling, each checked against the wire.**
    *Context* is read as **effort**: a role's facts on the wire are
    provider, model, effort (a level word) and priority (a boolean), per
    `reply/roles` rows, and there is no context knob to select — so effort
    takes the slot the operator called context. *Context window usage* IS
    stated — the `agent` row carries `context` (`prompt_tokens`, `window`,
    the server's own unclamped `percent`; `reply::agent::Fullness`) — so it
    is painted as `context N%` and asks nothing upstream. *Yellow* is the
    **annotation accent**: `docs/STYLE.md` §1's palette has no yellow, and
    the accent that says *something is happening here, not wrong* is the
    nearest it has; the check is the attention accent, the palette's green.

    **What each selector holds, lists and fires.** The selectors speak for
    the **worker** role — the role that answers — on the aimed workspace, read
    off the `roles` read, which now **stands on the aim** rather than on the
    tuning pane (`offframe::asker::workspace`): a selector that knew its fact
    only while a pane two gestures away was open would usually be blank.
    *Workspace* lists every workspace the roster holds (with its server where the
    seat reaches more than one), reloads with `workspaces`, and a pick aims
    — the switcher's own door, `Model::aim_at` with the server's opening.
    *Provider* lists the providers read's rows, reloads with `providers` —
    a read that stands on the aim beside the roles' (bl-7a77; it stood on
    the login pane alone before) — and a pick opens the *model* list on that row rather than writing,
    because a provider alone is half an assignment and the write takes the
    pair (`Edit::ready`'s rule). *Model* lists what that row offers
    (`Model::offers`, the tuning pane's own store, filed by the answer to
    the `models` read it posts) with the worker's model first where the
    listing lacks it, and a pick is `model` for the worker — the role
    tuning's own door (`Model::post_model`). *Effort* lists the four levels
    and *priority* `on` and `off`; both reload with `providers`, a pick is
    `effort` or `priority` for the worker, and each stands only where the
    worker's provider row's capability flag allows it — or where the
    providers read has not said, because absence is not evidence. With
    nothing aimed only the workspace stands.

    **The dropup** (`status/dropup.rs`, `model/dropup.rs`): a click on a
    segment opens a framed list over it, opening upward from the rule — the
    overlay's frame over a span (`overlay::float_at`), a child region laid
    on a foreground layer whose foot is the rule's top edge (§4.45 item 3:
    the rule is under the box, so the list floats over the box, as the
    workspace's floats over the bar's rows; it was laid at the central
    panel's foot while the rule stood there), never an `egui::Area`, so its
    first frame paints. **It is as wide as its longest row,
    capped at the pane** (bl-8d6d), each row measured as the row paints it
    (`theme::paint::row_width`, at the body size) — measured at the rule's
    small size it came out short, and a long model id was cut with `…` at a
    desk width with room to spare. It lists the **cached** rows at
    once and posts the read that refreshes them; the segment's mark is `⏶`
    in faint ink at rest, `↻` in the annotation accent while the read is
    out, and `✔` in the attention accent from the answer until the list
    closes — the answer of the kind the list asked (`Selector::answered_by`)
    is what turns it, and a refusal of that read's op puts the mark back to
    rest while the notice says why (a reload that never ended would claim
    one was still coming). ↑↓ walk, Enter picks, Esc closes — taken at the top
    of the frame ahead of the prompt and of the overlay's Esc — and a click
    on a row picks it, a click anywhere else or on its segment again closes
    it; an overlay or the completion menu puts it down. Each row wears the
    op its pick spends, each segment the read it posts. The role tuning
    stays as the per-role editor (`/roles`); **`/model` and Ctrl+O open the
    model's dropup** — from under a pane too: opening a dropup puts down
    whatever stands, the enrollment excepted, as Ctrl+X does (bl-27dd;
    a dropup opened under a cover was put down by it the same frame). A Shift+Tab off the prompt lands on the rule — the name,
    then the selectors right to left — and a Tab walks them in order
    (§4.45 item 3: the Tab order is the laying order, and the rule is laid
    before the box since it stands under it);
    the one that holds the keyboard wears the brand ring
    (`theme::paint::link`, bl-8e3e — STYLE §1's one stroke); Enter or
    Space opens it, as a click does.

    **The width**: what does not fit drops from the right — the
    diagnostics, then priority, then effort — and workspace, provider and
    model never drop: they are cut with `…` to one shared width, a short
    one keeping its words and the rest sharing what is left, beside a name
    held to a quarter of the line.

**Colour stays the six-state ruling** (`docs/STYLE.md` §1, operator
2026-09-06): the baseline's gold is its brand, and this seat keeps its own —
the brand on the prompt glyph and the operator's gutter, the attention accent
on the ask and a dropup's check, every other accent read off the wire's own
words, and the frames, stems, gutters and the rule's dashes in the ink scale.
**The frame is the one new shape**, and `docs/STYLE.md` §1's *no outlines* is
amended by exactly it: an outline marks what FLOATS — an overlay, the menu,
a dropup, the ask — and the panel at the head of a transcript, and nothing else is boxed.
A stroke on a thing that floats says what is over what; a stroke on a control
was the grid of chips the rule was written against.

**Keyboard, whole**: **the caret lives in the prompt line** — whenever
nothing else holds the keyboard and nothing floats over the prompt — the
ask standing included (bl-1252) — the box takes it back, at launch, after the switcher picks,
after Esc and after an overlay closes (bl-b1a1); a keystroke never needs a
click first. Enter sends; Shift+Enter breaks a line; `/` opens the
menu and Tab takes a row; Ctrl+X opens the switcher; Ctrl+O opens the
model's dropup, as `/model` and a click on the rule's model do (item 11); `/stop` and
`/interrupt <words>` are typed, not chorded (item 3); Esc closes a dropup,
then the overlay, then the menu, then the notice; ↑↓ walk whatever is open
and Enter picks in a dropup; on an empty draft a digit, ↑↓ and Enter answer
the ask; Ctrl+T unfolds the
dock; in the switcher the arrows walk onto `+ new` and `+ new workspace`
too, and Enter begins or founds (bl-f991). Every act is a slash verb or a
key, and every key calls the door the verb calls.

**What is proven, and how it stays proven.** Every assertion keeps reading the
glyphs that reached the glass (`src/paint_probe.rs`): the rule's segments are
runs, its selectors and a dropup's rows are tagged controls (a dropup is a
world, `snapshot::worlds::dropped`), the menu's rows are tagged controls, an overlay is a world, and the
ask's digits are controls the reach walk reaches. The reach bound (§4.11,
`snapshot::reach`) tightens rather than loosens — every pane is two gestures
from the field: `/`, the word.

**The order it lands in**, each its own ball nested under bl-7859 and
delivered into its worktree: the slash verbs, the menu and the overlay frame
first (every pane re-homed; the strips go); then the one column — the
switcher, the rule, the prompt line, the policy and drags deleted; then the
ask above the field; then the transcript idiom and the two panels; then the
dock.

### 4.43 The fallback face: CJK read off the system, never shipped (bl-3956)

**egui's four bundled faces have no ideograph, kana or hangul**, so CJK typed
into the box, a `/find` needle or a transcript painted as tofu. Operator ruling
2026-10-13: **load a fallback font at startup from a fixed list of system
paths; no embedded font, no new crate, no new setting.** At window start
(`src/main.rs`, the creation closure, once — not `theme::install`, which runs
every frame) the seat reads the first file of `theme::FALLBACKS`
(`src/ui/theme/fonts.rs`) that exists and carries an sfnt tag (TrueType,
OpenType or a collection; egui panics on bytes it cannot parse, so a file that
is not a font costs a face, never the window) and pushes it **last** in both
the proportional and monospace families: every glyph the bundled faces have
keeps its face, and only what was tofu gains one. The list, in order, each
taking face 0:

    /usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf
    /usr/share/fonts/google-droid-sans-fonts/DroidSansFallbackFull.ttf
    /usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc
    /usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc
    /usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc
    /System/Library/Fonts/PingFang.ttc
    /System/Library/Fonts/Hiragino Sans GB.ttc

Droid comes first because it is one face in ~4 MB where the Noto collection is
five in ~19.5 MB; face 0 of the Noto collection is its Japanese one. **A box
with none of them keeps today's face and says nothing.** **Cost, measured on a
stock Linux desktop holding the Droid file:** resident memory +8 MB (110 → 118
MB at rest: the file's bytes stay resident and the face is parsed), and the
first frame +3 ms (2.6 → 5.4 ms, warm page cache). **Emoji are not helped**:
the only system emoji face on such a box is colour (Noto Color Emoji, Apple
Color Emoji), which epaint draws nothing of, so none is listed and an emoji
past the bundled 887 code points (🦀) is still tofu.

**Why not a tracked font**: it would be the first binary in the tree, and the
disclosure gate's allowlist of unreadable files is empty by design (CLAUDE.md
*The disclosure gate*: "lernie's allowlist of tracked binaries is EMPTY") — a
font is content no rule can read, and it would add 4–16 MB to every download
for the operators who do not type CJK, when the ones who do already have a
face on their system. **The paint harness never installs it**: `theme::fonts`
takes the list as a parameter, the window passes `FALLBACKS` and the
offscreen frames pass nothing, so the snapshot matrix is the same bytes on
every box. `theme::tests`' act-glyph assertion stays about the BUNDLED faces:
a control's glyph may not lean on a face the box may not have.

---

### 4.44 The bar, the tabs, and the workspace that is not a choice made in the chat (bl-f772)

Operator ruling 2026-10-13, verbatim: *"I think it's time to add the
left-side bar. This bar is the pretty traditional list-of-chats view.
Sessions, showing their name and title. Expand when you mouse over them,
collapse to only a few characters wide when not (and remain expanded for
1.5s after mousing out, to reduce friction there). At the bottom, the
workspace selection. Different chats appear in different workspaces. I think
I was wrong earlier about the leftmost element of the chat's options being
workspace: a chat should not transition workspaces anyhow. Changing workspace
is really getting a whole new list of chats, providers, etc; an entirely
different yog backend. They should never crosstalk, so it's not really a
choice that you make _in_ the chat. New chat button also goes, top right on
this bar. Add a pin open/pin closed button that stops the auto-hiding of the
chat interface. We're also going to add tabs. Tabs are very straightforward:
just different views to see different open chats."*

**Three readings, each checked against the wire.** *Sessions* are the
wire's **conversations** — `Query::Conversations`' rows (`ConvRow`), the
rows the switcher paints. *Title* is the row's **preview** — its first
line, `ConvRow::preview` — because the wire carries no title, and the first
line is what a traditional chat list shows under a name. *The chat
interface* whose auto-hiding the pin stops is read as **the bar**: nothing
else on the glass hides itself.

**This section amends §4.42 items 1, 7 and 11**, each of which carries a
line at its head saying so. Item 1's *no side column* is withdrawn for the
one column this section adds; item 11's leftmost selector is withdrawn,
since a workspace is not a fact of a conversation; item 7's switcher stands,
as the keyboard's door and the whole seat's view.

1. **The bar** (`src/ui/bar.rs`; bl-46b9). **Amended by §4.45 items 1 and
   2 (bl-4792): the linger is 0.75 s, and both widths below are the
   defaults of two the operator drags.** It is a left panel
   (`egui::SidePanel::left`) standing the window's full height beside the
   column, painted before the column so the column fits beside it at every
   size of the snapshot matrix (`snapshot::SIZES`: 400, 900 and 1400 wide).
   It lists **the aimed workspace's conversations** — `Model::convs`, the
   one listing the seat holds for the aimed workspace, the rows the switcher
   paints under that workspace — one row each, in the listing's own order, with
   the subtree's indent (`ConvRow::depth`). It is a second painter of one
   listing and not a second listing: no row, state or count lives on the
   bar that `convs::` does not already spell (`convs::mark`, `convs::age`,
   `convs::words`), and a conversation appears on it the frame the
   listing answers, as it does in the switcher.

   **Two widths, and the pointer decides.** Collapsed, the bar is
   **a glyph column** `bar::SHUT` wide (80 points) — room for the state
   glyph and, by §4.45 item 2, as much of a name as it holds (it was the first three characters), and the
   panel's margin — so the eye keeps the count and the greens with the
   transcript at full width. Expanded, it is `bar::open_width` wide — a
   function of the window rather than a constant, since it is a quarter of
   the desk size's width (`bar::DESK`, 350 points) and never more than two
   fifths of the window's (160 at the matrix's phone) — and each row says the state glyph
   in its state's ink, the name in body ink (the selected conversation's in
   the brand), the preview beneath it in weak ink on one line cut with `…`,
   and the age at the right end in weak ink. The bar expands the frame the
   pointer enters it — **its edge easing** from one width to the other in
   `bar::EASE` (0.12 s, `ctx.animate_value_with_time` on the input clock;
   operator 2026-10-13: *very fast, but just less jarring than the
   immediate snap*), while what it holds switches at once, so the open rows
   ride out with the edge and the collapsed ones never stand in a wide bar —
   and **stays expanded for 1.5 s after the pointer
   leaves** (`bar::LINGER`), timed on egui's input clock
   (`ctx.input(|i| i.time)`, `Bar::left_at`) with a repaint asked for at
   the moment it lapses — a pointer that strays off a row for a moment is
   not made to re-cross the edge. Hover state and the clock are read, never
   stored past the frame they are needed: `Bar::left_at` is the last
   instant the pointer was seen over the bar, refreshed every frame it is
   and forgotten when the linger lapses, and while it is held the pointer
   is judged against the open width, so a pointer resting in the open bar
   with no event to wake a frame keeps it open. With nothing to list the
   open bar says why — the workspace answered empty, the workspace not answered yet
   (`convs::NOT_ANSWERED`), nothing aimed — and the collapsed bar says
   nothing. Its controls are painted before every other panel, so they
   lead the Tab order.

   **The pin** (`bar::PIN`, at the bar's head on the left) stops the
   auto-hiding: pinned, the bar stays expanded whatever the pointer does;
   unpinned, the pointer rules as above. It is one toggle with two words,
   `pin` while unpinned and `unpin` while pinned, and **it persists**
   (`Place::pinned`, the place file) because where the bar stands is a
   layout the operator chose and §4.39's place file is where such choices
   live. Collapsed, the head shows the pin's glyph alone, `📌`
   (`theme::glyph::PIN`, held to the body font's glyphs by the theme's
   glyph test). **It is a door at the heading size with `S` of air inside
   its seat** (`theme::paint::heading_link`; bl-bc9d, operator 2026-10-14:
   *the + and pin buttons are too small*), so the head is a comfortable
   target and stands a row taller than a body line.

   **`+ new`** stands at the bar's head on the right (bl-d357's `+ new`
   from under the workspace, on the bar too): it puts the caret in the field in
   start mode on the aimed workspace, the switcher's own door
   (`composer::start`), and stands only while a workspace is aimed. Collapsed,
   it is the bare `+`. It is the pin's door at the pin's size, heading-sized
   with `S` of air inside the seat (bl-bc9d).

   **The foot is the workspace** — the one place on the glass a workspace is
   chosen. It is item 11's workspace selector moved whole: the aimed workspace's
   name with `⏶` inside its door (5), a click opening the dropup (`status/dropup.rs`,
   `Selector::Workspace`) **upward from the bar's foot** listing every workspace
   the roster holds with its server where the seat reaches more than one,
   reloading with `workspaces`, a pick aiming (`Model::aim_at`) — which is
   a new listing, a new providers read, a new roles read and a new set of
   tabs (2), because the ruling's *entirely different yog backend* is the
   fact the aim already is. `+ new workspace` (bl-673e) is the dropup's
   last row under this box's own server where that server is held
   (`Pick::Found`), beside the choice it belongs to. **The switcher keeps
   its row too** (bl-46b9): it is the keyboard's door (4), and the walk,
   the field's hints and the refusal's remedy all name it there. With
   nothing aimed the foot says `workspace ⏶` and the list is the whole of
   what the bar shows. Collapsed, the foot is as much of
   the workspace's name, with the selector's mark.

   **The list stands inside the bar, and holds it open** (bl-46b9). It is
   the rule's dropup (`status::dropup::stand`) laid on a foreground layer
   whose room is the bar's own width and whose bottom is the foot's top;
   while it stands the bar is open whatever the pointer does, because a
   list opened from the bar is the bar in use. Laid over the column it
   covered the rule's first segments at every size, which the matrix's
   blank assertion read as a defect — and it was one: the column keeps
   every word it had while the bar is in use.

   **A row is the one door.** A click on a row is `Model::select` on its
   root id — the switcher's, the queue's and the rule's one door — and so
   opens its tab (2). Each row wears the `act:` tag the switcher's rows wear;
   the pin wears none (it posts nothing), `+ new` wears the start's. The
   bar never covers an overlay and no overlay covers it: it stands beside
   the column whatever the column shows, because what it lists is not what
   the overlay is about.

   **And a trashcan stands at the open row's right end while the pointer
   is over it** (bl-e837; operator 2026-10-14: *on mouseover, a trashcan;
   confirmation dialogue unless ctrl is held down*): `theme::glyph::TRASH`
   in faint ink under a seat of its own (`theme::paint::Row::door`), lit
   under the pointer as every door is, wearing `delete-agent`'s tag; the
   age steps left of it, and the row keeps its wash while the pointer is on
   it. Its click asks — a confirmation standing on the box where the ask
   stands (`bar/confirm.rs`), naming the conversation, `keep` first and
   `delete` after, Esc and Enter — and Ctrl+click fires the delete at once.
   It is §4.42 item 5's `/delete` by a second door and the same envelope
   (`Model::post_delete`, the one composition both go through), the seat
   typing the name the row bears.

2. **The tabs** (`src/ui/tabs.rs`; bl-03c1) are a strip at the top of the
   column, above the transcript, one tab per **open** conversation of the
   aimed workspace: the name, with the asking `?` in the attention accent
   before it while that conversation asks (`convs::mark`), the selected
   tab in body ink over a brand hairline and the rest in weak ink, and a
   close `×` in faint ink after each name. **Open is a navigation the
   operator performed** — the class of fact `Model::opened` already holds
   and no read can answer — so `Model::tabs` is a `Vec<Tab>` of the
   conversations selected since their tab was closed, in the order they
   were first selected, each with the aim it was selected under
   (`Tab { aim: Aim, root_id: String }`), and the strip paints the ones
   whose aim is the current one. It is filled by `Model::select` and by
   nothing else: every door that selects — the bar, the switcher, the
   queue, the rule's name, a start resolving (`Model::resolve`) — opens a
   tab by selecting, so there is no second act that opens one. A click on a
   tab selects; the close removes the tab, and where it was the selected
   one the neighbour to its left (or right, at the left end) is selected,
   or the selection is put down to the workspace where it was the last. **A tab
   whose conversation the listing no longer knows leaves with it**
   (bl-6dfd's rule): a selection the server put down is a tab put down. The
   strip stands only while a tab stands; a workspace with nothing open is the
   transcript's own empty column. **A start's tab is its claim's**
   (bl-03c1): the receipt selects the minted name, so the tab opens under
   it; no listing prunes that name while the claim stands, and the listing
   that resolves it (`Model::resolve`) renames the tab to the root in place
   rather than opening a second. Tabs are per run, never in the place
   file: which conversations are open is where an operator *was*, and a
   seat that reopened yesterday's tabs would be claiming a navigation
   nobody performed today. Each tab wears the `act:` tag its select
   spends; the close wears none.

3. **The rule without the workspace** (item 11 amended): the line is
   `— provider ⏶ | model ⏶ | effort ⏶ | priority ⏶ | <diagnostics> —
   <name>`; the four selectors and the diagnostics are as item 11 says, and
   item 11's *with nothing aimed only the workspace stands* becomes *with
   nothing aimed no selector stands* — the foot of the bar is where that is
   answered. The width rule loses its first word: provider and model never
   drop and share one cut width; priority, then effort, drop before them.
   `Selector::Workspace` keeps its rows, its read and its pick, and loses
   its segment: `status::selectors` no longer lists it, and the bar's foot
   is its one segment.

4. **What stays.** The switcher (item 7, Ctrl+X) stands as the keyboard's
   door and the one view across every server; the bar is one workspace's view
   with the pointer. Both paint `convs::` rows, so a glyph, a word or a
   count changes in one place for both. The rule's name stays the
   switcher's door (item 10). The seat's card still names Ctrl+X among the
   ways in, and the field with nothing aimed still says so.

5. **What lifts under the pointer, and where a selector's mark stands**
   (bl-c72e; operator 2026-10-13: *mouseover highlight for interface
   elements in lists to show that you're actually about to select it*, and
   *the ^ arrow next to options menus should be a part of the button (it's a
   natural place to click)*). **One wash, one painter**: every pickable row —
   the bar's, the switcher's, a dropup's, the completion menu's, the queue's,
   the ask's choices — is `theme::paint::row`/`marked`, and every door that
   is a run — the rule's selectors and name, the bar's pin, `+ new` and foot,
   a tab and its close — is `theme::paint::link`/`link_marked`/`tab`, one
   private `door` under all three. Under the pointer each wears
   `theme::hover()`: **the brand at three quarters of the selection tint's
   strength** (`BRAND × TINT_ALPHA × 0.75`; half read as faint on the real
   window, bl-232e), not a new colour. It replaced `SURFACE`,
   which was the hover until here and lifted nothing where it stood on a
   popup's own `SURFACE` — a dropup's and the menu's rows did not tint at
   all — and barely lifted the glass. A wash lifts every ground it stands
   on; a chosen row keeps its `RAISED`, the stronger tint, under the pointer
   too. A tab was a sensed label and is now a `tab` door, so it lifts with
   the rest.

   **A row is its whole block** (bl-232e). A row with lines beneath its
   head — the bar's open row (glyph and name, the age at the right end, the
   preview beneath) and the switcher's (the headline, the failure's own
   words, the preview) — is one `theme::paint::Row`: one seat over every
   line, edge to edge of the list, so the hover wash, the chosen `RAISED`,
   the state rule and the focus ring all span exactly what a click takes,
   and a click on the preview selects as a click on the name does. It was a
   one-line row with labels hung under it, so the tint covered the name line
   alone. The collapsed bar's rows are the same painter with nothing
   beneath. The switcher's `show N`/`hide N` stays a row of its own: it is a
   different act. The queue's rows are not pickable as a whole (their acts
   are their controls) and keep their lines as labels.

   **The mark is in the door.** A selector's `⏶` (and its `↻` and `✔`) is
   painted inside the same seat as its words — one galley, `<words>
   <mark>`, the mark in its own ink (`link_marked`) — so a click on the mark
   opens the dropup, the wash covers both, and the run is **one stop in the
   Tab order** as it already was (the separate mark was a label, never a
   stop). The door's accessible name stays its words: the mark is the
   affordance, not the name. The rule's width rule measures the mark as the
   space and glyph the door lays (`status::cost`).

**What this does not solve, and the principle that answers it.** A bar
listing one workspace is blind to another workspace's asking conversations; the
seat-wide green is the switcher's head, the seat panel and `/attention`
(item 11's placement, kept), and a bar that showed every workspace's rows would be
the switcher again with the pointer over it. The 1.5 s is a number read off
the ruling and held in one constant; the width constants are the bar's own
and the matrix proves the column still fits beside them at 400 wide.

### 4.45 The chat as state: the box beside the transcript, a sent message on the glass at once, the bubble, the trail that folds, the door that lights, and the bar that is dragged (bl-4792)

Operator ruling 2026-10-14, verbatim: *"Animation is good, but we can
reduce the out of frame time to 0.75. Width on the side panel should be
draggable, but while closed and while open. Flip all the chat controls to
under the conversation box, so the box is right next to the chat. Then,
(and this was implemented in an earlier version) when you send, it
immediately snaps up, with a text color indication that it hasn't been
acked by the remote machine yet. The design sensibility is that state
should always be immediately represented: there should never be a
half-second where your message is nowhere in the interface. Tool calls
should autocollapse when done. In general, all clickable interface elements
should have highlighting on mouseover. User, tool call, intermediate
response, and final response should all have bubbles under them demarcating
them, with slightly different colors. Not jarring, just enough that you know
immediately, 'I'm reading the message I wrote'"*

**One sentence of it is the principle and the rest are its instances**:
*state should always be immediately represented*. A message the operator
sent is state the moment Enter is pressed, whatever the server has heard;
which width the bar stands at is state the operator set; whether a trail is
still running is state its fold should say; what a pointer is over is state
the glass should show; and who said a block is state a fill can say faster
than a glyph can be read. Each item below is that one rule applied to one
surface, and none adds a fact the model did not already hold — the one new
field (item 4) records a gesture the seat itself made.

**This section amends §4.42 items 3, 4, 9 and 11 and §4.44 item 1**, each
of which carries a line at its head saying so, and `docs/STYLE.md` §2 and
§5, which stated the orders and the *never bubbled* rule this section
changes.

1. *Amended by §4.46 item 4: the number is the `linger` knob's default.* **The linger is 0.75 s** (`bar::LINGER`; bl-7578). The number read off
   the ruling, in the one constant §4.44 holds it in; nothing else about the
   bar's opening and closing changes, the ease included.

2. *Amended by §4.46 item 4: the two widths are the `bar_shut` and `bar_open` knobs, which a drag writes; `Place::dragged` goes.* **Both of the bar's widths are dragged** (`bar::Bar`, `place.rs`;
   bl-7578). The bar's right edge is a drag handle in both of its states:
   dragged while collapsed it sets the collapsed width, dragged while
   expanded the expanded one — which width a drag writes is which state the
   bar was in when the drag began, so a bar that opens under the pointer
   mid-drag does not hand the drag to the other width — and the edge is not
   the bar: a pointer reaching for the collapsed bar's edge does not open
   what it came to drag, and while the edge is in hand the bar stays in the
   state it was grabbed in, whatever the pointer crosses. Each is a number the
   operator set, which is §4.39's class of fact — *per-seat UI state … the
   seat's own* — so each lives in the place file (`bar_shut`, `bar_open`,
   floats read through serde and never through a cast), and §4.44's two
   constants become **the defaults a seat that dragged nothing gets**. The
   policy keeps the floors and the ceiling, applied by clamping on every
   frame and never by overwriting (§4.39: *a window narrower than the stored
   width clamps it and does not overwrite it*): the collapsed width is never
   less than the glyph column with its margins nor more than the expanded
   one, and the expanded width never more than two fifths of the window.
   The eased travel between the two (§4.44 item 1, bl-c72e) is unchanged: a
   drag moves one end of the travel and the ease still runs between the
   ends. **And a collapsed bar dragged wider shows more of each name**
   (bl-773e): a collapsed row paints the whole name and the foot the whole
   workspace, each elided with `…` at the width it has, as every row on the
   glass is (STYLE §5) — §4.44's *first three characters* was a cut at the
   default width and is withdrawn, since a width the operator set is the
   width to use.

3. **The box stands directly under the transcript, and every control
   stands under the box** (`src/ui/shell.rs`; bl-45ab). The bottom panels
   are laid so the rule takes the window's bottom edge, the dock (§4.42
   item 9) stands on the rule while anything runs, the prompt line stands on
   those, and the transcript fills what is left — so the words an operator
   types are beside the words they are answering, with no line of facts
   between. **The ask stays above the box** (§4.42 item 8): it is the
   model's question and not a control of the chat, and the box under it is
   where its answer is typed; it is also transient, standing only while the
   selected conversation asks. A selector's dropup still opens upward from
   its segment's left edge, which is now over the box: it is laid on a
   foreground layer anchored to the rule's rect, as the bar's foot lays the
   workspace's list inside the bar (§4.44 item 1), and it covers the box
   only while it stands. The slash menu keeps floating over the transcript's
   foot, directly above the box it completes. The notice too long for the
   rule still stands whole above the rule (§4.42 item 11) — above the rule,
   under the box. **The Tab order follows**: egui walks widgets in the order
   they are laid, and bottom panels are laid in their stacking order read
   upward — the rule, the dock, the box, the ask — so a Shift+Tab off the
   box lands on the rule's name and walks its selectors right to left, and
   a Tab off the box goes on up into the transcript; every control is one
   walk away, as before, and no binding is added to put the rule first.

4. **A sent message is on the glass the frame it is sent** (`Model::sent`,
   `composer::fire`, `chat::rows`; bl-ca3d). The ruling's own sentence: *there
   should never be a half-second where your message is nowhere*. The box
   composes a `message` or `interrupt` act and clears the draft, and in that
   same frame the model records what it composed — the conversation it was
   addressed to, the words, and how many entries the committed transcript
   held — beside the outbox. The transcript paints each such record as an
   operator row at its end, after the committed rows and the live fold, with
   the prompt's glyph in the gutter and **the words in the pending ink**
   (`INK_WEAK`, the ink of what is not yet settled) until the act's receipt
   lands, then in body ink. It is a row of the transcript and not a notice:
   it stands where the message will stand, under the operator's bubble (7).
   Three endings, each decided where the fact arrives:
   - **The receipt** (`Model::receipt`, the `message` and `interrupt` ops):
     an `ok` outcome marks it acknowledged, which is the ink changing and
     nothing else — the row stays until the transcript carries the words,
     because the read that will carry them lands at ask cadence and a row
     retired on the receipt would be a message nowhere for exactly the
     half-second the ruling forbids.
   - **The transcript** (`chat::rows`): the frame the committed transcript
     holds an operator delivery with those words at an index past the
     recorded length, the record is dropped — the words are now the
     server's own row, and one row, never two.
   - **A refusal**: the record is dropped and **the words go back in the
     box**, the start's own rule (bl-b180: *the server's refusal retires the
     start into its own sentence with the goal back in the box*) — a refused
     message cleared from the draft and gone from the transcript would be
     state represented nowhere. **Ahead of whatever has been typed since**,
     with a line break, where the start's rule gives words back only to an
     empty box: a goal overwriting a newer draft would lose the newer of the
     two, but a refused message was a row on the glass and the box is the
     only place left for it, so it goes in front and neither is lost. An act
     that never left this box (`Model::acted`, REMOTE §3's *unsent*) drops
     the record the same way, because no receipt and no read will ever
     retire it, and the words are the refund's (bl-e85f). A lost reply
     leaves the row standing in the pending ink beside the in-doubt notice,
     which is the honest picture: the server may have it.
   The records are keyed by conversation, so a tab change keeps each with its
   own transcript, and they are per run like the tabs: a seat that reopened
   carrying yesterday's unacknowledged words would be claiming a gesture
   nobody made today. Both deposits go through one door (`Model::deposit`),
   the box's Enter and the slash line's `/interrupt`, so there is no deposit
   on this window that is not on the glass the frame it is composed.

5. **The working folds in three stages** (`chat::trail`; bl-d50e, then
   bl-0174). Operator ruling 2026-10-14, second round, verbatim: *"the
   experience that I'm going for is verbose while it's happening, but
   compact when it's done. While the tools are going, they should be
   expanded, then once the tool call is done, it should collapse to just
   one line for call and one line for output (truncated, it's to preview,
   not to show the whole content), similarly for intermediate outputs, etc.
   Once you get to the final output, the whole intermediate segment,
   including tool calls and intermediates, thinking, etc, should
   collapse."* **A turn's working** is every row between the operator's
   turn and its final prose: the calls, their results, the reasoning, and
   the intermediate prose between them. It is one block on the trail's
   bubble (7), and what it shows is decided by the state of each step and
   of the turn — never by size, which is bl-90d0's rule kept:
   - **A step in flight stands whole**: the call with its input head, its
     output as it arrives, a thinking or an intermediate prose as it
     streams. The working is open while any step is in flight.
   - **A finished step is compact**: a call whose result landed is its call
     line and ONE line of its output, elided with `…` at the width it has
     — a preview, not the content; a finished thinking or intermediate
     prose (one a further step followed) is one elided line. Each compact
     step is a door that opens it whole in place, and the fold's head and
     whole (`chat::fold`) are what it opens to.
   - **A settled turn's working is one line** — `+ worked · N calls · M
     notes`, the counts of what it holds — standing between the operator's
     turn and the final answer, so the transcript at rest reads as question
     and answer. A click opens it to the compact steps; a second click on a
     step opens that step.
   An explicit toggle is remembered as before, the toolkit's memory of a
   view state — of ONE step's, in the conversation it was clicked in: every
   id under the transcript is pushed under the conversation's (bl-b83d), so
   the same row of another conversation is in its stage until somebody
   toggles it there — and the stages decide only where nobody toggled. The
   `response` elbow that parts the working from the prose after it stands
   only while the working is open; folded, the one line is the parting.
   Nothing is dropped at any stage: every word is one or two clicks away,
   and the head counts what it hides.

6. **Every clickable element lights under the pointer** (`theme::paint`;
   bl-52bb). §4.44 item 5 gave the hover wash to every row an operator can
   pick; this item gives it to every seat that senses a click — the trail's
   fold head, the panels' `▸` folds, the dock's line and rows, the ask's
   cells, the overlay name lines, the queue's, records', config and tuning
   panes' controls, the enrollment's — each through `theme::paint`'s door or
   the row idiom, or wearing egui's own button hover where it is already a
   button. The instrument is a test that hovers each tagged control of each
   snapshot world and reads a fill under the pointer, so a control that
   arrives bare fails the suite rather than the eye (`src/snapshot/lit.rs`,
   assertion (d)). *Lit* means the fills inside the control's rectangle
   changed between the frame at rest and the frame with the pointer on it,
   and what it does not judge is stated as a fact of the control, never as
   a name: a control egui does not report hovered with the pointer at its
   centre (disabled, scrolled past its clip, or under an overlay), a
   selected one (`RAISED` outranks the wash), and a field that takes text
   (its answer is the caret). A label that senses a click is the one bare
   shape the tree cannot tell from prose, which is why the audit was by
   hand and the dock's line went through the door.

7. **Four bubbles, by role** (`theme::bubble`, `chat`; bl-d50e). Under each
   block of the transcript stands a filled block at `RADIUS` — the gutter
   glyph kept inside it, STYLE §5's *read by its gutter* standing — and
   the fill says the role: **the operator's turn** on the brand's tint (the
   brand is the operator's colour on every surface, §4.42 item 4); **a
   trail** on the ground inside a hairline, the lowest step; **intermediate
   prose** — the model's prose a further trail follows within the same turn,
   and the live turn's prose while it streams, since what follows it is not
   known yet — on `SURFACE`; **the turn's final prose** on `RAISED`. The
   model's three are the elevation ramp rising toward what is read, and the
   operator's is the one hued fill, so *I'm reading the message I wrote* is
   answered by hue and the rest by height. The fills are named once in
   `theme` and derived from the palette — no new literal — and they are
   faint: the ruling's *not jarring*. Which bubble a prose row gets is
   decided in the projection (`chat::rows`), where the turn's shape is
   known, and never by the painter counting.

**What this does not solve, and the principle that answers it.** The
pending row says what this seat sent and nothing about what the server did
with it; the server's own row, when it lands, is the record, and the seat's
row leaves the frame it does. A drag that narrows the collapsed bar below
its own glyph is clamped, not refused, so the floor is one number and not a
sentence. A control egui paints as a button already lights; the audit's
rule is that nothing on the glass senses a click without saying so, not
that everything goes through one painter.


### 4.46 The interface told, keyed and tuned: the wheel, hover help on every control, every act on the keyboard, and the settings pane (bl-5369)

Operator ruling 2026-10-15, verbatim: *"Sensitivity in the scroll is a
little low. Something that we had in an earlier version of this was
keyboard shortcuts on everything and mouseover strongs on everything. I'd
like to bring that back. Every interface element should have a short delay
(half second?) mouseover hover help text, which should define what the
element does. Project lifecycle for handling any interface elements should
include updating as necessary any strings. All actions should have keyboard
shortcuts; the whole interface should be operable wihtout a mouse. Also,
let's get a settings pane and knobs for pretty much everything."*

**The principle under it is §4.45's, read for the hand instead of the
eye**: what a control does is state, and the glass says it before the hand
commits — under the pointer, as a sentence; on the keyboard, as a chord the
sentence names. The *earlier version* is the window yog carried before
its severance (bl-7942), where every control carried a `HOVER` constant
beside its word and the acceptance suite drove the keys; the shape returns,
the mechanism is this seat's. Four items, each one rule applied to one
surface.

1. **The wheel step is a setting, not the toolkit's constant.** egui scrolls
   40 points per wheel notch on native (its own *decided by consensus*
   number, `Options::line_scroll_speed`), which on a 24-point row is under
   two rows a notch. The seat installs the step from the settings
   (`settings::Knob::wheel`, default **100 points** per notch, item 4) in
   the one call that installs the style (`theme::install`), so the number
   has one home and the pane changes it live.

2. **Every control tells** (`theme::paint`; `snapshot::told`). A control is
   painted by one of the theme's painters — a door (`link`, `tab`,
   `heading_link`, `link_marked`), a row (`row`, `Row`), a
   button (`button`, new: the one home of `egui::Button`), a toggle
   (`toggle`, new: `egui::Checkbox`), a seat (`seat`, new: the former
   `selectable_label`) and a slider (`slider`, new) — and **every painter
   takes the control's help sentence** beside its word, so a control
   without one does not compile. The sentence **defines what the control
   does** in one line, in the operator's terms and never the wire's: *keeps
   the bar open whatever the pointer does*, not *sets Place::pinned* —
   and where a chord fires the same act, the sentence ends with it after
   ` · ` (`keys::chord_for`, item 3), so the help is also how the keys
   are learnt. It is shown as egui's tooltip after the pointer has rested
   on the control for `help_delay` (default **0.5 s**, item 4), in the
   overlay's own box and ink (STYLE §5), and never while the pointer
   moves. **The sentence is authored at the control, beside its word**, as
   a constant where the word is one (`bar::PIN` and `bar::PIN_HELP`) and
   composed where the word is a name (a tab's: *shows this conversation*),
   because a table from label to sentence is the translation table PARITY
   §4 refuses, and it drifts the first time a word is reworded.

   **Three things hold it, and none is a reviewer.** The compiler: no
   painter without a sentence. The confinement rule
   `rules/no-bare-control.yml`: `ui.button`, `Button::new`,
   `selectable_label`, `checkbox`, `small_button`, `ui.link`,
   `Slider::new` and `DragValue::new` are confined to `src/ui/theme/paint/`
   with the other four rules' discipline — one location, an `ignores` list
   that is the inventory, a fixture that must flag. And the instrument
   **assertion (e), every painted control tells** (`snapshot::told`, the
   twin of `lit`): every painter records the seat it painted in test
   builds (the bridge `act::seat_of` reads, extended from the tagged set to
   the painted set), the matrix puts the pointer at each seat's centre,
   steps the clock past the delay with the pointer still, and reads back
   whether a tooltip opened for that seat
   (`egui::popup::was_tooltip_open_last_frame`); a seat with none is a
   complaint, and a run that judged under the floor is a bridge gone
   quiet. The `lit` instrument's subject widens to the same set.

3. **Every act is on the keyboard, and the bindings are one table**
   (`keys::BINDINGS`). *Operable without a mouse* is read as two halves,
   both already partly true. **Every word is a keyboard path**: a slash
   verb is typed, and every pane, every wire verb and every door the menu
   lists is reached by `/` and its word (§4.42 item 5) — so a word gains no
   chord for being a word. **Every control is a Tab stop**: egui focuses
   every click seat in paint order, fires it with Enter or Space, and the
   brand ring says where the keyboard is (bl-2d6b, bl-8e3e); that is reach
   for the folds, the rule's selectors, the bar's doors and every button on
   every pane. What chords add is **the acts that are neither a word nor a
   stop worth tabbing to**, and the set is deliberately small and
   conventional:

   | chord | does |
   |---|---|
   | Ctrl+, | opens the settings pane (item 4) |
   | Ctrl+B | pins the bar open, or unpins it |
   | Ctrl+N | begins a conversation on the aimed workspace (`+ new`) |
   | Ctrl+W | closes the selected tab |
   | Ctrl+PageDown, Ctrl+PageUp | the next tab, the previous |
   | PageDown, PageUp | scroll the transcript a page, from the box too |
   | Ctrl+End, Ctrl+Home | the transcript's end, its head |
   | Ctrl+=, Ctrl+-, Ctrl+0 | scale up, down, back to 1 (item 4's `scale`) |
   | F1 | `/help` |

   with the eleven that stood (Enter, Shift+Enter, `/`, Tab, ↑↓, Esc's
   order, Ctrl+X, Ctrl+O, Ctrl+T, the ask's digits). **Two kinds of row,
   one table**: a *fired* row is a chord and the act it fires, run from the
   table from anywhere on the glass; a *told* row — the box's Enter, the
   menu's Tab, the walk's arrows, the ask's digits, Esc's order — is read
   where it is read and stands in the table so the printed list is whole.
   PageUp and PageDown
   reach past the box because the box does not take them: a key the draft
   has no cursor move for is not the box's (`keys::boxes`' gate reads the
   box, this reads the key). They scroll nothing while a pane covers the
   transcript, which is then not what the hand is on; and the transcript's
   scroll area is built **unstuck and uneased on the frame a key scrolls
   it** (`chat::scroll`), because egui undoes a delta handed in from
   outside on an area stuck to its end, and an eased scroll lands on a
   later frame than the one that loosened the stick. The tab chords
   saturate at the strip's ends as the walk does, and with nothing
   selected go to the first. Ctrl+Plus fires the scale-up row for a
   keyboard whose `=` is `+` under Shift, and is not printed. **The scale chords are the seat's, not
   egui's**: egui binds the same chords to a zoom it keeps in its own
   memory for the run (`Options::zoom_with_keyboard`), which would fight a
   setting installed every frame, so that option is off and the chords
   move the knob, which persists.

   **The table is the one home.** `keys::handle` runs it, `/help` prints
   it where it printed `commands::KEYS` (which goes), and a control bound
   by a row cites the row in its help (item 2): a chord added without a
   row is a chord nothing fires, and a row added is told on the glass
   without a second edit. §4.42 item 11's key list and §4.44's are
   superseded by the table where they differ. **A chord that opens a pane
   opens it alone** (bl-8c80): the words the chords reach go through
   `Model::uncover`, the one loop the switcher's and the dropup's doors
   already spent, so §4.42 item 7's *one stands at a time* holds from under
   anything — Ctrl+, under the switcher opened the settings pane beneath it.
   Nothing fires under the enrollment. **A standing pane leads the Tab
   order** (bl-99ed): the bar is painted first, so Tab under a pane walked
   its pin, its `+ new` and every row before reaching the pane; now, while
   a pane stands and nothing holds the keyboard, the pane's first painted
   control takes it (`overlay::FIRST`, read down in `theme::paint::told`,
   the one function every control passes through) — the twin of the
   prompt line taking the caret back at rest. The switcher is excepted,
   because its keys are the walk and a focused row would be what Enter
   fired. A slider that holds the keyboard wears the brand ring, as a row
   and a door do, and its trough is raised on the overlay's surface.

4. **The settings pane, and the knobs are one table** (`src/settings.rs`,
   `src/ui/settings.rs`; `/settings`, Ctrl+,). **Everything on it is a
   fact about this seat** — REMOTE §7's *per-seat UI state*, the class the
   tuning pane (§4.17) is deliberately not — so it crosses no boundary and
   has no standing read: the pane paints the table and a change applies at
   the next frame. Each knob is a row of `settings::KNOBS` — its key, its
   label, its help sentence (item 2's, shown under the control in weak ink
   and as its hover), its kind (a toggle, or a number with its unit, floor,
   ceiling and step) and its default — and the pane, the place file and
   the defaults test all read the table, so a knob added is painted,
   remembered and judged with no second edit. The readers read typed
   fields of `Settings`, never the table. The knobs:

   | key | what it tunes | default |
   |---|---|---|
   | `wheel` | points scrolled per wheel notch (item 1) | 100 |
   | `help` | whether controls tell under the pointer | on |
   | `help_delay` | seconds the pointer rests before one tells | 0.5 |
   | `scale` | the window's scale, text and air alike (egui's zoom factor) | 1 |
   | `linger` | seconds the bar stays open after the pointer leaves (§4.45 item 1) | 0.75 |
   | `ease` | seconds the bar's edge takes to travel; 0 snaps (§4.44 item 1) | 0.12 |
   | `bar_shut` | the bar's collapsed width in points — what a drag sets (§4.45 item 2) | 80 |
   | `bar_open` | its open width — the other drag | a quarter of the desk |
   | `bubbles` | the fills under blocks by role (§4.45 item 7) | on |
   | `fold` | whether a working folds itself once the answer stands (§4.45 item 5) | on |
   | `confirm` | whether a delete asks first; off, the trashcan deletes at once (§4.44 item 1) | on |
   | `dock` | whether the dock stands unfolded when the window opens (§4.42 item 9) | off |

   and the pin, which is the same fact `bar::PIN` toggles and stays where
   it is (`Place::pinned`), painted on the pane as a thirteenth row. **The
   two widths move here from `Place::dragged`**: a drag writes the knob
   and the knob paints the drag, one home, so a seat that dragged before
   this landed reads the defaults once. A number is a slider — arrows move
   it while it holds the keyboard, which is item 3's reach — and a toggle
   is a check; `defaults` at the foot puts every knob back. **Persistence
   is the place file's** (§4.13): a `settings` object beside the aim,
   holding only the knobs set off their default, so a seat that touched
   nothing writes nothing and a default moved in a later build reaches it
   — the rule the dragged widths already kept. `theme::install` installs
   what the toolkit holds (the wheel, the tooltip delay, the zoom) from
   the settings each frame, and the bar, the chat and the delete read
   theirs.

5. **The lifecycle rule, which is what the ruling's second sentence asks
   for.** A control is three strings declared together — its word, its
   help and, where one is bound, its chord's row — and one painter reads
   all three. There is no step to remember: a control added without a
   sentence does not compile, one painted outside the painters fails the
   rule, one whose sentence stopped opening fails the instrument, and a
   chord without a row fires nothing. Changing a word is changing the
   line its sentence is on.

**What this does not solve, and the principle that answers it.** A
sentence under the pointer is not a manual: it says what the control does
and nothing about why, which is `/help`'s and the verb's `detail`. A
knob is offered for what the seat already decided by a constant, not for
every number in the theme: the palette, the type scale and the spacing are
the language (STYLE §1) and a knob on each is a window with no look. The
scale knob is the one knob over all of them. Tab reaches every stop in
paint order, and a long list is still a long Tab; the chords and the words
are what make the long way short, and the switcher (§4.42 item 7) is still
the keyboard's door to a conversation.

## 5. Module map

| Path | What it is | Cap band |
|---|---|---|
| `src/main.rs` | the process entry: argv in, the environment folded once, a stream and an exit code out. The one `tarpaulin.toml` exclusion, and it is honest because it decides nothing. | small |
| `src/lib.rs` | the crate doc and the module list. | small |
| `src/cli.rs` | the command line as a **pure function**: arguments in, a `Decided` out. No argv, no environment, no streams, no exit. | ~210 |
| `src/cli/start.rs` | the composite start's grammar: the work target, positional because the §3.4 rung is what it decides, and `--role <name>`, written because it is stated on the FIRE and not on the stage (REMOTE §9.21, PROTOCOL 18). A tail this arm cannot read leads with the quoting, because it is almost always an unquoted goal. | ~80 |
| `src/cli/answer.rs` | `answer`'s own grammar: the three words and the optional fourth, the reach (PROTOCOL 18). A word the wire REQUIRES and an operator may leave off is neither a parameter nor a flag, so the word answers the field from the narrow constant and reads a misspelling here — `enroll`'s grade's own rule. | ~60 |
| `src/cli/enroll.rs` | `enroll`'s own grammar: its three words, the grade read off a closed set of two, and the two optional words — `--at <host>:<port>`, the route the new box will dial, and `--into <dir>`, where this box writes the material down (§4.15, bl-971c). Its own file for the reason yog's line reader cut the same seam at: a verb whose grammar is more than words is its own file. | ~120 |
| `src/cli/decided.rs` | what one invocation decided to DO — the value `run` hands back and the whole of what `src/main.rs` acts on. Split from the deciding at the cap: a word added moves the match, a kind of act moves this. | ~100 |
| `src/cli/verdict.rs` | what an invocation says, and with what exit code: the four constructors, the two codes, and the one-line pointer a refusal carries instead of the whole usage (§4.10, bl-b232). | ~110 |
| `src/cli/text.rs` | what this binary says about itself: the version line, and the usage whose verb section is derived. | ~75 |
| `src/paths.rs` | the two roots — what the operator carried here, and what the seat generates about itself — from one ladder and no knob of its own. Neither variable set is a refusal, never a guess. | ~130 |
| `src/place.rs` | where the seat was pointed and how its servers were arranged, remembered between runs (§4.13, §4.39): the aim, the open server, what each server's last opening ranks as, the workspace last aimed under each, the bar's pin (§4.44) and the settings set off their defaults (§4.46 item 4). Every way the file can be wrong is absence, per field. | ~190 |
| `src/settings.rs` | **the knob table** (§4.46 item 4): every seat-local setting as one row — key, label, help sentence, kind with its unit and bounds, default — the typed `Settings` the readers read, and the two projections, from the table's JSON and to it, writing only what is off its default. | ~250 |
| `src/ui/settings.rs` | the settings pane (`/settings`, Ctrl+,): the knob table painted, a slider per number and a check per toggle, each over its sentence, and `defaults` at the foot. Paints the table; decides nothing. | ~150 |
| `src/envelope.rs` | the gesture envelope from the seat's side: is it one, which workspace does it name, did the last reply say ok. **One table, not two** — the read answers through the write. | ~150 |
| `src/seat.rs` | one gesture spent: routed, asked, and answered as this seat's product. | ~100 |
| `src/seat/route.rs` | which channel a gesture goes down, what it carries there, and what this box calls the channel it chose (§4.7). | ~190 |
| `src/seat/holds.rs` | what this box says it holds, said without dialling any of it: the listing, the typed channel set the window stamps its rows with, and the one spelling of a channel's name. | ~150 |
| `src/seat/fan.rs` | a gesture that names no workspace, asked of every channel this box holds — the union, stamped with where each answer came from, where an unprovisioned bare channel is an absence rather than a section. That enumeration is `held`, and `aimed` counts it too. | ~130 |
| `src/seat/aimed.rs` | a workspace-less ACT down one channel (bl-1bb9): the one `--on` named, opened by `dial`, or the one the box holds — refused, naming them, where it holds more. | ~55 |
| `src/seat/start.rs` | the §8.1 start family's two acts, spelled as one word — the composite, and the local between them. | ~80 |
| `src/seat/follow.rs` | holding the line on one conversation until it comes to rest (§4.10, bl-3dca, bl-f076): the reads it holds across the server's step boundaries, and the sink the product is written to as it arrives. | ~190 |
| `src/seat/follow/rest.rs` | when that watch stops and what it says then, split from the file above at the cap on the seam the word has (bl-3a1f): the three readings of the standing row — working, at rest, or PARKED at the capability boundary — the inbox read that says whether a rest is one (bl-87ab, bl-3ecd), and the two sentences an ending can be. | ~170 |
| `src/seat/model.rs` | the role assignment, against the list the same seat can already fetch (§4.10, bl-1e5a): the read that goes ahead of the write, the warning a listing nobody offers earns, and every way a silent read costs the assignment nothing. | ~95 |
| `src/channel.rs` | one wire to one server: ask, follow, and what the server at the far end could SPELL as of the last dial (§4.9). | ~210 |
| `src/channel/dial.rs` | how a socket is obtained and first spoken on: the ladder climbed for a line, the preface and the request down it, and the sentence a failed write earns (§4.40). Split from the root at the budget. | ~90 |
| `src/channel/frame.rs` | the framing. | ~105 |
| `src/channel/hello.rs` | the version preface: write the major and this build's edition, confirm, and the refusal that names both numbers. The edition refuses nothing (§4.9). | ~120 |
| `src/channel/hello/ledger.rs` | the discipline in force, and the re-export of the constant `build.rs` compiles from the repo-root `PROTOCOL` file. Split from the file above at PROTOCOL 18 (bl-c515) because the two grow for different reasons — that one is what the preface DOES, this one is the record of what it has carried. | ~90 |
| `src/channel/hello/ledger/exact.rs` | the closed era, 2 through 18: one entry per bump, whole and unedited, in a module of nothing but prose. Split off at 19 (bl-aa01) when the major stopped moving for additions — that ledger will never gain a nineteenth entry. | ~245 |
| `src/channel/edition.rs` | what each end can SPELL (§4.9, REMOTE §3.2): the floor, this build's edition, and the ledger of post-floor paths — vendored, and held equal to `corpus/shapes.json` by its own tests. | ~90 |
| `src/channel/tls.rs` | the mTLS configuration. | ~90 |
| `src/channel/leaf.rs` | the grade, read off this box's own leaf: the one fault it names, and the DER walk that names it. | ~200 |
| `src/channel/reach.rs` | why an exchange produced no answer, and the one fact a sentence cannot carry: whether the request crossed (§4.22). | ~70 |
| `src/channel/material.rs` | what the operator carried here, and what its absence means — the two remedies its absence earns, and which of them is the caller's fact (§4.5, bl-ad7f, bl-5cbe). | ~160 |
| `src/channel/entries.rs` | the client-side workspaces this box holds elsewhere. | ~165 |
| `src/channel/clock.rs` | the seat's one reading of time, injected: monotonic for a held line's silence and a DHT query's deadline, the workspace for a BEP 44 sequence (§4.40). The DHT reads the system's always — its waits are on a real socket — and only its own suite steers it (bl-7b83). | ~50 |
| `src/channel/ladder.rs` | the four rungs a dial climbs — held line, direct address, then the commons' two — the knobs the roving rungs turn, and each climb's transcript handed to `say` (§4.40). | ~165 |
| `src/channel/ladder/commons.rs` | rungs 3 and 4, the ones that touch the DHT: the re-call from cached presence, else presence read; either way the call written — the observed address beside the local ones — and the punch, each step said, and the cache kept only for a call that landed (§4.40). Split from the ladder at the budget. | ~165 |
| `src/channel/say.rs` | what the rendezvous path says on stderr (yog REMOTE §13.4, "The operator's view of the loop"): every line built here, families and counts and never an address; a transcript identical to the entry's last is not said again; the sink injected (§4.40). | ~205 |
| `src/channel/line.rs` | one line to one server: what a rung handed back, the ping discard (counted, so it can be said), the liveness check and why it failed, and what is kept of it between asks (§4.40). | ~180 |
| `src/channel/rendezvous.rs` | the two files an entry may carry beside `ca.pem`, and the four HKDF derivations both ends must agree on (§4.40). | ~130 |
| `src/channel/rendezvous/item.rs` | presence and the call: the sealed byte format, mirrored from the server and pinned to its bytes (§4.40). | ~140 |
| `src/channel/rendezvous/punch.rs` | the simultaneous open from one port, and the box's route-local addresses (§4.40). | ~190 |
| `src/dht.rs` | the seat's DHT client root: the walk's parameters, the client, the id it queries as, and the vote on where the commons sees it (§4.40). | ~150 |
| `src/dht/bencode.rs` | bencode, canonical out and strict in. | ~180 |
| `src/dht/krpc.rs` | the KRPC query shape and the two datagrams read back, a reply's BEP 42 `ip` among them; compact address parsing. | ~135 |
| `src/dht/mutable.rs` | BEP 44's signed mutable item: sign, verify, the target, the exact bytes a signature covers. | ~145 |
| `src/dht/lookup.rs` | the iterative walk: the bootstrap as a door asked `find_node` and re-asked when dry — only at the addresses that answered (yog bl-f519) — the window refilled on every event, bounded twice over against a hostile commons. | ~130 |
| `src/dht/frontier.rs` | a walk's state — every node heard of, asked, replied, and the door addresses that answered — and the frontier read off it: the K closest live nodes, the next to ask, whether any is in the air. | ~140 |
| `src/dht/flight.rs` | the window of queries in the air, each with its own deadline read off the client's clock; waiting for one event at a time. | ~85 |
| `src/dht/items.rs` | the two BEP 44 verbs over the walk: `get` the newest verified item, `put` at the closest token holders, every one at once, and a walk that found no holder its own error, sending nothing (yog bl-f519). | ~70 |
| `src/dht/transport.rs` | the datagram seam: one trait, and the std UDP socket that fills it. | ~75 |
| `src/dht/tests/fake.rs` | a fake DHT node on loopback UDP, with a store the test can read and a mood for each way a node misbehaves — and the one rule for which queries a mood leaves unanswered. `cfg(test)`. | ~230 |
| `src/dht/tests/steered.rs` | a walk whose silence the fakes declare: every answer owed is waited for, and the client's clock moves only past silence, so a deadline under test never bets on the box's load (bl-7b83). Every DHT suite walk that can wait runs on it (bl-73f2). `cfg(test)`. | ~80 |
| `src/dht/tests/fake/wire.rs` | the fake node's datagrams: a reply, an error, a BEP 42 claim, compact routing. `cfg(test)`. | ~55 |
| `src/render.rs` | the rendering (§4.37): the two forms, the one place a reply stream becomes text — a frame read, then rendered, refused or named unreadable — and the held read's frame, rendered against the fold its follower holds. | ~140 |
| `src/render/parts.rs` | the vocabulary every rendering is built from, and where the present/absent branches live so they are not written forty-one times. | ~115 |
| `src/render/answer.rs` | the one dispatch — an answer, and the family that renders it. A kind added to the census is a match arm missing here. | ~85 |
| `src/render/workspaces.rs` | the roster, the decision queue and the live tail — plus the line a watch ends on, which is the seat's own narration rather than a frame's rendering and rides the same gutter (§4.37). | ~210 |
| `src/render/workspaces/convs.rs` | the conversation index, which is the one of the four whose listing has a RULE and not just a row shape: the fold to roots, and the subtree named on its root's line (§4.11, bl-96cd). | ~110 |
| `src/render/chat.rs` | the conversation as prose: its transcript, and its own whole row with the server's offers on it. | ~165 |
| `src/render/records.rs` | one conversation's records: its steps, its worktree, its inbox, its spine, and the commit governing it. | ~155 |
| `src/render/step.rs` | one step drilled into — its captured documents printed as what they are, never parsed. | ~75 |
| `src/render/policy.rs` | what a workspace's policy is written in: its roles, its lineages, one config file, and the machines registered to it. | ~100 |
| `src/render/tasks.rs` | the balls, the board, one workspace's own balls and the delivery attempts. What those attempts CHANGED split out at the cap: a listing of rows and a diff are two shapes. | ~120 |
| `src/render/work.rs` | the work diff as a diff (§4.37, bl-2e1d): the two commits a comparison resolved to, the totals folded over the churn, and the mark an elision earns. | ~145 |
| `src/render/reads.rs` | the three reads whose subject is a server rather than a workspace: its verb table, a search, the trail. | ~70 |
| `src/render/acts.rs` | what an act came back saying: a captured run, a staged start, a sign-in, and the receipts that carry one fact. | ~165 |
| `src/reply.rs` | the reply vocabulary's module list, the three outcomes one frame can be, and the four-rung decode policy stated once. The census split out at the cap (`reply/kinds.rs`) on the seam this row used to name. | ~180 |
| `src/reply/kinds.rs` | **the one census** — every kind this window draws, and the captured run three of the ops come back as. It moves every time a pane lands, where the policy beside it almost never does. | ~180 |
| `src/reply/read.rs` | reading one frame — the dispatch off `kind`, and the refusal that wears none. | ~75 |
| `src/reply/fields.rs` | the strict field readers — rung 1, in one place, every refusal naming its field. | ~110 |
| `src/reply/roster.rs` | the workspace enumeration and how current it is. | ~145 |
| `src/reply/convs.rs` | one workspace's conversation list, and the two token fields rung 3 lives on. | ~160 |
| `src/reply/transcript.rs` | the conversation's entries — the envelope of one, and which origin wrote it. | ~155 |
| `src/reply/transcript/blocks.rs` | what one model entry says: the canonical blocks, and the provider's own counters. | ~115 |
| `src/reply/start.rs` | the start family's three receipts: the staged body carried whole, one of those per candidate of a spread (§4.36), and the minted name. | ~110 |
| `src/reply/enrolled.rs` | a new box's material, and the one envelope a camera carries it in — the six fields and the optional rendezvous pair spelled once, read and re-said. | ~175 |
| `src/seat/enroll.rs` | the §8.4 act from argv: one gesture, and its envelope said as a symbol, as a line, and — where the operator names a destination — as an entry. | ~135 |
| `src/seat/enroll/entry.rs` | the envelope decomposed into §4.6's four files — six with the rendezvous pair, under `rendezvous`'s names — under `material`'s own names, created narrow and never over material already there. | ~110 |
| `src/ui/enroll.rs` | the enrollment pane: a name, a grade, and the symbol that comes back — the one pane that covers another. | ~140 |
| `src/ui/enroll/symbol.rs` | the symbol as geometry (bl-5e0e): the module pitch, which is a whole number of device pixels and as many of them as the pane has room for, and the one mesh every dark module is drawn in rather than a feathered rectangle apiece. | ~120 |
| `src/qr.rs` and `src/qr/*` | a QR symbol drawn by this crate: the field, the tables, the zigzag, the four scoring rules, and the terminal rendering. Seven files, none over 250. | ~250 |
| `src/ui/model/enroll.rs` | an enrollment between the control that opened it and the symbol it ends at, and the only secret this window holds. | ~135 |
| `src/reply/stream.rs` | the live tail's fold: the prose that accretes, the tool window beside it, and the append a follower answers a frame with. | ~195 |
| `src/reply/stream/window.rs` | the tool window (REMOTE §5.5): two transitions a call, merged onto one call by the id both carry, where the exit code's presence is the status. | ~95 |
| `src/reply/steps.rs` | the steps a conversation's loop has taken: one strict row per step, the nested spend, and the class tokens carried verbatim with `"none"` the one word the pane reads (§4.18). | ~135 |
| `src/reply/files.rs` | what a conversation's worktree holds: the listing whose absence is a fact, the bounded preview's three classes plus the rung-3 word, and where the work lands (§4.18). | ~125 |
| `src/reply/rail.rs` | the conversation's spine (§4.29): the notches, the two absences each is read as a pair, and the label derived off the commit that stores it. | ~170 |
| `src/reply/governing.rs` | which config commit a conversation resolves its policy from (§4.29): one enum rebuilt off one key, and the server's own two wordings rather than a third. | ~120 |
| `src/reply/agent.rs` | the conversation's own row, whole (§4.32): twenty-one facts, every optional one read as an absence, the priced figure read by the ball pane's own reader rather than respelled, and the context percent carried rather than divided out again. | ~240 |
| `src/reply/step.rs` | one step's records (§4.32): the record vocabulary, the capture-log vocabulary beside it, and the parsed tree that is deliberately not decoded. | ~155 |
| `src/reply/inbox.rs` | the undelivered mail (§4.32): the forgiving parse read as a reading, and the two facts only a result message states. | ~85 |
| `src/verbs.rs` | the typed gesture surface: which words exist, which module each row lives in, and what the table is for. | ~155 |
| `src/verbs/verb.rs` | what a verb IS: the row, the usage it computes, the flags its own word can raise (§4.10, bl-9fd1), and the two doors onto the one envelope it becomes — the second stating an optional string field, for the one request that carries one (bl-971c). Split from the surface at the workspace, because the two change for different reasons. | ~160 |
| `src/verbs/rows.rs` | the reads, the deposit and the advance, as data — each with the typed door the window composes by name — beside the one table that enumerates every row wherever it is declared. | ~240 |
| `src/verbs/enroll.rs` | the enrollment: its row, `address` — the one optional field a request on this surface carries (§4.15, bl-971c) — and the door both faces compose it through. One subject, one file: its reply is the only one this seat renders rather than prints. | ~70 |
| `src/verbs/conversation.rs` | the conversation's own four acts as rows — the cut, the kill, the change of lineage and the unmaking. Here and not in the exemption ledger because every one of them answers a captured run, which is a kind this seat already paints. | ~110 |
| `src/verbs/queue.rs` | the decision queue's three ops as rows — the fan that names no workspace, the raise and the answer, the last of which replies with a queue rather than a receipt (§4.19). | ~90 |
| `src/verbs/trail.rs` | the trail's read as a typed door rather than a row, its bound being a number, and the depth BOTH faces ask with stated once (§4.27, §4.10) — beside its two acts, which carry no field at all and are therefore rows (§4.35). | ~100 |
| `src/verbs/records.rs` | the conversation's records as rows — the steps ledger and the worktree listing, each with its typed door, admitted by the same test the four acts passed once §4.18 decoded their kinds. | ~65 |
| `src/verbs/spine.rs` | the conversation's spine as rows — the notches and the governing commit — beside the `fork` door that shares their subject and cannot be a row, because it carries a list (§4.29). | ~115 |
| `src/verbs/candidates.rs` | the n-attempt path's three envelopes — the spread, the acceptance and the release — as doors and never rows, because argv has no channel selector for a gesture naming no workspace (§4.36). | ~105 |
| `src/verbs/capability.rs` | the boundary's three acts as rows — the parked call's answer with its three verdicts, and the floor's two directions, neither of which can be refused (§4.34). | ~110 |
| `src/verbs/workspace.rs` | the workspace's own three acts as rows — the unmaking, whose `typed` is an arming rather than a parameter (§4.20), and the pin pair, which are assertions rather than a toggle (§4.25). | ~100 |
| `src/reply/help.rs` | one server's own verb table (§4.21): five required strings a row, the classification carried verbatim, and the headline the pane paints. It is the same shape `src/snapshot/parity/roster.rs` reads the parity roster off. | ~85 |
| `src/reply/search.rs` | what a needle found (§4.21): the four facts about a match, the four address fields that are optional because a hit is one of three shapes, and the unreadable list that is a different claim from finding nothing. | ~135 |
| `src/verbs/window.rs` | the window's own two ops (§4.21) — the server's verb table, which has no argv row because its word is `lernie help`'s, and the search, which does. | ~80 |
| `src/verbs/prices.rs` | the money ops (§4.41): `prices` as a row, `price` and `ceiling` as typed doors because their figures are numbers that may be absent, and the one reading of a typed rate or bound as a JSON number. | ~120 |
| `src/reply/prices.rs` | the price table, the bound, the world's spend against it and what a ceiling act woke (§4.41) — rates held as JSON numbers so they read back as written. | ~125 |
| `src/cli/prices.rs` | `price` and `ceiling` as argv spells them: a word that is not a number refused here with the door's usage, and the trailing `--on <channel>` read off. | ~75 |
| `src/ui/prices.rs` | the prices pane (§4.41): each channel's table, its bound and spend, and the acts addressed down that channel, each disabled with its reason. | ~150 |
| `src/ui/model/pricing.rs` | the prices pane between frames: the per-channel tables, the four draft boxes, the roster check and the four acts. | ~180 |
| `src/ui/commands.rs` | the commands pane (§4.21): what this window takes first — the menu's rows and the keys read off `keys::BINDINGS` (§4.46 item 3) — then one section per channel of what that server answers to, each row's line, sentence, page and classification. | ~110 |
| `src/ui/find.rs` | the find pane (§4.21): the needle, the act that is disabled until there is one, the hits, and the standing sentence saying why none of them can be aimed at (yog bl-ef16). | ~150 |
| `src/ui/model/window.rs` | the window's two panes between frames — which one stands as one field rather than two flags, the per-channel filing both share, the needle that is not spent on firing, and the roster refresh. | ~175 |
| `src/reply/doctor.rs` | whether this box is wired (§4.21): one row per check — three required fields and the remedy, optional and verbatim — the headline both faces paint, and the count of failing rows that is the only tally. | ~75 |
| `src/verbs/doctor.rs` | the doctor's envelope (§4.21) — a door, because its one workspace is optional: bare it fans, named it is routed. | ~35 |
| `src/ui/doctor.rs` | the doctor pane (§4.21): one section per channel, one row per check in the colour of its state, the remedy under a row that does not hold. | ~95 |
| `src/ui/model/doctor.rs` | the doctor pane between frames — the ask, bare or carrying the aimed workspace, cleared on asking, and each channel's checks replacing its own section. | ~70 |
| `src/verbs/tests/corpus/emits/ledger.rs` | the frames this seat's encoder cannot compose, by op, count and reason — split from the mechanism at the cap, because a builder changing and a surface not being built are two different reasons to edit. | ~120 |
| `src/verbs/start.rs` | the start family's two envelopes — doors without rows, and why — the two rungs a `dir` chooses between, the join that fires a rung's prefill ahead of what was typed (§4.10, bl-4371), and the born-on role's field and the one word plan mode is (REMOTE §9.21). | ~175 |
| `src/verbs/doors.rs` | the words whose grammar the gesture table cannot express: a word, a usage line and prose, with no envelope builder behind it. | ~190 |
| `src/verbs/help.rs` | the two rosters and one word's page, answered with no server up. | ~110 |
| `src/ui.rs` | the window's module list and what a frame may not do. | small |
| `src/ui/model.rs` | the model's module list and re-export surface. Four pieces split out at the cap onto seams this row used to name: the door a reply comes in through (`model/absorb.rs`), the aim (`model/aim.rs`), the accordion (`model/servers.rs`), the records pane's seven answers held as one value (`model/records.rs`'s `Records`), and the struct itself (`model/held.rs`). | ~95 |
| `src/ui/model/held.rs` | **what the window holds between frames** — the snapshot a frame reads, and every field documented where it is declared. | ~285 |
| `src/ui/model/held/asked.rs` | the two questions asked of the model that no pane owns: how many rows the composer's field stands at (§4.39), and whether a keyboard walk owes its selection a place on the glass — which since the fold asks no pane, there being one list. Split from the file above at the cap when the drag added the second — a field is what a pane learned to hold, a question is what two panes need one answer to. | ~35 |
| `src/ui/model/aim.rs` | which workspace the window is aimed at — the address every composed gesture is built from — and the two readings taken of its channel: the one a gesture goes down, and whether this box holds one by that name at all. | ~50 |
| `src/ui/model/fold.rs` | **one server, every name it is held under** (§4.6, bl-13b9): the roster folded by the address each channel dials — the server the seat card, the switcher and the seat-wide waiting count read, its lead, its workspaces once each, and which channel a gesture at a workspace goes down. A query over the roster, never a second one. | ~140 |
| `src/ui/model/servers.rs` | **the accordion** (§4.39): which server is open, what each server's last opening ranks as, the workspace last aimed under each, the three derivations over them — which one is open on a seat nobody has told, the order the pane paints, and what an opening aims at — and the `+`'s own act, which opens a server, clears the selection and asks for the composer's box. The seat's own and in the place file, never across the boundary. | ~170 |
| `src/ui/model/absorb.rs` | **the one door a reply comes in through**: what is filed, what becomes the notice, and the act's receipt — the same door knowing which act it answers (§4.26). | ~230 |
| `src/ui/model/absorb/unanswered.rs` | the leg that brought no reply at all: a channel this seat could not reach, said on its own section, and an act that earned no answer, said in the bar — with its words given back where nothing crossed (§4.26). | ~100 |
| `src/ui/model/notice.rs` | what the seat last heard that was not content: six kinds — five failures and one act's receipt (§4.34, §4.36) — and the line that says whose sentence each is. | ~170 |
| `src/ui/model/posted.rs` | a gesture on its way out: whether a lost reply leaves it in doubt (§4.22), and the channel it is addressed to where its envelope names no workspace (§4.30) — both recorded at the control because neither can be computed. | ~110 |
| `src/ui/model/delete.rs` | **a conversation's deletion** (§4.44 item 1; §4.42 item 5; bl-e837): the name the server compares, the confirmation's two answers, and `post_delete` — the one composition of `delete-agent` the trashcan and `/delete` share. | ~60 |
| `src/ui/model/acts.rs` | what a control does, whichever control did it — the one home a binding and a click share: the aim, the selection, Escape's ladder, the enrollment's four and the workspace's pin. | ~220 |
| `src/ui/model/channel.rs` | what a channel is, what a gesture aimed down one must be addressed as, and what its section says when it has no workspaces. | ~110 |
| `src/ui/model/start.rs` | a start between its two acts: what is held, what each receipt does to it, the born-on role written onto the body the fire carries back (§4.39), and the refusal that retires it with the goal back in the box (§4.26). | ~255 |
| `src/ui/model/start/spread.rs` | the same start with n in the middle (§4.36): the obligation it is over, the fan its staging receipt composes, and the one fire per candidate the fan's own answer does. | ~130 |
| `src/ui/model/claim.rs` | the claim a start leaves on the selection: the row it stands in for, what is not asked about it, and the answer that spends it. | ~130 |
| `src/ui/model/subtree.rs` | what hangs under a conversation, and whether it is on the glass: the descent fold `Model::rows` applies, and the set of what an operator has opened (§4.11). | ~85 |
| `src/ui/model/tabs.rs` | **the tabs between frames** (§4.44 item 2): `Tab`, the strip's query, the open `Model::select` performs, the close onto a neighbour or the workspace, and the two ways the server takes one — a listing that no longer knows it, a selection it put down — plus a start's claimed name renamed to the root it resolves to. | ~125 |
| `src/ui/model/sent.rs` | **a sent message between frames** (§4.45 item 4): `Sent`, the one door both deposits go through that records what it composed beside the act, the selected conversation's records as a query, and the three endings decided where each fact arrives — the receipt that changes the ink, the read that carries the words and retires the record, the refusal or unsent act that drops it with the words back in the box ahead of the draft. | ~170 |
| `src/ui/roster.rs` | **the window's one list** (§4.39): the servers this seat reaches as an accordion, the open one's workspaces under it and the aimed workspace's conversations under their own row — the word on the glass, the sections, what each says when it has none, and the header naming the address it dials. The strip of window-level acts split out at the design-time budget (`roster/acts.rs`), one workspace's own row with it (`roster/workspace.rs`), and the server's own row and order with them (`roster/server.rs`). | ~190 |
| `src/ui/roster/server.rs` | **one server's row and the window's cursor track** (§4.39): the row an operator clicks to open one (the `+` that stood at its right edge is `+ new` under each workspace, §4.42 item 7), and the three kinds of stop the walk makes — a row it only stands on (a server's, or a begin: `+ new`, `+ new workspace`; bl-f991); a workspace under the open one, which it aims; and a conversation under the aimed workspace, which it selects. The track is a query off the same rows and the same order the paint draws, so a key cannot walk where a click cannot reach. | ~180 |
| `src/ui/roster/workspace.rs` | one workspace's row: the line it wears, the row this seat holds no name for, the eight per-workspace controls that hang off the aimed one — the pin among them, whose word and op follow the row's own rank (§4.25) — under those, the aimed workspace's conversations (§4.39), and `+ new` last under every workspace (§4.42 item 7). | ~150 |
| `src/verbs/tuning.rs` | the role-tuning family: the `roles` read and the `model` assignment as rows, and `effort` and `priority` as doors without rows — a nullable level and a bool are not named strings. | ~155 |
| `src/reply/queue.rs` | the decision queue's rows: what is asking, why, the flag somebody raised on it and the invocation parked at its boundary — three nullable facts each read as an absence (§4.19). | ~165 |
| `src/reply/ops.rs` | the trail's rows: what crossed the boundary, how it ended in the server's own words, and where its alarm stands — a classification that crosses so no seat re-derives it (§4.27). | ~95 |
| `src/reply/roles.rs` | what one workspace's roles are set to: four required fields and the effort, which reports rather than asserts and so is an option carried verbatim. | ~80 |
| `src/ui/tuning.rs` | the tuning pane: what a workspace's roles are set to, the four seats and one toggle that retune each, and the assignment editor under its own row. **The settings surface `src/snapshot/reach.rs` was written to say this seat did not have.** | ~190 |
| `src/ui/model/tuning.rs` | the tuning pane between frames — a two-state enum rather than a flag beside an option — and the four acts its controls spend. | ~190 |
| `src/ui/tuning/offers.rs` | what a provider row offers, on the tuning pane (§4.17): the error-accent mark on a row whose model its provider's listing lacks, and the offered ids under the editor, each filling the model field. Split from `tuning.rs` on the read's seam — `roles` there, `models` here. | ~55 |
| `src/ui/model/tuning/offers.rs` | the offerings between frames (§4.17): which provider rows the pane asks, each stamped answer filed under its row, and the one fact a listing in hand supports — absence is not evidence. | ~70 |
| `src/ui/queue.rs` | the decision queue (§4.19): the union across channels, a section a server (bl-52d2), every line a row can carry with the flag leading, the acknowledgement, the choices `ask::choices` offers a held row — the three verdicts, and `pass` at each wider reach the row permits, each naming its reach for itself (§4.34, PROTOCOL 18; bl-da16) — and the way out to the conversation. Its beats split at the cap on the subject's own seam — `tests.rs` is what the pane says, `tests/answer.rs` the one act whose subject is the invocation behind the row. | ~250 |
| `src/ui/clear.rs` | the place a trail is cut in (§4.35): the servers the cut reaches, named; the way out first; and the act last, with no arming because the wire offers no field to arm on. | ~100 |
| `src/ui/trail.rs` | the trail (§4.27): the union across channels, what ran and how it ended in the server's words, and the standing that is silence for a clean run and its own word for every other. | ~150 |
| `src/ui/records.rs` | the records pane (§4.18): the frame, the steps half and the files half, every empty state its own sentence, every line a pure function beside the paint. Its five other halves are files of their own (§4.29, §4.30). | ~275 |
| `src/ui/records/spine.rs` | the pane's third half (§4.29): the governing commit, the notches, the cards off them, and the one fork control an operable notch carries. | ~180 |
| `src/ui/records/workflow.rs` | the workflow mark on the governing half (§4.29, REMOTE §9.24): its three renderings, one `workflow` control per lineage the workspace holds, and the clear where a mark is read. | ~85 |
| `src/ui/records/steps.rs` | the steps half: the orphan banner, one row per step with its drill-in, and the step's strings. Split from `records.rs` at the cap. | ~150 |
| `src/ui/records/files.rs` | the files half: the worktree's listing, its preview in each of its four classes, and the sentence for a listing cut short. | ~75 |
| `src/ui/records/header.rs` | the pane's header (§4.32): the conversation's own row, said several facts to a line because the pane has to fit the window. | ~215 |
| `src/ui/records/cascade.rs` | the pane's one act on the conversation itself (§4.20, §4.32): the stop that takes the subtree, its arming, and the one site the boundary's `children` field has in this window. | ~95 |
| `src/ui/records/drill.rs` | one step's records under the row that addresses them (§4.32): the control on the row, and the bytes rather than a tree. | ~155 |
| `src/ui/records/mail.rs` | the undelivered mail (§4.32): the deposit's header with each unstated fact said as itself, and its body. | ~90 |
| `src/ui/model/queue.rs` | the queue between frames — a flag, the per-channel filing, its fold by server (`sections`, bl-52d2), and the roster lookup that is the one place a row's address is resolved (§4.19). | ~135 |
| `src/reply/spend.rs` | what a ball has cost and what the sum is over: the money as upstream rendered it, and the attribution that says the classification and the clause because the clause alone cannot say it (§4.28). | ~80 |
| `src/reply/board.rs` | the fleet board: every live ball in its column, the column and the binding as two facts, and the armed loops — which ride here because there is no `fleet` read on this wire and are absent rather than empty (§4.31). | ~180 |
| `src/reply/balls.rs` | the balls themselves — the whole box's binding table and its three absences, one workspace's own balls with the figure that is required there and nowhere else, and the branch a workspace tracks its tasks on (§4.31). | ~110 |
| `src/verbs/balls.rs` | the balls family as rows: four reads — two naming no workspace and two naming one, a division read off the parameters rather than listed twice — and the three acts that claim, release and deliver a ball; `marks` is here as its READ, the amending form being a write with a confirmation to design (§4.31, §4.35). | ~175 |
| `src/verbs/balls/edit.rs` | the two authoring acts, which can never be rows: `create`'s body and each of `update`'s three may be ABSENT, and an empty string asks upstream to blank a field nobody touched (§4.35). | ~105 |
| `src/ui/board.rs` | the ball pane (§4.31): the union across channels, the aimed workspace's section between its two halves, the two emptinesses that are the union's rather than the workspace's, and the claim offered on a row nobody holds (§4.35). | ~200 |
| `src/ui/board/acts.rs` | the authoring block: one block with two subjects, the way out first, and the enablement that arms the one act with no undoing (§4.35). | ~205 |
| `src/ui/board/rows.rs` | the words a board row wears — every line a pure function of the row, the loop's line being the server's own with the two facts it omits hung off it. | ~135 |
| `src/ui/board/workspace.rs` | the aimed workspace's half: what it holds, what each cost, the branch it tracks on, four emptinesses because they are four different claims, and the two controls that open the authoring block (§4.35). | ~100 |
| `src/ui/model/board.rs` | the ball pane between frames — the two per-channel unions, the workspace's two answers, and the retirement that takes the workspace half and leaves the pane (§4.31). | ~120 |
| `src/ui/model/board/acts.rs` | what the model does with the five acts: the `--as` stamp read off the channel, the two openings, and the one door that sends down one channel (§4.35). | ~130 |
| `src/ui/model/board/block.rs` | the block itself — what it holds, and the four gestures whose absence IS the enablement of the control that spends them (§4.35). | ~155 |
| `src/reply/diff.rs` | what one attempt changed — the row the work diff answers with and the same row nested in every science attempt, one type because upstream has one encoder; binary read off the shape, and the patch deliberately unread (§4.32). | ~115 |
| `src/reply/science.rs` | every delivery attempt of one workspace: what it was fired with, what it cost, what was said to it and how it ended — the outcome a token with whatever that token can say (§4.32). | ~155 |
| `src/verbs/fleet.rs` | the fleet family — six rows and one door, the door because a cap is a number; and the file it is written in names no reply at all, because two families share one (§4.32). | ~150 |
| `src/ui/fleet.rs` | the fleet pane (§4.33): five acts in the unmaking's order, three boxes and the two controls each holds down, and the two standing listings under them. | ~205 |
| `src/ui/fleet/candidates.rs` | the n-attempt path on the listing that already names every value it takes (§4.36): which act a row earns, the two words and the count the three spend, and the acceptance addressed down one channel. | ~180 |
| `src/ui/fleet/rows.rs` | the words a fleet row wears — the receipt that names its op, an attempt's lines and no others, and the churn read off the shape. | ~125 |
| `src/ui/model/fleet.rs` | the fleet pane between frames (§4.33) — three words for its own acts and two more for the candidates' (§4.36), the receipt filed under the op that earned it, and the retirement that takes the pane and both its answers with the workspace. | ~175 |
| `src/ui/model/trail.rs` | the trail between frames — a flag and the per-channel filing, on the queue's own terms, with the read standing because a trail is what is happening (§4.27). | ~70 |
| `src/ui/model/records.rs` | the records pane between frames: **the seven answers as one value** (`Records`), its open/close acts, the retirement with its subject, and the one `covered` question every pane shares. | ~140 |
| `src/ui/model/spine.rs` | the pane's one draft (§4.29) — the two words a fork is composed from, why the goal is spent on firing and the role is not, and the act that spends them. | ~85 |
| `src/ui/model/deep.rs` | the one read this pane posts rather than stands, and why the answer's own `seq` is the whole of what says which row it belongs to (§4.32). | ~55 |
| `src/reply/config.rs` | one config file as the typed thing it is: the bytes, the settings the schema found in them, and the bounds that ride a control (§4.30). | ~130 |
| `src/reply/lineages.rs` | the config lineages one workspace holds — the listing the file read indexes into, with the tip in both spellings (§4.30). | ~70 |
| `src/reply/proposals.rs` | what a reviewer has staged for one workspace's config, and — where the read named one — that proposal whole (REMOTE §9.22, PROTOCOL 18). `fresh` crosses as the server's own reading and is never inferred here from an empty lineage list. | ~130 |
| `src/reply/clients.rs` | the machines registered in one workspace: presence as an observation, the advertised set as a statement, and the consent whose absence is a reading (§4.28). | ~110 |
| `src/reply/providers.rs` | what a workspace can sign in to, and what one row offers: the four required fields, the block whose absence is the whole of *signable*, and the two capability booleans (§4.24). | ~110 |
| `src/reply/login.rs` | one sign-in run as the server streams it — both tagged streams, the fold a frame is an append onto, and the two settled facts whose absence is a reading (§4.24). | ~115 |
| `src/verbs/proposals.rs` | the learning loop's two gestures: the listing at both its depths (the id is an optional string and rides the `stating` door), and the settle whose id and verdict are both required and neither defaulted. | ~100 |
| `src/verbs/config.rs` | the config family: the lineage listing as a row, and the one envelope the read and the write both are — the five places a config file lives, typed, and `text` the whole of what separates the halves (§4.30). | ~190 |
| `src/verbs/clients.rs` | the one op on the tool-host surface a seat is owed, and the four that are a machine's — with why `invocations` must never be asked from here (§4.28). | ~65 |
| `src/verbs/login.rs` | the sign-in family as rows: the table, the offering, the act that starts a run in the workspace, and the lane that streams it — four ops, one subject (§4.24). | ~115 |
| `src/ui/config.rs` | the config pane: what can be read — the four destinations that are picked and the fifth that is typed (§4.30). | ~215 |
| `src/ui/config/settings.rs` | the typed view: every setting the server's schema found in those bytes, its bounds, and the server's own judgement of the value (§4.30). | ~50 |
| `src/ui/config/edit.rs` | the editor: the box the new bytes are typed into, the enablement that is its arming, and the sentence for a file that moved under it (§4.30). | ~90 |
| `src/ui/config/proposals.rs` | what a reviewer has staged for the workspace's config (§4.30, bl-a1d6): the listing in the server's own standing, the named proposal whole and verbatim, and the two verdicts armed by its id under §4.20. | ~180 |
| `src/ui/clients.rs` | the clients pane: the three empty states, the two lifetimes on a row, and the consent said on every tool (§4.28). | ~130 |
| `src/ui/login.rs` | the login pane: the workspace's two sentences, the provider table, and the two controls on a row. The followed run's half split out at the design-time budget (`login/run.rs`). | ~200 |
| `src/ui/login/run.rs` | what one sign-in printed: both streams, the settled exit and the run-by-hand command (§4.24). | ~70 |
| `src/ui/model/config.rs` | the config pane between frames — which file it is pointed at, the two reads that stand on it, the box, and the write it composes (§4.30). | ~195 |
| `src/ui/model/config/draft.rs` | the box and the two readings it takes against the server's answer: is there anything to write, and did the file go somewhere neither end of this box put it (§4.30). | ~75 |
| `src/ui/model/config/proposal.rs` | the staged proposals between frames — which row is named, the depth the standing read is asked at (the named id while the listing still carries it), the arming, and the verdict it composes (§4.30). | ~140 |
| `src/ui/model/clients.rs` | the clients pane between frames — the aim that gates it, and what it retires with (§4.28). | ~55 |
| `src/ui/model/listing.rs` | the three panes that are pure listings, and the one field that says which is standing (§4.28). | ~70 |
| `src/ui/model/login.rs` | the login pane between frames — two questions rather than two modes — the three acts its controls spend, and where the server is, read off the channel stamp. | ~165 |
| `src/state.rs` | **the link** (§4.12): what the frame and the off-frame threads say to each other, and the crate's one lock. `settle` is the frame's whole side of it, and it publishes two projections — what to ask, and what to keep (§4.13). Its second tenant is what worked for an entry this run — held lines, endpoints, the punch port (§4.40). | ~235 |
| `src/state/traffic.rs` | what crosses the lock — a worker's report in its kinds — among them a routed gesture's reply stamped with the op it answers (§4.26) and a `models` answer stamped with the provider row asked (§4.17) — and the standing question set the frame publishes, whose open pane is one field rather than a flag apiece (§4.12). | ~205 |
| `src/offframe.rs` | the four off-frame threads (§4.12): the one leg both fanning workers share, the filing every answer goes through, and the pump that is a cadence rather than a timeout. | ~120 |
| `src/offframe/asker.rs` | one pass of the standing set, in two halves: the reads whose subject is every channel, the nest under the focus, and the channel that costs only itself (§4.12). The selected conversation's reads lead every leg they have not yet been asked at (bl-1f22). The seam is the wire's own, so a pane asking at both widths appears in both halves rather than as a case (§4.31). | ~190 |
| `src/offframe/asker/workspace.rs` | the second half, split out at the design-time budget: every question that names one workspace, asked of the aimed workspace. It is where six panes' pane-keyed reads nest, so it is the half that grows. | ~150 |
| `src/offframe/poster.rs` | one pass of the outbox (§4.12): the gesture that is routed and its reply stamped with the op (§4.26) and with the channel it crossed (§4.7), the channels a workspace-less one goes down — the one it named, or every one where it named none (§4.30) — and which sentence a failed leg earns. | ~165 |
| `src/offframe/follow.rs` | one pass of the follow lane: the held read on the focused conversation, stamped with what it is about (§4.12). | ~90 |
| `src/offframe/signin.rs` | one pass of the sign-in lane: the held read on the followed provider row, stamped with what it is about (§4.24). | ~90 |
| `src/ui/unmake.rs` | the unmaking pane (§4.20): the workspace it would unmake, the refusal stated before the act, the arming box, and the act that is on the glass without being live until the name matches. | ~135 |
| `src/ui/model/unmake.rs` | an unmaking between frames — the subject it holds rather than follows, the arming that is a readiness test and is never spent, and whether it has been asked. | ~105 |
| `src/ui/convs.rs` | **the conversations that stand under a workspace** (§4.39): the rows the roster paints beneath the aimed one, the two emptinesses left once the pane became a level, the headline, the age, and `continues` — whether a thread goes on past a row, which is what the connectors are drawn from. The row itself is `convs/row.rs`, its acts `convs/menu.rs`. It stays beside the roster rather than under it because `ui/queue.rs` reads the same age and the same uncertainty mark. | ~155 |
| `src/ui/convs/row.rs` | one threaded row: the row with its state rule, the lines hung beneath it at its words' edge, the subtree control as a row under them, and the connectors — a rail per ancestor thread that continues, an elbow into the words. Its indent starts one step under the workspace it hangs from (§4.39), and every connector reads that one function. | ~200 |
| `src/ui/convs/menu.rs` | the conversation row's context menu (§4.23): the acts that fire on the row, the three that lead somewhere and spend nothing, and the admission test that separates them — a door taking the workspace and the conversation, and nothing else. | ~150 |
| `src/ui/model/fill.rs` | which box on the composer was asked for the cursor — the two parameter boxes a row menu names (§4.23), and the draft itself, which the server row's `+` names (§4.39) — and the one door that goes there. Taken once, by the frame that paints the box. | ~95 |
| `src/ui/chat.rs` | the transcript read by its gutter and its bubble (§4.42, §4.45): the glyph per speaker, each block on the fill its role wears, the turn's working gathered into one block on the trail's bubble, the `response` line while that block is open and the hairline between turns, and the sent row in the pending ink at the end. | ~180 |
| `src/ui/chat/scroll.rs` | a scroll the keyboard asked of the transcript (§4.46 item 3): the ask held in the toolkit's memory across the frame, spent inside the area as a page or an end, the area built unstuck and uneased on that frame, and how far down it stands written back for the suite. | ~70 |
| `src/ui/chat/working.rs` | the working in three stages (§4.45 item 5): the grouping's counted line as a door, each step whole while in flight or compact once finished — a call's line and one elided line of its output, a note one elided line — each compact step a door that opens it whole in place, the defaults from the turn's state and a toggle remembered. | ~160 |
| `src/ui/chat/trail.rs` | one step of the working, whole (§4.42 item 4, §4.45 item 5): its mark on its stem, its name — a door that shuts it where it was opened from compact — a result folded to its head, and an intermediate prose on its own fill inside the working's. | ~100 |
| `src/ui/chat/panel.rs` | the two cards at the head of the pane (§4.42): the conversation's facts, and the seat's — each server with what it holds, the version, the two ways in — where nothing is selected. | ~120 |
| `src/ui/ask.rs` | **the ask** (§4.42): the selected conversation's queue row above the rule while it asks — its words, its choices as numbered rows (the one list the queue pane paints too; the `pass` reaches follow the held row's `scopes`, bl-da16, `tests/scopes.rs`), faint while its server will not dial. | ~270 |
| `src/ui/ask/keys.rs` | the digit, the arrows and Enter an empty prompt lends the ask, and the field's hint while it asks (bl-1252). | ~75 |
| `src/ui/dock.rs` | **the live dock** (§4.42): what runs under the aimed workspace, one line — a door — above the rule or a row apiece when Ctrl+T unfolds it. | ~100 |
| `src/ui/chat/rows.rs` | one conversation as rows: every committed entry, projected, with the live fold painted in the `Streaming` entry's place and nowhere else (§4.9, bl-f6b5). | ~160 |
| `src/ui/chat/rows/live.rs` | the turn that has not settled — the two halves of the prose and the tool window under them, plus `half`, the rule both routes share and neither may copy. | ~105 |
| `src/ui/chat/rows/sent.rs` | what this seat sent, as rows (§4.45 item 4): `whole`, the pane as data — the committed rows and the live fold, then one operator row per standing record for the selected conversation, pending until acknowledged — which paints what stands and retires nothing. | ~40 |
| `src/ui/chat/rows/role.rs` | a row's role (§4.45 item 7): whose weight a delivered message is said in, the bubble a row is born on, and the pass over each turn's shape that makes prose a trail follows — or the running turn's — intermediate and marks the running turn's rows live. | ~80 |
| `src/ui/chat/code.rs` | fenced code in what was said (bl-beef): the split on fences, pure, and the block painted monospace on the surface, scrolled sideways rather than wrapped. | ~100 |
| `src/ui/chat/code/inline.rs` | the prose between fences (bl-f5a7): code spans and heading lines split, pure, and laid as one wrapped job with each run in its face. | ~115 |
| `src/ui/chat/fold.rs` | what a machine's answer hides when it is folded, and the two counts a reader chooses between. | ~90 |
| `src/ui/composer.rs` | what an operator types, and the gesture it becomes — one box, three subjects, and the one read of Enter both of the box's modes share (§4.39); a slash line is the menu's and not the deposit's (§4.42); a deposit is recorded as sent the frame it is composed (§4.45 item 4). | ~200 |
| `src/ui/slash.rs` | **the slash verbs** (§4.42): the rows this seat offers right now — the conversation's, the workspace's, the window's — each the word its control carried; what a typed line parses to; what firing a word does; and the completion menu over the box, a window of rows around the chosen one. | ~300 |
| `src/ui/overlay.rs` | **the one frame every pane floats in** (§4.42): the box at the bottom of the pane it is called in, the name line with the subject and *Esc closes* on it, and the order the panes are tried in. A child region, never an `Area`. | ~135 |
| `src/ui/composer/start.rs` | the half that begins a conversation rather than continuing one — the deposit's own field at the deposit's own rows, with `start` inside it (§4.39). | ~90 |
| `src/ui/keys.rs` | the keyboard: the table below run first, then the one cursor track's walk, which is the selection — over three kinds of row since the fold (§4.39) — and the one binding that is not a walk, Enter on the row the cursor stands on (a server's, or a begin), fired only while no control holds the keyboard. There is no `Pane`: one list needs no field saying which. The registry of boxes that take text split out at the cap (`keys/boxes.rs`). | ~200 |
| `src/ui/keys/bindings.rs` | **the bindings table** (§4.46 item 3): every key the window binds as one row — a *fired* row is a chord and the act it fires, run here from anywhere; a *told* row is a key read where it is read, in the table so the printed list is whole. `run` fires the rows, `/help` prints them, and `cited` writes a chord after a control's sentence. | ~280 |
| `src/ui/keys/acts.rs` | what each fired row does: one arm per act, each naming a door a control already fires — the tab step, the scroll ask, the scale knob moved through its own row. Split from the table at the cap. | ~110 |
| `src/ui/keys/boxes.rs` | **every box on the glass that takes text**, by the id it wears, and the gate that asks whether one of them holds the keyboard right now — a comparison against a named list and never egui's *is anything focused at all* (§4.23). | ~100 |
| `src/ui/shell.rs` | the layout: the bar, then one column (§4.42, §4.44, §4.45) — the transcript, the prompt directly under it, the dock and the rule under the prompt, the menu and the overlay over it, and what stands down under what. | ~60 |
| `src/ui/bar.rs`, `bar/drag.rs`, `bar/rows.rs`, `bar/foot.rs`, `bar/confirm.rs` | **the bar** (§4.44 item 1; §4.45 items 1–2): the left panel listing the aimed workspace's conversations, a glyph column until the pointer is over it, lingering 0.75 s after it leaves, pinned by its head, `+ new` beside the pin; its two widths, each dragged at its right edge (`drag.rs`: the seat, the floor and the ceiling, which width a drag writes held from its first frame) and remembered in the place; the aimed workspace's conversations as `convs::` spells them, each row's trashcan under the pointer (`rows.rs`; bl-e837) and the confirmation its click opens on the box (`confirm.rs`: `keep` and `delete`, Esc and Enter); and the workspace at its foot, with the workspace's dropup standing inside it. | ~215, ~130, ~110, ~60, ~85 |
| `src/ui/status.rs` | **the status rule** (§4.42): the state slot — the notice, the selected conversation's word and age, or `ready` — the facts between it and the name, which facts fit at a width, and the name at the right end; the model and the name as doors, each the menu row it fires (item 10). | ~250 |
| `src/ui/status/selectors.rs`, `status/diagnostics.rs`, `status/dropup.rs` | **the rule's line** (§4.42 item 11, §4.44 item 3): the four selectors and their marks, the workspace's run the bar's foot paints, this conversation's diagnostics and the aimed server's dial sentence, and a selector's dropup — its list over the segment, its keys and its picks. | ~120 each |
| `src/ui/status/waiting.rs` | **the seat-wide waiting count off the rule** (item 11, bl-4cac): leaving out a channel that will not dial, painted as the queue's door on the seat panel and at the switcher's head. | ~35 |
| `src/ui/model/dropup.rs` | **a dropup between frames**: which selector is open, its cursor, whether its read has landed, the read it posts and what a pick fires (`dropup/choices.rs`: what each lists). | ~200 |
| `src/ui/switcher.rs` | **the switcher** (§4.42): the roster as an overlay, opened by Ctrl+X or `/conversations`, kept open by the walk and closed by a choice. | ~55 |
| `src/ui/tabs.rs` | **the tabs** (§4.44): the strip over the transcript, one tab per conversation opened in the aimed workspace, filled by `Model::select` alone, each with its close. | ~110 |
| `src/ui/theme.rs` | **the visual language's tokens** (§4.38, `docs/STYLE.md`): the ground ladder, the ink scale, the six states and their accents, the wire's states read onto them, the speaker weights, and the spacing and type scales — every byte once. | ~250 |
| `src/ui/theme/bubble.rs` | the four bubbles by role (§4.45 item 7): the operator's the brand at half a tint, the trail on the ground inside a hairline, intermediate prose on the surface, final prose raised — derived from the palette, no literal. | ~50 |
| `src/ui/theme/glyph.rs` | the glyph an act or a gutter wears: every character a control or a transcript row spells, asserted to have a glyph in the body font. | ~60 |
| `src/ui/theme/paint/knob.rs` | the slider, the settings pane's number knob (§4.46 item 4): the one home of egui's slider, taking its help sentence; it steps on the arrows while it holds the keyboard — egui's own arrow moves a point of the run and rounds to nothing on a coarse step — and takes the keyboard on a press. | ~60 |
| `src/ui/theme/paint.rs` | **the anatomy as paint** (§4.38, `docs/STYLE.md` §5): the row with its state rule (`paint/row.rs`: one line, or a block of lines under one seat), the rail and elbow, the ruled block, the section and the field — every shape a pane puts on the glass that is not a bare label, written once — **and every control** (§4.46 item 2): the door, the button, the toggle, the seat and the slider each take the control's help sentence beside its word, attach it as the tooltip and, in test builds, record the seat for the instruments; `rules/no-bare-control.yml` confines egui's own control constructors to this directory. | ~160 |
| `src/ui/theme/visuals.rs` | the one adapter into egui: the tokens installed as a `Style` once per frame — which slot each fills, and the one stroke left on the glass. | ~90 |
| `src/ui/theme/fonts.rs` | **the fallback face** (§4.43): the fixed list of system paths a CJK face is kept at, and egui's faces with the first readable one pushed last in both families. | ~110 |
| `src/mark.rs` | the seat's own mark: the two inks, the three shapes, the two emissions — and where a desktop actually looks for one. | ~160 |
| `src/mark/shape.rs` | the three primitives, each answering one geometry two ways: is this point inside me, and what element am I. | ~145 |
| `src/mark/raster.rs` | the pixel loop: supersampling and nothing else. | ~120 |
| `src/qr.rs` | a QR symbol drawn by this crate: bytes in, a grid of modules out, and what it emits stated once. | ~130 |
| `src/qr/gf.rs` | GF(2⁸) and the Reed-Solomon check bytes — the field itself, with no lookup tables. | ~95 |
| `src/qr/version.rs` | how big the symbol has to be: the block table at correction level M, and the geometry that follows. | ~200 |
| `src/qr/bits.rs` | the payload as a bit stream: header, terminator, padding, blocks, interleave. | ~130 |
| `src/qr/matrix.rs` | the grid: the furniture a scanner finds it by, the zigzag, and the eight candidates. | ~250 |
| `src/qr/mask.rs` | the eight masks, and the four penalty rules that choose between them. | ~145 |
| `src/qr/block.rs` | the symbol on a terminal: two module rows a line, carrying its own paper. | ~75 |
| `examples/icon.rs` | `make icon`'s machinery — re-emit `assets/lernie.svg` from the generator that defines it. | ~30 |
| `assets/lernie.svg` | the mark, as the hicolor theme reads it. A derivation, pinned byte for byte. | derived |
| `assets/lernie.desktop` | the freedesktop entry: how a Wayland compositor finds the mark at all. | config |
| `src/paint_probe.rs` | **the one paint walk**, and its projections. `cfg(test)`. | ~160 |
| `src/snapshot.rs` | **the seat rendered off-screen**: the matrix's sizes, where a shot lands, and the one settled frame. Every size is judged — the width gate went with bl-dfda, the policy now answering at every width. `cfg(test)`. | ~120 |
| `src/snapshot/worlds/column.rs` | the column's own states — the ask, the dock, the tabs, the notice — split from `worlds.rs` at the cap. `cfg(test)`. | ~60 |
| `src/snapshot/worlds/covered.rs` | the covered states, one world per pane that stands over the conversation, each answered because an unanswered pane offers no control at all and `PARITY.md` §5 reads unproven as red. `cfg(test)`. | ~175 |
| `src/snapshot/worlds.rs` | the named world states the matrix photographs — the shapes the window takes with nothing covering it, the covered ones being `worlds/covered.rs`'s; built from the window fixtures rather than from a second set — the first-run one seeded the way `src/main.rs` seeds a roster, because an empty `Vec` of channels is a state no box reaches and no pane has a sentence for, and the accordion's one photographing a seat with one server open and another closed, which a one-server seat never is. The whole screen set is part of the parity instrument, which is why every pane's world is here and why each is answered. `cfg(test)`. | ~160 |
| `src/snapshot/reach.rs` | assertion (a): the walk to each of the seat's seven covered panes and back, asked of the accessibility tree. Two legs per pane, and three in the narrow shape — the length is the bound, and the bound is a fact about the shape. `cfg(test)`. | ~175 |
| `src/snapshot/blank.rs` | assertion (b): every rectangle the layout put content in, read off the rendered glass. `cfg(test)`. | ~145 |
| `src/snapshot/clipped.rs` | assertion (c): no control laid out wholly off the window, and none offered without a rectangle. `cfg(test)`. | ~70 |
| `src/snapshot/told.rs` | assertion (e): every painted control tells (§4.46 item 2) — the pointer put at its centre and held still past the delay, and egui asked whether that seat's tooltip opened. `cfg(test)`. | ~100 |
| `src/snapshot/lit.rs` | assertion (d): every painted control lights under the pointer (§4.45 item 6; the subject widened from the tagged set by §4.46 item 2) — the pointer put at its centre, the fills inside its rectangle read off the one paint walk and compared with the frame at rest; what is not judged is a fact the tree or egui states. `cfg(test)`. | ~135 |
| `src/snapshot/parity.rs` | **the interface-parity gate** (yog's `docs/PARITY.md` §5): the `act:` tags read off the same accessibility tree, and the four assertions over roster, inventory and ledger. `cfg(test)`. | ~125 |
| `src/snapshot/parity/roster.rs` | which ops owe this seat a control, read off the vendored corpus's `reply/help` rows and decided nowhere here. `cfg(test)`. | ~80 |
| `src/snapshot/parity/exempt.rs` | `parity.toml`, and the strict subset of TOML it is allowed to be — no crate parses it. `cfg(test)`. | ~90 |
| `parity.toml` | the exemption ledger: one line per control-classed op this seat does not surface, each citing the ball that will build it. It records what is not RENDERED and never what is refused (§4.9). | config |
| `src/ui/act.rs` | the `act:<op>` token a control carries, written into AccessKit's `author_id` where no label can drift into it — and, under test, the bridge back from the tree's node id to egui's seat, which the lit assertion asks egui about. | ~110 |
| `src/paint_probe/frame.rs` | how a frame is produced: the offscreen input, the persistent window, the click. | ~120 |
| `corpus/` | yog's wire conformance corpus, vendored: `shapes.json` — every field path stamped with the edition it appeared at — `request/` whole, and the reply frames filed under `answers/`/`refusals/`/`unpainted/`/`unreadable/`. The directory a reply frame sits in **is** this seat's assertion; `corpus/README.md` is the contract. | docs |
| `.github/workflows/ci.yml` | the gate, run by a machine: the pinned tools, then `make ci`. Called by `release-plz.yml` on a push; triggered directly only by a pull request. | config |
| `.github/workflows/store-scan.yml` | the published store ref, judged by the source's own rule table. | config |
| `.github/workflows/release-plz.yml` | the release path (§6.2) and the proof of the seat's one artifact (§6.3). Its FILENAME is matched literally against the registry's trusted-publisher claim — renaming it stops publishing. | config |
| `release-plz.toml` | the four release decisions the workflow reads: the tag spelling, no generated changelog, no semver check across the fence, and the bump markers. | config |
| `scripts/mac-verify.sh` | reads the produced Mach-O and says what it IS — architecture, filetype, target OS, every `LC_LOAD_DYLIB`, whether it is signed at all — with five malformed inputs it must refuse first. | ~230 |
| `tests/packaged_files.rs` | what `cargo publish` would ship, over the real `cargo package --list` — the manifest's `include` allowlist, restated so a widening edit is red here. | ~200 |
| `scripts/refresh-corpus.sh` | the vendoring, from a yog checkout. It copies and sweeps; it never classifies. | ~90 |
| `src/test_support/corpus.rs` | the one walk over the corpus, the stamped shape record, and the protocol stamp checked on every file read. `cfg(test)`. | ~160 |
| `src/test_support/corpus/editions.rs` | projecting a frame back to an older edition and mutating a word in it — the two servers a seat meets that nobody can vendor. `cfg(test)`. | ~130 |
| `src/reply/tests/corpus.rs` | the replay, reply direction: every frame lands in the class its directory names, every upstream shape is classified exactly once, and an unpainted kind is refused BY NAME. | ~175 |
| `src/reply/tests/editions.rs` | the grows-only replay: every answer at every edition of this major, and every vocabulary word replaced by one no build has heard of. | ~110 |
| `src/verbs/tests/corpus.rs` | the replay, request direction, read half: every frame in the vocabulary decodes as a gesture and routes by the address its shape carries. | ~120 |
| `src/verbs/tests/corpus/emits.rs` | the write half: every frame this seat composes round-trips, and what it cannot compose is recorded by count and reason. | ~110 |
| `src/test_support.rs` | the scaffolding the suite shares and nothing production reads: the throwaway directory, a loopback address held dead for the process — bound, never listening, so no later bind is handed its port (bl-73f2) — and the two things that live here because the seat may not do them — mint a certificate, and listen. `cfg(test)`. | ~115 |
| `src/test_support/wire.rs` | a data root with a server standing behind one of its channels — the one fixture for both arrangements, the flat root and one entry. `cfg(test)`. | ~50 |
| `src/test_support/window.rs` | the window's fixtures: the rows a pane is built from, and the model at work every pane's suite clicks in. `cfg(test)`. | ~155 |
| `src/test_support/window/panes.rs` | the fixtures for the covering panes whose subject is a FOCUS — the aimed workspace, one of its rows — each open and answered. `cfg(test)`. | ~100 |
| `src/test_support/window/panes/records.rs` | the records pane's own fixture, split out at the budget because it is the one covering pane with seven reads under it. `cfg(test)`. | ~230 |
| `src/test_support/window/panes/union.rs` | the fixtures for the four whose subject is EVERY CHANNEL (§4.19, §4.21, §4.27): the decision queue, the verb table, a search and the trail. Split on the seam DESIGN draws, not at the cap. `cfg(test)`. | ~185 |
| `src/test_support/window/panes/fleet.rs` | the fleet pane's fixtures (§4.32) — its own file rather than the ball pane's neighbour, on the seam DESIGN draws: that pane is two widths and this one is one. `cfg(test)`. | ~130 |
| `src/test_support/window/panes/board.rs` | the ball pane's fixtures (§4.31) — the one pane whose subject is both every channel and the aimed workspace, so it is neither file's neighbour and gets its own. `cfg(test)`. | ~130 |
| `src/test_support/glass.rs` | **the one door to a GPU renderer in the test binary** (bl-f824): a `TestRenderer` held, created and dropped under one process-wide lock, because the system Vulkan loader is not safe across threads — one test's `vkEnumeratePhysicalDevices` `dlclose`d a driver library while another test called through it, and the whole binary died of SIGSEGV with no test named. `cfg(test)`. | ~50 |
| `src/test_support/mint.rs` | the operator's out-of-channel act, performed by the suite. **The crate's one spawn site.** | ~200 |
| `src/test_support/server.rs` | the stand-in server: a real listener, a real handshake, a real preface. | ~150 |
| `src/test_support/roving.rs` | the far end of a PUNCHED wire: a server serving a held line — pinging it and counting the pings, ending it every way a line can end — a reset or garbage only on the test's word that the answer was taken (bl-73f2) — or calling back what the seat wrote to its inbox (§4.40). `cfg(test)`. | ~255 |
| `src/test_support/clock.rs` | the suite's clock: an offset the test advances by hand (§4.40). `cfg(test)`. | ~60 |

**The three confined files, and two of them do not exist.** A confinement rule
names its one location *before* the first site is written, or the first site
picks the location by being written.

| Kind | Confined to | Rule | State |
|---|---|---|---|
| `unsafe` block or `unsafe fn` | `src/sys.rs` | `unsafe-outside-sys.yml` | **absent, and expected to stay so.** A seat dials a socket, frames JSON and paints; upstream's raw effects are all server-side. An ask to create this file is an ask to explain what the seat is now doing. |
| `Mutex` / `RwLock` | `src/state.rs` | `locks-outside-state.yml` | **occupied, by the tenant the rule was written for, and by one more** (§4.12, §4.40). The rule named this file before there was anything in it, and the off-frame threads filled it; the punched wire added the table of what worked for an entry this run, because a held connection is shared by every thread and outlives the channel that made it — the state this chokepoint exists to inventory, in the one file. |
| Building and forking a child | `src/test_support/mint.rs` | `no-bare-command.yml`, `no-bare-fork.yml` | **occupied, by test scaffolding.** The seat forks nothing in production, so the rule names the one place that does rather than an aspirational file nothing would ever go in. If production ever needs a child, the location moves to `src/spawn.rs` in the commit that writes the first site — never a second entry, because two confined files are two inventories. |

There is **no fork lock**, and that is a fact about this tree rather than a
disagreement with the hazard: ETXTBSY needs a write-then-exec pair, and the
crate's one child is `openssl` off `PATH`, which no test writes. The day a test
writes a program and runs it, the lock lands in the confined file with it — at
the fork, never at the write.

---

## 6. Publication: what was deferred, and what it became

This section was *Deferred, with the ball that pays for it*, and both its rows
have since been built — so it is now the record of how the crate reaches the
registry, kept here rather than deleted because the reasoning is what a later
editor needs and the section is where they will look for it. Each row still
says what it costs and what it still defers.

### 6.1 The first publish, which happened (bl-11fc, bl-f468, bl-3be4)

`publish = true` since bl-f468. What was ever deferred here was not the flag —
it was the apparatus that has to exist before a flag can be flipped safely, and
that landed with bl-11fc: `Cargo.toml` declares an `include` ALLOWLIST (never
an `exclude`, for the asymmetry AGENTS.md states), and `tests/packaged_files.rs`
holds it in both directions over the real `cargo package --list`. With no list
this crate packages 298 files, `scripts/leak-fixtures/` among them; with it, 116
`src` paths and four tree files.

The half no gate can hold — history, other refs, commit messages, repository
text nobody committed, Actions logs, published versions, and the fence — is
AGENTS.md's *Before a publish*, and bl-f468 ran it in full and recorded every
verdict. It stays a checklist rather than a target because every item is a
one-time judgement whose remedy is destructive. There is still no
`make publish`.

**0.1.0 is published** (bl-3be4, 2026-08-30). What had stopped it was never in
this tree: the registry crate `lernie` already carries the litany era's 0.0.x
line and is set to accept Trusted Publishing only, so a token `cargo publish`
is refused whatever the manifest says. Two owner acts opened it — relax the
setting, or land §6.2's workflow at the filename the crate's existing trusted
publisher already names — and the second is the one taken. The registry setting
stands; the route to it is the workflow, and there is no hand-run alternative
to fall back to.

The fence is now fixed in the public record: crates.io serves `lernie` 0.1.0
with the fence-stating README rendered on it, `published_by` null (no human
account published it — the trusted-publishing exchange did), tag `v0.1.0`, and
a GitHub Release beside it. §0's rule is no longer a claim about a future
version.

### 6.2 The release path, which now exists (bl-459d)

`.github/workflows/release-plz.yml`, `release-plz.toml`, and a
`scripts/mac-verify.sh` beside them. Four things about it are decisions rather
than mechanics, and each was made once:

**The CI gate is CALLED, never observed.** The release job's gate is
`uses: ./.github/workflows/ci.yml` inside its own run, with `needs: ci` on the
release job — not a `workflow_run` trigger watching CI from a distance. The
registry REFUSES an OIDC token minted under `workflow_run`, so the observed
shape publishes nothing, quietly, forever. `ci.yml` lost its push arm in the
same edit, so the gate still runs exactly once per push.

**The filename is fixed by the registry, not chosen here.** crates.io matches
`release-plz.yml` literally against the OIDC claim. Renaming it breaks
publishing until the registry entry is updated. There is no credential in this
repository and there should never be one — that is what trusted publishing buys.

**The tag is bare `v<version>`.** The sibling litany crate prefixes its tags
because ONE history there holds both eras of this name; this repository was
founded fresh for the seat, holds one crate, will never carry an litany commit,
and had no tags at all. The litany-era `lernie` tags live in the other
repository, where nothing this tree pushes can reach them. `release-plz.toml`
carries that reasoning beside the key.

**The macOS job reports; it does not gate.** bl-9380 measured that the seat
cannot be cross-built for darwin from Linux — the window links Apple
frameworks, they ship only in Apple's SDK, and the SDK agreement refuses both
hosting that SDK (§2.7) and running any part of it on non-Apple hardware
(§2.5) — and the operator ruled that a GitHub Actions macOS runner IS
Apple-branded hardware in the licence's sense. So the released tree is built
natively on `macos-14` and then READ rather than trusted: `mac-verify.sh`
reports architecture, filetype, target platform, minimum OS, every
`LC_LOAD_DYLIB` (each must be a stock `/usr/lib` or `/System/Library` path, or
the artifact acquired a dependency nobody chose) and whether a code signature
is present at all, with five fabricated malformed inputs it must refuse first.
It hangs off the release job's `releases_created` and checks out the tag that
job cut, so it runs once per published version and its verdict names one;
nothing `needs:` it, so a broken mac leg is visible and never stands between a
green tree and the registry. What it proves, and why it attaches nothing, is
§6.3.

**Nothing is still deferred here.** No container image is a ruling, §6.4;
signing and notarization are not needed, §6.3 says why.

### 6.3 The artifact is the crate, and no binary is attached to a release (bl-9380)

bl-9380 left one thing open after the macOS job landed: the artifact was built
and read on every push and attached to nothing. Four honest answers were on the
table, and three were refused on the record before this section was written:
cross-linking against SDK stubs in a Linux container (the licence, above);
a macOS runner leg as the artifact's one home, uploading what it built (below);
and something neither sibling repo needed (nothing was found — the seat's
difference from the foot and the server is the window, and the window is the
whole of what the licence reaches). **The fourth holds: `cargo install lernie
--locked` on the mac IS the artifact, and the repository ships none.**

Three facts make that the ruling rather than a shrug:

- **It is the seat's unit of install already, not a mac exception.** The
  Linux seat box installs by `cargo install lernie --root ~/.local --locked
  --version <v> --force` on an hourly timer (bl-155a, README *Continuous
  deployment for a seat box*): the reconciler carries no build, because "a
  seat's unit of install is a published version, and the registry already
  serves it". A mac is the same box with a different SDK. One artifact, one
  home — the registry — on every platform.
- **An attached binary would be a worse product, not a convenience.** The
  linker's signature is ad-hoc; a copy that arrives over a network carries a
  quarantine attribute, and an arm64 mac refuses to start it until somebody on
  that mac clears the attribute by hand. Making the download simply run means
  notarization, which is a credentialled act needing an Apple Developer
  identity and a secret in this repository — and this repository holds no
  secret and takes no dependency on one (§6.2: that is what trusted publishing
  bought). A binary compiled on the box carries no quarantine attribute at all
  and runs the moment `cargo install` returns. So the asset would have needed
  a credential to be worth attaching, and without the credential it is a file
  that does not work as shipped.
- **The Linux binary follows the same rule, and that closes the row §6.2 used
  to defer.** No upload job for any platform. The two headless siblings attach
  a Linux tarball because a tarball is what their boxes install; a seat box
  never did.

**So the macOS job is the proof of the artifact, not its home.** It builds
what the operator's `cargo install` builds — the released tag, natively, with
the SDK the runner already has — and reads the Mach-O it produced. It runs
exactly when a version publishes, on that version's tag, and refuses if the
tag and the checked-out manifest disagree, so a green mac leg is a statement
about a version an operator can name and install. It used to run twice per
landing, on trees nobody would install; now it runs once, on the one they
would.

**What this does not solve, stated rather than implied.** A `cargo install`
on a mac yields a binary launched from a terminal — no `.app` bundle, no dock
mark, no Launchpad entry; the freedesktop seats `make install` lays down have
no macOS analogue here. That is a bundle question with the same notarization
answer waiting behind it, and it is not this ruling's. And "it links, and the
file has the shape of a working mac binary" is still the whole of what the mac
leg proves — nothing on the runner runs it.

### 6.4 The seat ships no OCI image: the reasoned exception to yog's §10.1 (bl-18c7)

yog's DESIGN §10.1 records the operator ruling that every component of the
four-way split ships one OCI image, *"for deployment sanity and for nothing
else"* — the image is the unit of install — and the ball that carried the
seat's half said the answer could be that it does not, provided the reasoning
is recorded in the seat's own repository as a doc amendment rather than a
silent omission. **This is that amendment: the seat ships no image, and the
question is closed rather than deferred.** It is the same ruling as §6.3 read
from the other side.

**An image is the unit of install for a box that takes images, and a seat's
box is a desktop.** The three headless components run where no toolchain, no
display and no operator is, and an immutable image is exactly what such a box
should be handed. A seat runs where all three are. Its unit of install was
settled before this question was asked — the published crate, compiled on the
box, by `make install` or the hourly reconciler on Linux (bl-155a) and by
`cargo install` on a mac (§6.3) — and that already buys what the ruling names:
one immutable version per install, an hourly convergence, and a yank as the
rollback lever. A second unit of install for the same component is two homes
for one artifact, and the second would be the one nobody's box uses.

**What a seat image would have to carry is host state, and what it could
carry is nothing the host lacks.** The layer would hold a display stack — GL
or EGL, the Wayland and X client libraries, xkbcommon, fonts, and a Vulkan ICD
or software rasterizer for a box with no GPU — and the run line would mount
the display socket through, pass the GPU device through, mount the seat's own
XDG state root (§4.13) and mount the wire material it dials with, which REMOTE
§1.4 says is never in a layer. Every one of those is something the host has by
being a desktop; the image would carry a copy of the host's own display stack
so that the host could lend the image its display. The run line would be
longer than the install it replaced, and would couple the seat to the
compositor, the device node and the socket path of one box — the coupling an
image exists to remove.

**The one face a container could lawfully host is not this product.** The
seat has a headless verb surface (§4.10: the typed argv is a serialization of
the same boundary), and a box with no display could script it from a
container. bl-9380 named a headless wire-client artifact as *a different ball
with a different product*, and it is: today one binary carries both faces, so
an image of it is the window's image with the frameworks and the display
libraries linked whether or not a verb ever paints. If that product is ever
cut — the window behind a feature, a binary that links nothing above the
socket — the image question returns with it, and lands on the container line
the siblings already run (yog §10.2's rule: a graph that links nothing above
libSystem crosses). Nothing here forecloses that; nothing here builds it.

**So there is no `Containerfile`, no `.containerignore`, no `make image` and
no `make image-scan`.** The image-side disclosure gate is §10.1's condition on
publishing an image, and a gate comes with the artifact it judges: a scan
target with no image to read is a gate that passes forever, which is the
failure every two-direction check in this tree exists to end. The naming fence
(§0) has one fewer surface for the same reason — an image tag inherits a crate
version, and there is no tag to inherit one.

**What this asks of yog's document.** §10.1 lists `ghcr.io/mudbungie/lernie`
among the four packages; that name will never exist. The amendment there is a
sentence citing this section, and it is yog's ball, not this one's.

