+++
title = "gate: alignment"
created = 1785124190
updated = 1785125418
claimant = "Gimbal"
parent = "bl-8fb1"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-8fb1"
on = "claim"
+++
PASS, checked against `docs/ARCHITECTURE.md`, `docs/PRINCIPLES.md`, `docs/TAXONOMY.md`.

**ARCHITECTURE §4.4 (adapter resolution + version guard).** Untouched. Runtime resolution stays "the `models.yaml` `adapter:` override, else the binding-injected target, else `bz` on `PATH`", and the load-time guard still demands the exact linked version. The change moves only where the INSTALL TEST lets `cargo install brazen` write; §4.4's claim that "`make install` installs the pinned binary (`cargo install brazen --version =<pin>`)" remains literally true — the user-facing recipe has no new flag and no new branch.

**ARCHITECTURE §2.1 banned terms.** No banned usage introduced: nothing added says bare "call", "turn", "session", "compression", "invocation" as a structural unit, "subagent", "exchange", "thread", or "agent" in the config sense. New wording is confined to existing vocabulary — install root, worktree, pin, adapter, gate.

**PRINCIPLES.**
- *Single source of truth*: the pin still has one home (the `brazen = "="` line in `Cargo.toml`, via `BRAZEN_PIN`); nothing about the version was copied. The redirect adds no second representation of any fact — it names a directory, not a version.
- *Add scrutiny, subtract mechanism / new flags are a smell*: no new make variable, no new target, no test-only branch in the recipe. The seam is cargo's own documented `CARGO_INSTALL_ROOT`, an existing explicit signal, set on the one `Command` the test already builds.
- *Severability*: deleting the single `.env("CARGO_INSTALL_ROOT", ...)` line restores the old behavior exactly; no core code has to be edited back.
- *If it can't be tested, it mustn't be built*: the isolation was demonstrated directly (shim-forced mismatch; the global binary's sha256 and mtime unchanged while the per-worktree root received brazen 0.0.4).
- *Minimize interface*: the isolation lives on the test, not on a make target, so it holds for every runner — `cargo test --test install`, bare `cargo test --workspace`, `make test`, `make test-install`. A Makefile-only fix would have left the exact path that produced the reported breakage (a plain `cargo test` run) still writing the machine-global binary.

**TAXONOMY.** No new terms of art coined; nothing in the taxonomy needed amending.

One deliberate asymmetry, recorded rather than hidden: `install-bz`'s idempotency guard reads `bz --version` from `PATH` while the write now goes to the per-worktree root, so under the test the guard can mismatch and re-invoke `cargo install` even though that root is already populated. Cargo's "already installed" short-circuit makes that cheap, and changing the guard to interrogate the install root would change USER-facing `make install` semantics — out of scope here and not obviously desirable.