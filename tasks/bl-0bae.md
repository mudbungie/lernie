+++
title = "Design: sandboxed tools — WASM/WASI capability clamp (v1.1 spec)"
created = 1783464230
updated = 1783467730
claimant = "Sandcastle-0bae"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-f739"
on = "claim"
+++
From ~/dev/harness spec §6, folded as lernie milestone v1.1 (sketch lands in D1; this ball writes the full spec section). Same §3.3 stdio tool contract, sandboxed host: tool artifacts compiled to wasm32-wasip2 run under wasmtime with WASI clamped to manifest-intersect-grant (fs scopes, net hosts, exec, clock, env); a native ELF tool requires the exec grant; the flavor is derived from the artifact, never a config field; default grant set is empty; a tool asking beyond its grant fails at load, loudly, before any model call. Deliverable: ARCHITECTURE v1.1 section + capability grammar in agent/role config — small enough to audit at a glance (if a grant needs a comment, the grammar failed).