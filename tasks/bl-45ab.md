+++
title = "the layout: the rule and the dock stand under the box, so the box stands directly under the transcript; the dropups anchor on the rule where it now stands"
created = 1792034029
updated = 1792034450
claimant = "Stacking"
parent = "bl-4792"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-b7cb"
on = "claim"
+++
DESIGN §4.45 item 3, amending §4.42 items 3, 9 and 11.

**The order.** egui stacks bottom panels upward in the order they are laid, so `shell.rs` lays the rule first (the window's bottom edge), the dock on it while anything runs, the box on those, and the ask on the box while the selected conversation asks; the transcript fills the rest. The ask stays above the box because it is the model's question and the box under it is where the answer is typed.

**The dropup.** A selector's list opens upward from its segment, which is now under the box, so it is laid on a foreground layer whose room is the column run down to the rule's top edge (the rule panel's rect, read off the panel's response) — the idiom the bar's foot already uses for the workspace's list. It covers the box only while it stands. The slash menu stays a child region at the central panel's foot, which is now the box's top: directly above the box it completes. The long notice is the rule panel's, so it stands above the rule and under the box with no change.

**The Tab order follows the laying order**, so it is the stacking order read upward: bar, rule, dock, box, ask, column. A Shift+Tab off the box lands on the rule's name and walks the selectors right to left; a Tab off the box goes on into the column, or round to the bar where nothing is laid after it (egui gives the wrap on the frame after). No binding is added to put the rule first: a hand-written hop from the box to the provider would cycle box → rule → box and leave the ask and the column unreachable. §4.42 item 11's keyboard sentence and §4.45 item 3 say so.

**Proof.** `shell::tests` renders the seated, docked and asked windows at the matrix's three sizes and reads the glass: the rule is the lowest line of the column, the box above it, the transcript above the box, the dock alone between box and rule, the ask alone between transcript and box. `status::tests::dropup` reads the model list's rows above the rule and over the box at the desk and the phone; `notice` reads the long notice under the box; `keys::tests::focus` walks back from the box onto priority and opens it.