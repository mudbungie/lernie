+++
title = "a row's door tells its own sentence: the row's tooltip stands down while the pointer is on the door"
created = 1792122132
updated = 1792122137
claimant = "Surveying"
parent = "bl-5369"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Fix for bl-7617 (filed with --parent, so landed here on the epic's branch). While the door is hovered the row attaches no tooltip, so the door's sentence shows; the bridge still records the row's seat every frame. A told-style test puts the pointer on the door and reads the door's tooltip open and the row's not.