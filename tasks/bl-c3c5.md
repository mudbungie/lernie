+++
title = "design doc: cryptographic agent attestation — per-agent keys, executor-mediated signing, inference-log witness (DESIGN_AGENT_ATTESTATION.md)"
created = 1785906145
updated = 1785906146
claimant = "Progresses"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Design-only ball: log the attestation design under docs/. Not scheduled for implementation; the deliverable is the spec itself.

Content settled in conversation with the user 2026-08-04:
- per-agent keypair minted at the fork, announced in the assembled context so every retained model call witnesses the binding
- signing is executor-mediated (the agent's transcript channel IS its identity); the key never enters the agent's environment
- verifier side (log extraction service, pubkey index, verification check) is explicitly NOT lernie — different trust domain
- corporate chain: lernie key optionally certified by an org intermediate; deployment attribution comes from gateway auth
- honest trust claims: machine-level custody boundary; tamper-evidence and distinct-context proof, not proof of cognition