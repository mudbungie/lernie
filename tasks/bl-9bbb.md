+++
title = "the doctor has no control or rendering on the seat: reply/doctor sits in corpus/unpainted/ and parity.toml's doctor row cites only the engine's ball (yog REMOTE bl-28f4, round-1 ruling 7)"
created = 1791438069
updated = 1791438069
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Filed during the 2026-10-07 backlog sweep: parity.toml has carried a `doctor` exemption row since bl-183b re-vendored PROTOCOL 17, and the row cites only yog bl-28f4 (the engine half). No ball on this board owned the seat half, so this one does.

## What upstream says it is (yog REMOTE, bl-28f4)

`/doctor` is the one read that answers "is this box wired": the wire material, the endpoint `wire/address` names, what the process actually bound, the git identity every step commits under; naming a workspace adds whether a conversation there would reach a model (the Prompt door's own gate, quoting the sentence it would refuse with) and who is registered to reach it. It reads and never writes, carries no tally (a seat counts the rows it was handed), and the workspace is optional — the box it is for may hold none.

The reply is `{"kind":"doctor","ok":bool,"rows":[{"check","fact","ok","remedy"?}]}`; the fixture is `corpus/unpainted/doctor.json` and `corpus/request/doctor.json` carries both request spellings (bare and with a workspace). The help roster classes it `control`.

## What this seat owes

1. Decode `reply/doctor` (`src/reply/`), moving the fixture from `corpus/unpainted/` to `corpus/answers/` — DESIGN §4.9: the decode and the pane arrive in one commit.
2. Compose `lernie doctor [<ws>]` in `crate::verbs` beside the other window-level reads (DESIGN §4.21: it addresses a wall only when a workspace is named).
3. A doctor surface on the window: one row per check — the check, the fact, ok/not-ok, and the remedy verbatim where present — on the window-level reads' standing (§4.21), tagged `act:doctor`. With a wall aimed, the read carries that wall's workspace; without, it is the bare form.
4. Delete the `doctor` line from parity.toml in the same commit and rewrite DESIGN §4.11's "the ledger's first line since it emptied is doctor" paragraph as the record of how the row left.
5. Tests: decode with and without `remedy` and with and without a workspace in the request; a snapshot-walk test that sees `act:doctor`; a paint test per ok state.

Done when the ledger row is gone, `corpus/unpainted/` no longer holds a doctor, and an operator on a seat can ask whether the box is wired without a shell on it.