+++
title = "Swap data plane to bz: exec per attempt, harness-owned retry, config fold, delete lernie-provider-anthropic"
created = 1783464230
updated = 1783464230
parent = "bl-00ae"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-507a"
on = "claim"
+++
Per ARCHITECTURE.md v0.4 §4.4 + §2.10. Assembler emits a typed brazen CanonicalRequest (linked crate; the fail-open extra map stays unreachable), serialized to bz stdin. Invoke bz --json --provider <row> once per attempt; append stdout verbatim to the step's response.json (attempt segments). Retry loop is the harness's: attempt cap + backoff from workflow.yaml; retryability = CanonicalError::retryable() on the in-band Error event; bz exit code is diagnostic only. Delete crates/lernie-provider-anthropic, the describe/complete machinery, auth_env/endpoint_env forwarding, the non-stream->JSONL synthesis path (brazen normalizes non-stream wire into the same events). Config: <harness-root>/providers.yaml -> models.yaml (models/capabilities/context_window only + optional adapter: override for a contract-compatible alternate binary); per-repo providers.yaml roles: {provider: <brazen row>, model, tools}. Hermetic tests: BRAZEN_CONFIG -> fixture TOML with httpmock endpoint rows (bz honors --config/BRAZEN_CONFIG > XDG). make install: cargo install brazen --version =<pin>; load-time guard bz --version == linked crate version (Decline illegal operations; skipped under adapter: override, where the in-band MessageStart.v=1 handshake governs). Drop legacy-vocabulary support from readers/fixtures (I1's transition ends here). Update README (§install steps 3-8 describe the old adapter flow), template, schemas/.