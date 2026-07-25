+++
title = "child terminal deposit does not launch the parent's driver — parent revival needs a manual scan"
created = 1784955704
updated = 1784955725
claimant = "Rushlight"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 6. ARCH promises 'a running child … revives [the stopped parent] by depositing its result (§2.11)' and 'normal operation needs zero scanning'. Shipped: src/prompt/dispatch/result_deposit.rs::deposit_terminal writes into the parent's inbox but never rides the probe-and-launch seam lernie message uses; verified live — parent sat at its old tip until lernie scan launched a driver. Fix: the terminal result deposit into the PARENT's inbox must probe the parent's executor lock and, on a free lease, detach-spawn 'lernie advance <ws> <parent>' — the exact seam lernie message already has (single source: reuse it, don't duplicate). Mind the §2.11 exit-protocol pins (epitaph-value launch decision: final-response launches; stopped/budget-exhausted never do) — read ARCH §2.11 and reconcile: the launch decision likely applies here too. Tests: child terminal deposit → parent delivered + stepped with no scan; stopped/budget epitaphs do not launch.