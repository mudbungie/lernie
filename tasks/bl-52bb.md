+++
title = "every clickable element on the glass lights under the pointer: the audit of every click seat outside chat.rs, and the ones that did not"
created = 1792034032
updated = 1792034032
parent = "bl-4792"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-b7cb"
on = "claim"
+++
DESIGN §4.45 item 6. Every seat that senses a click wears the hover wash (theme::hover) or egui's own button hover: audit every Sense::click() and every Label-with-sense, ui.button, selectable text and custom-painted control under src/ui (chat.rs is bl-4792's bubbles ball's territory) — the dock's line and rows, the panels' ▸ folds, the ask's cells, the overlay name lines, the queue, the records, the config and tuning panes, the enrollment — and route the bare ones through theme::paint's door or the row idiom. A test on the offscreen context hovers each tagged control of each snapshot world and asserts a fill appeared under the pointer, so a later control cannot arrive bare.