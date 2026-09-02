+++
title = "kittest snapshot harness: render the real seat headless, PNG matrix, reachability assertions"
created = 1788329700
updated = 1788329725
claimant = "Snapseat"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Close the ergonomic feedback loop for the gross-defect class: agents must be able to SEE the seat without a compositor. Wayland blocks live capture; egui_kittest does not touch the compositor — it runs the real UI code off-screen and rasterizes PNGs (wgpu-backed snapshot feature), plus drives interaction through the AccessKit tree.

Dependency ruling (operator, 2026-09-01): egui_kittest + wgpu are APPROVED as dev-dependencies. Dev-only — they must not enter the published binary; put them under [dev-dependencies] and confirm 'cargo package --list' / the release artifact are unchanged.

Step 0, prove the precondition: since the remote split the window is a wire client on loopback, so the harness should be able to construct the seat against an arbitrary loopback endpoint. Prove it. If construction is entangled with anything beyond the wire address, THAT is in scope — untangle it; the operator ruled the loopback path should work.

Then:
1. Harness: kittest drives the real seat update loop against a fixture endpoint. The yog store has a sibling ball (bl-8741 there) delivering named deterministic world states; until it lands, a canned local endpoint or recorded wire session is an acceptable stand-in — do not block on it.
2. Snapshot matrix: PNGs across viewport sizes (include phone-shaped, e.g. 400x800) and both themes, one per named world state, written somewhere an agent can Read.
3. First standing assertions, gate-checked: (a) settings panel reachable from the main screen in a bounded number of gestures at every matrix size — an AccessKit query, not a pixel test; (b) a blank-region detector — a large rect of near-zero pixel variance inside the layout fails; (c) no interactive AccessKit node clipped fully off-screen.
4. Do NOT pin golden images as gates — pixel-diff gates rot with every font/theme tweak. PNGs are for eyes; invariants are for the gate.

Verify every premise against the tree before building — this body was written from design docs and memory, not a fresh read of this repo.