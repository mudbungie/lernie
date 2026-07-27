+++
title = "Correct bl-9253's close record: the published 0.0.1 tarball is CLEAN — no check.exit"
created = 1785130057
updated = 1785130057
priority = 1
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
bl-9253's close message states 'Known blemish: 0.0.1 tarball carries a stray check.exit (bl-c786, fixed on main; 0.0.2 clean).' That is factually wrong, verified twice by downloading and listing the published lernie-0.0.1.crate: 266 files, NO check.exit, no *.profraw, no check.log. Why: tag v0.0.1 resolves to 38efef3 — the check.exit REMOVAL commit itself — and the release job checks out the branch (ref: github.ref_name), not the triggering run's SHA, so it packaged main's tip after the fix. There is no reopen verb; this ball IS the correction record. Close it by appending nothing further — anyone auditing 0.0.1's provenance should find this ball adjacent to bl-9253. Do not yank or respin 0.0.1 on the strength of the erroneous close message.