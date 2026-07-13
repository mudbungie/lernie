+++
title = "lernie stop: discover executor pid via lock fd, not response.json"
created = 1783914391
updated = 1783915927
claimant = "Companions-aafc"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["stop", "spec"]
+++
## Problem
ARCH §2.9 pins stop's pid discovery to scanning /proc/*/fd for the writer holding the latest step's response.json open. That fd is open only while a model call is in flight (§4.4). During tool execution, inbox drains, and between-step gaps there is no open response.json — so `lernie stop` finds no pid and cannot signal, exactly when a long-running tool makes stopping most wanted. (Backoff sleeps ARE covered: the fd spans a step's attempts.)

## Fix
Discover the executor by who holds the agent's inbox-directory flock fd open (src/prompt/inbox/lock.rs) — the whole-loop liveness signal by construction (§2.11: held across tool execution, drains, backoff alike). Same /proc scan, different target path. response.json fd-scan may remain as the which-model-call-pid refinement; the lock fd is the authoritative is-anyone-driving discovery.

## Deliverables
- ARCHITECTURE.md §2.9 amended (discovery mechanism wording).
- Stop implementation walks /proc for the inbox-dir fd holder.
- Test: stop lands during a (mocked-slow) tool execution window.