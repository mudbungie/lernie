# The wire corpus

The frames this seat is judged by, both directions. **The frames are not
this repository's.** They are yog's generated wire conformance corpus (yog's
`corpus/README.md`, REMOTE §3), emitted from the boundary that *is* the
protocol authority and vendored here verbatim by
`scripts/refresh-corpus.sh`. The seat reimplements the reply vocabulary rather
than linking a shared crate, so nothing but a fixture can catch a field its
decoder silently drops — and a fixture it wrote itself catches only what it
already thought of.

**A fixture that fails is a seat defect until proven otherwise.** The corpus
comes off the codec the protocol is defined by, so a disagreement is this
end's, unless `PROTOCOL` itself moved — which the fixture's own stamp says, and
which `src/reply/tests/corpus.rs` fails on in a sentence naming both numbers.

## Layout

    shapes.json        vendored: every shape upstream has, its field signature,
                       and the protocol the corpus as a whole is for
    request/           vendored whole: one file per request `op`
    answers/   \
    refusals/   > the assertion layer — reply frames, filed by what `read` must
    unreadable/       answer for them

## The three directories are the assertion

There is no sidecar, no manifest and no expected-value file. A reply frame's
directory **is** its expectation, and the replay asserts nothing else:

| Directory | What `read` must answer |
|---|---|
| `answers/` | `Read::Answer` — this build paints it |
| `refusals/` | `Read::Refusal` — the engine said no, in its own words |
| `unreadable/` | `Read::Unreadable` — this seat cannot read it |

Field-level assertions are **not** here. They live in each type's own unit
tests, where a failure names the field. This replay answers one question — does
every frame still land in the class it belongs to — and that is the question a
corpus is for.

## Two provenances, and the bytes say which

A file is one of two things, and nothing has to be listed anywhere for the
replay to tell them apart:

- **A vendored fixture** — yog's own envelope, `{"direction", "shape",
  "protocol", "frames"}`, carrying N frames. It is named for its shape, which
  is how the refresh finds it again, and it is copied in unmodified.
- **A frame this repository wrote** — the bare JSON body of one reply frame,
  with no envelope around it. That is the drop-in shape for a frame captured
  off a live engine, and it is how the malformed and rung-3 readings are held:
  a codec generates only what it can encode, so upstream cannot emit
  `not-an-object.json` or a classification word no build knows.

A file named exactly for a wire shape is the vendored one. Anything else is
named for what it *is* — `workspaces-empty.json`, not `workspaces-ok.json`.

## `unreadable/` is the ledger, and the refresh writes into it

Most of it is not malformed at all. Thirty-odd of its files are perfectly good
frames of kinds this build does not paint — a board, an inbox, a decision
queue. They are unreadable *to this seat*, which is the honest reading of
DESIGN §4.9: the vocabulary decodes only what the window renders.

So the directory doubles as **the ledger of what is not painted yet**, and the
refresh is what keeps it honest. `scripts/refresh-corpus.sh` never classifies:
a shape already filed goes back where it was, and a shape yog has **grown**
lands here, as a new file the diff shows. A kind moves to `answers/` in the
release that starts painting it, and the diff of that move is the record of
exactly what the release added. A kind that ought to be painted and sits here
is a filed ball, not an oversight.

## The request direction

`request/` is upstream's whole request vocabulary, sixty-odd ops. A request has
no `Read` class, so its two assertions are upstream's own contract for a client
(`src/verbs/tests/corpus.rs`):

- **Every frame decodes as a gesture this seat can route.** Including the ops it
  has no word for — those cross through `lernie ask` unchanged, and the thing
  that costs something if it is wrong is the workspace slot the gesture is
  routed by. The expected slot is read off `shapes.json`'s signature rather
  than off the seat's own rule, so it is a second opinion.
- **Every frame the seat's encoder can compose round-trips**, key for key and
  therefore byte for byte. The frames it *cannot* compose are recorded in
  `UNEMITTED` with the reason — a count that moves fails until the reason is
  rewritten, which is the decision being recorded rather than passed over.

## Refreshing

    scripts/refresh-corpus.sh ../yog

The seat **vendors** rather than reading a checkout at test time, and the
reason is not taste: this crate must build and its suite must pass on a box
that has never held a yog checkout, and a test that reads a path nobody
configured is a test that is skipped everywhere but one machine. What vendoring
costs is staleness, and that is what the protocol stamp is for.

## Both directions, everywhere

The replay fails a directory that enumerates **nothing**, a fixture carrying no
frame, a file at the corpus root that no class claims, and a vendored file that
`shapes.json` does not name — as well as the reverse, a shape upstream has that
nothing here files. A broken walk must not pass as a clean corpus.

## Nothing real

The vendored frames carry only synthetic content — house workspace and
conversation names, `/ws`-style paths, fabricated ball ids — and upstream keeps
them that way. A frame captured off a live engine carries workspace names,
conversation ids, transcript prose and sometimes an address; every one of those
is disclosure on a published branch. Rewrite the values before the file lands.
`make leak-scan` catches the mechanical half and cannot catch the rest.

## The pair in `refusals/`

`unknown-workspace.json` and `unregistered.json` are near-identical on purpose.
A workspace this client is not registered in answers exactly as one that does
not exist — under the protocol's own rule that everything out of scope is
*absent* rather than *forbidden*, because a scope error that confirms existence
is a disclosure. The two files are the record that a seat must not try to tell
them apart.
