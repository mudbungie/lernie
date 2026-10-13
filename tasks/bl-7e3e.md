+++
title = "a long notice in the rule's state slot is cut at the window edge mid-word, and it pushes every other segment off the rule — the waiting count, the engines and the conversation's name all vanish while it stands"
created = 1791867659
updated = 1791867659
parent = "bl-957a"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Gesture: any act whose notice is longer than the rule — e.g. 'seen' on a busy fixture (bl-6802's unreadable-answer sentence), at 1200 px and narrower.
Expected (DESIGN 4.42 item 2: 'the notice stands in the state slot'; segments drop from the right, 'the name last'; bl-3d0f ruled a notice must never be cut, because the remedy is the half that is lost): the whole sentence is readable and the rule still says where you are.
Observed: status::fitting reserves the state run's full width before any fact, so a notice wider than the rule leaves room for nothing: the facts and the name are dropped, and the notice itself runs past the frame, clipped mid-word ('...nothing here renders tha'). The remedy clause is exactly what is lost.
Why it matters: every refusal and every unreadable answer is a notice; the operator reads the start of a sentence whose end is the fix, and loses the rule's other facts until Esc.