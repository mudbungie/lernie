+++
title = "Config machinery has no runtime caller: wire manifest/version/loaders/schema-gen into verbs or subtract them"
created = 1784524950
updated = 1784524950
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-231c phase 3: narrowing the library to the command surface removed clippy's avoid-breaking-exported-api exemption and unmasked that Manifest, Version, config::schemas, Workflow::load/typed_events, ModelsConfig::load and several file-loaders are tested but reachable from no verb entry — real machinery awaiting wiring (context assembly §5.2, model resolution §4.2), currently held by commented #[allow(dead_code)] markers referencing bl-231c. Either wire each into its consuming verb path or subtract it (PRINCIPLES: build less). Grep for 'allow(dead_code)' markers citing bl-231c to enumerate.