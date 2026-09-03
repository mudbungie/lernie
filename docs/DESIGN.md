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
are three threads (§4.12) — the asker over the standing question set, the poster
draining what a click composed, and the follow lane holding one connection open
on the focused conversation. **Everything the window does is reachable from the
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

**Nine kinds, because nine are painted.** The engine's reply surface is
forty-odd variants and most of them belong to panes that do not exist here.
What is carried is the roster (`workspaces`), the conversation list
(`conversations`), the conversation itself (`transcript`), the live tail
(`follow`), a captured run (`outcome`), the detached advance's receipt
(`nudged`), the start family's two — the staged body (`prepared`) and the
minted name (`started`) — and a new box's material (`enrolled`, §4.15), plus
the refusal envelope, which is not a kind at all. A kind nothing renders is a kind nobody has to carry, and the ball that
lands a pane is the ball that adds its kind.

**A protocol bump is not a shopping list, and PROTOCOL 4 is the worked example**
(bl-d774). REMOTE §9.10 and §9.11 put four new facts on the wire in one
unreleased cycle: a `failure` clause on the conversation row, the same clause on
the §6 queue row and on the `agent` answer, and a `flag` object beside a new
`flagged` signal token on the queue row. **This seat consumed exactly one of
them** — the conversation row's clause, because the conversation list is the
pane that paints the row it hangs on, and a red row that says nothing about why
it is red is a list an operator opens one by one to learn the one thing every
row in it says. The other three ride through unread and their shapes stay in
`corpus/unreadable/`, which is the ledger doing its job rather than a shortfall:
nothing here paints an `agent` answer or a decision queue, so a field carried
for them would be a field held for no glass. **The number moves for the wire;
the fields move for the panes**, and the two are decided separately.

**PROTOCOL 5 and 6 are that sentence with the second half empty**, and the
pair is why the ordinary bump costs this seat an integer and nothing else. 5
(bl-e6ee, REMOTE §9.12) took `branch` off `reply/governing` for `follows` and
`diverged_lineages`; 6 (bl-675e, REMOTE §9.13) gave `reply/providers`' rows
`effort` and `priority`, two booleans saying which tuning that provider row
takes. **Neither shape is decoded here**, so both times the seat paid the
number and no field, and the correct amount of new paint was none.

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

**Six rows and eight gestures — the table and the roster are not one list.**
The *roster* is the gestures whose replies §4.9 paints, and it grows with the
paint surface: the ball that lands a pane adds its kind and its gesture in the
same breath. The *table* is the subset a word can spell, and it is six because
two of them cannot be. `prepare` carries a payload rung and `prompt` carries a
prepared body, and a nested object is not a word an operator types — which is
exactly the case the paragraph above refuses to special-case. So they are typed
doors with no row (`src/verbs/start.rs`), and what argv types instead is the
composite below.

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
the composer. Every one of them is a function of the [`Model`] it is handed.

**The frame never dials.** Nothing under `src/ui/` opens a socket, reads a file,
blocks or waits; the one thing a frame *produces* is `Model::outbox`, the
gestures a click composed, drained by whoever can send them. A frame that posted
its own act would be a frame that waits on one, and a window that waits is the
failure a seat has no excuse for. What fills the model is §4.12.

**It fires through the verb table.** A composed deposit is built by
`crate::verbs` — the same rows §4.10's command line spends, through a door whose
arity is its signature — so a click and a typed command build one object and
there is no second spelling of a gesture anywhere in this crate.

**What does not fit is scrolled, and the conversation has a floor**
(`src/ui/shell.rs`, `widths`; bl-e5d2). Both list panes hold a list and neither
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
| `src/channel/material.rs` | what the operator carried here, and what its absence means. | ~110 |
| `src/channel/entries.rs` | the client-side workspaces this box holds elsewhere. | ~165 |
| `src/reply.rs` | the reply vocabulary's roster: the nine kinds, the three outcomes one frame can be, and the four-rung decode policy stated once. | ~190 |
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
| `src/verbs.rs` | the typed gesture surface: what a verb is, and the one envelope a row becomes. | ~135 |
| `src/verbs/rows.rs` | the six rows, as data — and the two acts the window composes by name. | ~105 |
| `src/verbs/start.rs` | the start family's two envelopes — doors without rows, and why. | ~80 |
| `src/verbs/doors.rs` | the four words this binary answers itself: a word, a usage line and prose, with no envelope behind it. | ~150 |
| `src/verbs/help.rs` | the two rosters and one word's page, answered with no engine up. | ~110 |
| `src/ui.rs` | the window's module list and what a frame may not do. | small |
| `src/ui/model.rs` | what the window holds between frames, and the one door a reply comes in through. | ~180 |
| `src/ui/model/notice.rs` | what the seat last heard that was not content: the three kinds, and the line that says whose sentence it is. | ~40 |
| `src/ui/model/acts.rs` | what a control does, whichever control did it — the one home a binding and a click share. | ~60 |
| `src/ui/model/channel.rs` | what a channel is, what a gesture aimed down one must be addressed as, and what its section says when it has no walls. | ~110 |
| `src/ui/model/start.rs` | a start between its two acts: what is held, and what each receipt does to it. | ~160 |
| `src/ui/model/claim.rs` | the claim a start leaves on the selection: the row it stands in for, what is not asked about it, and the answer that spends it. | ~130 |
| `src/ui/roster.rs` | every workspace this seat can reach, grouped by channel. | ~120 |
| `src/ui/convs.rs` | the aimed wall's conversations. | ~110 |
| `src/ui/chat.rs` | one conversation as rows, and the live fold the lane hands over whole. | ~170 |
| `src/ui/composer.rs` | what an operator types, and the gesture it becomes — one box, three subjects. | ~90 |
| `src/ui/composer/start.rs` | the half that begins a conversation rather than continuing one. | ~55 |
| `src/ui/keys.rs` | the keyboard: which list the arrows belong to, the walk that is the selection, and the one gate. | ~160 |
| `src/ui/shell.rs` | the layout, the width policy the panes yield by, and the notice that stands where content would have been. | ~110 |
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
| `src/snapshot.rs` | **the seat rendered off-screen**: the matrix's sizes, where a shot lands, the one settled frame, and which widths the layout still promises a shape. `cfg(test)`. | ~125 |
| `src/snapshot/worlds.rs` | the four named world states the matrix photographs, built from the window fixtures rather than from a second set. The walk's screen set is part of the parity instrument, which is why the start's own screen is one of them. `cfg(test)`. | ~80 |
| `src/snapshot/reach.rs` | assertion (a): the walk to the seat's one covered pane and back, asked of the accessibility tree. `cfg(test)`. | ~100 |
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

