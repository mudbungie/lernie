# The reply corpus

Reply frames as the wire carries them, one per file, replayed through
`lernie::reply::read` by `src/reply/tests/corpus.rs`.

Every file here is **exactly what a reader receives**: the JSON body of one
reply frame, with no envelope of this repository's own around it. That is the
whole drop-in contract — a frame captured off a live engine, or emitted by a
conformance corpus upstream, is copied in as a file and needs no translation.

## The three directories are the assertion

There is no sidecar, no manifest and no expected-value file. A frame's
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

## Why `unreadable/` holds valid answers

Most of it is not malformed at all. `unpainted-kind-*.json` are perfectly good
frames of kinds this build does not paint: a board, a decision queue. They are
unreadable *to this seat*, which is the honest reading of DESIGN §4.9 — the
vocabulary decodes only what the window renders. The start family's two left
this directory when the start pane landed; the diff of that move is what the
paragraph below is about.

So this directory doubles as **the ledger of what is not painted yet**. When a
pane lands and its kind becomes paintable, its fixture moves to `answers/`, and
the diff of that move is the record of exactly what the release added. A kind
that ought to be painted and sits here is a filed ball, not an oversight.

## Adding to it

- Name a file for what it *is*, not for the assertion — the directory carries
  that. `workspaces-empty.json`, not `workspaces-ok.json`.
- Keep it small enough to read. A fixture nobody reads is a fixture nobody
  maintains.
- **Nothing real.** A frame captured off a live engine carries workspace names,
  conversation ids, transcript prose and sometimes an address; every one of
  those is disclosure on a published branch. Rewrite the values before the file
  lands. `make leak-scan` catches the mechanical half and cannot catch the rest.
- The replay fails a directory that enumerates **nothing**, in both directions:
  a broken walk must not pass as a clean corpus.

## The pair in `refusals/`

`unknown-workspace.json` and `unregistered.json` are near-identical on purpose.
A workspace this client is not registered in answers exactly as one that does
not exist — under the protocol's own rule that everything out of scope is
*absent* rather than *forbidden*, because a scope error that confirms existence
is a disclosure. The two files are the record that a seat must not try to tell
them apart.
