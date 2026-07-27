+++
title = "Correct bl-9253's close record: the published 0.0.1 tarball is CLEAN — no check.exit"
created = 1785130057
updated = 1785130123
priority = 1
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
CORRECTION RECORDED (Tumult): bl-9253 close message claimed the 0.0.1 tarball carries a stray check.exit — WRONG, per two independent downloads of the published lernie-0.0.1.crate: 266 files, no check.exit, no profraw, no check.log. Cause of the error: the orchestrator assumed the tag pointed at the pre-bl-c786 commit; release-plz actually tagged the newer tip that already contained the cleanup. This ball is the durable correction record (closed balls have no file to amend; absence is the record, this is the errata). Session memory corrected in the same pass. No yank consideration needed — the artifact was always clean.