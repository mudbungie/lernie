+++
title = "the seat CD can only be seated over ssh: a box with no sshd cannot arm its own timer"
created = 1788673590
updated = 1788673831
claimant = "Cantaloups-H"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
tags = ["usability-r1"]
+++
`scripts/deploy/seat.sh` takes an ssh destination and only an ssh destination —
`scp` the three files, `ssh` to arm the timer, `ssh` to run the first reconcile.
`make deploy-seat HOST=<ssh-host>` is the one door.

But the seat is a WINDOW, and the box most likely to run a window is the
workstation somebody is sitting at — which is exactly the box least likely to
be running an sshd. Observed on a live seat box: no sshd, so the seat timer
could not be armed at all, and the installed lernie was whatever the last hand
install left. That box cannot ssh to itself and there is no other door.

Add the local carrier. `scripts/deploy/verify.sh` in yog states the idiom this
repo should reuse: one payload, and the only thing `--local` changes is whether
it goes through `ssh` or through `sh` right here. So:

  * `seat.sh --local` (or a bare `seat.sh` with no host) copies the three files
    into `~/.local/bin` and `~/.config/systemd/user`, arms the timer and runs
    the first reconcile synchronously, with no ssh hop and no sshd.
  * `make deploy-seat` with no `HOST` seats THIS box; `HOST=<ssh-host>` keeps
    seating that one. One target, two carriers, no new verb.

The install and the arming are byte-identical either way — only the carrier
differs — so a local seating is not a second recipe that can drift from the
remote one.