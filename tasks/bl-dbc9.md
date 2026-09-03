+++
title = "conversation management at the row: a right-click context menu carrying the conversation's acts"
created = 1788405831
updated = 1788405840
claimant = "Rowmenu"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Operator requirement (2026-09-03, paraphrased): conversation management reachable from the conversation itself — a right-click menu on the desktop, matching a long-press menu on mobile. The unifying fact: egui synthesizes a secondary click from a touch long-press, so both clients implement THE SAME design — the conversation row's context menu — and the platform-native trigger falls out free. The android twin rides its own ball (yog-android store, the conversation-acts ball carries the ruling).

Scope: the acts this seat already speaks for a conversation, surfaced on its ROW's context menu (egui response.context_menu on the conversations-list row): stop, interrupt, retarget, delete (with the arming — decide how the name-arming works inside a menu; the DESIGN 4.20 idiom may mean delete in the menu OPENS the arming rather than firing), records…, flag (needs its reason box — same question), plus aim-independent conveniences if cheap (mark seen when the row carries attention). The menu is a second gesture path to controls that mostly exist in the composer/panes — do not duplicate the act plumbing, reuse the same verb builders; parity: menu items that compose gestures carry the same act: tags (the walk takes a union, two controls per op is lawful).

Constraints: works in both layout shapes (narrow: the row lives on the conversations column); the keyboard story stays whole (F1 everything-keyboard-operable — the composer/pane paths remain the keyboard route, state that in DESIGN; if egui offers a keyboard menu affordance cheaply, take it); snapshots regenerated with a world/walk that opens the menu if the harness can (kittest can send secondary clicks — assert at least that the menu opens and its items carry their tags); DESIGN gains the menu idiom as its one home.

Verify premises against the tree — the seat moved a lot in the last day (DESIGN 4.11-4.22).