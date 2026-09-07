+++
title = "three answers that mis-report: --json follow prints prose, follow right after start calls a starting conversation 'at rest', and ask names a workspace field that is present"
created = 1788746112
updated = 1788746744
claimant = "Cantaloups-S8"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r2"]
+++
Round-2 devadmin lane. Three separate sightings of `lernie follow` and
`lernie ask` answering something other than what the flag or the field promised.

## 1. `--json follow` prints prose

`lernie --help`: "`--json` prints the frames exactly as they crossed instead,
one envelope per line, which is what a script wants."

    $ lernie --json follow ops KhakiArchway
    KhakiArchway is at rest (quiescent) — nothing more will arrive until it is
    nudged or messaged

That is the whole output — one line, not JSON, on a stream a script is parsing
one envelope per line. Every other frame in the same run is a proper envelope:

    {"kind":"follow","ok":true,"stream":{},"tools":[{"input":"…","tool":"alpha2_Bash","tool_use":"toolu_01…"}]}

so a reader that `json.loads` each line dies on the terminator, on the one line
that says the watch is over.

## 2. `follow` immediately after `start` answers "at rest (stopped)"

    $ lernie start ops '<goal>'
    staged in ops
    started NoodleGarden
    $ lernie --json follow ops NoodleGarden
    NoodleGarden is at rest (stopped) — nothing more will arrive until it is
    nudged or messaged

Exit 0, immediately. Seconds later the same conversation's row read
`"state": "live"` and it went on to run four routed commands. Re-attaching by
hand then streamed all of them. So the documented sequence — start, then watch —
does not work: `follow` reads the pre-driver state, calls it rest and leaves.
The operator's only recourse is to guess how long to sleep first.

## 3. `ask` names a field that is present and correct

    $ lernie ask '{"op":"config","workspace":"ops","target":{"file":"brazen"}}'
    refused: missing or non-string field "workspace"

`workspace` is there, at the top level, and it is a string. The envelope's real
fault is that the config family carries its workspace INSIDE `target` —
`{"op":"config","target":{"workspace":"ops","file":"brazen"}}` works — which
`src/envelope.rs` states well in its own comment ("Three shapes, because yog's
own typed table has three … one level down inside `target` on the config family,
whose destination *is* its address"). None of that reaches the refusal, which
names the field and not the place, on the one op where the place is the whole
question. The two adjacent refusals are both good — `config: target is not an
object`, `config: missing target` — so this is the one shape in the family that
mis-reports.

`lernie ask` is documented as "not a fallback: it is the surface, and the typed
verbs are its shorthand", and the config family has no typed verb for a raw
read, so this is the only door to it.

p3 each; filed together because they are one class — an answer that is right
about the world and wrong about what the caller asked. Item 2 is the one that
costs a user something every time.