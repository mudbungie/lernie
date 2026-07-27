+++
title = "ETXTBSY retry envelope is wall-clock; README's determinism rule says attempt count — conform the code"
created = 1785124474
updated = 1785124474
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Raised by Ratchet while closing bl-9300/bl-2061 (recorded on bl-ec65's body). README's coverage-determinism section states: 'A retry budget is a count of attempts, never a wall-clock deadline.' The production envelope in src/prompt/tool/subprocess.rs is still a wall-clock deadline (Instant::now() < deadline); its retry arm's coverage is safe only by a probability argument. DECIDED (Tumult): the README rule is the invariant — convert the envelope to an attempt count so the structure, not probability, guarantees the retry arm is exercised. Keep observable behavior equivalent (attempts x interval ~ the current 200ms envelope); update the etxtbsy tests (src/prompt/tool/tests/etxtbsy.rs) and the with_etxtbsy_budget seam from bl-7a3f coherently — the injected budget becomes an attempt count too. No README change needed; the code conforms to it.