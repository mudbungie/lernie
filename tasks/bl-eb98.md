+++
title = "Unify the §3.3 tool-resolver third hop with the injected driver target (linked-host correctness)"
created = 1784525020
updated = 1784525020
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Residual from bl-231c (see the phase-3 comment on CurrentExeResolver in src/prompt/tool/spawn.rs): the tool-resolution third hop still execs '<current_exe> tool <name>'. Correct for the exec binding; wrong for a linked host driving verbs in-process — current_exe is the host binary, so the hop would exec '<host> tool <name>'. Thread Fx::driver_target into SpawnTool/CurrentExeResolver (re-signs SpawnTool::new; ~18 unit-test call sites) so ARCH §2.11's 'the driver target is injected at the binding, not resolved by name' holds for the tool seam too, and delete the residual current_exe from the library. While in there, consider making the §2.9 prelude-per-verb map a query on cmd::Command (fn preludes(&self)) so the bin's coverage-exempt match can't drift from a library-defined, tested fact.