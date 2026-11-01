+++
title = "the CLI listing, the records header and the watch's held line each name the tool ahead of the server's hold sentence, which opens with it"
created = 1793512458
updated = 1793512458
parent = "bl-4446"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["ui"]
+++
Siblings of bl-9f2b, which deduped the ask's 'waiting for your approval' line. Three more renderers write the tool (and the tool_use id) ahead of the hold's reason sentence, which already opens with the tool and its input: render::workspaces::parked ('held: {tool} ({tool_use}) — {reason}'), ui::records::header::parked ('held at {tool} ({tool_use}) — {reason}'), seat::follow::rest::held_at ('{agent} is holding {tool} ({tool_use}) for your answer after {age} — {reason}'). Expected: the sentence carries the line (bl-5507's rule, DESIGN 'Why a row asks is the server's sentence'), the tool_use id kept where it is spent, the bare tool only where no sentence came. Rough edge, not a wrong answer.