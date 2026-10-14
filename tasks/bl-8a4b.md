+++
title = "the strings: every sentence the window says to an operator — notices, refusals as painted, hints, overlay names, empty states, the start's and the ask's lines — inventoried and the opaque ones reworded into plain words that name the door that mends it; engine sentences the seat cannot reword are filed against yog with the wording proposed"
created = 1791951826
updated = 1791952002
claimant = "LaneStrings"
parent = "bl-3c8a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Every sentence the window can paint, inventoried; the opaque ones reworded so a person who never read the code knows what happened and what to do next, the next thing named as the slash word or key that does it. The seat's frame on engine refusals is kept (reasoning in DESIGN §4.42 item 11), and a refused role write now carries the seat's own `/config` door, decided by the op refused, never by the engine's prose.

## Inventory

Verdict: P = plain (kept), O = opaque (reworded).

| site | today's words | v | new words |
|---|---|---|---|
| notice.rs `Notice::line` Refused | the engine refused: {said} | P | kept (frame states the act did not happen and whose words follow) |
| notice.rs `Notice::refused` (new) | — | — | {said} — /config opens the settings editors (for refused `model`/`effort`/`priority`) |
| notice.rs Unreadable | this seat could not read the answer: {why} | O | lernie could not read the engine's answer ({why}) — updating lernie may fix it |
| notice.rs Unreachable | this seat could not reach it: {name}: {why} | O | lernie could not reach {name}: {why} |
| notice.rs InDoubt | `{op}` is IN DOUBT: it reached the engine and no answer came back ({why}), so it may have run. This seat never resends an act — the world is the record, and the reads on this window are already asking it again. | O | {your message / the new conversation / the nudge} reached the engine but no answer came back ({why}), so it may have run — look before doing it again; this window is already re-reading |
| notice.rs Unsent | `{op}` was not sent: it never left this seat ({why}), so nothing happened — it is safe to do it again. | O | {act} never reached the engine ({why}), so nothing happened — it is safe to do it again |
| notice.rs answered | answered {tool} ({tool_use}): {verdict} for this {scope} — … | O | {verdict} {tool} for this {scope} — the conversation is running again / it stays parked |
| notice.rs seen | seen {name} — it is still asking … / off the queue; N still waiting | P | kept |
| notice.rs floored(true) | a floor stands over this conversation: every tool call but a read waits for you | O | auto-approval is off for this conversation: every tool call but a read waits for you — /restore gives it back |
| notice.rs floored(false) | the floor is lifted — its tool calls are adjudicated by the ordinary policy again | O | auto-approval is back for this conversation — its tool calls follow the workspace's usual rules again |
| notice.rs delivered | delivered onto {t} {at}, {from}, off the pinned {base} — the ball is not closed and what its close delivers is unchanged; "no source ref" | O | … based on {base} — the ball stays open, and closing it still delivers what it would have; "no branch to deliver" |
| notice.rs retired | …its source ref went with it: this project's retention says the keep had expired / …its source ref, and so its whole diff, is still addressable | O | …its branch is deleted: this project keeps them only so long / …its branch, and so its changes, are kept |
| model/acts.rs enrolled-while-closed (was Unreadable) | the engine enrolled X while this pane was closed, so the material is gone — enroll again to mint another | O | the engine enrolled X after the enroll pane closed, so its code is gone — /enroll makes another (now a note, not an "unreadable") |
| model/enroll.rs too long | the engine's material will not fit one symbol: … | O | the enrollment is too long to draw as one QR code: … |
| reply/read.rs unknown kind | reply: this seat does not paint a "k" answer. The kind is refused by name … corpus/unpainted/ and parity.toml record between them. | O | a "k" answer, which this version of lernie does not show |
| reply/* parse errors (missing or non-string field "x", "… row: not an object") | as is | P (detail) | kept inside the Unreadable frame, which now carries the remedy |
| render.rs CLI unreadable | this seat cannot read that answer: {why} (`--json` prints the frame) | O | lernie could not read that answer ({why}) — updating lernie may fix it; `--json` prints it as it came |
| render receipts nudged / flagged / acked / trail-cleared | nudged — the advance is running · flagged — it is on the queue now · the trail's watermark is where you are · the trail is truncated | O,P,O,O | nudged — it is working again · (kept) · every alarm on the trail is marked seen · the trail is cleared |
| reply/transcript.rs wound refused | {provider} refused the last call — sign it in with /login, or move the role with /roles | P→tightened | … — /login signs it in, or /roles picks another |
| reply/transcript.rs wound output_limit | …cut off at its output cap — raise the role's output limit, or ask for less in one step | O | …cut off at its output limit — /config raises the limit, or ask for less in one step |
| reply/transcript.rs wound no_response | the last step produced nothing — /nudge runs it again | P | kept |
| chat/rows.rs WOUND, unknown entry | it stopped · X (word, which this seat cannot read) | P,O | kept · X (word, which this version of lernie cannot read) |
| model/channel.rs `Held::said` Unheld (card, slot, ask hint via `diagnostics::down`) | {wire sentence} | O | cannot reach this engine: {wire sentence} |
| channel/hello.rs mismatch | wire protocol mismatch: this seat speaks version N, the engine speaks M. There is no negotiation — upgrade the older component until both speak one version. | O | this lernie speaks protocol N and the engine speaks M — update whichever is older, lernie or the engine, until both match |
| channel material / dial verify / ladder sentences | provisioning instructions naming the four files | P (CLI-shared) | kept whole: they carry the only remedy, and `lernie entries` prints them; framed on the glass by "cannot reach this engine:" |
| composer HINT / BREAKS / NOWHERE / UNAIMED parts | say it — Enter sends · Shift+Enter breaks a line · nothing is aimed · / for a verb · Ctrl+X for a conversation or a new workspace | P | kept |
| composer/start.rs goal, UNSIGNED | what this conversation on {ws} is for — Enter begins it · no provider yet — /login or /config, then pick one on the rule | P | kept |
| composer/found.rs | new workspace: <name> <what it is for> — Enter founds it | P | kept |
| model/start.rs lines | started «x» in ws · not started in ws: {why} · starting in ws: goal… | P | kept |
| chat/panel.rs WAYS | /help for commands · Ctrl+X for a conversation · /roles for the model · /login for a provider | O (one part) | … · Ctrl+O for the model · … (§4.42 item 11: Ctrl+O opens the model's dropup) |
| chat/panel.rs engine line | N wall(s), M conversations | O | N workspace(s), M conversations |
| chat/panel.rs FOUNDING | Ctrl+X then + new workspace to found one | P | kept |
| chat.rs NO_CONVERSATION / NO_AIM | Ctrl+X picks a conversation, or begin one in the box below · Ctrl+X picks a workspace to begin in, or a conversation | P | kept |
| ask.rs HEADING / HINT / ask::hint | it is asking · a digit picks · arrows move · Enter answers · it is asking — 1–6 answers, or say it — Enter sends | P | kept |
| queue.rs parked | held at the boundary: {tool} ({tool_use id}) — {reason} | O | waiting for your approval: {tool} — {reason} |
| queue.rs HEADING / NOT_ANSWERED / NOTHING / GO / WIDER | waiting on you · … · nothing is waiting on you — a conversation that asks will appear here | P | kept |
| overlay HINT | Esc closes | P | kept |
| slash NOTHING / HINT / fire.rs needs-words | nothing here answers to that — Esc clears it · /flag needs its reason after it — /flag <reason> … | P | kept |
| status/dropup NOTHING / HINT | nothing heard yet · arrows walk · Enter picks · Esc closes | P | kept |
| dock | + N running · name · Ctrl+T | P | kept |
| bar/rows EMPTY / NOTHING_AIMED | no conversations here yet — + new begins one · pick a workspace below | P | kept |
| convs NO_CONVERSATIONS / NOT_ANSWERED | … · waiting to hear about this wall | O (2nd) | waiting to hear about this workspace |
| roster NOT_ANSWERED / NO_WALLS | waiting to hear from this channel · this engine holds no workspace this seat is in | O | waiting to hear from this engine · this engine holds no workspace this box is in |
| roster/wall NO_NAME_HERE | no entry here names it, so nothing typed here can address it | O | this box has no connection set up for it, so nothing here can reach it |
| commands (/help) ENGINES / NO_OPS | what these engines answer — each engine's own op table, in its own spelling; the words above are what this box takes · this engine names no op | O | what each engine answers to — its own commands, for the lernie command line; the words above are what this window takes · this engine lists no commands |
| commands keys | Esc closes the overlay, then the menu, then the notice · Ctrl+O changes the model a role runs on · 1 to 9 answers the ask by its number · Ctrl+X every engine, its walls… | O (stale/jargon) | Esc closes a dropup, then the overlay, then the menu, then the notice · Ctrl+O changes the model this workspace runs on · 1 to 6 answers the question it is asking, by its number · …its workspaces… |
| slash/rows.rs summaries | the conversation's own ledger · run this wall's balls · the aimed wall's · its walls | O | this conversation's steps, files and history · this workspace's balls · this workspace's · its workspaces |
| verbs summaries on the menu: stop / nudge / retarget / delete / pin / unpin / unmake / enroll / login / ops (trail) / prices / ack / clear-trail | kill the driver held on a conversation; `children` takes the subtree with it · start a driver on a conversation that has gone quiet · settle a conversation onto the head of the config lineage governing it · …the typed name arms taking its children too · rank this workspace first in every seat's list of walls… · take this workspace back out of the pinned run · unmake a workspace; the typed name is the arming · mint a new box's material… · what this workspace's wall can sign in to · every act that crossed the boundary… · the price table… and the world's spend… · acknowledge every alarm on the ops trail · truncate the ops trail… | O | stop a conversation's work now; `children` stops what it started too · set a conversation that has gone quiet working again · move a conversation onto the latest version of its settings · delete a conversation; typing its name confirms it, and takes its children too · keep this workspace at the top of every list of workspaces · let this workspace fall back into name order · delete a workspace; typing its name confirms it · let a new box connect: its credentials as a code to scan, a line to paste or, if asked, four files · the providers this workspace can sign in to · everything done on this engine, from any window or box, newest last · what each model costs, the spend ceiling, and what has been spent · mark every alarm on the trail seen · clear the trail; the clearing is the new trail's first row |
| verbs summaries kept | interrupt, revoke, restore, flag, attention, roles, clients, search, doctor, models→"the models one provider offers", login→"sign this workspace in to one provider" | P / O | models and login reworded as shown |
| login pane | waiting to hear what this wall can sign in to · this wall routes no provider · sign in to this row · waiting to hear what this row offers · this row offers no model id · this wall is held on another box … lands in its wall · if the flow opens an authorize URL, it redirects to the ENGINE's loopback … · run it by hand, on the box that holds this wall: | O | …this workspace can sign in to · this workspace uses no provider — /config names one · sign in to this provider · …this provider offers · this provider offers no model · this workspace lives on another box, and the sign-in runs there — the credential is kept there, not on this one · if the sign-in opens a web page, that page expects a browser on the engine's box: finish it there, or forward the port by hand · run it by hand, on the box this workspace lives on: |
| login/run.rs | nobody has signed in to this row here · signed in · the sign-in ended with status N | P | kept |
| tuning pane | waiting to hear what this wall's roles are · this workspace's config declares no role · asking for "x", which this seat has no seat for · hint "provider row" | O | …this workspace's roles are · this workspace's settings name no role — /config adds one · asking for "x", a level this version of lernie does not offer · hint "provider" |
| config pane | waiting to hear which lineages this wall holds · this wall holds no config lineage · this file has no typed settings — its bytes are the whole of it · pick a file to read · the box holds what the file holds · this file has changed on the engine… | O,O,O,P,P,P | waiting to hear which settings this workspace holds · this workspace has no settings yet · this file has no fields to edit one by one — edit its text instead |
| config/proposals | nothing is staged for this wall · accept moves the lineage onto it and every conversation on that lineage picks it up… reject deletes the staging branch | O | nothing is staged for this workspace · accept puts it into the settings, and every conversation on them picks it up at its next step; reject throws it away |
| clients pane | waiting to hear which machines this wall holds · no machine is registered in this workspace: registering one is an operator's act on the server that holds it, never a gesture from here | O | …this workspace holds · no machine is registered in this workspace yet — /enroll lets one join |
| find pane NOT_ADDRESSABLE | a hit says where it is in the engine's own path spelling, which no gesture from this box can carry — so these are read here and aimed at by hand | O | a hit says where it is as the engine names it, which this window cannot open — Ctrl+X goes to the conversation by hand |
| find other | type what to look for… · nothing has been looked for yet · this engine found none of it — try fewer words | P | kept |
| trail pane | waiting to hear what has crossed the boundary · nothing has crossed the boundary yet — every act from any seat lands here | O | waiting to hear what has been done on these engines · nothing has been done here yet — every act, from any window or box, lands here |
| clear pane REACH | one gesture cuts the trail of every engine this box holds: | O | this clears the trail on every engine this box reaches: |
| board pane BINDINGS | every ball⇄workspace binding | O | which workspace holds which ball |
| board/acts CLAIM | claim it for this wall | O | claim it for this workspace |
| board other (UNAIMED, NOTHING, UNFILEABLE, UNARMED…) | aim at a workspace to see the balls it holds … | P | kept |
| prices pane | what the world is charged · no row is priced, so no count carries money · hint "input output [cache-read [cache-write]] per Mtok" | O | what the models cost · no model has a price yet, so nothing shows a cost — price one below · "input output [cache read [cache write]], USD per million tokens" |
| fleet pane | waiting to hear what this wall's agents have done · this wall's agents have delivered nothing yet… · this wall's agents have changed nothing yet | O | …this workspace's agents… (×3) |
| records pane | waiting to hear what this conversation's loop has done · its loop has taken no step yet | O | waiting to hear what this conversation has done · it has taken no step yet |
| records/files, drill | a "x" preview/record, which this seat cannot show | O | …which this version of lernie cannot show |
| records/header ABSENT | the engine's snapshot does not carry it | O | the engine did not say |
| records/spine ROLE_SAID | a role is a name the fork point's config declares, and this seat cannot list them yet | O | type a role this workspace's settings name — /roles lists them |
| records/steps orphaned, records/spine notches, workflow, governing (reply/governing.rs) | an orphaned N tail · workflow.yaml marked to config/{lineage} … · policy held at X — N diverged config lineages | O (left) | not reworded: a power-user ledger pane whose nouns are git's and the engine's; flagged for a later pass |
| doctor pane | is this box wired · waiting to hear whether this box is wired · this engine ran no check | P | kept |
| unmake pane | not armed — type the name above, exactly · asked — waiting for the engine · refused unless this engine owns the workspace and nothing in it is live | P | kept |
| enroll pane | enroll a box · not written down anywhere — scan it now, or enroll again | P | kept |

Swept: 381 string literals in src/ui, src/reply and the shared receipts, plus the channel's failure sentences; about 250 reach the glass (the rest are widget ids and parse details). Changed: 93, plus one new seat door.

## Engine's sentences (proposed for yog)

- `` `<id>` is not a plain model id — yog writes it as a block value in providers.yaml; use the Config editors for anything else `` → `` `<id>` is not a plain model id, so it cannot be set from the model list — /config edits providers.yaml by hand ``
- `this client is registered in no workspace here` (roster refusal) → `this box is in no workspace on this engine yet — Ctrl+X, then + new workspace, founds one`
- `sign in first: … holds no credential — write its key into this wall with /config <row>` → drop *wall*: `sign in first: <provider> has no key yet — /login signs in, or /config <row> takes a key`
- `unknown workspace "<name>"` → `no workspace named "<name>" on this engine — Ctrl+X lists the ones it holds`
- the `doctor` rows' check names (wire material, address, anchors) → each row's sentence should end with what to do, not only what failed
- the ask's raw signal words where the engine wrote no `says` sentence (e.g. `held  mail  flagged`) → the engine should always write `says`
- the `wounded` class `refused` reason text from the adapter (provider error JSON) → a one-line summary, the raw body behind it