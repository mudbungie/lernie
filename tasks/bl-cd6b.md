+++
title = "adapter stderr is discarded — startup failures masquerade as killed-mid-stream"
created = 1784955704
updated = 1784955704
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 9. A bz that fails to start (e.g. malformed brazen config TOML) yields a 0-byte response.json and 'adapter stream ended without a terminal end (killed mid-stream, §2.9)' — the real error (bz's stderr) is discarded. Fix: capture the adapter subprocess's stderr per attempt into the step record (e.g. steps/<id>/<NNN>/stderr.log beside response.json — a diagnostic artifact, written never read, matching §2.3's pattern) and include its tail in the surfaced error when the stream ends without a terminal end AND no stop signal is pending. Align ARCH §2.3/§4.4 step-record inventory docs. Tests: failing-adapter fixture surfaces the stderr text; clean runs write empty/no stderr artifact.