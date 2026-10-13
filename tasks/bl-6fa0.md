+++
title = "lane: the verbs and the overlays — every slash noun and verb, the completion menu's narrowing and window, Tab/Enter/Esc order, every overlay at desk/narrow/phone widths, /flag and /delete prefill, /find, the reach bound, keyboard-only operation"
created = 1791867477
updated = 1791869505
claimant = "LaneVerbs"
parent = "bl-957a"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Lane 2 — verbs and overlays (LaneVerbs). Living scenario log.

Rig: isolated yog 0.0.76 'yog fixture busy' on 127.0.0.1:7742 under ~/.cache/stress/verbs (seat roots there too); Xvfb :92 1440x900; seat built from each claimed worktree (release); keys by XTEST (xdotool windowfocus + key/type); screenshots by ffmpeg x11grab, kept in the session scratchpad. Widths: desk 1440x900, narrow 900x600, phone 420x820 (xdotool windowsize).

| # | gesture | expected | observed | verdict |
|---|---|---|---|---|
| 1 | launch, nothing aimed, type / | menu with window verbs | no box at all ('pick a workspace…'); only door is Ctrl+X | fail bl-fa26 (duplicate of bl-2cb0, landed by LaneAsks; verified on main) |
| 2 | Ctrl+X anywhere (real window) | switcher | nothing: winit delivers Event::Cut | fail bl-8c49 (fixed) |
| 3 | / at head of draft | menu: verb + summary from the verb table | as expected; rows follow offers | pass |
| 4 | /r, Down Down, Tab | narrows; window around choice; Tab completes '/restore ' | as expected | pass |
| 5 | Esc on menu | menu clears | clears draft AND dropped the caret; also dismissed a standing notice in the same press | fail bl-b1a1 (fixed) |
| 6 | / mid-draft | no menu | no menu (parsed only at head) | pass |
| 7 | slash line + Enter | never sent to the model | never sent (composer returns on slash lines) | pass |
| 8 | Ctrl+X pick, then type | typing lands in the box | lands nowhere until clicked | fail bl-b1a1 (fixed) |
| 9 | every noun at desk: records attention help doctor trail board prices find enroll roles login clients config fleet unmake conversations | name line + Esc closes; content whole or scrolling | all open and close; enroll keeps whole cover; subject sometimes on 2nd line (p3, unfiled) | pass |
| 10 | /trail /board /records with an asking conversation selected | fill in a beat | 'waiting to hear' ~30 s: attention read starves the serial pass | fail bl-5439 (fixed: attention on its own lane) + yog bl-d124 (open, engine) |
| 11 | /help | the window's words | engine op table dressed as slash words the box does not take | fail bl-55ed (fixed) |
| 12 | help vs lernie help | same words, same order | different subjects by design; window half now read off the menu table in its order | pass after bl-55ed |
| 13 | overlays at narrow and phone | nothing off the glass | all fit/wrap/scroll; menu windows rows; rows elide at phone | pass |
| 14 | /find + 500-char needle | wraps in box | box widened past the window (egui one-line field allocates its overflow) | fail bl-4f8d (fixed) |
| 15 | /find with CJK/emoji | reads back | tofu boxes | fail bl-3956 (open, fonts) |
| 16 | /nosuchverb, //, / Enter | nothing fires, says so | 'nothing here answers to that'; bare / Enter fires first row (/plan) | pass |
| 17 | /flag, /interrupt with no words | fires nothing, says why | fired nothing, said nothing | fail bl-df70 (fixed) |
| 18 | /delete with no or wrong name | fires nothing | deleted the conversation (wire deletes a leaf on any word) | fail bl-df70 (fixed) |
| 19 | /stop children | subtree stopped | 'children' dropped | fail bl-0740 (fixed) |
| 20 | /pin summary, /prices rate hint, /fleet empty copy | true words, whole | 'strip' named; hint cut; 'below' and one sentence twice | fail bl-0740 (fixed) |
| 21 | Esc order: overlay, menu, notice | in that order | holds after bl-b1a1 (menu Esc no longer also dismisses) | pass |
| 22 | Ctrl+X over an open overlay | replaces it | stacks; choosing lands back in the pane | fail bl-339d (fixed) |
| 23 | Tab from the box | ring on what it lands on | lands on the rule's new segment control, no ring | fail bl-8e3e (open) |
| 24 | rapid /trail Enter Esc x15; Down/Up held 2.5 s in menu | no crash; saturates | as expected | pass |
| 25 | parity walk / reach (snapshot tests) | green | green | pass |

Also seen, outside this lane: the fixture engine's board/balls/search read ball stores outside the fixture root (yog bl-d124 body); the rule's 'plan' toggle is not shown while on (LaneDaily's).

Final check on merged main (release build, same rig): typing at launch with nothing aimed opens the menu; Ctrl+X → pick → typing lands; /trail fills with an asking conversation selected; /delete without the name says what arms it and fires nothing.
Open question for the operator: while the ask stands the caret deliberately stays off the box (digits answer the ask), so typing a slash word there needs a click first.