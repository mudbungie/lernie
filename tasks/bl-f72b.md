+++
title = "Open the dispatch role set: lernie dispatch <role> accepts any config-defined role"
created = 1784337602
updated = 1784337602
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["design-settled"]

[[blockers]]
id = "bl-e6d4"
on = "close"

[[blockers]]
id = "bl-92ec"
on = "close"

[[blockers]]
id = "bl-4b21"
on = "close"
+++
## Settled design (from bl-3a85, the role design ball — do not re-litigate)
A role is a pure config-selection key (docs/TAXONOMY.md §1 'Role', docs/ARCHITECTURE.md §4.3): the role set is OPEN — a role is valid iff the governing config commit lists roles.<name> in providers.yaml AND carries souls/<name>.md — and any valid role is dispatchable by any dispatcher, with no constraint relative to the parent's role.

## The shipped contradiction
src/prompt/dispatch_cli.rs::run_with hard-codes a closed role match: ROLE_WORKER (goal required) / ROLE_COMPACTOR (goal forbidden, boilerplate goal) / other => DispatchCliError::UnknownRole. Consequences:
- `lernie dispatch verifier ... --goal ...` is refused, though verifier is a legitimate config role (§6 gate).
- The model-facing dispatch tool (src/prompt/tool/builtin/dispatch/) validates the role OPENLY against the governing config (validate.rs: roles.<name> + souls/<name>.md) and then re-enters `lernie dispatch <role>` — which refuses anything beyond the two names. So a model dispatching any third role passes validation and then fails at the CLI: two paths, two answers.
- The workflow interpreter sidesteps the CLI entirely (child_result/verifier.rs calls child_dispatch::run directly), which is why the v0.7 verifier gate works — but §3.4 says the CLI is the sole front door, so the CLI must agree with the open validation.

## What to implement
1. dispatch_cli validates role validity (roles.<name> present + souls/<name>.md present in the governing config commit) BEFORE forking — no branch debris on an invalid role. Reuse/hoist the tool's validate_role so the check has ONE home (single source of truth); the tool then either delegates or keeps the pre-flight, but not two divergent copies.
2. Goal policy generalizes: --goal is required for every role EXCEPT compactor, whose goal is procedure-generated (the compaction procedure owns it, §2.7) and for which --goal stays rejected. No role enumeration remains in the CLI.
3. UnknownRole becomes the config-validation failure (role not in providers.yaml / soul missing), naming the config commit consulted.
4. Update §2.5/§3.4 shipped-state notes and the §4.3 divergence note (ARCHITECTURE.md) that records this defect once fixed.