+++
title = "lernie config: declined/failed authoring pass leaks the transient checkout and wedges the verb"
created = 1784955704
updated = 1784959264
claimant = "Pintle"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 3. An authoring pass that changes nothing was DOCUMENTED as declined but the decline path (git commit exit 1) skipped teardown: .config-author remained and every later `lernie config` failed with "worktree add ... already exists"; a failed --from also left a dangling refs/heads/config/<name>.

== STATE 2026-07-24 (Pintle) — IMPLEMENTED, GATE GREEN, COMMIT IN FLIGHT, CLOSE HELD ==

Worktree: /home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/lernie/bl-54aa (claimed by Pintle). All work is in that worktree.

--- What was built ---
1. NEW src/template/checkout.rs — teardown made structural:
   * `Checkout<G>` drop guard. Construction = the `git worktree add`; `Drop` = `worktree remove --force`; the ref the add created (`-b`, i.e. fork/orphan) is deleted with it unless `landed()` is called. So the decline, a git failure, an editor failure, a descriptions-refresh failure and any early `?` all leave the workspace as found — no .config-author, no dangling config/<name>. `landed()` reports teardown failure (success path); `Drop` swallows it (failure paths).
   * `heal(git, repo, path)` — crash-debris decision: HEAL, never refuse. Every pass starts with `worktree prune` + `worktree remove --force <path>` + `remove_dir_all` (NotFound = Ok). Unconditional, so there is no leftover special case. Rationale in the doc comment: ARCH 2.11 "the next touch heals"; the transient checkout holds no authored history, so the cost is only a killed pass unsaved edit. A path surviving all three steps surfaces as Error::Io (no silent wedge).
   * `path(workspace)` + AUTHOR_DIR — one definition of .config-author (was duplicated in template/mod.rs and authoring/mod.rs).
2. src/template/mod.rs — `commit_checkout(git, author, msg) -> io::Result<bool>`: `add -A`, then `status --porcelain`; empty stage returns Ok(false) = declined. Decides from the index rather than parsing gits refusal text. No longer removes the worktree (the guard does). `scaffold` now uses the same guard (one teardown for both authoring acts) and ignores the bool (embedded template always writes files).
3. src/template/authoring/mod.rs — `pub enum Pass { Landed, Declined { target: String } }`; `author`/`from_cli` return `Result<Pass, Error>`; `materialize` returns the guard and passes `created = Some(config_ref)` for Fork/Orphan, None for Advance (a failed advance must never delete a pre-existing branch).
4. DECLINE EXIT CONTRACT (chosen, documented): a declined pass is a SUCCESS, exit 0, and `lernie config` prints ONE stdout line via the existing `Outcome::Line` (no new mechanism):
     config/default unchanged: the edit changed nothing, so no config commit was authored
   so empty stdout means a commit landed (machine-readable). Rationale: authoring is a user act (ARCH 2.3) and a user who saves no change has declined it; ARCH 3.4 one-product-on-stdout is honoured.
5. Docs: ARCH 2.2 gains a new paragraph "Teardown is structural, and a declined pass is a clean outcome" (defines "declined pass", the ref rule, and the heal-on-next-pass rule). README "Authoring config commits" gains "Declining is fine, and leaves nothing behind" with the literal line and the crash note.
6. Tests: NEW src/template/authoring/tests_teardown.rs (7 tests) — decline is a success with the checkout gone; decline then immediate re-author lands; declined fork leaves no ref; failed fork/orphan leave neither checkout nor ref; killed-pass debris (hand-made `worktree add` + dirty file) is healed by the next pass; unremovable path surfaces as Io. Plus cmd::tests::verbs::config_reports_a_declined_pass_as_a_clean_line (two noop-editor passes: the first lands the 3.3 descriptions refresh, the second declines). Existing stub gits now report `status` dirty; scaffold run-sequence assertions updated (add -A, status, commit, remove --force).
7. INCIDENTAL (needed for a deterministic gate): src/prompt/tests/advance.rs `free_within_gives_up_on_a_genuinely_held_lease` deadline 20ms -> 500ms. At 20ms a loaded box expired the deadline before the first retry, so the retry arm (line 116) went uncovered and tarpaulin fell to 99.98%.

--- Verification ---
`make coverage` GREEN in the worktree: "100.00% coverage, 4396/4396 lines covered", 0 test failures (log: <scratchpad>/covrun.log). fmt-check + clippy -D warnings clean. All files under the 300-line cap.
IMPORTANT ENVIRONMENT NOTE: the on-PATH `bz` is 0.0.4 while this branch pins brazen =0.0.3, which fails 5 e2e wire tests with a version-guard error. Run every gate/commit/close with PATH prefixed by /tmp/claude-1000/-home-mark-dev-lernie/fe8c0ced-96b4-45ac-8547-30d9edb8e1e3/scratchpad/bz003/bin (a real bz 0.0.3). Also `prompt::tool::tests::errors::spawn_retries_past_transient_etxtbsy` flakes under machine load average ~25 (ETXTBSY 200ms retry budget) — retry the run, it is not a code fault.

--- Remaining steps ---
a. The commit is being retried in a detached loop (<scratchpad>/commit54aa.sh, message in <scratchpad>/msg54aa.txt, status in <scratchpad>/commitstatus.log). If it is not in `git log` in the worktree, re-run: `cd <worktree> && PATH=<bz003/bin>:$PATH git commit -F <scratchpad>/msg54aa.txt` (the pre-commit hook runs make check, ~14 min; NEVER --no-verify).
b. Delivery is SERIALIZED. Hold the close until bl-bdc7, bl-a1a1, bl-0135, bl-aabb all appear in `git log --oneline main`. Then merge main into the worktree, re-run the gate if main moved, and close from the repo root:
   cd /home/u/dev/lernie && bl close bl-54aa --as Pintle -m "..."  (retry once on a store race; bare re-run on an unseen-diff refusal)
c. Then verify and close the three gate children of bl-54aa (tests / docs / alignment).