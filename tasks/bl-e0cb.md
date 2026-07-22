+++
title = "Manifest has no runtime caller because §5.2 context assembly is unimplemented: the assembler ignores manifest.yaml"
created = 1784525323
updated = 1784698014
claimant = "Prostheses-e0cb"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-9e2d. `Manifest`/`RoleRules`/`OverflowPolicy` and `Manifest::load` parse and validate `manifest.yaml` (ARCH §5.2 context-assembly rules: per-role `pinned`, `order`, `budget_tokens`, `overflow`), and the template ships a populated manifest into every config commit — but nothing reads it at runtime. `src/prompt/dispatch/assembler.rs` assembles context from `messages/` alone: no pinned head, no manifest `order` over `summary/**`/`skills/**`, no token budget, no overflow policy. So the type is held by `#![allow(dead_code)]` in src/config/manifest.rs.

Not a wiring job — implementing §5.2 assembly is the work. When it lands, `Manifest` needs a `parse(raw, origin)` seam like Workflow/PerRepoProviders/Version (§2.2: control is read from the config commit's tree, never a worktree file); `load(path)` was kept only because the template scaffold test reads it from a temp file.

bl-9e2d already subtracted the redundant file-loaders (ModelsConfig::load, Workflow::load, PerRepoProviders::load) and wired Version (§10 schema guard) and check_workflow_against_roles (§4.3) into src/prompt/resolve.rs.