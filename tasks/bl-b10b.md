+++
title = "cargo deny fails every gate on RUSTSEC-2026-0349: rustls 0.23.45 in the lockfile predicts the server's key-exchange group from an unauthenticated ServerHello; the fix is 0.23.46 and the move is a lockfile update of an approved dependency"
created = 1792034084
updated = 1792034084
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The builder's make check fails at the deny step for every tree until the lockfile carries rustls ≥ 0.23.46. No manifest change: rustls is the approved TLS dependency and this is its patch line moving. cargo update -p rustls, commit the lockfile.