+++
title = "verify the three-stage fold and the elided collapsed bar on the glass, then install and relaunch"
created = 1792036018
updated = 1792037257
claimant = "Watching"
parent = "bl-4446"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-773e"
on = "claim"

[[blockers]]
id = "bl-0174"
on = "claim"
+++
Verification of bl-4446 on the shipped 0.1.102 seat, driven on a private Xvfb display against an isolated engine laid by `yog fixture busy` (0.0.82) under the lane's own XDG roots, with two conversations grafted into the laid world: one settled turn carrying two Bash calls with prose between and after them, and one in-flight turn whose second call's step is held open the way the fixture's own streaming step is. Two parks were grafted as held marks on the workspace repo, one whose reason sentence names the secret class and one the process class. Shots are kept beside the lane brief, one per item. No code changed in this ball; the operator's window was not relaunched (the binary was installed from main by the orchestrator).

| # | item | gesture | observed | verdict |
|---|---|---|---|---|
| 1 | three-stage working | read the settled turn; click its head; click a step; open the running turn | settled: one line `+ worked · 2 calls · 2 notes` between the operator's turn and the final answer, no step text, no response elbow. Head click: compact steps — each call its call line and one output line, each note one line — with the response elbow standing. Step click: the call opens whole in place. Running turn: the in-flight call stands whole, the finished call compact, no elbow | PASS, with bl-b83d |
| 1b | the toggle's memory | open step 1 in the settled turn, switch to the running turn, fold its step 1, switch back | the running turn's step 1 arrived open though nobody toggled it there, and folding it folded the settled turn's step 1: the memory is keyed by row index alone (`working.rs`, `trail.rs`) | FAIL → bl-b83d (p2) |
| 2 | collapsed bar elides at its width | read at the default width; drag the collapsed edge to about twice it | default: each name ends in `…` after one glyph; dragged: `Ball bl-2007: c…` and the rest, more characters at the wider edge, still elided | PASS |
| 3 | bar head | read the open head; click pin; click + new; unpin and record the ease at 30 fps | pin and + new stand heading-sized, a row taller than a body line; pin toggles to unpin and the bar stays open with the pointer gone; + new puts the caret in the box in start mode on the aimed wall; through the ease the head shrinks to its glyphs and the rows elide, nothing overlapping in any frame | PASS, with bl-6ef6 |
| 4 | trashcan | hover a row; leave; click; Esc; click then Enter; Ctrl+click another row | trashcan at the hovered row's right end with the age stepped left, gone when the pointer leaves; click opens the confirmation on the box naming the conversation with keep and delete and the Enter/Esc hint; Esc keeps with nothing posted; Enter posts delete-agent and the row leaves the listing on the next read; Ctrl+click deletes at once with no confirmation | PASS |
| 5 | the ask's scopes | a park whose sentence names the secret class; one naming the process class | secret: `1. seen · 2. pass · 3. refuse · 4. hold` and the hint says 1–4; process: six rows and the hint says 1–6 | PASS, with bl-9f2b |

By eye: the ask's reason line names the tool twice (seat prefix and engine sentence) — bl-9f2b (p3); start mode's hint says no provider yet while the rule shows a provider and a model — bl-6ef6 (p3, may be the fixture's provider-less wall). The known trailing blank line in an opened call result (bl-63ea) still stands. The fixture cannot produce a park on its own; the held mark was written by hand in the shape the engine reads.