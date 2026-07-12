+++
title = "Inbox substrate: executor flock, deposit, lernie message verb, message tool [substrate]"
created = 1783831462
updated = 1783831462
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["substrate"]
+++
Implement ARCHITECTURE §2.11 (spec refined via bl-4298): (1) the executor lock — flock(2) on the agent's inbox directory fd, acquired non-blocking at executor start, held for the whole step loop, inherited across the §6 exec baton; kernel state, observed never stored, no stale cleanup. (2) Deposit — create-only new file at inbox/<agent-id>/<sender>-<NNN>.md, temp-path + atomic rename, from:/deposited_at: frontmatter; path carries framing, frontmatter carries asserted facts, body is content. (3) lernie message <workspace> <agent> <content> CLI verb and the message built-in tool ({agent, content} -> {status: deposited}; sender from LERNIE_CONV_BRANCH, never model-supplied). (4) Deposit-starts-a-driver: post-deposit non-blocking lock probe; finding the recipient quiescent, launch a driver (lernie advance) and exit — launching is not driving; writer/driver totality per §2.11.