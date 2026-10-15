+++
title = "every clickable element on the glass lights under the pointer: the audit of every click seat outside chat.rs, and the ones that did not"
created = 1792034032
updated = 1792034494
claimant = "Lighting"
parent = "bl-4792"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-b7cb"
on = "claim"
+++
DESIGN §4.45 item 6: every seat that senses a click wears the hover wash (`theme::hover`) or egui's own button hover. Two halves.

**The audit**, by hand, of every click seat under `src/ui` outside `chat.rs`/`chat/` (the bubbles ball's). Every `ui.button`, `Button::new`, `selectable_label`, `SelectableLabel` and checkbox lights through egui's own ladder — `theme/visuals.rs` installs `widgets.inactive` on `SURFACE` and `widgets.hovered` on `RAISED`, which differ, and a selectable label paints `RAISED` only under the pointer. Every row — the roster's, the switcher's, the bar's, the dock's, the ask's, the slash menu's, the dropup's, the enrollment's grades — is `theme::paint::row`/`Row`; every link and tab is `theme::paint::door`; both wear the wash. The one bare seat was the dock's folded line: an `egui::Label` with `Sense::click()`, which lit nothing. It is now `theme::paint::link` at the same size in the same accent. The composer's field is a field, not a seat; `find`'s `allocate_exact_size` is `Sense::hover()` for layout and not a seat.

**The instrument**, `src/snapshot/lit.rs`, assertion (d) of the matrix: for each world at each size, every tagged control is hovered at its centre and the fills inside its rectangle (`paint_probe::fills_of`, the one walk) compared with the frame at rest — lit means changed. What it does not judge is a fact of the control, not a list: a control egui does not report hovered (disabled, scrolled past its clip, under an overlay), a selected one (`RAISED` outranks the wash; egui writes selection as *toggled*, so a checked box — toggled but a value — is still judged), and a field that takes text. The way from a tree node back to egui's seat is a test-only bridge in `ui::act::tag` (`seat_of`): AccessKit's node id is the egui id's value, and egui answers nothing about a bare number. The matrix holds the count of judged controls to a floor, and the detector's failing arm is proved on a toy window painting one tagged bare label.

**What it found that the audit did not**: the bar's selected conversation row and its aimed wall row read as not lit — because they are *chosen*, which is the exclusion above; the first run read selection off `is_selected`, which egui never writes, and the fix was to read `toggled`.

**What the tree cannot see**: a `Label` with `Sense::click()` has the same role and the same click action as selectable prose, so the instrument judges tagged controls and the Label shape is held by the audit. An ast-grep rule banning `Label…sense(click)` would close it mechanically, and it waits on `chat.rs`'s fold head and `chat/panel.rs`'s `▸` folds (the bubbles ball's), which carry that shape today.