+++
title = "Run tests/install.rs in the close gate (outside tarpaulin)"
created = 1784955529
updated = 1784959182
claimant = "Lodestar"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Harrow audit finding 7. tests/install.rs is cfg_attr(tarpaulin, ignore) and `make check` was `fmt-check lint coverage` — tarpaulin only — so the install contract (first thing every user touches; carries the include_dir! embedded-asset seam) never ran in the pre-commit/close gate.

## Implemented (Lodestar, 2026-07-24/25)

New `make check` composition: **fmt-check + lint + coverage + test-install**.

Makefile gains a `test-install` target (`cargo test --test install`), added to `.PHONY` and appended to `check` AFTER coverage (cheap failures first). Its comment block explains why it is separate (the test shells out to `make install` — a release build plus `cargo install brazen` — which contends with tarpaulins target/ lock) and states that a future tarpaulin-ignored sibling belongs on that same line, not in a new target.

**Other tarpaulin-ignored tests: NONE.** `grep -rn "cfg_attr(tarpaulin" tests/ src/ crates/` returns only tests/install.rs (line 49, plus a line-12 comment). Nothing to fold in.

**Standalone runtime:** `cargo test --test install` = 1m03s wall cold (debug build included); the test body itself 45s. Inside a warm `make check` the step takes ~45-60s.

**Global side effect, accepted:** the test runs `make install`, which does `cargo install brazen --version =<Cargo.toml pin>` onto the users cargo bin. Observed: it rolled bz 0.0.4 back to 0.0.3. Correct end state for this tree (the §4.4 load-time guard rejects a non-pin bz) but it mutates the machine outside the repo on every gate run, and parallel agents fighting over global bz is a live hazard.

Docs updated to the four-part composition: README CI line (§ top), README build-target table (`make check` row + a new `make test-install` row), README "Pre-commit hook" item 3, and the `.githooks/pre-commit` header comment + its echo line.

## State

Worktree /home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/lernie/bl-f01f, branch work/bl-f01f. Work is COMMITTED as ccc36ee "Run the install contract in the close gate, outside tarpaulin [bl-f01f]" (3 files: Makefile, README.md, .githooks/pre-commit; +41/-14), plus later merges of main. No Rust changed.

**Verified green:** the commit passed the full pre-commit gate — fmt-check, clippy, tarpaulin 100.00% (4355/4355), and then `cargo test --test install` -> `test make_install_lays_down_skeleton_idempotently ... ok`. The same log shows that test as `ignored` under tarpaulin, which is exactly the gap this ball closes.

## Remaining steps

1. `bl close bl-f01f --as Lodestar -m "..."` FROM /home/u/dev/lernie (running it inside the worktree fails "no balls checkout here").
2. Sweep the three gate children, parent bl-f01f: **bl-9efb (gate: tests), bl-d4d3 (gate: docs), bl-c3af (gate: alignment)** — verify then `bl close <id> --as Lodestar -m "..."`.

## Gate recipe — REQUIRED, or the gate fails

The e2e suite requires the `bz` on PATH to equal the Cargo.toml brazen pin (=0.0.3). Parallel agents keep reinstalling bz 0.0.4 globally, which makes coverage fail with: `bz version "0.0.4" does not match the linked brazen crate "0.0.3"`. A private pinned bz is installed at

    /tmp/claude-1000/-home-mark-dev-lernie/fe8c0ced-96b4-45ac-8547-30d9edb8e1e3/scratchpad/bz03/bin

(note: bz03, not bz003). Prefix it on PATH for every commit and for `bl close`:

    env PATH="/tmp/.../scratchpad/bz03/bin:$PATH" bl close bl-f01f --as Lodestar -m "..."

If that scratchpad is gone: `cargo install brazen --version =0.0.3 --locked --root <somewhere>`.

## Environment notes (not defects in this change)

- The box is oversubscribed by parallel agents (load avg 20-35, up to 66 concurrent tarpaulin processes). systemd-oomd SIGTERMs the coverage run — it surfaces as `make: *** [Makefile:71: coverage] Terminated`. Pure retry.
- Two load-sensitive coverage/timing flakes cost ~8 attempts: `src/prompt/tests/advance.rs:116` (a 5ms sleep in a retry loop that only executes if the deadline has not already elapsed) and `prompt::tool::tests::errors::spawn_retries_past_transient_etxtbsy`. Main commit 6a8ab4d [bl-1c2e] fixed these; merge main before retrying.
- A full `make check` under this load takes 10-13 minutes.