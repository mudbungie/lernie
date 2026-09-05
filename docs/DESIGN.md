# lernie — DESIGN

**Status: a working seat, whole.**
`lernie <verb>` and `lernie ask` open the channel a gesture's workspace names —
the box's own engine, or one of the client-side workspaces it holds elsewhere —
over a real mTLS handshake with a real version preface, carry the operator's
envelope across, and print the reply stream. `lernie entries` says what this box
holds without dialling any of it, and `lernie help` says what a verb takes
without dialling anything at all. `src/verbs/` is the gestures typed (§4.10)
and `src/reply/` reads their answers back as the nine kinds a window paints
(§4.9). **`lernie start` begins a conversation** — the start family's two acts,
staged and fired in one process (§4.10). The suite proves all of that against a
stand-in engine that speaks the protocol, at 100% coverage.

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
  engine's, on the engine's box, over a trust root that already exists. The act
  crosses a channel **already** authenticated at operator grade on material
  that already moved out of channel; the box being enrolled is not a party to
  it and has no channel yet; and what reaches that box reaches it through a
  camera pointed at a screen, which is the same class of act as carrying a file
  on a stick and is mediated by the same operator. The seat asks, and paints.
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

**Where a gesture names its workspace is three holders and one table**
(bl-4a36, and yog's twin bl-523f in the typed table): top level; one level down
inside `prepared` on the two envelopes carrying a prepared start; and one level
down inside `target` on the config family, whose destination *is* its address —
the wall whose file the act edits. The third row is the one that had to be
argued for: without it a config act aimed at an entry under a local rename
resolves to no entry, falls through to this box's own engine, and writes the
wrong wall's file with nothing painted. The two tables are fixed together or
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

**The grade is read off this box's own leaf before anything is dialled, and it
is a DIAGNOSIS rather than an enforcement** (`src/channel/leaf.rs`, REMOTE
§4.2). A certificate carries one of two grades, spelled as the subject's
organizational unit; a seat is operator grade by definition, and enforcement of
that is the engine's, at the chokepoint where the client identity is already
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
  leaves the engine would have accepted — an outage bought with a diagnostic.
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

**The fallthrough is not silent, and a name with no reader does not take it**
(bl-d574). The flat engine's own workspaces are named and held nowhere else, so
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

A self-provisioning engine writes `127.0.0.1:0` and only the listener knows what
it became — it tells its own in-process window in RAM. A separately installed
seat cannot be told that, so a flat root naming `:0` refuses with the sentence
rather than the raw connect error port zero would otherwise earn.

### 4.9 The reply vocabulary is reimplemented, and it decodes only what it paints (§8, §9.7)

`src/reply/`. REMOTE §8 is explicit that *"what the seat reimplements is this
document … no shared protocol crate was created and none should be"* — a shared
crate would make the versioned authority a dependency for one of the four
components and an authority for the other three. So the reply spellings are
read off REMOTE and implemented here, exactly as the android client does.

**Nineteen kinds, because nineteen are painted.** The engine's reply surface is
forty-odd variants and most of them belong to panes that do not exist here.
What is carried is the roster (`workspaces`), the conversation list
(`conversations`), one workspace's role tuning (`roles`, §4.17), the
conversation itself (`transcript`), the live tail
(`follow`), the conversation's records pair — the steps its loop took
(`steps`) and what its worktree holds (`files`), both §4.18's — the decision
queue (`attention`) and the receipt that raises a row onto it (`flagged`),
both §4.19's, the window's own two — one engine's verb table (`help`) and what
a needle found (`search`), both §4.21's — the login pane's three: what a wall
can sign in to (`providers`), what one of its rows offers (`models`) and one
sign-in run (`login`), all §4.24's — a captured
run (`outcome`), the detached advance's receipt
(`nudged`), the start family's two — the staged body (`prepared`) and the
minted name (`started`) — and a new box's material (`enrolled`, §4.15), plus
the refusal envelope, which is not a kind at all. A kind nothing renders is a kind nobody has to carry, and the ball that
lands a pane is the ball that adds its kind.

**`roles` is the worked example of that last sentence** (bl-4a2c). It sat in
`corpus/unreadable/` from the PROTOCOL 6 refresh — a perfectly good frame of a
kind nothing painted — and the commit that moved it to `corpus/answers/` is the
commit that built the pane. The diff of that move is the record of what the
release added, which is exactly what the ledger is for. The records pair
(bl-2cf7) is the pattern's second spend: `steps` and `files` made the same
move in the commit that built §4.18's pane, `help` and `search` made it in the
commit that built §4.21's two (bl-40ec), the sign-in family's three
(`providers`, `models`, `login`) made it in the commit that built §4.24's pane
(bl-e3c5), and the five conversation reads still in `unreadable/` (`agent`,
`step`, `inbox`, `rail`, `governing`) are the ledger holding the two balls that
will move them (bl-3257, bl-b52c).

**A protocol bump is not a shopping list, and PROTOCOL 4 is the worked example**
(bl-d774). REMOTE §9.10 and §9.11 put four new facts on the wire in one
unreleased cycle: a `failure` clause on the conversation row, the same clause on
the §6 queue row and on the `agent` answer, and a `flag` object beside a new
`flagged` signal token on the queue row. **That release consumed exactly one of
them** — the conversation row's clause, because the conversation list is the
pane that paints the row it hangs on, and a red row that says nothing about why
it is red is a list an operator opens one by one to learn the one thing every
row in it says. The other three rode through unread and their shapes stayed in
`corpus/unreadable/`, which was the ledger doing its job rather than a
shortfall: nothing here painted an `agent` answer or a decision queue, so a
field carried for them would have been a field held for no glass. **The number
moves for the wire; the fields move for the panes**, and the two are decided
separately.

**And the fields moved when the pane did** (bl-f0ef). §4.19 lands the decision
queue, so `attention` and `flagged` moved to `corpus/answers/` in the commit
that built it and two of PROTOCOL 4's four facts — the queue row's `failure`
clause and its `flag` object with the `flagged` token beside it — are painted.
The fourth is still unread and its ledger line still stands: nothing here paints
an `agent` answer, and that half belongs with the provider rungs (bl-b180,
bl-e3c5) rather than with a queue. The interval between the bump and the paint
is the ledger's whole product — one diff, naming exactly what a release added.

**PROTOCOL 5 and 6 are that sentence with the second half empty**, and the
pair is why the ordinary bump costs this seat an integer and nothing else. 5
(bl-e6ee, REMOTE §9.12) took `branch` off `reply/governing` for `follows` and
`diverged_lineages`; 6 (bl-675e, REMOTE §9.13) gave `reply/providers`' rows
`effort` and `priority`, two booleans saying which tuning that provider row
takes. **Neither shape was decoded here at the time**, so both times the seat
paid the number and no field, and the correct amount of new paint was none.
**One of the two has since been claimed and the interval is the ledger's
product**: §4.24's pane reads `reply/providers` — the two booleans among its
fields — and the commit that built it is the diff saying so, three protocol
bumps after the fields landed. `reply/governing` is still nobody's, and its
ledger line still stands (bl-b52c).

**PROTOCOL 7 is the first bump this seat reads a field out of, and no pane
paints it** (bl-8758 upstream, bl-38d4 here). Every `reply/help` row gained
`surface`, classing the op `control` or `machine` — and the consumer is a GATE,
not a pane: it is the roster §4.16's parity assertion judges this window
against. That is a third answer to *what does a bump cost this seat*, beside
"an integer" and "an integer and some paint": a field can be load-bearing for
what the suite may assert while the glass stays exactly as it was. It does not
loosen §4.9 — the reply vocabulary still decodes only what the window renders,
and `reply/help` is still filed in `corpus/unreadable/` because nothing paints
a help pane. What reads the field is the corpus walk, which reads every frame
whatever this build does with it.

**The per-bump ledger is the `PROTOCOL` constant and this section does not
restate it** — one fact, one home, and a list here would rot the way every
count this repository stopped writing down rotted. What belongs here is the
rule the two bumps are instances of, and the one trap they exposed.

**A shape whose meaning moves under an unchanged spelling is the one drift a
corpus replay cannot catch.** 5 is the plainest case: `oid` stopped naming the
fork commit and started naming the followed lineage's head, so the bytes stay
well-formed, the fixture stays in the class it was already in, and every
mechanical check this repository owns stays green while a future pane paints a
plausible number that has been wrong since the bump. The ledger catches a shape
that CHANGED; only prose catches a shape that changed its MIND, so that trap is
written at `PROTOCOL` where whoever lands the pane will be reading.

**And a bump reaches the WRITE direction even when it paints nothing.** 6 added
two ops rather than only a field, and a new op is free by §3's rule — the peer
refuses an unknown one in band, by name. Free of a *decode* is not free of an
obligation: `src/verbs/tests/corpus.rs` replays every op in the vocabulary,
including the sixty-odd this seat has no word for, and asserts each routes by
the address upstream's own signature says it carries. That is where a miss
would cost something and would do it silently — a gesture routed by a slot the
seat does not look in goes down the wrong channel — so the request half is
checked on every bump whether or not a pane moved.

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
own in-band correction and is the upgrade prompt; an unknown **token** inside a
row keeps its word and paints as itself, because refusing there would drop a
whole readable listing to avoid one word; an unknown **field** is ignored
structurally, which is the other half of §3's rule that a new field is not a
protocol bump. Nothing is a panic path and nothing is defaulted to a known
neighbour: a token painted as a word it is not is a lie, where a token painted
as itself is merely unstyled.

**One deliberate divergence from the engine's own reader**, recorded at the
site (`src/reply/stream.rs`): a `follow` frame's delta token takes rung 3 where
the engine's decoder refuses. The two readers are not doing one job — the
engine's reads bytes the engine wrote, so a mismatch there means its own codec
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
starts holding nothing. `Stream::absorb` is the engine's own operation copied
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

**The corpus is a directory, not a test table** (`corpus/`, with its own
README), and **the frames in it are not this repository's** (bl-6b8e). yog
generates a wire conformance corpus from the boundary that *is* the protocol
authority (yog bl-32cb, REMOTE §3) and commits it; `scripts/refresh-corpus.sh`
vendors it here verbatim. A seat that judged its own decoder by fixtures it
wrote itself would be checking what it already thought of, which is the one
thing a reimplemented vocabulary cannot afford.

**The layout reconciles two shapes and keeps the better half of each.** The
frames and their stamps are upstream's, whole; the **directory is still this
seat's assertion** — `answers/`, `refusals/`, `unreadable/` — because a
classification is a decision this end makes and never one copied in. So the
refresh does not classify: a shape already filed goes back where it was, and a
shape yog has grown lands in `unreadable/` as a new file the diff shows. A
silent pass is the outcome the arrangement excludes. There is still no
manifest and no expected-value sidecar; a frame captured off a live engine
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

**The table and the roster are not one list, and neither count is written
here** — two were, and both rotted the way every restated count in this suite
rots; `src/verbs/rows.rs`'s `TABLE` is the one home.
The *roster* is the gestures whose replies §4.9 paints, and it grows with the
paint surface: the ball that lands a pane adds its kind and its gesture in the
same breath. The *table* is the subset a word can spell, and it trails the
roster by exactly four gestures that cannot be rows. Two modules hold the
four and each is the same rule
applied. `src/verbs/start.rs`: `prepare` carries a payload rung and `prompt`
carries a prepared body, and a nested object is not a word an operator types, so
what argv types instead is the composite below. `src/verbs/tuning.rs` (§4.17):
`effort` carries a level that is a string **or null** — null being the whole of
what *off* means, so a row would have to send a fifth word the boundary refuses
by name — and `priority` carries a bool. Each is exactly the case the paragraph
above refuses to special-case, and each is a typed door with no row.

**The one argument the seat settles itself is `enroll`'s grade** (bl-07b9).
`grade` is a closed set of two words the boundary defines (REMOTE §8.4) and
this binary already holds them — `lernie help enroll` says so — so a typo used
to cost a round trip and come back `unknown grade "OPERATOR"`: true, and naming
neither word that would have worked. It is read in `cli::run` for exactly the
reason a hand-written envelope is: decided entirely by what was typed, the
caller's typo, earns the usage, costs no connection. **It is not a second
authority on grades** — the engine still decides what a grade means and whether
this box may ask for one at all, which §8.4 makes unknowable here — and the two
words are read off `ui::Grade`'s own list rather than a second copy of it.

**The four structural words are DOORS, and they have pages** (`src/verbs/doors.rs`,
bl-6bda, bl-81dd). `start`, `ask`, `entries` and `help` are answered by this
binary and cannot be rows of the gesture table for the reason above; the defect
was that the help surface did not know it. The usage listed eleven words,
`lernie help <word>` answered a page for seven, and the other four were refused
with *"no verb named `ask`"* — false in the only sense the operator means it,
byte for byte what `lernie help bogus` earns, on the one surface whose whole job
is to answer with no engine up. The same lookup gap had a second face on argv:
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

**`lernie start <workspace> <goal>` is a serialization of BOTH acts, not a
third gesture** (`src/seat/start.rs`). Nothing new crosses the wire: a
`prepare` goes out, its answer is held in a local, and a `prompt` carrying that
answer goes out after it — which is exactly what the window holds in its model
(§4.11). Both reply streams print, in order, because both are the engine
answering; the exit code is the *fire's*, so a stage that answered no staged
body exits non-zero with its own frames on stdout. The one outcome two acts
have and one does not — a stage that landed and a fire that never left this box
— gets its own sentence, because the workspace exists and nothing is running,
and the remedy is to type it again (yog §8.1: the steps are convergent).

**The bare rung, and only the bare rung.** yog's §3.4 gives the payload three:
bare, a work directory, a ball. This seat composes the bare one. A directory
needs a field that refuses a path that is not there; a ball needs a project, a
picker and the §3.5 join states. Each is unbuilt rather than unreachable, and
each arrives with the surface that composes it — a seat that guessed a rung
would found a claim nobody asked for.

**Positional and context-free, unlike the engine's own line.** yog's line reader
is terse because a seat with a focus supplies the address; REMOTE §8.5 says a
seat with no selection *"spells its targets out"*, and a one-shot process is
that seat. Copying the line's grammar would mint a selection type that is always
empty. The corollary is the verbatim payload: the line takes a message's content
as its whole tail because a line has no quoting, argv does, so arity here is
exact and the shell is what makes a sentence one argument.

**Help's subject is this binary rather than a world**, which is why it is
answered in the seat with no dial, no engine and no material — a binary an
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

`src/ui/`. Four panes and a notice bar: the roster grouped by the channel each
row came down, the aimed wall's conversations, the selected conversation, and
the composer — with seven more that COVER the conversation when they are open
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
policy used to say, and it ends: past the point where the two list panes are on
their own floor, nothing yields and the conversation goes under `CHAT_FLOOR`.
At a phone-shaped viewport that was three columns of about 120 points each,
with every line in every one of them wrapped to two or three words, and a band
of the window painted by no panel at all. The answer is not more yielding —
there is none left to do — it is the **covering-pane idiom read across the
whole layout**: below the width at which the yield can still keep the
conversation its floor, the window shows one `Column` (channels /
conversations / conversation) and a bar naming the three, which is a surface
you navigate to, act in, and come back from exactly as §4.15's enrollment and
§4.17's tuning pane are.

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
  the pane when three do. So the heading moved out of `roster`, `convs` and
  `chat` into `src/ui/shell.rs` — two nodes reading `channels` would be two
  things an operator and the accessibility tree both have to tell apart — and
  bl-e5d2's *heading outside the scrolled region* became structural rather than
  a convention each pane keeps.
- **The narrow shape is navigated explicitly and by nothing else.**
  `Model::column` moves in the bar and under a left/right key, and NOT as a
  side effect of aiming or selecting: those two are one door for a click and a
  keypress (§4.11's own rule), so a walk down the roster would have jumped the
  glass to another column on the first arrow. The bar stands down under a
  covering pane, as the composer does, because a modal's way out is its own
  control. The composer stands down off the conversation's own column, which is
  *"the conversation is not on the glass while a pane covers it"* read
  literally. And the arrows belong to whichever column shows, spent once per
  frame in `src/ui/keys.rs`, so no pane asks which shape it is in.

**What does not fit is scrolled, and the conversation has a floor**
(`src/ui/shell/policy.rs`, `widths`; bl-e5d2). Both list panes hold a list and neither
had an answer for one longer or wider than its box: the overflow was cut at the
panel edge mid-glyph, with nothing on the glass saying anything had been cut,
and the arrow walk moved the selection onto rows the pane had never painted —
the exact disagreement `roster::aimable` exists to prevent. Two rules close it.
**The list scrolls** (the heading stays out of the scrolled region: it is the
one thing on the pane that is always painted, and it carries the mark saying
whose the arrows are), and **a keyboard walk brings its selection along**
(`Model::reveal`, set by `keys::walk` and taken once by the pane the arrows
belong to — the keyboard is the only surface that can move a selection out of
view, since a click names a row the operator is looking at and a scroll IS the
operator choosing what to look at). Sideways, the side panels used to keep
their widths and the central panel absorb the whole loss, so at 900 points the
pane the window exists for was a ~140-point strip while the roster kept 280.
The rule is the other way round: the conversation keeps a floor and the two
list panes yield to it, together and in proportion to what each is worth, until
they reach their own floor — past which nothing yields, because two panes
showing nothing buys the chat pane a width it still cannot use. The policy is a
pure function of one number, so it is a value a test reads back rather than a
layout somebody has to look at.

**A section header names the address it dials** (`ui::Channel::dials`,
bl-77df). Two entries naming one address are two trust relationships that
happen to terminate at one listener (§8.2), which is lawful — and an entry
naming the address this box's own engine listens on is the same thing by
accident, which paints every workspace of that engine twice, under two headers,
with nothing on either saying they are the same server. `lernie entries` prints
the address under every row and the window dropped it. The seat is the only
thing that can see the duplication and it can see it for free, so the address
goes on the header and a duplicate is self-evident. The unaddressable row's own
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
The bar is for what an **engine said** about a gesture, and for a frame this
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
reframe is one level down: a section is either a list of walls or a **reason
there is none**, and the reason is three facts and not one — nothing has
answered down this channel yet (the `convs` pane's own doctrine, one noun
over), this box cannot dial it and knows why off its own files (the sentence
`crate::channel` already computes and `entries` already prints — one home, both
surfaces), or the engine answered and holds no workspace. `seat::channels`
seeds the second at boot; the first roster answer spends whichever was standing.

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
conversation on the very wall being enrolled into — beneath the symbol, so the
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
wall this window had not been ANSWERED about was painted with the sentence
reserved for a wall that answered nothing. That is a definite fact about a wall
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
(`ui::Channel::address`, §8.2 read from the seat's side). This box's own engine
rewrites nothing, so a row is addressed by its own name; an entry resolves by
its **leaf** and by nothing else, so the leaf is the address of the one
workspace it names. The third case is real and is painted rather than hidden: an
entry's engine may answer a workspace the entry does not name, which no envelope
this seat can write will reach. Dropping it would hide a workspace the operator
has, and addressing it by the leaf would aim a gesture at a different wall, so
it is shown and said to be unreachable.

**One composer with two subjects, and a start is where the third pane's
refusal went.** A wall with a conversation on it is spoken to; a wall with
none is where a conversation is *begun*. That second case used to be half of a
refusal — *pick a workspace and a conversation* — and the refusal was the
start's own case wearing a sentence. A second box beside this one would be the
same box twice, each with its own Enter, on a face that is four panes wide.
What is left refusing is the one case that is neither: no wall aimed at, so
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
outright while the conversation is live, and charging a retype for the engine's
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
  is three refusals per pass against an address the engine cannot resolve, and
  the operator's own new conversation painted as unknown for the whole of a
  healthy start. An empty conversation pane is what the world honestly holds.
- **A row stands where the conversation will be.** `Model::rows` is the list
  every surface paints and walks — the pointer's and the keyboard's one list, so
  the row is not a case each of them carries. It is what nothing observed: no
  lock, no completed step, flagged uncertain, which is exactly what the engine's
  own classifier answers for a conversation it cannot probe. **It must not read
  `live`**, which would claim a driver this seat has never seen. It carries the
  operator's goal, because a start that shows nothing between Enter and the
  driver's first write reads as a window that did nothing.
- **It is spent where it was made.** `Model::resolve` retires the claim on the
  answer that makes the conversation addressable — a listing row carrying that
  `name`, which the engine hands over exactly when a stored fact backs it — and
  moves the selection **only while the selection is still the name the claim put
  there**. A start can take a minute, and an operator who read something else in
  that minute is not yarded back. No claim survives being spent, and a claim
  whose row never arrives is inert: the name stays selected, the row stays
  faded, and one arrow key leaves it.

**And the composer stands down while it stands.** A deposit or an advance
composed against a claimed name is a gesture this end already knows the engine
will refuse, so the start's own sentence stands where the box was — the same
shape as the second-start refusal above, and the same reason: not a disarmed
control, but no control.

**Nothing that arrives is dropped.** `Model::absorb` is the one door: an answer
is filed, and a refusal or an unreadable frame becomes the notice bar, standing
where that content would have been. That is §4.9's rung 2 honoured on the glass
rather than only in the type — and the two read differently, because a refusal
is the engine's sentence and an unreadable frame is a statement about this seat,
of which only the second is fixed by an upgrade. It is a **bar and not a
modal**: the engine refusing a deposit says nothing about the roster beside it.

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
- **The gate is the composer's box by name, not egui's "is anything focused".**
  `wants_keyboard_input` answers *is any widget focused*, buttons included, so
  tabbing to Send would have turned the arrows off. The box wears one id and the
  gate compares against it — which is also what lets Escape mean *leave the box*
  inside it and *dismiss the notice* outside it, two contexts that never
  overlap.

**The mark is arithmetic, and where it is SEEN is not where a client sets it**
(`src/mark.rs`). A window that names no icon wears the generic one every
unnamed client gets, and the seat's own mark is three shapes — a ring for the
engine held elsewhere, a filled disc for the seat that is here, and the one
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
  cadence: every channel's roster, the aimed wall's conversations, the selected
  conversation's transcript, and whichever covering pane's read is up. The
  questions **nest** — the aim is asked only where there is one, the
  conversation only where one is selected — and a channel that will not answer
  costs only itself, which is REMOTE §8.2's *"a refusal is one entry's, never
  the set's"* one layer above the file it was written about.
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
decides whether it is still wanted.** The engine was asked about one
conversation, or one provider row, and answers about that one — so only this end
knows the operator has moved on, and only the frame knows what it is looking at
right now. The guard is therefore a pure comparison at the settle rather than a
poll racing the socket the lane is parked on. Each lane's fold lives for exactly
one `tick`, which is how REMOTE §5.5's *"onto an empty fold"* is implemented
with no flag and no field: a read boundary is a local variable's scope.

**The standing set is a QUERY, never stored** (`Standing::of`). It is derived
from the model on every settle — every channel's roster, the aim, the selection,
and which covering pane is open — so there is nothing to invalidate and nothing
that can disagree with the focus: a click changes the model, and what is asked
next follows from it. Two consequences are worth stating because both were
decisions.

- **A pane's read is keyed on the PANE, not on the aim** (§4.17). A seat with no
  configuration surface open asks nothing about a file nobody is looking at; the
  reads are cheap, and a standing question nobody has a use for is still one the
  engine answers on every beat, forever. `Standing::open` is **one field rather
  than a flag per pane** (§4.24), because no two covering panes ever stand
  together and four bools would make three-at-once representable.
- **Not every selection is a question.** A conversation this window has just
  started is selected under a name the engine resolves nowhere until its driver
  writes the branch, so `Model::asked` leaves it out and the standing set never
  asks about it (§4.11's claim).

**The one lock, and the rule named the file before there was anything in it**
(`rules/locks-outside-state.yml`; §5's confinement table). `src/state.rs` is its
only tenant and there should not be a second: everything above the socket is a
pure function of what it is handed, and everything below it is one thread on one
connection. A poisoned lock is recovered rather than propagated — a worker that
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
against the stand-in engine with no thread at all, and the one end-to-end beat
is about the threading rather than about what the passes do. **A stop is seen
between passes, not during one**: a held lane is parked on a socket read, so it
learns of a stop when the engine writes, when the connection closes, or when the
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
stale aim needs no validation either — a wall that has gone is a wall no channel
resolves, and §4.12's standing set already declines to ask about one, so
checking it here would be a second answer to a settled question.

**It is a JSON object and not two lines**, so the next fact §7 names is a key
beside this one rather than a format: an unknown key is ignored and a missing
key is absence, which is §4.9's rungs 3 and 4 applied to this box's own file.

**And it costs the frame nothing.** The aim is already across the lock —
`Standing` is published on every settle — so the place is a projection of what
the workers were reading, read once at boot and written once after the event
loop returns. No fourth thread, no write on a frame, and nothing new crossing
the lock to carry it.

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

`src/reply/enrolled.rs`, `src/seat/enroll.rs`, `src/ui/enroll.rs`, `src/qr/`.

**What the act is.** `enroll` names a workspace, a name and a grade, and the
engine mints that box's leaf on its own CA, seats the registration, answers the
material and shreds the key. The seat sends it over an entry it already holds
at operator grade and shows the answer as a QR symbol. §3 above records why
this does not lift REMOTE §1.4, and REMOTE §8.4 is the authority for the wire.

**The reply is not the product, and that is the one place this seat departs
from its own shape.** Every other verb hands its reply stream to stdout —
`seat::ask` is the whole of it. This one must not: the answer carries a private
key for a box that does not exist yet, and stdout is a scrollback, a shell
history and whatever it was piped into. So `enroll` has an arm of its own all
the way up to `cli::Decided`, and what it prints is the picture and the three
fields that are not secret. `lernie ask` still prints the raw frame, which is
correct — an operator who spells the envelope by hand has asked for the stream,
and nothing here is a boundary against the operator. What the arm buys is that
the *ordinary* path leaves nothing where nobody chose to put it.

**Nothing is written down, and it is asserted over the tree.** No file, no
cache, no log line, no temporary anything, on either face: the command line
holds it in locals, the window holds it in `Model::enroll` and drops it with
the pane, and closing that pane is a control whose whole product is the
forgetting. `seat::enroll`'s suite walks a throwaway root before and after the
act and compares — over the **tree** rather than over the paths the code
happens to know about, because a defect here is precisely a path nobody thought
of.

**The QR encoder is this crate's own** (`src/qr/`), byte mode at correction
level M, no new dependency. A QR symbol is a fully specified algorithm rather
than a research problem, and the manifest's dependency set is an approved list
a ball has to argue its way onto. REMOTE §8.4 measures the envelope at 1567
bytes and states the rule as *PEM as minted, at level M or lower*; level M
carries 2331. The encoder's own module doc holds the rulings, and its suite
pins three whole symbols and all forty versions against an **independent**
implementation — because an encoder that agrees with itself proves nothing, and
two reference implementations disagreeing with each other is how two of this
one's choices were settled.

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

`src/ui/tuning.rs`. **The seat's second covering pane, and the first that is a
place rather than a moment.** §4.15's enrollment covers the conversation because
what it holds is a private key and the act is *look at this now and close it*.
This one covers it for a plainer reason: it is a surface an operator navigates
to, acts in, and comes back from — which is what §4.11's reach assertion was
written about and recorded that this seat did not have. It does now, and the
walk covers both panes.

**Nothing on it is a preference.** Every row is a fact about the WALL, held in
that workspace's config on the engine, and every control is one gesture across
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
name — a config written by a hand, or by a newer engine — is §4.9's rung 3 on
the glass: painted as itself, because four unselected seats would otherwise say
the level is *off*, which is a different claim from the one the file makes.

### 4.18 The records pane: the conversation's own ledger, on the tuning pane's standing (bl-2cf7)

`src/ui/records.rs`. **The third covering pane, and §4.17's shape one noun
over**: the subject is the SELECTED CONVERSATION where tuning's is the aimed
wall, and every joint carries across. bl-213c built the conversation's ACTS —
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
and a seat with no records surface open asks nothing on any beat. The pane
holds no state at all — not even a draft, which is one less than tuning holds
— and it goes down when its subject moves: selecting another conversation or
aiming at another wall retires the pane and both answers, because a pane
about *the selected conversation* left standing over a new selection would
paint one conversation's records under another's name for a beat.

**One control opens it and carries both reads' tags** (`act:steps act:files`),
leading the composer's second row — the composer being the pane that already
acts on the selected conversation, and the chat pane being a pure projection
no control may be the first exception to. The deeper reads stay in the
exemption ledger on two seams: the spine (`rail`, `governing`, and the `fork`
whose `from` only the rail can offer — bl-b52c) and the depth under this pane
(`agent`, `step`, `inbox` — bl-3257).

### 4.19 The decision queue: the pane about no focus (bl-f0ef)

`src/ui/queue.rs`. **The fourth covering pane, and the first whose subject is
neither the aimed wall nor the selected conversation.** REMOTE §6 is the
engine's *what needs you* — a flattened roster across every enumerated
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
that somebody *wrote* rather than something the engine observed.

Five decisions, and each is a choice rather than a mechanism:

- **Its control hangs off the roster, above the channels and off no row.** The
  roster is the one pane that is already the union across channels, which is
  what the queue is; and because `attention` addresses nothing, the control
  needs no aim and is offered on a seat that has aimed at nothing — which is the
  seat most likely to be asking the question. It sits outside the roster's
  scrolled region for the reason the heading does, and stands down under a
  covering pane exactly as the two per-wall controls do.
- **Nothing on the glass can retire it.** §4.17's pane goes down when the aim
  moves and §4.18's when the selection does, because each is *about* one of
  them. This one is about everything, so aiming and selecting leave it standing
  — which is the same rule read on a pane whose subject nothing on the glass can
  move, not an exception to it. The read stands on the PANE
  (`Standing::queue`), and there the argument is sharper than tuning's: a fan
  costs one round trip per channel per beat, so a seat with nothing open pays
  none of them.
- **A fan's answer replaces its OWN channel's section**, leaving every other
  standing — §8.2's *"a refusal is one entry's, never the set's"*, the shape the
  roster already keeps.
- **A row is addressed off the ROSTER, never off the section it came down.** A
  queue row names its workspace as *its host* names it, which is not what a
  gesture from this box carries when an entry renames. The window's half of that
  mapping has one home — a roster row and `ui::Channel::address` — so
  `Model::wall` asks it there. Two things follow and both are the point. A
  channel stamp becomes a display fact that cannot mis-aim a gesture, which
  matters because a receipt's stamp is the *aimed* channel rather than the one
  that answered (`src/offframe/poster.rs`) and `seen`'s reply is a queue rather
  than a receipt — the residual that leaves is one beat of a section under the
  wrong header, and it is bl-c70d's, filed with the fix: the seat-side name is
  a fact `seat::route` already has and discards. And a row for a wall this seat
  holds no name for is honestly unaddressable, and says so in the roster's own
  sentence, rather than being aimed at a different wall by a guess.
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

**What is painted and not actionable, on purpose.** `held` — the invocation
parked at the conversation's capability boundary — is read and shown, because it
is what makes a row answerable rather than merely readable. The control that
releases or declines it is `answer`, and that belongs to the tool-host surface
this seat does not have; its `parity.toml` line still cites bl-e53c. Saying what
is parked is worth more than saying nothing while that pane is unbuilt.

**And one control on it crosses no wire.** *go to it* aims at the row's wall and
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
  `Model::aim` when it fired would unmake a wall the operator armed a different
  one for. This is the opposite of what the tuning pane does (§4.17) and it is
  the opposite for the right reason: a tuning write that lands on the wrong wall
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
  unless the workspace is the engine's own, nothing in it is live, and the typed
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
  the engine declines while anything in the workspace is live, which is exactly
  the state an operator reaches for this control in. Clearing the box would
  charge a retype for the engine's *no*, a toll on the safe path — where
  clearing a sent message costs nothing, because it was sent. Firing is said
  instead (`asked — waiting for the engine`), for the reason the enrollment says
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

### 4.21 The window's own reads: the three ops that name no workspace (bl-40ec)

`src/ui/roster/acts.rs`, `src/ui/commands.rs`, `src/ui/find.rs`,
`src/ui/model/window.rs`, `src/verbs/window.rs`. **The sixth and seventh
covering panes, and the one control that opens nothing.** §4.19's queue was the
first surface whose subject is neither the aimed wall nor the selected
conversation; these are the rest of that family, and the family has a
mechanical definition rather than a taste: **an op with no `workspace`
parameter cannot name a channel, so its subject is every channel this box
holds** (`crate::verbs::Verb::addresses_a_workspace`). Four ops are in it —
`attention`, `workspaces`, `help`, `search` — and until this ball three of the
four reached no gesture at all.

**The mechanism all three needed was one, and it was in the poster.**
`crate::seat::route` resolves an envelope's workspace over this box's entries
and falls through to the flat root when there is none. That is right for
`lernie ask`, where an operator named one channel by hand, and wrong for a
frame: a window that composed `workspaces` would have asked this box's own
engine and said nothing about the rest, which is bl-0d54's defect one surface
over. `src/offframe/poster.rs` now reads the envelope — a gesture naming no
workspace is **fanned** over every channel the standing set holds, each answer
stamped with the channel it came down and each failure reported against it.
`crate::seat::fan` is that rule on argv and `crate::cli::Decided::Fanned` is
it in the command line, so all three surfaces read one predicate. The leg
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
  these does: a verb table is fixed for the life of an engine build, and a
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
- **The engine's `help` is not `lernie help`, and only one of them has an argv
  row.** `crate::verbs::help` answers *what does this BINARY take*, from a
  table compiled into it, with nothing provisioned and no engine up; the wire's
  `help` answers *what does that ENGINE offer*. Two subjects, one word, and
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
ENGINE'S OWN ABSOLUTE PATH while every gesture this box composes carries a name
(REMOTE §8, *"paths never cross the wire"*). So the keys a row spells are the
keys the acts take and the values are not: feed one back and it earns `unknown
workspace`. The find pane therefore offers **no control that spends a hit**,
and says why on the glass rather than in a comment — a *go to it* here would
have to guess a name off a path, which is the mis-aim `Model::wall` exists to
refuse one layer down, and a control that silently did nothing would be worse.
That is §4.19's decision about a parked invocation read one noun over: saying
what was found is worth more than saying nothing while the address it names
cannot be spent. **It is not in `parity.toml`** — the op is surfaced, and that
file records absences.

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
an engine to hang up once and answer the second dial, and asserts the far end
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
`Notice::InDoubt` says it reached the engine and may have run, so this seat
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
is the one with no test seam for it."* The stand-in engine now takes
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

**The rule: an item whose act takes only the wall and the conversation FIRES;
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
subject is the aimed wall.** §4.17's tuning pane is what a wall's roles are set
to; this is what that wall can sign in to, and the act that signs it in.

**The act is the ENGINE'S and this crate never spawns one.** `bz --login` runs
on the engine, inside the named workspace's wall, so the credential lands where
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
  credential lands on the ENGINE while the operator is looking at the table, so
  a row that said *no credential* says otherwise on the next beat with nothing
  asked again.
- **`models` is POSTED** (§4.21's rule). What a row offers is fixed for the
  life of the engine's own answer, so a standing read would spend a round trip
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
address.** The wall's name and the channel it came down, and — for a wall held
elsewhere — that it is held elsewhere, read off the roster's own client-side
channel stamp (`ui::Channel::named_there`, §8.2). Two entries naming one
listener are still two trust relationships, and an address in this sentence
would be this end telling the operator where a credential is about to land.

**The loopback remedy is STATED and never built** (REMOTE §8.3). A row whose
flow wants a browser at the engine's own loopback completes only from a browser
on that box, or through a port-forward — *an operator act on boxes the operator
administers*, which upstream rules is a remedy and not a channel feature.
**It is said for every row on a wall held elsewhere rather than for the
browser-only ones**, and that is a limit rather than a choice: the `device`
column is on yog's provider ROW and not on the view that crosses the wire
(bl-7c9f), so this seat cannot tell the two apart. A stated remedy that is
sometimes unnecessary beats a silence that is sometimes wrong. If the column
ever reaches the view, the sentence narrows to the rows that need it and
nothing else here moves.

**The run-by-hand fallback is the engine's sentence with one of ours beside
it.** `fallback` is composed by the end that knows the wall and arrives only on
a non-zero exit. The original surface (yog bl-1ddb) spelled a local wall's
fallback as an `exec` on this box and an entry-held wall's as something else;
there is no `exec` in this crate and both are the same case now — **an act the
operator runs on the box that HOLDS the wall** — so what the pane adds is one
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

**And the pane goes with the wall.** `Model::aim_at` retires it and all three
answers, which is §4.17's rule in its sharper form: the act signs a credential
into ONE wall's store, so a pane left standing over a new aim would offer to
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

## 5. Module map

| Path | What it is | Cap band |
|---|---|---|
| `src/main.rs` | the process entry: argv in, the environment folded once, a stream and an exit code out. The one `tarpaulin.toml` exclusion, and it is honest because it decides nothing. | small |
| `src/lib.rs` | the crate doc and the module list. | small |
| `src/cli.rs` | the command line as a **pure function**: arguments in, a `Decided` out. No argv, no environment, no streams, no exit. | ~160 |
| `src/cli/verdict.rs` | what an invocation says, and with what exit code: the four constructors and the two codes. | ~95 |
| `src/cli/text.rs` | what this binary says about itself: the version line, and the usage whose verb section is derived. | ~75 |
| `src/paths.rs` | the two roots — what the operator carried here, and what the seat generates about itself — from one ladder and no knob of its own. Neither variable set is a refusal, never a guess. | ~130 |
| `src/place.rs` | where the seat was pointed, remembered between runs. Every way the file can be wrong is one answer: no place. | ~85 |
| `src/envelope.rs` | the gesture envelope from the seat's side: is it one, which workspace does it name, did the last reply say ok. **One table, not two** — the read answers through the write. | ~150 |
| `src/seat.rs` | which engine a gesture reaches, and what it carries there. | ~190 |
| `src/seat/holds.rs` | what this box says it holds, said without dialling any of it: the listing, the typed channel set the window stamps its rows with, and the one spelling of a channel's name. | ~150 |
| `src/seat/fan.rs` | a gesture that names no workspace, asked of every channel this box holds — the union, stamped with where each answer came from. | ~80 |
| `src/seat/start.rs` | the §8.1 start family's two acts, spelled as one word — the composite, and the local between them. | ~80 |
| `src/channel.rs` | one wire to one engine: dial, ask, follow. | ~150 |
| `src/channel/frame.rs` | the framing. | ~105 |
| `src/channel/hello.rs` | the version preface. | ~85 |
| `src/channel/tls.rs` | the mTLS configuration. | ~90 |
| `src/channel/leaf.rs` | the grade, read off this box's own leaf: the one fault it names, and the DER walk that names it. | ~200 |
| `src/channel/reach.rs` | why an exchange produced no answer, and the one fact a sentence cannot carry: whether the request crossed (§4.22). | ~70 |
| `src/channel/material.rs` | what the operator carried here, and what its absence means. | ~110 |
| `src/channel/entries.rs` | the client-side workspaces this box holds elsewhere. | ~165 |
| `src/reply.rs` | the reply vocabulary's roster: the kinds (`Reply` is the one census), the three outcomes one frame can be, and the four-rung decode policy stated once. | ~200 |
| `src/reply/read.rs` | reading one frame — the dispatch off `kind`, and the refusal that wears none. | ~75 |
| `src/reply/fields.rs` | the strict field readers — rung 1, in one place, every refusal naming its field. | ~110 |
| `src/reply/roster.rs` | the workspace enumeration and how current it is. | ~145 |
| `src/reply/convs.rs` | one workspace's conversation list, and the two token fields rung 3 lives on. | ~160 |
| `src/reply/transcript.rs` | the conversation's entries — the envelope of one, and which origin wrote it. | ~155 |
| `src/reply/transcript/blocks.rs` | what one model entry says: the canonical blocks, and the provider's own counters. | ~115 |
| `src/reply/start.rs` | the start family's two receipts: the staged body carried whole, and the minted name. | ~90 |
| `src/reply/enrolled.rs` | a new box's material, and the one envelope a camera carries it in — the six fields spelled once, read and re-said. | ~130 |
| `src/seat/enroll.rs` | the §8.4 act from argv: one gesture, a symbol printed instead of the answer, and nothing written down. | ~90 |
| `src/ui/enroll.rs` | the enrollment pane: a name, a grade, and the symbol that comes back — the one pane that covers another. | ~145 |
| `src/qr.rs` and `src/qr/*` | a QR symbol drawn by this crate: the field, the tables, the zigzag, the four scoring rules, and the terminal rendering. Seven files, none over 250. | ~250 |
| `src/ui/model/enroll.rs` | an enrollment between the control that opened it and the symbol it ends at, and the only secret this window holds. | ~135 |
| `src/reply/stream.rs` | the live tail's fold. | ~105 |
| `src/reply/steps.rs` | the steps a conversation's loop has taken: one strict row per step, the nested spend, and the class tokens carried verbatim with `"none"` the one word the pane reads (§4.18). | ~135 |
| `src/reply/files.rs` | what a conversation's worktree holds: the listing whose absence is a fact, the bounded preview's three classes plus the rung-3 word, and where the work lands (§4.18). | ~125 |
| `src/verbs.rs` | the typed gesture surface: what a verb is, and the one envelope a row becomes. | ~135 |
| `src/verbs/rows.rs` | the reads, the deposit, the advance and the enrollment, as data — each with the typed door the window composes by name. | ~105 |
| `src/verbs/conversation.rs` | the conversation's own four acts as rows — the cut, the kill, the change of lineage and the unmaking. Here and not in the exemption ledger because every one of them answers a captured run, which is a kind this seat already paints. | ~110 |
| `src/verbs/queue.rs` | the decision queue's three ops as rows — the fan that names no workspace, the raise and the answer, the last of which replies with a queue rather than a receipt (§4.19). | ~90 |
| `src/verbs/records.rs` | the conversation's records as rows — the steps ledger and the worktree listing, each with its typed door, admitted by the same test the four acts passed once §4.18 decoded their kinds. | ~65 |
| `src/verbs/workspace.rs` | the wall's own act as a row — the one whose product is that its subject is gone, and the one whose `typed` is an arming rather than a parameter (§4.20). | ~50 |
| `src/reply/help.rs` | one engine's own verb table (§4.21): five required strings a row, the classification carried verbatim, and the headline the pane paints. It is the same shape `src/snapshot/parity/roster.rs` reads the parity roster off. | ~85 |
| `src/reply/search.rs` | what a needle found (§4.21): the four facts about a match, the four address fields that are optional because a hit is one of three shapes, and the unreadable list that is a different claim from finding nothing. | ~135 |
| `src/verbs/window.rs` | the window's own two ops (§4.21) — the engine's verb table, which has no argv row because its word is `lernie help`'s, and the search, which does. | ~80 |
| `src/ui/commands.rs` | the commands pane (§4.21): one section per channel of what that engine answers to, each row's line, sentence, page and classification. | ~110 |
| `src/ui/find.rs` | the find pane (§4.21): the needle, the act that is disabled until there is one, the hits, and the standing sentence saying why none of them can be aimed at (yog bl-ef16). | ~150 |
| `src/ui/model/window.rs` | the window's two panes between frames — which one stands as one field rather than two flags, the per-channel filing both share, the needle that is not spent on firing, and the roster refresh. | ~175 |
| `src/ui/roster/acts.rs` | the strip above the channels: the four ops whose subject is every channel, and the one of them that opens nothing (§4.21). | ~65 |
| `src/verbs/start.rs` | the start family's two envelopes — doors without rows, and why. | ~80 |
| `src/verbs/doors.rs` | the four words this binary answers itself: a word, a usage line and prose, with no envelope behind it. | ~150 |
| `src/verbs/help.rs` | the two rosters and one word's page, answered with no engine up. | ~110 |
| `src/ui.rs` | the window's module list and what a frame may not do. | small |
| `src/ui/model.rs` | what the window holds between frames. The door a reply comes in through split out at the cap onto the seam this row used to name (`model/absorb.rs`). | ~195 |
| `src/ui/model/absorb.rs` | **the one door a reply comes in through**, and the leg that brought none: what is filed, what becomes the notice, and why an unreachable channel is neither. | ~140 |
| `src/ui/model/notice.rs` | what the seat last heard that was not content: the three kinds, and the line that says whose sentence it is. | ~40 |
| `src/ui/model/posted.rs` | a gesture on its way out, and whether a lost reply leaves it in doubt — recorded at the control because it cannot be computed (§4.22). | ~65 |
| `src/ui/model/acts.rs` | what a control does, whichever control did it — the one home a binding and a click share. | ~60 |
| `src/ui/model/channel.rs` | what a channel is, what a gesture aimed down one must be addressed as, and what its section says when it has no walls. | ~110 |
| `src/ui/model/start.rs` | a start between its two acts: what is held, and what each receipt does to it. | ~160 |
| `src/ui/model/claim.rs` | the claim a start leaves on the selection: the row it stands in for, what is not asked about it, and the answer that spends it. | ~130 |
| `src/ui/roster.rs` | every workspace this seat can reach, grouped by channel, and the two per-wall controls that hang off the aimed row. The strip of window-level acts above the channels split out at the design-time budget (`roster/acts.rs`). | ~260 |
| `src/verbs/tuning.rs` | the role-tuning family: the `roles` read and the `model` assignment as rows, and `effort` and `priority` as doors without rows — a nullable level and a bool are not named strings. | ~155 |
| `src/reply/queue.rs` | the decision queue's rows: what is asking, why, the flag somebody raised on it and the invocation parked at its boundary — three nullable facts each read as an absence (§4.19). | ~165 |
| `src/reply/roles.rs` | what one workspace's roles are set to: four required fields and the effort, which reports rather than asserts and so is an option carried verbatim. | ~80 |
| `src/ui/tuning.rs` | the tuning pane: what a wall's roles are set to, the four seats and one toggle that retune each, and the assignment editor under its own row. **The settings surface `src/snapshot/reach.rs` was written to say this seat did not have.** | ~185 |
| `src/ui/model/tuning.rs` | the tuning pane between frames — a two-state enum rather than a flag beside an option — and the four acts its controls spend. | ~180 |
| `src/ui/queue.rs` | the decision queue (§4.19): the union across channels, every line a row can carry with the flag leading, the answer and the way out to the conversation. | ~210 |
| `src/ui/records.rs` | the records pane (§4.18): the steps half and the files half, every empty state its own sentence, every line a pure function beside the paint. | ~240 |
| `src/ui/model/queue.rs` | the queue between frames — a flag, the per-channel filing, and the roster lookup that is the one place a row's address is resolved (§4.19). | ~135 |
| `src/ui/model/records.rs` | the records pane between frames — a flag, because it holds nothing — its open/close acts, the retirement with its subject, and the one `covered` question seven panes share. | ~70 |
| `src/reply/providers.rs` | what a wall can sign in to, and what one row offers: the four required fields, the block whose absence is the whole of *signable*, and the two capability booleans (§4.24). | ~110 |
| `src/reply/login.rs` | one sign-in run as the engine streams it — both tagged streams, the fold a frame is an append onto, and the two settled facts whose absence is a reading (§4.24). | ~115 |
| `src/verbs/login.rs` | the sign-in family as rows: the table, the offering, the act that starts a run in the wall, and the lane that streams it — four ops, one subject (§4.24). | ~115 |
| `src/ui/login.rs` | the login pane: the wall's two sentences, the provider table, and the two controls on a row. The followed run's half split out at the design-time budget (`login/run.rs`). | ~200 |
| `src/ui/login/run.rs` | what one sign-in printed: both streams, the settled exit and the run-by-hand command (§4.24). | ~70 |
| `src/ui/model/login.rs` | the login pane between frames — two questions rather than two modes — the three acts its controls spend, and where the engine is, read off the channel stamp. | ~165 |
| `src/state.rs` | **the link** (§4.12): what the frame and the off-frame threads say to each other, and the crate's one lock. `settle` is the frame's whole side of it. | ~175 |
| `src/state/traffic.rs` | what crosses the lock — a worker's report in its four kinds, and the standing question set the frame publishes, whose open pane is one field rather than a flag apiece (§4.12). | ~195 |
| `src/offframe.rs` | the four off-frame threads (§4.12): the one leg both fanning workers share, the filing every answer goes through, and the pump that is a cadence rather than a timeout. | ~120 |
| `src/offframe/asker.rs` | one pass of the standing set: the questions nested, the pane-keyed reads, and the channel that costs only itself (§4.12). | ~145 |
| `src/offframe/poster.rs` | one pass of the outbox (§4.12): the gesture that is routed, the one that is fanned, and which sentence a failed leg earns — an act's or a read's. | ~120 |
| `src/offframe/follow.rs` | one pass of the follow lane: the held read on the focused conversation, stamped with what it is about (§4.12). | ~90 |
| `src/offframe/signin.rs` | one pass of the sign-in lane: the held read on the followed provider row, stamped with what it is about (§4.24). | ~90 |
| `src/ui/unmake.rs` | the unmaking pane (§4.20): the wall it would unmake, the refusal stated before the act, the arming box, and the act that is on the glass without being live until the name matches. | ~135 |
| `src/ui/model/unmake.rs` | an unmaking between frames — the subject it holds rather than follows, the arming that is a readiness test and is never spent, and whether it has been asked. | ~105 |
| `src/ui/convs.rs` | the aimed wall's conversations: the four emptinesses, the truncating headline with the selection drawn under it, and the two lines hung beneath a row. The row's own acts split out onto the seam a gesture draws (`convs/menu.rs`). | ~245 |
| `src/ui/convs/menu.rs` | the conversation row's context menu (§4.23): the acts that fire on the row, the three that lead somewhere and spend nothing, and the admission test that separates them — a door taking the wall and the conversation, and nothing else. | ~150 |
| `src/ui/model/fill.rs` | which of the composer's two parameter boxes a row menu asked for the cursor in, and the one door that names a conversation and goes there (§4.23). Taken once, by the frame that paints the box. | ~85 |
| `src/ui/chat.rs` | one conversation as rows, and the live fold the lane hands over whole. | ~170 |
| `src/ui/composer.rs` | what an operator types, and the gesture it becomes — one box, three subjects, and the row of verbs that advance the turn. | ~150 |
| `src/ui/composer/acts.rs` | the second row: the acts that spend no words — kill the driver, retarget, raise a flag, and the unmaking with the name that arms its descendants. Its two boxes wear ids and take the cursor a row menu asked for (§4.23). | ~165 |
| `src/ui/composer/start.rs` | the half that begins a conversation rather than continuing one. | ~55 |
| `src/ui/keys.rs` | the keyboard: which list the arrows belong to, the walk that is the selection, and the gate — every box that takes text, named (§4.23). | ~250 |
| `src/ui/shell.rs` | the layout: the two shapes a window takes, each column's heading, the narrow shape's navigation bar, and the notice that stands where content would have been. | ~210 |
| `src/ui/shell/policy.rs` | **the width policy**: the yield the list panes give the conversation, the two shapes and where they meet, and the three columns a window is made of. A pure function of one number, so what the window does as it narrows is a value a test reads back. | ~180 |
| `src/ui/theme.rs` | the ink a row is painted in. | ~70 |
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
| `src/snapshot/worlds.rs` | the eleven named world states the matrix photographs, built from the window fixtures rather than from a second set — the first-run one seeded the way `src/main.rs` seeds a roster, because an empty `Vec` of channels is a state no box reaches and no pane has a sentence for. The walk's screen set is part of the parity instrument, which is why the start's own screen, the tuning pane's two, the records pane's, the decision queue's and the window's own two are among them. `cfg(test)`. | ~200 |
| `src/snapshot/reach.rs` | assertion (a): the walk to each of the seat's seven covered panes and back, asked of the accessibility tree. Two legs per pane, and three in the narrow shape — the length is the bound, and the bound is a fact about the shape. `cfg(test)`. | ~175 |
| `src/snapshot/blank.rs` | assertion (b): every rectangle the layout put content in, read off the rendered glass. `cfg(test)`. | ~145 |
| `src/snapshot/clipped.rs` | assertion (c): no control laid out wholly off the window, and none offered without a rectangle. `cfg(test)`. | ~70 |
| `src/snapshot/parity.rs` | **the interface-parity gate** (yog's `docs/PARITY.md` §5): the `act:` tags read off the same accessibility tree, and the four assertions over roster, inventory and ledger. `cfg(test)`. | ~125 |
| `src/snapshot/parity/roster.rs` | which ops owe this seat a control, read off the vendored corpus's `reply/help` rows and decided nowhere here. `cfg(test)`. | ~80 |
| `src/snapshot/parity/exempt.rs` | `parity.toml`, and the strict subset of TOML it is allowed to be — no crate parses it. `cfg(test)`. | ~90 |
| `parity.toml` | the exemption ledger: one line per control-classed op this seat does not surface, each citing the ball that will build it. | config |
| `src/ui/act.rs` | the `act:<op>` token a control carries, written into AccessKit's `author_id` where no label can drift into it. | ~85 |
| `src/paint_probe/frame.rs` | how a frame is produced: the offscreen input, the persistent window, the click. | ~120 |
| `corpus/` | yog's wire conformance corpus, vendored: `shapes.json`, `request/` whole, and the reply frames filed under `answers/`/`refusals/`/`unreadable/`. The directory a reply frame sits in **is** this seat's assertion; `corpus/README.md` is the contract. | docs |
| `.github/workflows/ci.yml` | the gate, run by a machine: the pinned tools, then `make ci`. Called by `release-plz.yml` on a push; triggered directly only by a pull request. | config |
| `.github/workflows/store-scan.yml` | the published store ref, judged by the source's own rule table. | config |
| `.github/workflows/release-plz.yml` | the release path (§6.2). Its FILENAME is matched literally against the registry's trusted-publisher claim — renaming it stops publishing. | config |
| `release-plz.toml` | the four release decisions the workflow reads: the tag spelling, no generated changelog, no semver check across the fence, and the bump markers. | config |
| `scripts/mac-verify.sh` | reads the produced Mach-O and says what it IS — architecture, filetype, target OS, every `LC_LOAD_DYLIB`, whether it is signed at all — with five malformed inputs it must refuse first. | ~230 |
| `tests/packaged_files.rs` | what `cargo publish` would ship, over the real `cargo package --list` — the manifest's `include` allowlist, restated so a widening edit is red here. | ~200 |
| `scripts/refresh-corpus.sh` | the vendoring, from a yog checkout. It copies and sweeps; it never classifies. | ~90 |
| `src/test_support/corpus.rs` | the one walk over the corpus, and the protocol stamp checked on every file read. `cfg(test)`. | ~130 |
| `src/reply/tests/corpus.rs` | the replay, reply direction: every frame lands in the class its directory names, and every upstream shape is classified exactly once. | ~140 |
| `src/verbs/tests/corpus.rs` | the replay, request direction, read half: every frame in the vocabulary decodes as a gesture and routes by the address its shape carries. | ~120 |
| `src/verbs/tests/corpus/emits.rs` | the write half: every frame this seat composes round-trips, and what it cannot compose is recorded by count and reason. | ~110 |
| `src/test_support.rs` | the scaffolding the suite shares and nothing production reads: the throwaway directory, and the two things that live here because the seat may not do them — mint a certificate, and listen. `cfg(test)`. | ~85 |
| `src/test_support/wire.rs` | a data root with an engine standing behind one of its channels — the one fixture for both arrangements, the flat root and one entry. `cfg(test)`. | ~50 |
| `src/test_support/window.rs` | the window's fixtures: the rows a pane is built from, and the model at work every pane's suite clicks in. `cfg(test)`. | ~155 |
| `src/test_support/window/panes.rs` | the covering panes' own fixtures — the seated model with each one open and answered, split at the design-time budget on the seam the parent's doc draws. `cfg(test)`. | ~210 |
| `src/test_support/mint.rs` | the operator's out-of-channel act, performed by the suite. **The crate's one spawn site.** | ~200 |
| `src/test_support/engine.rs` | the stand-in engine: a real listener, a real handshake, a real preface. | ~150 |

**The three confined files, and two of them do not exist.** A confinement rule
names its one location *before* the first site is written, or the first site
picks the location by being written.

| Kind | Confined to | Rule | State |
|---|---|---|---|
| `unsafe` block or `unsafe fn` | `src/sys.rs` | `unsafe-outside-sys.yml` | **absent, and expected to stay so.** A seat dials a socket, frames JSON and paints; upstream's raw effects are all engine-side. An ask to create this file is an ask to explain what the seat is now doing. |
| `Mutex` / `RwLock` | `src/state.rs` | `locks-outside-state.yml` | **occupied, by the tenant the rule was written for** (§4.12). The rule named this file before there was anything in it, and the off-frame threads filled it. There should not be a second: everything above the socket is a pure function of what it is handed, and everything below it is one thread on one connection. |
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
this tree: the registry crate `lernie` already carries the engine era's 0.0.x
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

**The tag is bare `v<version>`.** The sibling engine crate prefixes its tags
because ONE history there holds both eras of this name; this repository was
founded fresh for the seat, holds one crate, will never carry an engine commit,
and had no tags at all. The engine-era `lernie` tags live in the other
repository, where nothing this tree pushes can reach them. `release-plz.toml`
carries that reasoning beside the key.

**The macOS job reports; it does not gate.** bl-9380 measured that the seat
cannot be cross-built for darwin from Linux — the window links Apple
frameworks, they ship only in Apple's SDK, and the SDK agreement refuses both
hosting that SDK (§2.7) and running any part of it on non-Apple hardware
(§2.5) — and the operator ruled that a GitHub Actions macOS runner IS
Apple-branded hardware in the licence's sense. So the artifact is built
natively on `macos-14` and then READ rather than trusted: `mac-verify.sh`
reports architecture, filetype, target platform, minimum OS, every
`LC_LOAD_DYLIB` (each must be a stock `/usr/lib` or `/System/Library` path, or
the artifact acquired a dependency nobody chose) and whether a code signature
is present at all, with five fabricated malformed inputs it must refuse first.
`needs: ci`, so it only ever builds a green tree; nothing `needs:` it, so a
broken mac leg is visible and never stands between a green tree and the
registry.

**Still deferred, each its own ball.** No upload job for the Linux release
binary, no container image, and no signing or notarization — the linker's
ad-hoc signature satisfies an arm64 mac's refusal to start unsigned binaries
and is not notarization, and a downloaded artifact's quarantine attribute is
cleared by a credentialled act on a mac by whoever publishes.

