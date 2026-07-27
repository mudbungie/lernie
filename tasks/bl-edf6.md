+++
title = "ETXTBSY retry envelope is wall-clock; README's determinism rule says attempt count — conform the code"
created = 1785124474
updated = 1785125158
claimant = "Keelson"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
DELIVERED as part of bl-6987's sweep (Keelson, 2026-07-26) — one landing commit, tagged [bl-6987]; this ball closes with an empty delivery pointing there. The decided direction was implemented exactly: src/prompt/tool/subprocess.rs's envelope is now ETXTBSY_RETRY_ATTEMPTS: u32 = 100 (x 2ms ETXTBSY_RETRY_INTERVAL = the same ~200ms observable envelope on an idle machine; longer under load, which is when the fork->exec window stretches). SpawnArgs.etxtbsy_budget is a u32 attempt count; the with_etxtbsy_budget seam (src/prompt/tool/spawn.rs) takes attempts; src/prompt/tool/tests/etxtbsy.rs conformed (retry-success: 1_000_000 attempts; give-up: 3 attempts against a permanent hold, making the retry-arm's coverage structural — exactly 2 retry sleeps every run; fixture self-exec envelope: 10_000 attempts). README unchanged — the code now conforms to its rule. Verified in bl-6987's 30/30 runs under load 65-85.