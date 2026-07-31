+++
title = "gate: tests"
created = 1785459718
updated = 1785459811
claimant = "pedantic-lernie"
parent = "bl-e4ef"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
GREEN. `make test` under the pin-keyed bz 0.0.5: all suites pass, 0 failures (995 lib + integration). `make check` at commit 3456828: fmt-check clean, clippy -D warnings clean, tarpaulin 100.00% coverage 5259/5259, tests/install.rs install contract ok.

brazen 0.0.4→0.0.5 API impact on lernie: none. The only model-facing change is `Message`'s hand-rolled Deserialize (brazen src/canonical/request_de.rs) normalizing a `user` turn bearing any `tool_result` to `Role::Tool`. lernie already emits `Role::Tool` for tool results (src/prompt/dispatch/assembler.rs: `Side::Tool => Role::Tool`), so the normalization is a no-op on every request lernie sends. `brazen::Event` and `brazen::Tool`, lernie's only other imports, are unchanged.