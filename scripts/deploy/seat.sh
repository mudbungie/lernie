#!/bin/sh
# Seat the continuous-deployment timer on a seat box (bl-155a; the local carrier
# is bl-bae7):
#
#   make deploy-seat                    # THIS box
#   make deploy-seat HOST=<ssh-host>    # that one
#
# **Two carriers, one payload.** The three files and the four arming commands
# are spelled once and do not know which way they arrived; only `put` and `run`
# below differ. That is the idiom yog's `scripts/deploy/verify.sh` states for
# its own `--local`, and it is the whole reason the local form is not a second
# recipe: two statements of one seating drift, and the copy that drifts is the
# one nobody re-reads.
#
# **The local carrier is not a convenience, it is the only door for most seats.**
# A seat is a WINDOW, and the box most likely to run a window is the workstation
# somebody is sitting at — which is the box least likely to be running an sshd,
# and it cannot ssh to itself. Until bl-bae7 that box could not arm its own
# timer at all, so its lernie was whatever the last hand install left.
#
# HOST, when given, is an ssh destination and the ONLY parameter — no address,
# account or machine name is committed anywhere in this tree. That is the leak
# gate's rule and the severability one at the same time: pointing this at a
# second seat is a different argument, not an edit, and a box that should stop
# tracking releases is one `systemctl --user disable` away from stopping, with
# no file in this repository to change.
#
# **It seats a timer; it does not deploy a build.** Nothing is compiled here and
# nothing is carried over the channel but three small text files. The box
# installs from crates.io on its own schedule from then on — which is the
# difference between this and an engine's deployment, where the image is the
# unit of install and a human carries it. A seat's unit of install is a
# published version, and the registry already serves it.
#
# **It restarts nothing, because there is nothing to restart.** A seat is a
# window somebody launched. An install replaces the binary by rename, so an open
# window finishes its session on the build it started under and the next launch
# is the new one. Read `lernie-update`'s header for the protocol-skew
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
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
where=${host:-this box}

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf '%s: %s\n' "${0##*/}" "$*" >&2; exit 1; }

# The carriers, and the whole of what a HOST changes. `run` takes one shell
# command; `put` takes a source file and a destination path RELATIVE to the
# home directory, so the payload below never has to know whose home it is.
if [ -n "$host" ]; then
    run() { ssh -n "$host" "$1"; }
    put() { scp -q "$1" "$host:$2"; }
else
    run() { sh -c "$1"; }
    put() { install -m 0644 "$1" "$HOME/$2"; }
fi

say "seating the reconciler on $where"
run 'mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"'
# To a temp name and then `mv` into place: the reconciler may be running right
# now (the timer is armed from the previous seating), and both carriers truncate
# before they write. rename(2) in the same directory means a running shell reads
# whole-old or whole-new and never a half file.
put "$here/lernie-update" .local/bin/.lernie-update.tmp
put "$here/lernie-update.service" .config/systemd/user/lernie-update.service
put "$here/lernie-update.timer" .config/systemd/user/lernie-update.timer
run 'chmod 0755 "$HOME/.local/bin/.lernie-update.tmp" && \
    mv -f "$HOME/.local/bin/.lernie-update.tmp" "$HOME/.local/bin/lernie-update"'

say "arming the timer on $where"
run 'systemctl --user daemon-reload; \
    systemctl --user reset-failed lernie-update.service 2>/dev/null; \
    systemctl --user enable --now lernie-update.timer'

# The verification, and it is the reconciler itself rather than a probe of one.
# `systemctl --user start` blocks on a `Type=oneshot` unit and exits non-zero
# when it fails, so this is a real end-to-end run — the index reached, the
# version compared, the build done if there was one — and not a status print.
# A first-ever seating builds the window's toolkit here, which is minutes.
say "running the first reconcile on $where (a first build is not quick)"
run 'systemctl --user start lernie-update.service' \
    || { run 'journalctl --user -u lernie-update.service --no-pager --lines=30' 2>&1 \
             | sed 's/^/  | /' >&2
         die "the first reconcile failed on $where (the timer is armed; it will retry)"; }

run 'systemctl --user status lernie-update.service --no-pager --lines=5 2>/dev/null \
    | sed -n "s/^ *[A-Za-z]*\[[0-9]*\]: //p"' || true
say "seated: $where tracks released versions hourly"
