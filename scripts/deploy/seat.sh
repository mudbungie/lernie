#!/bin/sh
# Seat the continuous-deployment timer on a seat box (bl-155a):
# `make deploy-seat HOST=<ssh-host>`.
#
# HOST is an ssh destination and the ONLY parameter — no address, account or
# machine name is committed anywhere in this tree. That is the leak gate's rule
# and the severability one at the same time: pointing this at a second seat is
# a different argument, not an edit, and a box that should stop tracking
# releases is one `systemctl --user disable` away from stopping, with no file
# in this repository to change.
#
# **It seats a timer; it does not deploy a build.** Nothing is compiled here
# and nothing is carried over the ssh channel but three small text files. The
# box installs from crates.io on its own schedule from then on — which is the
# difference between this and the engine's deployment, where the image is the
# unit of install and a human carries it. A seat's unit of install is a
# published version, and the registry already serves it.
#
# **It restarts nothing, because there is nothing to restart.** A seat is a
# window somebody launched. An install replaces the binary by rename, so an
# open window finishes its session on the build it started under and the next
# launch is the new one. Read `lernie-update`'s header for the protocol-skew
# consequence of that — a seat may run ahead of its engine for up to an hour,
# and the hello's refusal is the designed behavior.
#
# Idempotent, and the upgrade path: re-run it to move a box to this checkout's
# reconciler.
#
# **Its last act runs the reconciler once, synchronously, and its exit code is
# this script's** — so seating a box either ends with the newest release
# installed on it or says why, rather than reporting that a timer was enabled
# and leaving the first real answer an hour away on a machine nobody is
# watching. That first tick is also the only one that proves the box can reach
# the index and has a toolchain at all, so it is the one worth waiting for.
set -eu

host=${1:-}
[ -n "$host" ] || { echo "usage: ${0##*/} <ssh-host>" >&2; exit 2; }
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf '%s: %s\n' "${0##*/}" "$*" >&2; exit 1; }

say "seating the reconciler on $host"
ssh -n "$host" 'mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"'
# To a temp name and then `mv` into place: the reconciler may be running right
# now (the timer is armed from the previous seating), and scp truncates before
# it writes. rename(2) in the same directory means a running shell reads
# whole-old or whole-new and never a half file.
scp -q "$here/lernie-update" "$host:.local/bin/.lernie-update.tmp"
scp -q "$here/lernie-update.service" "$here/lernie-update.timer" \
    "$host:.config/systemd/user/"
ssh -n "$host" 'chmod 0755 "$HOME/.local/bin/.lernie-update.tmp" && \
    mv -f "$HOME/.local/bin/.lernie-update.tmp" "$HOME/.local/bin/lernie-update"'

say "arming the timer on $host"
ssh -n "$host" 'systemctl --user daemon-reload; \
    systemctl --user reset-failed lernie-update.service 2>/dev/null; \
    systemctl --user enable --now lernie-update.timer'

# The verification, and it is the reconciler itself rather than a probe of one.
# `systemctl --user start` blocks on a `Type=oneshot` unit and exits non-zero
# when it fails, so this is a real end-to-end run — the index reached, the
# version compared, the build done if there was one — and not a status print.
# A first-ever seating builds the window's toolkit here, which is minutes.
say "running the first reconcile on $host (a first build is not quick)"
ssh -n "$host" 'systemctl --user start lernie-update.service' \
    || { ssh -n "$host" 'journalctl --user -u lernie-update.service \
             --no-pager --lines=30' 2>&1 | sed 's/^/  | /' >&2
         die "the first reconcile failed on $host (the timer is armed; it will retry)"; }

ssh -n "$host" 'systemctl --user status lernie-update.service --no-pager \
    --lines=5 2>/dev/null | sed -n "s/^ *[A-Za-z]*\[[0-9]*\]: //p"' || true
say "seated: $host tracks released versions hourly"
