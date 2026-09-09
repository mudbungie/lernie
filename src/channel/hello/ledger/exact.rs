//! **The exact era: one integer per change, 2 through 18.**
//!
//! Every entry below was written under the discipline `PROTOCOL` carried
//! until 19 — the number moved for *any* change to an existing shape, a
//! gained field included, and a client re-pinned for each one. It is kept
//! whole and unedited in reasoning because it is the record of what this seat
//! did about each: a corpus replay catches a shape that CHANGED, and only
//! words catch a shape that changed its MIND.
//!
//! It is a module of nothing but prose, split off [`super`] (bl-aa01) on the
//! seam that era's end drew: that file is the discipline in force and grows
//! only when a MAJOR is cut, and this one is closed — 18 is its last entry
//! and there will never be a nineteenth.
//!
//! **2 was yog bl-77be's bump**, and it was the second clause of that rule
//! rather than the first: four shapes grew an optional field
//! (`request/advertise` and `reply/clients` gained a tool's `subject_cwd`
//! consent, `request/invoke` and `reply/invocations` gained the subject's
//! `cwd`), and — the part no ledger can see — REMOTE §5.5 changed what a
//! follow frame's `text`/`thinking` are taken to say, from the whole
//! accumulated answer to what landed since the previous frame. The spelling
//! did not move; the meaning did, which is exactly what this integer is for.
//!
//! **3 and 4 are two bumps of one unreleased cycle** (REMOTE §9.10, §9.11),
//! and the pair is why this integer is not a count of releases. 3 gave
//! `reply/conversations`' row, the §6 queue row and the `agent` answer a
//! `failure` clause — why the conversation's latest model call failed — and 4
//! gave the queue row a `flag` object beside a new `flagged` signal token.
//! Each is a gained field, which §3's rule bumps whether or not a reader needs
//! it; the ledger's granularity is per bump and not per release, so a shape
//! touched twice in one cycle costs two integers. Neither number was ever
//! spoken by a peer.
//!
//! **This seat consumes exactly one of the four fields** (bl-d774), which is
//! DESIGN §4.9's rule holding rather than a shortfall: `failure` reaches
//! [`crate::reply::convs::ConvRow`] because the conversation list is the pane
//! that paints the row it hangs on, and the `agent`, `attention` and queue
//! shapes stay in the corpus ledger under `unpainted/` because no pane here
//! reads them. A field is carried by the release that paints it.
//!
//! **5 is the first clause of the rule and the one this seat cannot check**
//! (REMOTE §9.12, upstream bl-e654; bl-e6ee here). `reply/governing` lost
//! `branch`, gained `follows` and `diverged_lineages`, and — the half no
//! signature can see — its `oid` **changed meaning under the same key**: it
//! named the `config/*` ancestor an agent's branch forked off, a commit that
//! never moved, and now names the commit control reads at each step boundary,
//! the followed lineage's head. The doctrine inverted with it, from
//! fork-is-the-freeze to follow-the-tip.
//!
//! **Nothing here decodes `governing`**, so this seat paid the integer and no
//! field, and that is the whole of what it owed: the shape falls to
//! [`crate::reply::read`]'s unknown-kind arm and its fixture asserts exactly
//! that from `corpus/unpainted/`. The trap is recorded here rather than
//! nowhere, because it is aimed at whoever lands the pane: a reader that took
//! `oid` for the fork commit would paint a plausible number that has been
//! wrong since this bump, and it is the one kind of drift a corpus replay
//! cannot catch — the bytes are well-formed and the field is spelled the same.
//!
//! **6 is a bump this seat paid for one row and not for the two ops beside
//! it** (REMOTE §9.13, upstream bl-23bd; bl-675e here). `reply/providers`'
//! rows gained `effort` and `priority`, two required booleans saying which
//! tuning knobs that provider row actually takes — a capability stated as a
//! column of the row it is about, rather than as a second answer a seat would
//! have to join back. The `/effort` and `/priority` **ops** that landed with
//! them moved nothing: a new op is a new spelling in an existing vocabulary,
//! and a peer that has not heard of one refuses it in band by name, which is
//! the first paragraph of this comment rather than an exception to it.
//!
//! Nothing here paints providers either, so this is the second consecutive
//! integer bought without a field — see [`crate::reply`] for why that is the
//! arrangement working. The request half is not free of obligation, though:
//! the two new ops carry a top-level `workspace` and `src/verbs/tests/corpus`
//! asserts this seat routes every vocabulary frame by the slot upstream's own
//! signature says it carries, which is where a miss would be silent.
//!
//! **7 is the first bump this end reads a field out of** (REMOTE §9.14,
//! upstream bl-8758; bl-38d4 here). Every `reply/help` row gained `surface`,
//! classing the op `control` — every seat owes it a discoverable interactable
//! — or `machine`, spoken by programs and owed nothing. It is wire-visible
//! because it rides a reply this seat vendors, and it is load-bearing here:
//! the field IS the roster the interface-parity gate judges this window
//! against (`crate::snapshot::parity`, yog's `docs/PARITY.md` §2). The bump
//! also carried the `roles` shape, which nothing here paints yet and which
//! therefore landed in the corpus ledger — `corpus/README.md` on why that
//! directory is the record rather than an oversight.
//!
//! **8 is a bump for a shape this seat does not speak either half of**
//! (REMOTE §5.1, upstream bl-66d4; bl-2604 here). `reply/advertised` gained
//! **`wrote`**, a required boolean saying whether the engine stored the
//! presented tool set or found it identical to what it held and wrote nothing.
//! It is required rather than optional-absent-reads-false on purpose:
//! absent would decode as *"nothing was restored"* — the reassuring answer —
//! on exactly the build too old to tell.
//!
//! **The fact is a foot's, not a seat's.** `advertise` is the tool host's
//! gesture, and a `true` on any re-presentation after the first is that host
//! learning some other connection bearing its certificate blanked its set
//! while it was executing and holding no parked read. A seat presents no set,
//! so this end has no gesture to receive the receipt for and nothing to say
//! about it: the fixture stays in `corpus/unpainted/`, the shape keeps
//! falling to [`crate::reply::read`]'s unknown-kind arm, and the pane that
//! would paint a tool host's state does not exist here yet (bl-e53c). Third
//! consecutive integer bought without a field, and — like the two before it —
//! the corpus refresh is the whole of what the re-vendor moved: one signature,
//! one fixture, nothing else in forty-seven shapes.
//!
//! **13 crosses five bumps at once, and only one of them is a break**
//! (bl-249b here; yog's own `src/wire/hello.rs` carries the ledger and REMOTE
//! §9.16-§9.18 the reasoning). The released engine moved five integers while
//! this build sat at 8, and the shape of the arrears is the point: four of the
//! five *gained* something, and a gain reaches a decoder that reads named
//! fields as nothing at all. What has to be paid is the one loss.
//!
//! - **9** — `reply/steps` **LOST** `auth_failed`, and `reply/transcript`
//!   gained the `wounded` entry beside it. This is the break: the field was
//!   read here with [`super::super::reply::fields::flag`], which is required,
//!   so at 13 every steps reply refuses on a key the engine no longer writes.
//!   REMOTE §9.16 names the replacement outright — *"a seat that read
//!   `auth_failed` reads `wound == \"refused\"` instead"* — and this build
//!   already decodes `wound` and `auth_row`, so the affordance is re-pointed at
//!   the vocabulary it was always one arm of, and no reading is lost.
//! - **10** — no field moved anywhere. `attention` became follow-class: the
//!   same ask and the same reply shape, but a *sequence*. It is the one class
//!   of change a field-signature ledger cannot see, and it is exactly why the
//!   preface is strict equality rather than a signature comparison.
//! - **11** — `reply/ops` gained `failed`, `exit_label` and `standing`.
//! - **12** — every queue row gained `says`, and `reply/acknowledged` arrived
//!   spelling rows the same way.
//! - **13** — `reply/config` gained `settings`, the file's own schema applied
//!   to the bytes answered beside it.
//! - **14** — two additive fields on two shapes in use, landed at one version
//!   because two bumps a minute apart would make every client re-pin twice for
//!   one wave. `request/enroll` gained an optional `address` — what the DEVICE
//!   will dial, which need not be the engine's own view of itself — and this
//!   seat composes no door for it (`verbs::tests::corpus::emits`'s ledger,
//!   bl-2f18). `reply/clients` gained an optional `last_seen`, and this seat
//!   paints the half that decides something: **absent is a machine that has
//!   never once dialled**, which on a terminal is the only reading available,
//!   since presence reads false for every row when every verb opens and closes
//!   its own connection.
//! - **15** — `reply/follow` gained `tools`, the TOOL WINDOW: an entry as a
//!   call is dispatched and another when its capture lands. Required rather
//!   than absent-reads-empty, on `reply/advertised`'s precedent at 8 — absent
//!   would read as *nothing ran*, the reassuring answer, on exactly the build
//!   that cannot tell. It is the first bump this seat spends on the lane it
//!   was already following: the CLI's follower now holds a fold so a closing
//!   entry that restates nothing can still be named
//!   (`crate::render::tail`), and the window paints the window under the live
//!   turn.
//! - **17** — two shapes at one version, both the same litany 0.0.11 pin
//!   landing upstream. A delivered row gained an optional `sender_name`, and
//!   the deposit envelope in `reply/inbox` the same fact as `from_name`: the
//!   sender's DISPLAY name, present exactly when the sender is an agent
//!   wearing one. Painted on both faces beside the handle and never instead of
//!   it (`reply::transcript::said_by`) — the framing sender is the addressing
//!   key and stays true once an agent is deleted and its name recycled, where
//!   every message a child sent was otherwise headed by sixty characters of
//!   timestamped hex. The version's other half is a VALUE and not a field: the
//!   §6 signal vocabulary gained `truncated`, which this seat already carries
//!   as itself on rung 3, so it costs the integer and no decode. And two new
//!   shapes arrived with it — `reply/doctor` and its request — which land in
//!   `corpus/unpainted/` because nothing here paints a doctor yet; the
//!   parity ledger carries the op's line, citing the ball that will.
//! - **16** — `reply/ops` rows gained `client`, the identity that made the act.
//!   Read strictly, like `standing`, and painted on both faces: it is the one
//!   fact on a trail row that nothing later can recover, because presence is a
//!   point-in-time observation by design.
//!
//! **The ledger absorbs the rest, which is the arrangement working rather than
//! a debt.** At the 13 refresh `ops` and `config` were still
//! `corpus/unpainted/` files here, so their gains cost a refresh and nothing
//! else — both have since been claimed by the panes that paint them, which is
//! why 16's gain on `ops` is a decode and not a line in that directory;
//! `acknowledged` and `login` were new shapes and landed there the same way;
//! and the `wounded` entry falls to
//! [`crate::reply::transcript::EntryKind::Unknown`], which is total by design.
//! Four new ops are classed `control` — `login`, `login-tail`, `pin`, `unpin`
//! — so `parity.toml` gains four lines rather than the gate reddening, each
//! citing the ball that will delete it.
//!
//! - **18 is three lanes in one night, four shapes, and one number**
//!   (bl-c515 here; yog bl-58bb, bl-ab53, bl-dd88, bl-94a5, bl-9ced). Upstream
//!   batched them on 14's own reasoning — two bumps a minute apart make every
//!   client re-pin twice for one wave — with a second argument this release
//!   has: under the four-repository gate a raise HOLDS yog's release until
//!   three consumer mains vendor it, so each extra integer is another window
//!   in which no published suite composes. **This seat's answer is a different
//!   one of DESIGN §4.9's readings per shape**, which is the whole of what the
//!   version cost:
//!   - **`reply/follow`'s tool-window entry gained `held`** — the capability
//!     control's reason for parking the call. A held invocation is stopped
//!     *before* the executor is entered, so litany lands neither of the two
//!     files the window is made of and the lane said nothing at the one moment
//!     the operator was the blocker. Presence is the status, `exit_code`'s own
//!     discipline on the same entry. **Decoded and PAINTED on both faces**,
//!     for the reason `sender_name` was at 17 — the defect is on the glass —
//!     and painted as a transition: the park is a row of its own, the dispatch
//!     row it replaces does not appear because there was no dispatch, and an
//!     answered park keeps its row and gains the two it was waiting for
//!     (bl-3a1f).
//!   - **`reply/steps` gained a fourth `framing` word, `in_flight`** — the
//!     step being written right now, told apart from the one an interrupt cut.
//!     A new VALUE and not a new key, which no field signature can see: rung 3
//!     already carries it, so it costs the integer and no decode, and what it
//!     owes is the assertion that the word arrives and is never read as
//!     `killed` — the word that makes an interrupt legible, and one a healthy
//!     conversation must not be described by.
//!   - **Two new ops and a new reply kind at no version cost** (REMOTE §9.22):
//!     `proposals`, `proposal` and `reply/proposals`, the learning loop's
//!     operator half. New ops are free by §3's rule, so what a client owes is
//!     a re-vendor and not a re-pin. This release paid the WIRE half whole and
//!     the GLASS half not at all: the reply is decoded, both gestures are
//!     composed, the command line paints the listing and the proposal whole,
//!     and `parity.toml` carries a line per op citing bl-a1d6.
//!   - **`request/answer` and `reply/answered` gained `scope`** — how far one
//!     capability answer stands: the held `call`, the `conversation` and its
//!     descent, or the `workspace`. Required in both directions rather than
//!     optional-defaults-to-`call`, because an absent field would let the two
//!     ends disagree about how wide the instruction was and *wider* is the
//!     reading nobody may arrive at by accident. So this seat states it on
//!     every gesture it composes, `call` included, and what stays optional is
//!     the operator's TYPING of it. It is the shape that cost a GRAMMAR: a
//!     word the wire demands and the row cannot carry rides a door of its own
//!     (`crate::cli::answer`) on `enroll`'s precedent, and the window offers
//!     the two wider reaches as seats that name their own reach rather than as
//!     a picker, because a sticky picker is exactly how *wider* is arrived at
//!     by accident.
//!   - **The `prepared` body gained `role`**, and every shape carrying one
//!     gained it with them — four (REMOTE §9.21). **The decode cost nothing**,
//!     which is rung 4 in the write direction paying for itself a second time:
//!     the body crosses back verbatim, so the field rode through untouched
//!     from the moment it existed. What it cost is the other kind of grammar —
//!     `prepare` answers `null` and the SEAT states what the operator asked
//!     for on the fire (`lernie start … --role <name>`), because which role a
//!     conversation is born on is the choice made between the two acts and
//!     plan mode is exactly that choice.
//!
//! **And 18 is the first version yog carries a `PROTOCOL_PUBLISHED` beside**
//! (yog bl-9ced), which is why four shapes could land on one integer: its
//! corpus ledger used to refuse a moved signature at the version the record
//! was last GENERATED at, a proxy that advances whether or not the bump ever
//! shipped. This seat needs no such constant — it VENDORS the corpus rather
//! than generating it, so *has this version shipped* is not a question it ever
//! has to ask.
