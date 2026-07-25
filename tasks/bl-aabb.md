+++
title = "bundle/replay: replayed workspace cannot be driven — no config/* ref survives"
created = 1784955704
updated = 1784959181
claimant = "Osprey"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 2, release-critical. `agents.bundle` carried only `refs/heads/agents/*` — no `config/*` refs — while governing-config derivation is merge-base against the `config/*` HEADS, so a replayed workspace failed with `fatal: Needed a single revision` (prompt) / `no config/* ancestor` (advance). Fix per ARCH §9.2/§2.2.

## Implemented (Osprey, 2026-07-24) — work committed on `work/bl-aabb`, NOT yet delivered

Worktree: `/home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/lernie/bl-aabb`.
Work commit: "bundle: carry the governing config lineage so a replay is drivable [bl-aabb]" plus merges of main.

**Design — the governing lineage rides the bundle as refs, at their heads.**
- `workspace::config_lineage(ws, agent_id, git) -> Vec<(config-ref, merge-base)>`: every `config/*` ref whose history reaches the branch. `governing_config` is now a fold over it (one place knows which lineages govern an agent).
- `archive::bundle` appends those ref names to `git bundle create`. Heads, not merge-bases: a config branch that advanced past the fork is no longer an ancestor of the agent, yet it is the ref the merge-base is taken *against*, so heads keep the replay's candidate set identical to the source's. An orphan config lineage (no shared ancestry) contributes nothing and does not ride. No sidecar — the refs are the single source.
- `archive::bundle_heads` derives the primary id over `refs/heads/agents/` only (a `config/*` head is not an agent).
- `src/archive/slices.rs` (new): slice-copy plumbing split out to keep `archive/mod.rs` under the 300-line cap.

**Delivery ordering: deliberately unchanged (deliver-before-resolve).** A delivered message is a committed transcript entry and warrant re-derives from the transcript tail, so the tail reads user-side and the next hop finds a model call due — landed-and-pending, never consumed-and-lost. Resolving first would make every no-op hop pay a config read (killing the §2.11 pin-1 terminator's "costs nothing but the probe" property) and would add a second guard for a fact the bundle now fixes at its source. Rationale is written into ARCH §9.2.

**Tests.** New `src/e2e/replay_drive.rs` (httpmock + real `bz`): `a_replayed_workspace_takes_a_fresh_prompt` (new→prompt→bundle→replay→prompt in the scratch drives to a landed assistant entry) and `a_replayed_agent_advances_on_its_governing_config` (replayed `governing_config` == source's; `lernie message` launches a driver that delivers `003-user.md` and steps `004-*`). Both fail with exactly the reported errors when the lineage is suppressed. Unit cases added for: lineage refs in the bundle arg list, orphan-lineage exclusion, lineage-enumeration failure arm, primary derivation over agent refs only, and `config_lineage_names_the_ref_the_merge_base_is_taken_against` (config head advanced past the fork).

**Docs.** ARCH §2.2 defines the term *governing lineage*; ARCH §9.2 Archive/Replay bullets + shipped-state note rewritten (fix, regression test, delivery-ordering decision); README §9 bundle/replay section rewritten.

## Verification status

Full close gate ran GREEN on 2026-07-24 21:46: fmt, clippy -D warnings, and **100.00% coverage, 4462/4462 lines**, with a pinned `bz` 0.0.3 first on PATH (`/tmp/claude-1000/-home-mark-dev-lernie/fe8c0ced-96b4-45ac-8547-30d9edb8e1e3/scratchpad/bz003/bin`, also `bzroot/bin`) — needed because the machine's PATH `bz` is 0.0.4 while main pins `=0.0.3` (that bump is bl-8c92/Umber's ball and must NOT ride this close).

The close then ABORTED in the bl-delivery plugin, not the gate:
`no-resurrection invariant: the squash of work/bl-aabb carries path(s) never authored on it since its fork: src/prompt/dispatch/assembler.rs, src/prompt/dispatch/assembler/body.rs, src/prompt/dispatch/assembler/body/tests.rs` — main advanced (bl-ae66, 803aa00) mid-close, so the branch's older copies of those files read as a resurrection. Cure: merge latest main, then close. Delivery is serialized by the coordinator; this ball is FOURTH — hold until `[bl-bdc7]`, `[bl-a1a1]`, and `[bl-0135]` all appear in `git log --oneline main`.

## Remaining steps to close

1. `cd <worktree> && git merge main --no-edit` (reconcile; the branch must carry main's newest assembler/* verbatim).
2. Ensure no local `brazen = "=0.0.4"` pin experiment is in the tree — the pin stays `=0.0.3` (`git status --short` must show nothing).
3. `PATH=<bz003>/bin:$PATH bl close bl-aabb --as Osprey -m "bundle: carry the governing config lineage so a replayed workspace is drivable"` from `/home/u/dev/lernie` (gate takes ~15 min; run detached and poll).
4. Sweep the gate children: bl-ad41 (tests), bl-83f5 (docs), bl-0a72 (alignment).